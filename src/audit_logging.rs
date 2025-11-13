use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: u64,
    pub event_type: String,
    pub user_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub severity: AuditSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone)]
pub struct AuditLogger {
    log_file: Arc<Mutex<BufWriter<File>>>,
    _retention_days: u32,
}

impl AuditLogger {
    pub fn new(log_path: &str, retention_days: u32) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(log_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(Self {
            log_file: Arc::new(Mutex::new(BufWriter::new(file))),
            _retention_days: retention_days,
        })
    }

    pub fn log_event(&self, mut event: AuditEvent) -> Result<(), Box<dyn std::error::Error>> {
        // Set timestamp if not already set
        if event.timestamp == 0 {
            event.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        // Set ID if not already set
        if event.id.is_empty() {
            event.id = Uuid::new_v4().to_string();
        }

        // Serialize and write event
        let event_json = serde_json::to_string(&event)? + "\n";
        let mut writer = self.log_file.lock().unwrap();
        writer.write_all(event_json.as_bytes())?;
        writer.flush()?;

        // Log to stderr for high severity events
        if matches!(event.severity, AuditSeverity::Critical | AuditSeverity::High) {
            eprintln!(
                "AUDIT [{:?}]: {} - {}",
                event.severity, event.event_type, event.action
            );
        }

        Ok(())
    }

    pub fn log_authentication(
        &self,
        user_id: &str,
        success: bool,
        ip_address: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = AuditEvent {
            id: String::new(),
            timestamp: 0,
            event_type: "authentication".to_string(),
            user_id: Some(user_id.to_string()),
            resource: "user_account".to_string(),
            action: if success { "login" } else { "login_failed" }.to_string(),
            result: if success { "success" } else { "failure" }.to_string(),
            ip_address,
            user_agent: None,
            metadata: std::collections::HashMap::new(),
            severity: if success {
                AuditSeverity::Low
            } else {
                AuditSeverity::Medium
            },
        };

        self.log_event(event)
    }

    pub fn log_api_access(
        &self,
        user_id: Option<String>,
        endpoint: &str,
        method: &str,
        status_code: u16,
        ip_address: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = AuditEvent {
            id: String::new(),
            timestamp: 0,
            event_type: "api_access".to_string(),
            user_id,
            resource: endpoint.to_string(),
            action: method.to_string(),
            result: status_code.to_string(),
            ip_address,
            user_agent: None,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("status_code".to_string(), status_code.to_string());
                map.insert("http_method".to_string(), method.to_string());
                map
            },
            severity: match status_code {
                200..=299 => AuditSeverity::Low,
                400..=499 => AuditSeverity::Medium,
                _ => AuditSeverity::High,
            },
        };

        self.log_event(event)
    }

    pub fn log_model_access(
        &self,
        user_id: Option<String>,
        model_name: &str,
        action: &str,
        ip_address: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = AuditEvent {
            id: String::new(),
            timestamp: 0,
            event_type: "model_access".to_string(),
            user_id,
            resource: model_name.to_string(),
            action: action.to_string(),
            result: "success".to_string(),
            ip_address,
            user_agent: None,
            metadata: std::collections::HashMap::new(),
            severity: AuditSeverity::Medium,
        };

        self.log_event(event)
    }

    pub fn log_admin_action(
        &self,
        admin_user_id: &str,
        action: &str,
        target: &str,
        ip_address: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = AuditEvent {
            id: String::new(),
            timestamp: 0,
            event_type: "admin_action".to_string(),
            user_id: Some(admin_user_id.to_string()),
            resource: target.to_string(),
            action: action.to_string(),
            result: "success".to_string(),
            ip_address,
            user_agent: None,
            metadata: std::collections::HashMap::new(),
            severity: AuditSeverity::High,
        };

        self.log_event(event)
    }
}

// Global audit logger instance
lazy_static::lazy_static! {
    static ref AUDIT_LOGGER: Arc<AuditLogger> = Arc::new(
        AuditLogger::new("logs/audit.log", 90).expect("Failed to create audit logger")
    );
}

pub fn get_audit_logger() -> Arc<AuditLogger> {
    AUDIT_LOGGER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_audit_logging() {
        let temp_file = NamedTempFile::new().unwrap();
        let logger = AuditLogger::new(temp_file.path().to_str().unwrap(), 7).unwrap();

        let event = AuditEvent {
            id: "test-id".to_string(),
            timestamp: 1234567890,
            event_type: "test_event".to_string(),
            user_id: Some("test_user".to_string()),
            resource: "test_resource".to_string(),
            action: "test_action".to_string(),
            result: "success".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            metadata: std::collections::HashMap::new(),
            severity: AuditSeverity::Low,
        };

        assert!(logger.log_event(event).is_ok());
    }
}
