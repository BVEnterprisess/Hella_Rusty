use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests: u32,
    pub window: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub default: RateLimit,
    pub endpoints: HashMap<String, RateLimit>,
    pub burst_limit: u32,
}

#[derive(Debug)]
struct ClientBucket {
    requests: Vec<Instant>,
    last_burst_reset: Instant,
}

pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<Mutex<HashMap<IpAddr, ClientBucket>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(&self, client_ip: IpAddr, endpoint: &str) -> Result<(), RateLimitError> {
        let mut clients = self.clients.lock().unwrap();

        let bucket = clients.entry(client_ip).or_insert_with(|| ClientBucket {
            requests: Vec::new(),
            last_burst_reset: Instant::now(),
        });

        // Clean old requests outside the window
        let limit = self.get_limit_for_endpoint(endpoint);
        let window_start = Instant::now() - limit.window;

        bucket.requests.retain(|&time| time > window_start);

        // Check burst limit (requests per second)
        if bucket.requests.len() >= self.config.burst_limit as usize {
            let time_since_last_burst_reset = Instant::now().duration_since(bucket.last_burst_reset);
            if time_since_last_burst_reset < Duration::from_secs(1) {
                return Err(RateLimitError::BurstLimitExceeded);
            }
            bucket.last_burst_reset = Instant::now();
            bucket.requests.clear();
        }

        // Check rate limit
        if bucket.requests.len() >= limit.requests as usize {
            return Err(RateLimitError::RateLimitExceeded);
        }

        // Add current request
        bucket.requests.push(Instant::now());

        Ok(())
    }

    fn get_limit_for_endpoint(&self, endpoint: &str) -> RateLimit {
        self.config.endpoints
            .get(endpoint)
            .cloned()
            .unwrap_or(self.config.default.clone())
    }

    pub fn get_client_stats(&self, client_ip: IpAddr) -> Option<usize> {
        let clients = self.clients.lock().unwrap();
        clients.get(&client_ip).map(|bucket| bucket.requests.len())
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    RateLimitExceeded,
    BurstLimitExceeded,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            RateLimitError::BurstLimitExceeded => write!(f, "Burst limit exceeded"),
        }
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiting() {
        let config = RateLimitConfig {
            default: RateLimit {
                requests: 10,
                window: Duration::from_secs(60),
            },
            endpoints: HashMap::new(),
            burst_limit: 5,
        };

        let limiter = RateLimiter::new(config);
        let client_ip = IpAddr::from([127, 0, 0, 1]);

        // Should allow first 10 requests
        for i in 0..10 {
            assert!(limiter.check_rate_limit(client_ip, "/api/test").is_ok());
        }

        // Should block 11th request
        assert!(limiter.check_rate_limit(client_ip, "/api/test").is_err());
    }
}