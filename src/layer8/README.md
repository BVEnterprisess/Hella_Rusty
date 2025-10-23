# Layer 8 (Resource Management)

Layer 8 is the resource management layer of Project Chimera's 8-layer autonomous AI system. It provides comprehensive GPU allocation, cost optimization, and compute resource scheduling across all layers.

## Overview

Layer 8 acts as the central resource coordinator for the entire autonomous AI system, managing:

- **GPU Resource Management**: Allocation and optimization of GPU resources for ML workloads
- **Cost Optimization**: Monitoring and optimizing compute costs across all layers
- **Resource Scheduling**: Intelligent scheduling of compute resources based on demand
- **Capacity Planning**: Predictive resource allocation based on historical usage patterns
- **Integration Management**: Coordinating resource requests from Layers 4, 5, and 7

## Architecture

### Core Components

#### ResourceAllocator
Main resource allocation engine that handles:
- Resource request validation and prioritization
- GPU, CPU, memory, and storage allocation
- Cost calculation and budget enforcement
- Allocation lifecycle management

#### CostOptimizer
Cost monitoring and optimization system that provides:
- Real-time cost tracking across all layers
- Budget utilization monitoring
- Cost optimization recommendations
- Automated cost alerting

#### GpuManager
GPU-specific resource management including:
- GPU discovery and status monitoring
- GPU allocation and deallocation
- Performance metrics collection
- Temperature and utilization monitoring

#### CapacityPlanner
Predictive capacity planning system featuring:
- Historical usage analysis
- Forecasting models for resource prediction
- Scaling recommendations
- Risk assessment and mitigation

#### IntegrationManager
Cross-layer communication and integration handling:
- HTTP-based API communication with other layers
- Resource request/response handling
- Health check coordination
- Integration status monitoring

#### ResourceMetrics
Comprehensive metrics collection and export:
- Prometheus metrics for monitoring
- Performance metrics tracking
- Cost metrics aggregation
- Resource utilization monitoring

## Integration Points

### Layer 4 (Execution) Integration
- Receives agent resource requirements
- Provides GPU allocation for WASM runtime
- Monitors execution performance metrics
- Handles resource scaling requests

### Layer 5 (Refinement) Integration
- Manages ML training resource allocation
- Provides GPU resources for optimization algorithms
- Monitors training performance and costs
- Handles model deployment resources

### Layer 7 (Evolution) Integration
- **Critical Integration**: Provides GPU resources for genetic algorithms
- Manages population evolution compute requirements
- Handles genome deployment resource allocation
- Monitors evolution pipeline performance

## Configuration

### Environment Variables

```bash
# Service Configuration
LAYER8_PORT=8008
RUST_LOG=info

# Database Configuration
POSTGRES_URL=postgres://chimera:password@localhost:5432/chimera
REDIS_URL=redis://localhost:6379/1

# Resource Limits
MAX_GPUS_PER_ALLOCATION=4
MAX_MEMORY_PER_GPU_GB=24
MAX_ALLOCATION_TIME_MINUTES=480

# Cost Configuration
COST_PER_GPU_HOUR=0.5
BUDGET_LAYER4=100.0
BUDGET_LAYER5=200.0
BUDGET_LAYER7=150.0

# Integration Endpoints
LAYER4_HEALTH_ENDPOINT=http://layer4-execution:8000/health
LAYER5_HEALTH_ENDPOINT=http://layer5-refinement:8002/health
LAYER7_HEALTH_ENDPOINT=http://layer7-evolution:8003/health
```

### Configuration File

Layer 8 uses a YAML configuration file located at `configs/layer8-config.yaml`:

```yaml
layer8:
  port: 8008
  gpu_limits:
    max_gpus_per_allocation: 4
    max_memory_per_gpu_gb: 24
    max_allocation_time_minutes: 480
    utilization_threshold: 0.8
  cost_settings:
    cost_per_gpu_hour: 0.5
    budget_limits:
      layer4: 100.0
      layer5: 200.0
      layer7: 150.0
  integration:
    layer4_endpoints:
      health_endpoint: "http://layer4-execution:8000/health"
      resource_endpoint: "http://layer4-execution:8000/api/resources"
      metrics_endpoint: "http://layer4-execution:8000/metrics"
    layer5_endpoints:
      health_endpoint: "http://layer5-refinement:8002/health"
      resource_endpoint: "http://layer5-refinement:8002/api/resources"
      metrics_endpoint: "http://layer5-refinement:8002/metrics"
    layer7_endpoints:
      health_endpoint: "http://layer7-evolution:8003/health"
      resource_endpoint: "http://layer7-evolution:8003/api/resources"
      metrics_endpoint: "http://layer7-evolution:8003/metrics"
  monitoring:
    metrics_interval_seconds: 60
    alert_interval_seconds: 30
    retention_days: 30
```

