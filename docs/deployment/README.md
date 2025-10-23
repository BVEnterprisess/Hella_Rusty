# 🚀 Project Chimera Deployment Guide

## Overview

Project Chimera is a sophisticated multi-agent AI orchestration platform with GPU optimization, self-evolving capabilities, and enterprise-grade DevOps infrastructure. This guide covers deployment, monitoring, and operational procedures.

## 🏗️ Architecture

### 8-Layer Autonomous AI System
**Current Implementation: 3/8 Layers Complete (37.5%)**

#### ✅ Implemented Layers
- **Layer 4 (Execution)**: WASM-based agent runtime with scheduling and metrics
- **Layer 5 (Refinement)**: ML optimization engine with pattern recognition and A/B testing
- **Layer 7 (Evolution)**: Genetic algorithm engine with genome management and population evolution

#### ❌ Pending Layers
- **Layer 1 (Discovery)**: Environmental awareness and data collection
- **Layer 2 (Planning)**: Strategic planning and task decomposition
- **Layer 3 (Validation)**: System integrity and safety validation
- **Layer 6 (Evolution)**: Advanced evolutionary algorithms
- **Layer 8 (Resource Management)**: GPU/compute resource allocation (Next Priority)

### Infrastructure Stack
- **Containerization**: Multi-stage Docker builds with GPU support
- **Orchestration**: Kubernetes with multi-layer service deployment
- **Monitoring**: Prometheus, Grafana, Jaeger, Alertmanager with layer-specific metrics
- **Logging**: Fluent Bit with centralized aggregation and structured logging
- **Security**: Trivy scanning, encrypted backups, RBAC, network policies
- **CI/CD**: GitHub Actions with multi-layer builds and comprehensive testing

## 🚀 Quick Start

### Prerequisites
- Docker and Docker Compose
- NVIDIA GPU with drivers (for GPU acceleration)
- 16GB+ RAM recommended
- 100GB+ storage for models and data

### ✅ Latest Implementation Status
**3/8 Layers Complete (37.5%)**
- **Layer 4 (Execution)**: ✅ FULLY IMPLEMENTED - WASM runtime, scheduling, metrics
- **Layer 5 (Refinement)**: ✅ FULLY IMPLEMENTED - ML optimization, pattern recognition
- **Layer 7 (Evolution)**: ✅ FULLY IMPLEMENTED - Genetic algorithms, genome management

**Build Status**: ✅ **SUCCESS** (2025-10-23 00:30:00 UTC)
- **Total Build Time**: ~135 seconds (all layers)
- **Profile**: Release (optimized)
- **Status**: All implemented layers compiled successfully

### Local Development Setup

```bash
# Clone and navigate to project
git clone <repository-url>
cd Project-Chimera

# Start all services
docker-compose up -d

# Verify services are running
docker-compose ps

# Access monitoring dashboard
open http://localhost:3000  # Grafana
open http://localhost:9090  # Prometheus
```

### Multi-Layer Services Overview

| Layer | Service | Port | Purpose | Status |
|-------|---------|------|---------|--------|
| **Layer 4** | Agent Service | 8000 | WASM agent execution runtime | ✅ Active |
| **Layer 4** | Scheduler | 8001 | Task scheduling and orchestration | ✅ Active |
| **Layer 5** | Refinement Engine | 8002 | ML optimization and pattern recognition | ✅ Active |
| **Layer 7** | Evolution Engine | 8003 | Genetic algorithms and genome management | ✅ Active |
| **Shared** | Redis | 6379 | Caching, queues, and message broker | ✅ Active |
| **Shared** | PostgreSQL | 5432 | Persistent data storage (genomes, metrics) | ✅ Active |
| **Shared** | MinIO | 9000 | Artifact storage (models, backups) | ✅ Active |
| **Monitoring** | Prometheus | 9090 | Metrics collection (all layers) | ✅ Active |
| **Monitoring** | Grafana | 3000 | Visualization dashboard | ✅ Active |
| **Monitoring** | Jaeger | 16686 | Distributed tracing | ✅ Active |
| **Monitoring** | Alertmanager | 9093 | Alert management | ✅ Active |

