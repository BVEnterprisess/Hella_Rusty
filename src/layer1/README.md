# Layer 1 (Discovery) - Environmental Awareness and Data Collection

Layer 1 provides comprehensive environmental awareness and data collection capabilities for Project Chimera. It serves as the foundation layer that discovers, monitors, and collects data about the system environment, providing critical information to higher layers for planning, validation, and execution.

## Overview

Layer 1 (Discovery) is responsible for:

- **Environmental Scanning**: Discovering and cataloging system components, network topology, and available resources
- **System Monitoring**: Continuously monitoring system health, performance, and availability
- **Data Collection**: Gathering data from multiple sources including system metrics, application logs, and external APIs
- **Integration Hub**: Managing communication and data sharing with other layers in the system

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Layer 1 - Discovery                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
│  │ Environmental│  │ System      │  │ Data       │  │ External│
│  │ Scanner     │  │ Monitor     │  │ Collector  │  │ API     │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────┤
│  │ Data Storage│  │ Event       │  │ Integration│  │ Metrics │
│  │ & Caching   │  │ Processor   │  │ Hub        │  │ & Alert │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────┘
└─────────────────────────────────────────────────────────────┘
                       │                    │
         ┌────────────▼────┐    ┌─────────▼─────────┐
         │                  │    │                   │
    ┌────▼────┐       ┌─────▼──────┐       ┌────▼────┐
    │Layer 2  │       │Layer 3     │       │Layer 4  │
    │(Planning)│     │(Validation)│       │(Execution)│
    └─────────┘       └───────────┘       └─────────┘
```

## Core Components

### 1. Environmental Scanner (`environmental_scanner.rs`)
Discovers and catalogs system components and resources:

- **System Probes**: Configurable probes for different system types (local, network, container, service)
- **Network Topology Discovery**: Maps network segments and connections
- **Resource Inventory**: Tracks available CPU, memory, disk, and network resources
- **Caching System**: Performance optimization with configurable TTL

**Key Features:**
- Concurrent probe execution with timeout handling
- Incremental scanning for large environments
- Configurable scan intervals and depth
- Comprehensive system capability detection

### 2. System Monitor (`system_monitor.rs`)
Provides continuous health checking and performance monitoring:

- **Health Checks**: CPU, memory, disk usage, and network connectivity monitoring
- **Performance Metrics**: Real-time collection of system performance data
- **Alert Generation**: Automated alerting for system issues and anomalies
- **Threshold Management**: Configurable alert thresholds for different metrics

**Key Features:**
- Multiple health check types (connectivity, performance, resource usage, security)
- Real-time performance monitoring with baseline establishment
- Automated alert generation and acknowledgment system
- Integration with metrics collection system

### 3. Data Collector (`data_collector.rs`)
Multi-source data ingestion and preprocessing:

- **Data Sources**: System metrics, application logs, network traffic, external APIs
- **Data Pipeline**: Validation, transformation, and normalization
- **Batch Processing**: Efficient batching and buffering for high-throughput scenarios
- **Quality Monitoring**: Data quality scoring and validation

**Key Features:**
- Pluggable data source architecture
- Real-time and batch data collection modes
- Data quality assessment and filtering
- Compression and retention policy management

### 4. Integration Hub (`integration_hub.rs`)
Inter-layer communication and data distribution:

- **Layer Connections**: Dedicated connections to Layer 2, 3, and 4
- **Event Routing**: Intelligent routing of discovery data based on type and priority
- **Message Queuing**: Reliable message delivery with retry mechanisms
- **Protocol Management**: Standardized communication protocols

**Key Features:**
- Event-driven architecture for responsive data sharing
- Priority-based message routing
- Connection health monitoring and failover
- Bidirectional communication support

### 5. Metrics Collection (`metrics.rs`)
Comprehensive metrics and monitoring:

- **Prometheus Integration**: Standard metrics export for monitoring systems
- **Performance Tracking**: Operation duration and throughput metrics
- **Health Scoring**: Calculated health scores based on multiple factors
- **Custom Metrics**: Layer-specific metrics for operational visibility

**Key Features:**
- Real-time metrics collection and aggregation
- Performance timer utilities for operation tracking
- Health score calculation algorithms
- Metrics formatting for logging and monitoring

## Configuration

Layer 1 is configured through the `DiscoveryConfig` structure:

```rust
let config = DiscoveryConfig {
    scanner: ScannerConfig {
        scan_interval_seconds: 300,  // 5 minutes
        max_scan_duration_seconds: 60,
        network_timeout_seconds: 10,
        cache_ttl_seconds: 3600,
        deep_scan_enabled: false,
    },
    monitor: MonitorConfig {
        check_interval_seconds: 60,  // 1 minute
        cpu_alert_threshold: 80.0,
        memory_alert_threshold: 85.0,
        disk_alert_threshold: 90.0,
        real_time_enabled: true,
    },
    collector: CollectorConfig {
        collection_interval_seconds: 30,
        max_batch_size: 1000,
        retention_hours: 168,  // 7 days
        compression_enabled: true,
        api_timeout_seconds: 30,
    },
    integration: IntegrationConfig {
        layer_timeout_seconds: 10,
        max_retry_attempts: 3,
        encryption_enabled: true,
        queue_size: 10000,
    },
};
```

## Usage

### Basic Usage

```rust
use layer1_discovery::*;

