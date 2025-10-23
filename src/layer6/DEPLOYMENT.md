# Layer 6 (Evolution) Deployment Guide

## Overview

Layer 6 provides advanced evolutionary algorithms and meta-learning capabilities for Project Chimera. This guide covers deployment procedures for both Docker Compose and Kubernetes environments.

## Architecture

Layer 6 consists of:
- **Meta-Learning Framework**: Algorithm portfolio management and performance tracking
- **Population Dynamics**: Multi-population structures with migration topologies
- **Adaptive Evolution**: Self-adaptive parameter control and strategy switching
- **Hyper-Heuristics**: High-level heuristic selection and generation
- **Fitness Landscape Analyzer**: Modality detection and algorithm recommendations
- **Integration Hub**: Inter-layer communication and evolution data routing

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
# Build and start Layer 6 service
docker-compose up evolution -d

# View logs
docker-compose logs -f evolution

# Check health
curl http://localhost:8082/health

# View metrics
curl http://localhost:8082/metrics
```

### Configuration

Key environment variables:

```bash
# Evolution Parameters
POPULATION_SIZE=100                 # Size of evolution population
MAX_GENERATIONS=1000               # Maximum generations per run
MUTATION_RATE=0.1                  # Mutation probability (0.0-1.0)
CROSSOVER_RATE=0.8                 # Crossover probability (0.0-1.0)
ALGORITHM_SELECTION_INTERVAL=50    # Generations between algorithm switches

# Feature Flags
ENABLE_META_LEARNING=true          # Enable meta-learning algorithm selection
ENABLE_POPULATION_DYNAMICS=true    # Enable multi-population dynamics
ENABLE_FITNESS_LANDSCAPE=true      # Enable landscape analysis

# Integration
REDIS_URL=redis://:password@redis:6379
POSTGRES_URL=postgresql://user:pass@postgres:5432/chimera
PROMETHEUS_GATEWAY=http://prometheus:9090

# Logging
RUST_LOG=info                      # Log level (trace, debug, info, warn, error)
```

## Kubernetes Deployment

### Prerequisites

1. **Kubernetes cluster** with adequate CPU/memory resources
2. **Secrets** configured for database credentials
3. **Persistent volumes** for population and algorithm data
4. **Network policies** allowing inter-layer communication

### Deployment

```bash
# Apply Layer 6 manifests
kubectl apply -f k8s/evolution-deployment.yaml

# Check deployment status
kubectl get pods -l app=chimera-evolution
kubectl get services -l app=chimera-evolution

# View logs
kubectl logs -l app=chimera-evolution -f

# Port forward for local access
kubectl port-forward svc/chimera-evolution-service 8082:80
```

### Scaling

```bash
# Scale evolution service
kubectl scale deployment chimera-evolution --replicas=3

# Check resource usage
kubectl top pods -l app=chimera-evolution
```

## Monitoring and Alerting

### Prometheus Metrics

Layer 6 exposes the following metrics:

- `evolution_generations_total` - Total generations processed
- `evolution_generation_duration_seconds` - Histogram of generation times
- `evolution_population_diversity` - Current population diversity ratio
- `evolution_best_fitness` - Best fitness score in current population
- `evolution_algorithm_success_total` - Successful algorithm executions
- `evolution_algorithm_failures_total` - Failed algorithm executions
- `evolution_meta_learning_accuracy` - Meta-learning prediction accuracy
- `evolution_migration_events_total` - Population migration events

### Grafana Dashboard

Access the Layer 6 monitoring dashboard at:
- **URL**: `http://localhost:3000/d/evolution-monitoring`
- **Metrics**: Real-time evolution performance and algorithm statistics

### Alerting Rules

Critical alerts configured:
- **EvolutionServiceDown**: Service unavailable
- **EvolutionAlgorithmFailed**: Algorithm execution failures
- **LowPopulationDiversity**: Population convergence issues
- **MetaLearningDegraded**: Meta-learning accuracy problems
- **HighEvolutionLatency**: Performance degradation