## 🔧 Configuration

### Environment Variables

Create a `.env` file with your configuration:

```bash
# Database
POSTGRES_DB=chimera
POSTGRES_USER=chimera_user
POSTGRES_PASSWORD=your_secure_password

# Redis
REDIS_PASSWORD=your_redis_password

# Monitoring
GRAFANA_ADMIN_PASSWORD=admin_password

# External APIs (add your keys)
OPENAI_API_KEY=your_openai_key
HUGGINGFACE_TOKEN=your_hf_token
```

### GPU Configuration

For GPU acceleration, ensure:

```dockerfile
# In docker-compose.yml
services:
  agent-service:
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

## 🛠️ Multi-Layer Development Workflow

### 1. Multi-Layer Build Process

```bash
# Build all implemented layers with comprehensive metrics
./build_layer4.sh

# Or build individual layers
cd src/layer4 && cargo build --release  # Layer 4 (Execution)
cd src/layer5 && cargo build --release  # Layer 5 (Refinement)
cd src/layer7 && cargo build --release  # Layer 7 (Evolution)

# Run tests for all layers
cargo test --workspace

# Check code quality
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. CI/CD Pipeline

The GitHub Actions workflows provide automated multi-layer builds:

```bash
# Main CI/CD pipeline with comprehensive metrics
.github/workflows/ci-cd.yml

# Layer-specific pipelines
.github/workflows/layer5-ci.yml
.github/workflows/layer7-ci.yml
```

**Pipeline Features:**
- Multi-layer parallel builds with timing metrics
- Comprehensive test execution across all layers
- Security scanning with Trivy
- Sandbox testing with Playwright
- Docker builds with multi-platform support
- Kubernetes deployment with canary releases

### 3. Testing Strategy

```bash
# Unit tests (all layers)
cargo test --workspace

# Integration tests (cross-layer)
cargo test --test integration

# Sandbox tests (safe deployment validation)
cd tests/sandbox && npx playwright test

# Load testing
# Use k6 scripts in tests/load/
```

### 4. Deployment Process

```bash
# Deploy to staging (automated via CI/CD)
# kubectl apply -f k8s/staging/

# Deploy to production (gated deployment)
# kubectl apply -f k8s/production/

# Check deployment status
kubectl get pods -l layer=execution  # Layer 4 agents
kubectl get pods -l layer=refinement # Layer 5 optimization
kubectl get pods -l layer=evolution  # Layer 7 evolution
```

## 📊 Multi-Layer Monitoring & Observability

### Layer-Specific Metrics

#### Layer 4 (Execution) Metrics
- **Agent Response Time**: p95 < 500ms
- **Task Success Rate**: >99.5%
- **Active Agents**: Count and distribution
- **WASM Runtime Performance**: Memory and CPU usage
- **Task Queue Depth**: Redis queue monitoring

#### Layer 5 (Refinement) Metrics
- **Optimization Accuracy**: >95% improvement rate
- **Pattern Recognition**: Detection rate and confidence
- **A/B Test Results**: Statistical significance tracking
- **ML Model Performance**: Training and inference metrics
- **Feedback Loop Latency**: <100ms processing time

#### Layer 7 (Evolution) Metrics
- **Evolution Convergence**: <100 generations for 5% improvement
- **Genome Deployment Success**: >99% successful hot-swaps
- **Population Diversity**: Genetic algorithm health
- **Fitness Evaluation**: Accuracy and performance
- **Resource Utilization**: GPU allocation efficiency

### System Health Metrics
- **Cross-Layer Integration**: Data flow success rates
- **Resource Usage**: CPU, memory, GPU, storage across all layers
- **Network Performance**: Inter-layer communication latency
- **Database Performance**: PostgreSQL and Redis metrics

### Accessing Dashboards