## API Endpoints

### Health Endpoints
- `GET /health` - Service health check
- `GET /health/ready` - Kubernetes readiness probe
- `GET /health/live` - Kubernetes liveness probe

### Resource Management Endpoints
- `POST /api/resources/allocate` - Allocate resources
- `DELETE /api/resources/{allocation_id}` - Release allocation
- `GET /api/resources/{allocation_id}` - Get allocation status
- `GET /api/resources/active` - List active allocations

### Cost Management Endpoints
- `GET /api/costs/metrics` - Get cost metrics
- `GET /api/costs/optimization` - Get optimization recommendations
- `GET /api/costs/budget` - Get budget utilization

### GPU Management Endpoints
- `GET /api/gpu/status` - Get GPU status
- `POST /api/gpu/allocate` - Allocate GPU resources
- `DELETE /api/gpu/{allocation_id}` - Release GPU allocation

### Integration Endpoints
- `GET /api/integration/status` - Get integration status
- `POST /api/integration/test` - Test layer connectivity
- `GET /api/integration/layer4` - Layer 4 integration status
- `GET /api/integration/layer5` - Layer 5 integration status
- `GET /api/integration/layer7` - Layer 7 integration status

### Metrics Endpoints
- `GET /metrics` - Prometheus metrics export
- `GET /api/metrics/summary` - Metrics summary
- `GET /api/metrics/layer/{layer}` - Layer-specific metrics

## Deployment

### Docker Build
```bash
# Build Layer 8 Docker image
docker build -f src/layer8/Dockerfile -t project-chimera/layer8:latest .

# Run locally
docker run -p 8008:8008 project-chimera/layer8:latest
```

### Kubernetes Deployment
```bash
# Deploy to Kubernetes
kubectl apply -f src/layer8/k8s-deployment.yaml

# Check deployment status
kubectl get pods -l layer=resource-management

# View logs
kubectl logs -l layer=resource-management -f
```

### Service Discovery
Layer 8 is discoverable via Kubernetes DNS:
- `layer8-resource-management.default.svc.cluster.local:80`
- Internal service mesh communication via Istio

## Monitoring

### Prometheus Metrics

Layer 8 exports comprehensive metrics for monitoring:

#### Resource Metrics
- `layer8_allocations_total` - Total resource allocations
- `layer8_allocations_active` - Currently active allocations
- `layer8_allocation_duration_seconds` - Allocation duration histogram
- `layer8_gpu_utilization_ratio` - GPU utilization percentage
- `layer8_resource_efficiency_ratio` - Resource allocation efficiency

#### Cost Metrics
- `layer8_cost_total` - Total cost counter
- `layer8_cost_by_layer_total` - Cost by requesting layer
- `layer8_cost_savings_ratio` - Cost optimization savings
- `layer8_budget_utilization_ratio` - Budget utilization percentage

#### Performance Metrics
- `layer8_allocation_latency_seconds` - Allocation request latency
- `layer8_integration_errors_total` - Integration error counter
- `layer8_resource_efficiency_ratio` - Resource utilization efficiency

### Grafana Dashboards

Layer 8 provides specialized Grafana dashboards:

1. **Resource Management Dashboard**
   - GPU utilization and allocation status
   - Resource allocation trends
   - Cost analysis and budget utilization

2. **Cost Optimization Dashboard**
   - Cost trends by layer
   - Budget utilization alerts
   - Optimization recommendations

3. **Integration Dashboard**
   - Cross-layer communication status
   - Integration health metrics
   - Service mesh performance

### Alerting Rules

#### Critical Alerts
- High allocation failure rate
- GPU resource exhaustion
- Budget overrun
- Integration failures

#### Warning Alerts
- High allocation latency
- Low resource efficiency
- Cost spikes
- Performance degradation

## Testing

### Unit Tests
```bash
# Run Layer 8 unit tests
cd src/layer8 && cargo test

# Run with verbose output
cargo test --package layer8-resource-management -- --nocapture
```

### Integration Tests
```bash
# Run integration tests
cargo test --package layer8-resource-management --test integration

# Test cross-layer integration
cargo test --workspace --test integration layer8
```