## Health Checks

### Endpoints

- **Health Check**: `GET /health` - Basic service health
- **Readiness Check**: `GET /ready` - Service readiness for traffic
- **Metrics**: `GET /metrics` - Prometheus metrics

### Manual Verification

```bash
# Test evolution functionality
curl http://localhost:8082/evolution/status

# Check population dynamics
curl http://localhost:8082/population/status

# Verify meta-learning
curl http://localhost:8082/meta-learning/status

# Test algorithm performance
curl http://localhost:8082/algorithms/performance
```

## Troubleshooting

### Common Issues

1. **Service won't start**
   ```bash
   # Check logs for errors
   docker-compose logs evolution

   # Verify dependencies
   docker-compose ps
   ```

2. **Evolution algorithms failing**
   ```bash
   # Check population data integrity
   kubectl exec -it deployment/chimera-evolution -- ls -la /app/populations

   # Verify algorithm configurations
   kubectl exec -it deployment/chimera-evolution -- cat /app/algorithms/config.json
   ```

3. **High resource usage**
   ```bash
   # Monitor resource consumption
   kubectl top pods -l app=chimera-evolution

   # Adjust resource limits in deployment
   kubectl edit deployment chimera-evolution
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
4. **Audit Logging**: Enable comprehensive audit trails for algorithm decisions

## Performance Tuning

### Resource Allocation

```yaml
resources:
  requests:
    memory: "2Gi"
    cpu: "1000m"
  limits:
    memory: "4Gi"
    cpu: "2000m"
```

### Algorithm Optimization

- Adjust population size based on problem complexity
- Tune mutation/crossover rates for specific domains
- Configure algorithm selection intervals appropriately
- Enable/disable advanced features based on requirements

## Backup and Recovery

### Data Backup

```bash
# Backup evolution data
kubectl exec deployment/chimera-evolution -- tar czf /tmp/evolution-backup.tar.gz /app/populations /app/algorithms

# Restore data
kubectl cp evolution-backup.tar.gz chimera-evolution-pod:/tmp/
kubectl exec deployment/chimera-evolution -- tar xzf /tmp/evolution-backup.tar.gz -C /
```

### Service Recovery

```bash
# Restart service
kubectl rollout restart deployment/chimera-evolution

# Rollback if needed
kubectl rollout undo deployment/chimera-evolution
```

## Integration Testing

### End-to-End Tests

```bash
# Run integration tests
docker-compose exec evolution cargo test --test integration

# Test with Layer 7 (basic evolution)
curl http://localhost:8082/integration/layer7/status

# Test meta-learning integration
curl http://localhost:8082/meta-learning/integration/status
```

## Advanced Configuration

### Multi-Population Setup

```yaml
env:
- name: POPULATION_COUNT
  value: "3"
- name: MIGRATION_TOPOLOGY
  value: "ring"  # ring, star, complete
- name: MIGRATION_INTERVAL
  value: "10"
```

### Algorithm Portfolio

```yaml
env:
- name: ENABLE_GENETIC_ALGORITHM
  value: "true"
- name: ENABLE_DIFFERENTIAL_EVOLUTION
  value: "true"
- name: ENABLE_PARTICLE_SWARM
  value: "true"
- name: ENABLE_CMA_ES
  value: "true"
```

## Support

For issues or questions:
1. Check logs: `kubectl logs -l app=chimera-evolution`
2. Review metrics: Grafana dashboard
3. Check alerts: Prometheus Alertmanager
4. Escalate to: Layer 6 team or DevOps on-call

## Version History

- **v1.0.0**: Initial implementation with basic genetic algorithms
- **v1.1.0**: Enhanced meta-learning and population dynamics
- **v1.2.0**: Multi-population support and fitness landscape analysis
- **v1.3.0**: Hyper-heuristics and advanced algorithm selection