- **Grafana**: http://localhost:3000 (admin/admin)
  - **Layer 4 Dashboard**: Agent performance and execution metrics
  - **Layer 5 Dashboard**: Optimization and ML performance
  - **Layer 7 Dashboard**: Evolution and genetic algorithm metrics
  - **System Overview**: Cross-layer integration and health
- **Prometheus**: http://localhost:9090 (query layer-specific metrics)
- **Jaeger**: http://localhost:16686 (distributed tracing across layers)

### Alert Configuration

Multi-layer alerts configured in `configs/alertmanager.yml`:

#### 🚨 Critical Alerts
- Layer 4: Agent service down, WASM runtime failures
- Layer 5: Optimization engine failures, pattern recognition errors
- Layer 7: Evolution pipeline failures, genome corruption
- System: Database connectivity issues, resource exhaustion

#### ⚠️ Warning Alerts
- Performance degradation across any layer
- Resource usage approaching thresholds
- Integration failures between layers
- Model accuracy dropping below thresholds

#### ℹ️ Info Alerts
- Deployment notifications for each layer
- Scheduled maintenance windows
- Performance optimization recommendations

## 🤖 AI/ML Operations

### Model Management

```bash
# Train new LoRA adapter
python tools/scripts/train_qlora.py \
  --model-base mistralai/Mistral-7B-Instruct-v0.1 \
  --dataset your-training-data \
  --output-dir adapters/new-capability

# Optimize performance
python tools/scripts/optimize_performance.py --optimize --report
```

### GPU Optimization

```bash
# Check GPU status
nvidia-smi

# Clear GPU memory if needed
python tools/scripts/optimize_performance.py --clear-memory

# Monitor GPU utilization
watch -n 1 'nvidia-smi --query-gpu=utilization.gpu,utilization.memory --format=csv'
```

## 🔒 Security

### Scanning

```bash
# Scan container images
trivy image your-image:tag

# Scan filesystem
trivy fs .

# Generate security report
trivy image --format html --output security-report.html your-image:tag
```

### Backup & Recovery

```bash
# Create manual backup
./tools/scripts/backup_databases.sh

# Check backup status
ls -la /backups/

# Restore from backup (emergency)
pg_restore -h localhost -U postgres -d chimera /backups/postgres_backup_*.sql
```

## 🚢 Multi-Layer Production Deployment

### Kubernetes Multi-Layer Deployment

```bash
# Deploy all implemented layers
kubectl apply -f k8s/layer4-deployment.yaml  # Layer 4 (Execution)
kubectl apply -f k8s/layer5-deployment.yaml  # Layer 5 (Refinement)
kubectl apply -f k8s/layer7-deployment.yaml  # Layer 7 (Evolution)

# Check deployment status for each layer
kubectl get pods -l layer=execution   # Layer 4 agents
kubectl get pods -l layer=refinement  # Layer 5 optimization
kubectl get pods -l layer=evolution   # Layer 7 evolution

# Scale individual layers
kubectl scale deployment layer4-execution --replicas=5
kubectl scale deployment layer5-refinement --replicas=3
kubectl scale deployment layer7-evolution --replicas=2
```

### Layer-Specific Service Discovery

```bash
# Layer 4 services
kubectl get services -l layer=execution

# Layer 5 services
kubectl get services -l layer=refinement

# Layer 7 services
kubectl get services -l layer=evolution

# Cross-layer service mesh
kubectl get virtualservices -A  # Istio service mesh
```

### CI/CD Pipeline

The GitHub Actions workflows provide comprehensive multi-layer deployment:

#### Main Pipeline (`.github/workflows/ci-cd.yml`)
1. **Multi-Layer Build**: Parallel builds with timing metrics
2. **Security Scanning**: Trivy vulnerability scanning
3. **Testing**: Unit, integration, and sandbox tests
4. **Docker Build**: Multi-stage builds for each layer
5. **Deployment**: Gated deployment with canary releases

#### Layer-Specific Pipelines
- **Layer 5 CI** (`.github/workflows/layer5-ci.yml`): Optimization engine deployment
- **Layer 7 CI** (`.github/workflows/layer7-ci.yml`): Evolution engine deployment