// Create and start discovery service
let config = DiscoveryConfig::default();
let mut service = DiscoveryService::new(config).await?;

service.start().await?;

// Get current system state
let system_state = service.get_system_state().await?;
println!("Discovered {} systems", system_state.environmental.systems.len());

// Perform full system scan
let scan_result = service.trigger_full_scan().await?;
println!("Scan found {} issues", scan_result.issues_found.len());

// Check service health
let health = service.health_check().await?;
println!("Service status: {:?}", health.status);

// Stop the service
service.stop().await?;
```

### Custom Health Checks

```rust
use layer1_discovery::*;

struct CustomHealthCheck {
    check_id: String,
}

#[async_trait]
impl HealthCheck for CustomHealthCheck {
    async fn check_health(&self) -> Result<HealthCheck, HealthError> {
        // Implement custom health check logic
        Ok(HealthCheck {
            check_id: self.check_id.clone(),
            system_id: "custom-system".to_string(),
            check_type: HealthCheckType::Custom("business-logic".to_string()),
            status: HealthStatus::Healthy,
            duration_ms: 50,
            error_message: None,
            metrics: HashMap::new(),
            timestamp: Utc::now(),
        })
    }

    fn get_check_id(&self) -> CheckId {
        self.check_id.clone()
    }

    fn get_check_name(&self) -> &str {
        "Custom Business Logic Check"
    }

    fn get_check_type(&self) -> HealthCheckType {
        HealthCheckType::Custom("business-logic".to_string())
    }
}
```

### Custom Data Sources

```rust
use layer1_discovery::*;

struct CustomDataSource {
    source_id: String,
}

#[async_trait]
impl DataSource for CustomDataSource {
    async fn collect_data(&self) -> Result<DataBatch, CollectionError> {
        // Implement custom data collection logic
        let data_points = vec![
            DataPoint {
                metric_name: "custom_metric".to_string(),
                value: 42.0,
                unit: "count".to_string(),
                tags: HashMap::new(),
                timestamp: Utc::now(),
            }
        ];

        Ok(DataBatch {
            source_id: self.source_id.clone(),
            timestamp: Utc::now(),
            data_points,
            quality_score: 0.95,
            metadata: HashMap::new(),
        })
    }

    fn get_source_id(&self) -> SourceId {
        self.source_id.clone()
    }

    fn get_source_type(&self) -> DataSourceType {
        DataSourceType::Custom("business-data".to_string())
    }
}
```

## Data Types

### Core Types

- **`DiscoveryConfig`**: Main configuration structure for all Layer 1 components
- **`SystemState`**: Complete snapshot of system environmental, monitoring, and collection state
- **`DiscoveredSystem`**: Information about a discovered system including resources and capabilities
- **`HealthCheck`**: Result of a health check operation with metrics and status
- **`DataBatch`**: Collection of data points from a single source
- **`Alert`**: System alert with severity, description, and acknowledgment status

### Error Types

- **`DiscoveryError`**: Main error type for Layer 1 operations
- **`ScanError`**: Errors related to environmental scanning
- **`HealthError`**: Errors related to health checking
- **`CollectionError`**: Errors related to data collection
- **`IntegrationError`**: Errors related to inter-layer communication

## Metrics

Layer 1 exports comprehensive metrics via Prometheus:

### Counters
- `layer1_systems_discovered_total`: Total systems discovered
- `layer1_health_checks_total`: Total health checks performed
- `layer1_data_points_collected_total`: Total data points collected
- `layer1_alerts_generated_total`: Total alerts generated
- `layer1_scan_errors_total`: Total scan errors
- `layer1_collection_errors_total`: Total collection errors

### Gauges
- `layer1_active_systems`: Currently active systems
- `layer1_active_data_sources`: Currently active data sources
- `layer1_system_health_score`: Overall system health score (0.0-1.0)

### Histograms
- `layer1_scan_duration_seconds`: Environmental scan duration
- `layer1_health_check_duration_seconds`: Health check duration
- `layer1_collection_duration_seconds`: Data collection duration

## Testing

Layer 1 includes comprehensive testing:

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test --test unit_tests
cargo test --test integration_tests
cargo test --test performance_tests

# Run tests with output
cargo test -- --nocapture

# Run tests for specific components
cargo test environmental_scanner
cargo test system_monitor
cargo test data_collector
cargo test integration_hub
cargo test metrics
```

