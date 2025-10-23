# Layer 1 (Discovery) Deployment Guide

## Overview

Layer 1 provides environmental awareness and system monitoring capabilities for Project Chimera. This guide covers deployment procedures for both Docker Compose and Kubernetes environments.

## Architecture

Layer 1 consists of:
- **Environmental Scanner**: Discovers system resources and configurations
- **System Monitor**: Performs health checks and resource monitoring
- **Data Collector**: Gathers metrics and logs from various sources
- **Integration Hub**: Coordinates with other layers (2, 3, 4)

## Docker Compose Deployment

### Prerequisites

1. **Docker and Docker Compose** installed
2. **Environment variables** configured (see `.env.example`)
3. **Required services** running:
   - Redis (message queuing)
   - PostgreSQL (metadata storage)
   - Prometheus (metrics collection)

### Quick Start

```bash
# Build and start Layer 1 service
docker-compose up discovery -d

# View logs
docker-compose logs -f discovery

# Check health
curl http://localhost:8081/health

# View metrics
curl http://localhost:8081/metrics
```

### Configuration

Key environment variables:

```bash
# Service Configuration
SCAN_INTERVAL=60                    # Seconds between discovery scans
DATA_RETENTION_HOURS=24            # Hours to retain collected data

# Feature Flags
ENABLE_SYSTEM_PROBES=true          # Enable system resource monitoring
ENABLE_NETWORK_PROBES=true         # Enable network connectivity checks
ENABLE_CONTAINER_PROBES=true       # Enable container status monitoring

# Integration
REDIS_URL=redis://:password@redis:6379
POSTGRES_URL=postgresql://user:pass@postgres:5432/chimera
PROMETHEUS_GATEWAY=http://prometheus:9090

# Logging
RUST_LOG=info                      # Log level (trace, debug, info, warn, error)
```

## Kubernetes Deployment

### Prerequisites

1. **Kubernetes cluster** with metrics-server
2. **Secrets** configured for database credentials
3. **Persistent volumes** for data storage
4. **Network policies** allowing inter-layer communication

### Deployment

```bash
# Apply Layer 1 manifests
kubectl apply -f k8s/discovery-deployment.yaml

# Check deployment status
kubectl get pods -l app=chimera-discovery
kubectl get services -l app=chimera-discovery

# View logs
kubectl logs -l app=chimera-discovery -f

# Port forward for local access
kubectl port-forward svc/chimera-discovery-service 8081:80
```

### Scaling

```bash
# Scale discovery service
kubectl scale deployment chimera-discovery --replicas=3

# Check resource usage
kubectl top pods -l app=chimera-discovery
```

## Monitoring and Alerting

### Prometheus Metrics

Layer 1 exposes the following metrics:

- `discovery_scans_total` - Total number of discovery scans performed
- `discovery_scan_duration_seconds` - Histogram of scan durations
- `discovery_resources_total` - Total resources discovered
- `system_health_checks_total` - Health check executions
- `data_collection_events_total` - Data collection events
- `discovery_scan_failures_total` - Failed scan attempts

### Grafana Dashboard

Access the Layer 1 monitoring dashboard at:
- **URL**: `http://localhost:3000/d/discovery-monitoring`
- **Metrics**: Real-time discovery performance and system health

### Alerting Rules

Critical alerts configured:
- **DiscoveryServiceDown**: Service unavailable
- **DiscoveryScanFailed**: Scan failures detected
- **SystemProbeFailed**: System monitoring issues
- **DataCollectionStalled**: Data collection stopped

## Health Checks

### Endpoints

- **Health Check**: `GET /health` - Basic service health
- **Readiness Check**: `GET /ready` - Service readiness for traffic
- **Metrics**: `GET /metrics` - Prometheus metrics

### Manual Verification

```bash
# Test discovery functionality
curl http://localhost:8081/discovery/status

# Check system monitoring
curl http://localhost:8081/system/status

# Verify data collection
curl http://localhost:8081/data/status
```

## Troubleshooting

### Common Issues

1. **Service won't start**
   ```bash
   # Check logs for errors
   docker-compose logs discovery

   # Verify dependencies
   docker-compose ps
   ```

2. **Discovery scans failing**
   ```bash
   # Check network connectivity
   kubectl exec -it deployment/chimera-discovery -- ping redis-service

   # Verify permissions
   kubectl auth can-i get pods --as=system:serviceaccount:default:default
   ```

3. **High resource usage**
   ```bash
   # Monitor resource consumption
   kubectl top pods -l app=chimera-discovery

   # Adjust resource limits in deployment
   kubectl edit deployment chimera-discovery
   ```

### Debug Mode

Enable debug logging:

```yaml
env:
- name: RUST_LOG
  value: "debug"
```

## Security Considerations

1. **Network Policies**: Restrict traffic to necessary services only
2. **RBAC**: Use least-privilege service accounts
3. **Secrets Management**: Store credentials in Kubernetes secrets
4. **Audit Logging**: Enable comprehensive audit trails

## Performance Tuning

### Resource Allocation

```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### Scan Optimization

- Adjust `SCAN_INTERVAL` based on environment size
- Enable/disable probe types based on requirements
- Configure data retention policies appropriately

## Backup and Recovery

### Data Backup

```bash
# Backup discovery data
kubectl exec deployment/chimera-discovery -- tar czf /tmp/discovery-backup.tar.gz /app/data

# Restore data
kubectl cp discovery-backup.tar.gz chimera-discovery-pod:/tmp/
kubectl exec deployment/chimera-discovery -- tar xzf /tmp/discovery-backup.tar.gz -C /
```

### Service Recovery

```bash
# Restart service
kubectl rollout restart deployment/chimera-discovery

# Rollback if needed
kubectl rollout undo deployment/chimera-discovery
```

## Integration Testing

### End-to-End Tests

```bash
# Run integration tests
docker-compose exec discovery cargo test --test integration

# Test with other layers
curl http://localhost:8081/integration/layer2/status
curl http://localhost:8081/integration/layer3/status
```

## Support

For issues or questions:
1. Check logs: `kubectl logs -l app=chimera-discovery`
2. Review metrics: Grafana dashboard
3. Check alerts: Prometheus Alertmanager
4. Escalate to: Layer 1 team or DevOps on-call

## Version History

- **v1.0.0**: Initial implementation with basic discovery
- **v1.1.0**: Enhanced system monitoring and alerting
- **v1.2.0**: Multi-layer integration and performance optimizations