#### Deployment Features
- **Blue-Green Deployments**: Zero-downtime layer updates
- **Canary Releases**: Gradual rollout with traffic splitting
- **Rollback Automation**: Instant rollback on failure detection
- **Integration Testing**: Cross-layer validation in staging

## 🛠️ Multi-Layer Troubleshooting

### Layer-Specific Issues

#### Layer 4 (Execution) Issues
**Agent Service Not Responding**
```bash
# Check Layer 4 logs
kubectl logs -l layer=execution

# Restart Layer 4 services
kubectl rollout restart deployment/layer4-execution

# Check WASM runtime health
curl http://layer4-service:8000/health
```

**High Task Queue Latency**
```bash
# Check Redis queue depth
kubectl exec redis-pod -- redis-cli LLEN agent-tasks

# Monitor task processing rate
kubectl logs -l layer=execution -f | grep "task-processed"
```

#### Layer 5 (Refinement) Issues
**Optimization Engine Failures**
```bash
# Check Layer 5 logs
kubectl logs -l layer=refinement

# Verify ML model health
curl http://layer5-service:8002/health/models

# Check pattern recognition accuracy
kubectl logs -l layer=refinement | grep "pattern-accuracy"
```

**A/B Test Issues**
```bash
# Check experiment status
curl http://layer5-service:8002/api/experiments

# Validate statistical calculations
kubectl logs -l layer=refinement | grep "statistical-test"
```

#### Layer 7 (Evolution) Issues
**Evolution Pipeline Stuck**
```bash
# Check evolution status
kubectl logs -l layer=evolution

# Monitor population convergence
curl http://layer7-service:8003/api/population/status

# Check genome deployment success rate
kubectl logs -l layer=evolution | grep "deployment-success"
```

### Cross-Layer Integration Issues
**Inter-Layer Communication Failures**
```bash
# Check service mesh connectivity
kubectl exec layer4-pod -- curl layer5-service:8002/health

# Verify message queue health
kubectl exec redis-pod -- redis-cli PING

# Check circuit breaker status
kubectl logs -l layer=execution | grep "circuit-breaker"
```

**Database Connectivity Issues**
```bash
# Check PostgreSQL health across layers
kubectl exec postgres-pod -- pg_isready -U chimera

# Verify database connections
kubectl logs -l layer=evolution | grep "database-connection"

# Check connection pooling
kubectl logs -l layer=refinement | grep "connection-pool"
```

### System-Wide Issues
**Resource Exhaustion**
```bash
# Check resource usage across all layers
kubectl top pods --all-namespaces

# Monitor GPU utilization
kubectl logs -l layer=evolution | grep "gpu-utilization"

# Check memory usage patterns
kubectl logs -l layer=refinement | grep "memory-usage"
```

**Monitoring and Alerting Issues**
```bash
# Check Prometheus targets for all layers
curl http://prometheus:9090/api/v1/targets

# Verify Grafana dashboards
curl http://grafana:3000/api/health

# Check alert manager status
curl http://alertmanager:9093/api/v1/status
```

## 📚 Additional Resources

- [API Documentation](docs/api/)
- [Architecture Overview](docs/architecture/)
- [Architecture Diagrams](./ARCHITECTURE_DIAGRAMS.md)
- [Build Metrics & Status](./BUILD_METRICS_README.md)
- [Performance Tuning Guide](docs/deployment/performance.md)
- [Security Best Practices](docs/deployment/security.md)
- [Layer4 Implementation](../src/layer4/README.md)
- [Layer5 Implementation](../src/layer5/)
- [Layer7 Implementation](../src/layer7/)
- [Implementation Checklists](../LAYER*_IMPLEMENTATION_CHECKLIST.md)

## 🆘 Support

For issues and questions:
1. Check the troubleshooting guide above
2. Review service logs with `docker-compose logs [service-name]`
3. Check Grafana dashboards for system health
4. Create an issue in the project repository

---

*This deployment guide is automatically generated and kept in sync with the Project Chimera infrastructure. Last updated: $(date)*