## Performance

Layer 1 is designed for high performance and scalability:

- **Concurrent Operations**: All major operations run concurrently using async/await
- **Efficient Caching**: Discovery results are cached to reduce redundant scanning
- **Batch Processing**: Data collection uses efficient batching for high throughput
- **Resource Optimization**: Configurable resource usage limits and monitoring

### Performance Targets
- **Scan Duration**: <60 seconds for typical environments
- **Health Check Latency**: <100ms per check
- **Data Collection Throughput**: >1000 data points/second
- **Memory Usage**: <100MB baseline + <10MB per 1000 systems
- **CPU Usage**: <5% continuous monitoring overhead

## Security

Layer 1 implements security best practices:

- **Minimal System Impact**: Non-intrusive monitoring with configurable resource limits
- **Secure Communication**: Encrypted inter-layer communication when enabled
- **Access Controls**: Configurable access controls for discovery operations
- **Audit Logging**: Comprehensive audit logging for all discovery activities
- **Data Protection**: Data sanitization and validation for collected information

## Integration

Layer 1 integrates with other Project Chimera layers:

### Layer 2 (Planning)
- Provides environmental context for resource allocation decisions
- Supplies system capability information for task planning
- Delivers performance data for optimization planning

### Layer 3 (Validation)
- Supplies system state for compliance validation
- Provides health data for system integrity checks
- Delivers security metrics for compliance monitoring

### Layer 4 (Execution)
- Provides execution environment information
- Supplies resource availability data
- Delivers performance monitoring for execution optimization

## Deployment

Layer 1 supports multiple deployment scenarios:

### Docker Deployment
```dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/layer1-discovery /usr/local/bin/
CMD ["layer1-discovery"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: layer1-discovery
spec:
  replicas: 3
  selector:
    matchLabels:
      app: layer1-discovery
  template:
    metadata:
      labels:
        app: layer1-discovery
    spec:
      containers:
      - name: discovery
        image: project-chimera/layer1-discovery:latest
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        env:
        - name: RUST_LOG
          value: "info"
```

## Monitoring and Alerting

Layer 1 integrates with Prometheus and Grafana for monitoring:

### Key Metrics to Monitor
- System discovery rate and success
- Health check success rates
- Data collection throughput and quality
- Alert generation rates and acknowledgment times
- Resource usage (CPU, memory, network)

### Important Alerts
- High scan error rates
- System health degradation
- Data collection failures
- Resource constraint warnings
- Integration communication failures

## Troubleshooting

### Common Issues

1. **High Scan Duration**
   - Reduce scan depth or increase timeouts
   - Check network connectivity
   - Verify system resource availability

2. **Health Check Failures**
   - Verify system accessibility
   - Check authentication credentials
   - Review firewall and security settings

3. **Data Collection Errors**
   - Verify data source availability
   - Check API credentials and permissions
   - Review data format compatibility

4. **Integration Issues**
   - Verify layer connectivity
   - Check message queue status
   - Review communication protocols

### Debug Commands

```bash
# Enable debug logging
export RUST_LOG=debug

# Check metrics endpoint
curl http://localhost:9090/metrics

# View system state
curl http://localhost:8080/state

# Trigger manual scan
curl -X POST http://localhost:8080/scan
```

## Contributing

When contributing to Layer 1:

1. **Add Tests**: All new features must include comprehensive unit and integration tests
2. **Update Documentation**: Keep README and API documentation current
3. **Performance Testing**: Validate performance impact of changes
4. **Security Review**: Ensure changes don't introduce security vulnerabilities
5. **Code Review**: Follow Rust best practices and project coding standards

## License

Layer 1 is part of Project Chimera and follows the same licensing terms.

## Support

For support and questions about Layer 1:

- Check the troubleshooting section above
- Review the implementation plan in `LAYER1_IMPLEMENTATION_PLAN.md`
- Consult the API documentation in the source code
- Open an issue in the project repository

---

**Layer 1 (Discovery)** provides the essential environmental awareness foundation that enables Project Chimera's intelligent planning, validation, and execution capabilities. Its comprehensive system discovery, monitoring, and data collection capabilities ensure that higher layers have accurate, real-time information about the operational environment.