### Performance Tests
```bash
# Run performance benchmarks
cargo test --package layer8-resource-management --test performance

# Load testing with k6
cd tests/load && k6 run layer8-load-test.js
```

## Development

### Building
```bash
# Build Layer 8
cd src/layer8 && cargo build --release

# Build entire workspace
cargo build --workspace
```

### Running Locally
```bash
# Run Layer 8 service
cd src/layer8 && cargo run

# With custom configuration
LAYER8_PORT=8008 RUST_LOG=debug cargo run
```

### Testing Integration
```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Run integration tests
cargo test --workspace --test integration

# Check integration status
curl http://localhost:8008/api/integration/status
```

## Security Considerations

### Access Controls
- Kubernetes RBAC for API access
- Network policies restricting communication
- Service mesh encryption for inter-layer traffic

### Data Protection
- Encrypted database connections
- Secure credential management via secrets
- Audit logging for all resource operations

### Compliance
- Resource usage tracking for cost compliance
- Budget enforcement and alerting
- Access logging for audit trails

## Performance Characteristics

### Benchmarks
- **Allocation Latency**: <100ms for resource allocation
- **GPU Discovery**: <5 seconds for GPU status updates
- **Cost Calculation**: <50ms for cost analysis
- **Integration Response**: <200ms for cross-layer communication

### Scalability
- **Max Allocations**: 1000+ concurrent resource allocations
- **GPU Management**: Support for 16+ GPUs per node
- **Cost Tracking**: Real-time cost monitoring for 1000+ allocations
- **Integration**: Support for 10+ layer integrations

## Troubleshooting

### Common Issues

#### High Allocation Latency
```bash
# Check GPU discovery performance
kubectl logs -l layer=resource-management | grep "gpu-discovery"

# Monitor resource contention
kubectl top pods -l layer=resource-management

# Check database performance
kubectl logs -l layer=resource-management | grep "database"
```

#### Integration Failures
```bash
# Check layer connectivity
curl http://layer8-service:8008/api/integration/status

# Test individual layer connections
curl http://layer4-service:8000/health
curl http://layer5-service:8002/health
curl http://layer7-service:8003/health

# Check service mesh logs
kubectl logs -l app=istio-proxy -c istio-proxy
```

#### Cost Tracking Issues
```bash
# Verify cost calculation
curl http://layer8-service:8008/api/costs/metrics

# Check budget configuration
kubectl get configmap layer8-config -o yaml

# Review cost optimization recommendations
curl http://layer8-service:8008/api/costs/optimization
```

### Health Check Procedures

#### Automated Health Checks
```bash
# Kubernetes health checks
curl http://layer8-service:8008/health
curl http://layer8-service:8008/health/ready
curl http://layer8-service:8008/health/live

# Service integration checks
curl http://layer8-service:8008/api/integration/status
```

#### Manual Verification
```bash
# Check resource allocation status
curl http://layer8-service:8008/api/resources/active

# Verify GPU management
curl http://layer8-service:8008/api/gpu/status

# Check cost metrics
curl http://layer8-service:8008/api/costs/metrics
```

## Future Enhancements

### Planned Features
1. **Advanced GPU Scheduling**: Multi-GPU workload optimization
2. **Spot Instance Integration**: Cost optimization with spot instances
3. **Resource Prediction**: ML-based resource forecasting
4. **Multi-Cloud Support**: AWS, GCP, Azure resource management
5. **Advanced Cost Analytics**: Detailed cost breakdown and trends

### Integration Roadmap
1. **Layer 1 Integration**: Environmental resource awareness
2. **Layer 2 Integration**: Planning-based resource allocation
3. **Layer 3 Integration**: Validation of resource allocations
4. **Layer 6 Integration**: Advanced evolution resource management

## Support

### Development Team
- **Layer 8 Development**: layer8-dev@project-chimera.com
- **Resource Management**: resource-mgmt@project-chimera.com
- **Cost Optimization**: cost-opt@project-chimera.com

### Documentation
- **API Documentation**: Available via OpenAPI/Swagger
- **Configuration Guide**: See `configs/layer8-config.yaml`
- **Deployment Guide**: See `k8s-deployment.yaml`
- **Troubleshooting**: See operations runbook

---

**Layer 8 Status**: ✅ **FULLY IMPLEMENTED** - Production Ready
**Integration Status**: ✅ **ACTIVE** - Integrated with Layers 4, 5, 7
**Next Priority**: 🔄 **Layer 2 (Planning)** - Required for autonomous operation
**Last Updated**: 2025-10-23