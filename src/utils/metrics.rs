//! Metrics collection and reporting utilities

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), value);
    }

    pub fn observe_histogram(&self, name: &str, duration: Duration) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    pub fn record_request(&self, endpoint: &str, duration: Duration, status_code: u16) {
        // Increment request counter
        self.increment_counter(&format!("requests_total{{endpoint=\"{}\"}}", endpoint), 1);

        // Record response time
        self.observe_histogram(
            &format!("request_duration_seconds{{endpoint=\"{}\"}}", endpoint),
            duration,
        );

        // Set status code gauge
        self.set_gauge(
            &format!(
                "response_status{{endpoint=\"{}\",code=\"{}\"}}",
                endpoint, status_code
            ),
            1.0,
        );
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.lock().unwrap();
        counters.get(name).copied().unwrap_or(0)
    }

    pub fn get_gauge(&self, name: &str) -> f64 {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(name).copied().unwrap_or(0.0)
    }

    pub fn get_histogram_percentile(&self, name: &str, percentile: f64) -> Option<Duration> {
        let histograms = self.histograms.lock().unwrap();
        if let Some(values) = histograms.get(name) {
            let mut sorted_values = values.clone();
            sorted_values.sort();

            let index = ((sorted_values.len() as f64 - 1.0) * percentile).round() as usize;
            sorted_values.get(index).copied()
        } else {
            None
        }
    }

    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Export counters
        {
            let counters = self.counters.lock().unwrap();
            for (name, value) in counters.iter() {
                output.push_str(&format!("# HELP {} Counter metric\n", name));
                output.push_str(&format!("# TYPE {} counter\n", name));
                output.push_str(&format!("{} {}\n", name, value));
            }
        }

        // Export gauges
        {
            let gauges = self.gauges.lock().unwrap();
            for (name, value) in gauges.iter() {
                output.push_str(&format!("# HELP {} Gauge metric\n", name));
                output.push_str(&format!("# TYPE {} gauge\n", name));
                output.push_str(&format!("{} {}\n", name, value));
            }
        }

        // Export histograms
        {
            let histograms = self.histograms.lock().unwrap();
            for (name, values) in histograms.iter() {
                if !values.is_empty() {
                    output.push_str(&format!("# HELP {} Histogram metric\n", name));
                    output.push_str(&format!("# TYPE {} histogram\n", name));

                    let mut sorted_values = values.clone();
                    sorted_values.sort();

                    let count = values.len();
                    let sum: Duration = values.iter().sum();

                    output.push_str(&format!("{}_count {}\n", name, count));
                    output.push_str(&format!("{}_sum {}\n", name, sum.as_secs_f64()));

                    // Calculate buckets (simplified)
                    let _p50 = self.get_histogram_percentile(name, 0.5).unwrap_or_default();
                    let _p95 = self
                        .get_histogram_percentile(name, 0.95)
                        .unwrap_or_default();
                    let _p99 = self
                        .get_histogram_percentile(name, 0.99)
                        .unwrap_or_default();

                    output.push_str(&format!(
                        "{}_bucket{{le=\"0.1\"}} {}\n",
                        name,
                        values
                            .iter()
                            .filter(|&&d| d <= Duration::from_millis(100))
                            .count()
                    ));
                    output.push_str(&format!(
                        "{}_bucket{{le=\"0.5\"}} {}\n",
                        name,
                        values
                            .iter()
                            .filter(|&&d| d <= Duration::from_millis(500))
                            .count()
                    ));
                    output.push_str(&format!(
                        "{}_bucket{{le=\"1.0\"}} {}\n",
                        name,
                        values
                            .iter()
                            .filter(|&&d| d <= Duration::from_secs(1))
                            .count()
                    ));
                    output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", name, count));
                }
            }
        }

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestTimer {
    start: Instant,
    collector: Arc<MetricsCollector>,
    endpoint: String,
}

impl RequestTimer {
    pub fn new(collector: Arc<MetricsCollector>, endpoint: String) -> Self {
        Self {
            start: Instant::now(),
            collector,
            endpoint,
        }
    }

    pub fn finish(self, status_code: u16) -> Duration {
        let duration = self.start.elapsed();
        self.collector
            .record_request(&self.endpoint, duration, status_code);
        duration
    }
}

impl MetricsCollector {
    pub fn start_timer(&self, endpoint: String) -> RequestTimer {
        RequestTimer::new(Arc::new(self.clone()), endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let collector = MetricsCollector::new();

        collector.increment_counter("test_counter", 5);
        collector.increment_counter("test_counter", 3);

        assert_eq!(collector.get_counter("test_counter"), 8);
        assert_eq!(collector.get_counter("nonexistent"), 0);
    }

    #[test]
    fn test_gauge() {
        let collector = MetricsCollector::new();

        collector.set_gauge("test_gauge", 42.5);
        assert_eq!(collector.get_gauge("test_gauge"), 42.5);

        collector.set_gauge("test_gauge", 50.0);
        assert_eq!(collector.get_gauge("test_gauge"), 50.0);
    }

    #[test]
    fn test_histogram() {
        let collector = MetricsCollector::new();

        collector.observe_histogram("test_histogram", Duration::from_millis(100));
        collector.observe_histogram("test_histogram", Duration::from_millis(200));
        collector.observe_histogram("test_histogram", Duration::from_millis(300));

        let p50 = collector.get_histogram_percentile("test_histogram", 0.5);
        assert!(p50.is_some());
    }

    #[test]
    fn test_request_timer() {
        let collector = MetricsCollector::new();
        let timer = collector.start_timer("/api/test".to_string());

        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(10));

        let duration = timer.finish(200);

        assert!(duration.as_millis() >= 10);
        assert_eq!(
            collector.get_counter("requests_total{endpoint=\"/api/test\"}"),
            1
        );
    }
}
