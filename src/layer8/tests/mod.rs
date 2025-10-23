//! # Layer 8 Tests
//!
//! Test module for the resource management layer. Includes unit tests,
//! integration tests, and performance benchmarks.

pub mod unit_tests;

#[cfg(test)]
mod integration_tests {
    use layer8_resource_management::{
        ResourceManager, ResourceConfig, ResourceRequest, ResourceRequirements, Priority,
        types::{ResourceError, AllocationStatus}
    };

    #[tokio::test]
    async fn test_resource_allocation_integration() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let request = ResourceRequest::new(
            "layer7".to_string(),
            ResourceRequirements {
                gpu_count: 1,
                gpu_memory_gb: 8,
                cpu_cores: 4,
                ram_gb: 16,
                storage_gb: 50,
                special_requirements: Vec::new(),
            },
            Priority::High,
        );

        let allocation = manager.allocate_resources(request).await;
        assert!(allocation.is_ok());

        let allocation = allocation.unwrap();
        assert!(matches!(allocation.status, AllocationStatus::Active));

        // Test deallocation
        let release_result = manager.release_allocation(allocation.allocation_id).await;
        assert!(release_result.is_ok());
    }

    #[tokio::test]
    async fn test_cross_layer_integration() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        // Test Layer 7 resource request
        let request = ResourceRequest::new(
            "layer7".to_string(),
            ResourceRequirements {
                gpu_count: 2,
                gpu_memory_gb: 16,
                cpu_cores: 8,
                ram_gb: 32,
                storage_gb: 100,
                special_requirements: vec!["evolution-compute".to_string()],
            },
            Priority::Critical,
        );

        let allocation = manager.allocate_resources(request).await;
        assert!(allocation.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_resources_handling() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        // Request more resources than available
        let request = ResourceRequest::new(
            "layer4".to_string(),
            ResourceRequirements {
                gpu_count: 10, // More than available
                gpu_memory_gb: 8,
                cpu_cores: 4,
                ram_gb: 16,
                storage_gb: 50,
                special_requirements: Vec::new(),
            },
            Priority::Normal,
        );

        let allocation = manager.allocate_resources(request).await;
        assert!(matches!(allocation, Err(ResourceError::InsufficientResources { .. })));
    }
}

#[cfg(test)]
mod performance_tests {
    use layer8_resource_management::{ResourceManager, ResourceConfig};
    use std::time::Instant;

    #[tokio::test]
    async fn test_allocation_performance() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let start_time = Instant::now();

        // Perform multiple allocations
        for i in 0..10 {
            let request = layer8_resource_management::ResourceRequest::new(
                "layer4".to_string(),
                layer8_resource_management::ResourceRequirements {
                    gpu_count: 1,
                    gpu_memory_gb: 8,
                    cpu_cores: 2,
                    ram_gb: 8,
                    storage_gb: 25,
                    special_requirements: Vec::new(),
                },
                layer8_resource_management::Priority::Normal,
            );

            let _allocation = manager.allocate_resources(request).await.unwrap();
        }

        let duration = start_time.elapsed();
        println!("10 allocations completed in: {:?}", duration);

        // Should complete within reasonable time
        assert!(duration.as_secs() < 30);
    }

    #[tokio::test]
    async fn test_concurrent_allocations() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        // Test concurrent allocation requests
        let mut handles = Vec::new();

        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let request = layer8_resource_management::ResourceRequest::new(
                    format!("layer{}", i % 3 + 4),
                    layer8_resource_management::ResourceRequirements {
                        gpu_count: 1,
                        gpu_memory_gb: 8,
                        cpu_cores: 2,
                        ram_gb: 8,
                        storage_gb: 25,
                        special_requirements: Vec::new(),
                    },
                    layer8_resource_management::Priority::Normal,
                );

                manager_clone.allocate_resources(request).await
            });
            handles.push(handle);
        }

        // Wait for all allocations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod security_tests {
    use layer8_resource_management::{ResourceManager, ResourceConfig, ResourceRequest, Priority};

    #[tokio::test]
    async fn test_request_validation() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        // Test with invalid requirements
        let invalid_request = ResourceRequest::new(
            "layer4".to_string(),
            layer8_resource_management::ResourceRequirements {
                gpu_count: 100, // Exceeds limits
                gpu_memory_gb: 8,
                cpu_cores: 4,
                ram_gb: 16,
                storage_gb: 50,
                special_requirements: Vec::new(),
            },
            Priority::Normal,
        );

        let allocation = manager.allocate_resources(invalid_request).await;
        assert!(allocation.is_err());
    }

    #[tokio::test]
    async fn test_cost_limit_enforcement() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let expensive_request = ResourceRequest::new(
            "layer7".to_string(),
            layer8_resource_management::ResourceRequirements {
                gpu_count: 4, // Very expensive
                gpu_memory_gb: 24,
                cpu_cores: 16,
                ram_gb: 64,
                storage_gb: 200,
                special_requirements: Vec::new(),
            },
            Priority::Normal,
        );

        // Set low cost limit
        let mut request = expensive_request;
        request.max_cost_per_hour = Some(1.0); // Very low limit

        let allocation = manager.allocate_resources(request).await;
        assert!(matches!(allocation, Err(ResourceError::CostLimitExceeded { .. })));
    }
}