//! Genome Management System for Layer 7 Evolution

use crate::types::*;
use async_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, warn};

/// Genome Manager handles storage, versioning, and lifecycle of agent genomes
pub struct GenomeManager {
    genomes: Arc<Mutex<HashMap<GenomeId, AgentGenome>>>,
    agent_genomes: Arc<Mutex<HashMap<AgentId, Vec<GenomeId>>>>,
    storage_backend: Arc<dyn GenomeStorage>,
    config: GenomeConfig,
    version_history: Arc<Mutex<HashMap<AgentId, Vec<GenomeVersion>>>>,
}

#[derive(Debug, Clone)]
struct GenomeVersion {
    genome_id: GenomeId,
    version: u64,
    created_at: DateTime<Utc>,
    fitness_score: f64,
    is_active: bool,
}

pub trait GenomeStorage: Send + Sync {
    async fn store_genome(&self, genome: &AgentGenome) -> Result<(), GenomeError>;
    async fn retrieve_genome(&self, genome_id: GenomeId) -> Result<AgentGenome, GenomeError>;
    async fn delete_genome(&self, genome_id: GenomeId) -> Result<(), GenomeError>;
    async fn list_genomes(&self, agent_id: AgentId) -> Result<Vec<GenomeId>, GenomeError>;
    async fn backup_genome(&self, genome_id: GenomeId) -> Result<(), GenomeError>;
}

