# 🚀 Project Chimera Deployment Guide

## Overview

Project Chimera is a sophisticated multi-agent AI orchestration platform with GPU optimization, self-evolving capabilities, and enterprise-grade DevOps infrastructure. This guide covers deployment, monitoring, and operational procedures.

## 🏗️ Architecture

### Core Components
- **AI Agents**: Multi-agent system with specialized capabilities
- **GPU Optimization**: Single GPU with memory-efficient processing
- **Self-Evolution**: Automated LoRA adapter training and deployment
- **Safety Systems**: Sandbox testing and gated deployments

### Infrastructure Stack
- **Containerization**: Docker with GPU support
- **Orchestration**: Kubernetes for production scaling
- **Monitoring**: Prometheus, Grafana, Jaeger, Alertmanager
- **Logging**: Fluent Bit with centralized aggregation
- **Security**: Trivy scanning, encrypted backups, RBAC

## 🚀 Quick Start

### Prerequisites
- Docker and Docker Compose
- NVIDIA GPU with drivers (for GPU acceleration)
- 16GB+ RAM recommended
- 100GB+ storage for models and data

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

### Services Overview

| Service | Port | Purpose |
|---------|------|---------|
| **Agent Service** | 8000 | Main AI agent API |
| **Redis** | 6379 | Caching and message broker |
| **PostgreSQL** | 5432 | Persistent data storage |
| **Prometheus** | 9090 | Metrics collection |
| **Grafana** | 3000 | Visualization dashboard |
| **Jaeger** | 16686 | Distributed tracing |
| **Alertmanager** | 9093 | Alert management |

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

## 🛠️ Development Workflow

### 1. Code Changes

```bash
# Make your changes
git checkout -b feature/new-agent-capability

# Run tests
npm test  # or equivalent for your stack

# Build locally
cargo build --release  # for Rust components
```

### 2. Testing

```bash
# Run unit tests
cargo test

# Run integration tests
npm run test:integration

# Run sandbox tests (safe deployment testing)
npx playwright test tests/sandbox/
```

### 3. Deployment

```bash
# Deploy to staging
./tools/scripts/deploy.sh staging

# Run sandbox tests
npm run test:sandbox

# Deploy to production (if tests pass)
./tools/scripts/deploy.sh production
```

## 📊 Monitoring & Observability

### Key Metrics to Monitor

1. **Agent Performance**
   - Response time (p95 < 2s)
   - Success rate (>99%)
   - GPU memory usage (<90%)

2. **System Health**
   - CPU usage (<80%)
   - Memory usage (<85%)
   - Disk usage (<80%)

3. **Business Metrics**
   - Active agents
   - Model accuracy
   - Training completion rate

### Accessing Dashboards

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686

### Alert Configuration

Alerts are configured in `configs/alertmanager.yml` and include:

- 🚨 **Critical**: Agent down, GPU failure, database issues
- ⚠️ **Warning**: High resource usage, degraded performance
- ℹ️ **Info**: Deployment notifications, maintenance windows

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

## 🚢 Production Deployment

### Kubernetes Deployment

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -l app=agent-service

# Scale agents
kubectl scale deployment agent-service --replicas=3
```

### CI/CD Pipeline

The GitHub Actions workflow (`.github/workflows/ci-cd.yml`) provides:

1. **Security Scanning**: Automated vulnerability checks
2. **Testing**: Unit, integration, and sandbox tests
3. **Building**: Multi-stage Docker builds
4. **Deployment**: Gated deployment to production

## 🛠️ Troubleshooting

### Common Issues

**High Memory Usage**
```bash
# Clear GPU cache
python tools/scripts/optimize_performance.py --clear-memory

# Check for memory leaks
python tools/scripts/optimize_performance.py --report
```

**Agent Not Responding**
```bash
# Check agent logs
docker-compose logs agent-service

# Restart agent service
docker-compose restart agent-service
```

**Database Connection Issues**
```bash
# Check database health
docker-compose exec postgres pg_isready -U postgres

# View database logs
docker-compose logs postgres
```

**Monitoring Issues**
```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Restart monitoring stack
docker-compose restart prometheus grafana
```

## 📚 Additional Resources

- [API Documentation](docs/api/)
- [Architecture Overview](docs/architecture/)
- [Performance Tuning Guide](docs/deployment/performance.md)
- [Security Best Practices](docs/deployment/security.md)

## 🆘 Support

For issues and questions:
1. Check the troubleshooting guide above
2. Review service logs with `docker-compose logs [service-name]`
3. Check Grafana dashboards for system health
4. Create an issue in the project repository

---

*This deployment guide is automatically generated and kept in sync with the Project Chimera infrastructure. Last updated: $(date)*