impl GenomeManager {
    /// Create a new genome manager
    pub async fn new(config: GenomeConfig) -> Result<Self, GenomeError> {
        let storage_backend = Arc::new(PostgresGenomeStorage::new().await?);

        Ok(Self {
            genomes: Arc::new(Mutex::new(HashMap::new())),
            agent_genomes: Arc::new(Mutex::new(HashMap::new())),
            storage_backend,
            config,
            version_history: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Store a new genome
    pub async fn store_genome(&self, genome: AgentGenome) -> Result<GenomeId, GenomeError> {
        // Validate genome size
        if genome.neural_weights.len() > self.config.max_genome_size {
            return Err(GenomeError::InvalidData(format!(
                "Genome size {} exceeds maximum {}",
                genome.neural_weights.len(),
                self.config.max_genome_size
            )));
        }

        // Compress if enabled
        let processed_genome = if self.config.compression_enabled {
            self.compress_genome(genome).await?
        } else {
            genome
        };

        // Store in memory
        let genome_id = processed_genome.id;
        self.genomes.lock().await.insert(genome_id, processed_genome.clone());

        // Update agent genome index
        let mut agent_genomes = self.agent_genomes.lock().await;
        agent_genomes.entry(processed_genome.agent_id)
            .or_insert_with(Vec::new)
            .push(genome_id);

        // Update version history
        self.update_version_history(processed_genome).await?;

        // Store in persistent storage
        self.storage_backend.store_genome(&processed_genome).await?;

        info!("Stored genome {} for agent {}", genome_id, processed_genome.agent_id);
        Ok(genome_id)
    }

    /// Retrieve a genome by ID
    pub async fn get_genome(&self, genome_id: GenomeId) -> Result<AgentGenome, GenomeError> {
        // Check memory cache first
        if let Some(genome) = self.genomes.lock().await.get(&genome_id).cloned() {
            return Ok(genome);
        }

        // Retrieve from storage
        let genome = self.storage_backend.retrieve_genome(genome_id).await?;

        // Cache in memory
        self.genomes.lock().await.insert(genome_id, genome.clone());

        Ok(genome)
    }

    /// Get current active genome for an agent
    pub async fn get_current_genome(&self, agent_id: AgentId) -> Result<Option<AgentGenome>, GenomeError> {
        let version_history = self.version_history.lock().await;
        if let Some(versions) = version_history.get(&agent_id) {
            if let Some(active_version) = versions.iter().find(|v| v.is_active) {
                return self.get_genome(active_version.genome_id).await.map(Some);
            }
        }
        Ok(None)
    }

    /// Get genome history for an agent
    pub async fn get_genome_history(&self, agent_id: AgentId) -> Result<Vec<AgentGenome>, GenomeError> {
        let agent_genomes = self.agent_genomes.lock().await;
        let genome_ids = agent_genomes.get(&agent_id).cloned().unwrap_or_default();

        let mut genomes = Vec::new();
        for genome_id in genome_ids {
            genomes.push(self.get_genome(genome_id).await?);
        }

        // Sort by version
        genomes.sort_by_key(|g| g.version);

        Ok(genomes)
    }

    /// Set a genome as active for an agent
    pub async fn set_active_genome(&self, agent_id: AgentId, genome_id: GenomeId) -> Result<(), GenomeError> {
        // Validate genome belongs to agent
        let genome = self.get_genome(genome_id).await?;
        if genome.agent_id != agent_id {
            return Err(GenomeError::InvalidData("Genome does not belong to agent".to_string()));
        }

        // Update version history
        let mut version_history = self.version_history.lock().await;
        if let Some(versions) = version_history.get_mut(&agent_id) {
            // Deactivate all versions
            for version in versions.iter_mut() {
                version.is_active = false;
            }

            // Activate the specified version
            if let Some(version) = versions.iter_mut().find(|v| v.genome_id == genome_id) {
                version.is_active = true;
            }
        }

        info!("Set genome {} as active for agent {}", genome_id, agent_id);
        Ok(())
    }

    /// Create a backup of a genome
    pub async fn backup_genome(&self, genome_id: GenomeId) -> Result<(), GenomeError> {
        self.storage_backend.backup_genome(genome_id).await?;

        // Update version history to mark as backed up
        let genome = self.get_genome(genome_id).await?;
        let mut version_history = self.version_history.lock().await;
        if let Some(versions) = version_history.get_mut(&genome.agent_id) {
            if let Some(version) = versions.iter_mut().find(|v| v.genome_id == genome_id) {
                // Mark as backed up (in a real implementation, this would track backup status)
            }
        }

        info!("Backed up genome {} for agent {}", genome_id, genome.agent_id);
        Ok(())
    }

    /// Clean up old genomes based on retention policy
    pub async fn cleanup_old_genomes(&self, agent_id: AgentId) -> Result<usize, GenomeError> {
        let mut version_history = self.version_history.lock().await;
        let versions = version_history.get_mut(&agent_id).ok_or_else(|| GenomeError::NotFound(genome_id))?;

        // Keep only the most recent N versions (configurable)
        let keep_count = self.config.backup_generations as usize;
        if versions.len() <= keep_count {
            return Ok(0); // Nothing to clean up
        }

        // Sort by version and keep the most recent
        versions.sort_by_key(|v| v.version);
        let to_remove = &versions[..versions.len() - keep_count];

        let mut removed_count = 0;
        for version in to_remove {
            self.storage_backend.delete_genome(version.genome_id).await?;
            removed_count += 1;
        }

        // Remove from version history
        *versions = versions[versions.len() - keep_count..].to_vec();

        info!("Cleaned up {} old genomes for agent {}", removed_count, agent_id);
        Ok(removed_count)
    }

    async fn compress_genome(&self, genome: AgentGenome) -> Result<AgentGenome, GenomeError> {
        // Simple compression using lz4 (in a real implementation)
        // For now, just return the original genome
        Ok(genome)
    }

    async fn update_version_history(&self, genome: AgentGenome) -> Result<(), GenomeError> {
        let mut version_history = self.version_history.lock().await;
        let versions = version_history.entry(genome.agent_id).or_insert_with(Vec::new);

        versions.push(GenomeVersion {
            genome_id: genome.id,
            version: genome.version,
            created_at: genome.created_at,
            fitness_score: genome.metadata.fitness_score,
            is_active: false, // Will be set active when deployed
        });

        Ok(())
    }
}

/// PostgreSQL-based genome storage implementation
pub struct PostgresGenomeStorage {
    // In a real implementation, this would contain database connection
}

impl PostgresGenomeStorage {
    pub async fn new() -> Result<Self, GenomeError> {
        // Initialize database connection
        Ok(Self {})
    }
}

impl GenomeStorage for PostgresGenomeStorage {
    async fn store_genome(&self, genome: &AgentGenome) -> Result<(), GenomeError> {
        // Store genome in PostgreSQL
        info!("Storing genome {} in PostgreSQL", genome.id);
        Ok(())
    }

    async fn retrieve_genome(&self, genome_id: GenomeId) -> Result<AgentGenome, GenomeError> {
        // Retrieve genome from PostgreSQL
        info!("Retrieving genome {} from PostgreSQL", genome_id);
        Err(GenomeError::NotFound(genome_id)) // Placeholder
    }

    async fn delete_genome(&self, genome_id: GenomeId) -> Result<(), GenomeError> {
        // Delete genome from PostgreSQL
        info!("Deleting genome {} from PostgreSQL", genome_id);
        Ok(())
    }

    async fn list_genomes(&self, agent_id: AgentId) -> Result<Vec<GenomeId>, GenomeError> {
        // List genomes for agent from PostgreSQL
        info!("Listing genomes for agent {} from PostgreSQL", agent_id);
        Ok(Vec::new()) // Placeholder
    }

    async fn backup_genome(&self, genome_id: GenomeId) -> Result<(), GenomeError> {
        // Backup genome to secondary storage
        info!("Backing up genome {} to secondary storage", genome_id);
        Ok(())
    }
}