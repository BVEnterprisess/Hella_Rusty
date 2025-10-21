# Project Chimera - AI Agent DevOps Environment

> ⚠️ **PROJECT STATUS: EARLY DEVELOPMENT (15% Complete)** ⚠️  
> **NOT PRODUCTION READY** - Most architectural layers are not yet implemented.  
> See [MASTER_COMPLETION_PLAN.md](./docs/MASTER_COMPLETION_PLAN.md) for complete roadmap.

## 📊 Current Reality

**What Actually Works**:
- ✅ Layer 4 (Execution Fabric): 85% complete, 63 tests passing
- ✅ DevOps infrastructure designs (configs, but not deployed)
- ✅ Comprehensive documentation

**What Does NOT Work Yet**:
- ❌ Layers 1-3, 5-8 (87.5% of system NOT IMPLEMENTED)
- ❌ API Gateway, Task Queue, Validation layers
- ❌ KPI Analysis, Training, Evolution engines  
- ❌ Model files and training pipeline
- ❌ Database schemas
- ❌ End-to-end integration
- 🚨 **CRITICAL SECURITY ISSUES**: Hardcoded credentials in version control

**Quick Links**:
- [Complete Status Report](./PROJECT_STATUS.md)
- [Master Completion Plan](./docs/MASTER_COMPLETION_PLAN.md) (1,888 hours remaining)
- [Layer 4 Final Report](./docs/LAYER4_FINAL_REPORT.md)

---

## 🚧 Vision & Architecture (Planned)

Project Chimera is designed as a self-evolving multi-agent AI orchestration platform with 8 architectural layers, single-GPU deployment support, and enterprise-grade observability.

### Planned 8-Layer Architecture

```
Layer 1: Discovery & Ingestion (API Gateway)        ❌ NOT IMPLEMENTED
Layer 2: Task Queue & Distribution (Redis Streams)  ❌ NOT IMPLEMENTED  
Layer 3: Validation & Preprocessing                 ❌ NOT IMPLEMENTED
Layer 4: Execution Fabric (WASM Agents)             ✅ 85% COMPLETE
Layer 5: KPI Collection & Analysis                  ❌ NOT IMPLEMENTED
Layer 6: Training Data Curation                     ❌ NOT IMPLEMENTED
Layer 7: Evolution Engine (LoRA/QLoRA)              ❌ NOT IMPLEMENTED
Layer 8: Resource Orchestration                     ❌ NOT IMPLEMENTED
```

**See [MASTER_COMPLETION_PLAN.md](./docs/MASTER_COMPLETION_PLAN.md) for full details.**

---

## ⚠️ CRITICAL: Do NOT Use Yet

### The instructions below are for the PLANNED system, not current state:

<details>
<summary>Click to expand future Quick Start (NOT FUNCTIONAL YET)</summary>

## 🚀 Quick Start (PLANNED - NOT WORKING)

### Prerequisites
- NVIDIA GPU (GTX 1660 or better)
- Docker and Docker Compose
- Kubernetes cluster (for production)
- 16GB+ RAM recommended

### Initial Setup

1. **Clone and prepare the environment:**
   ```bash
   git clone <repository-url>
   cd project-chimera
   chmod +x tools/scripts/*.sh
   ```

2. **Start local development environment:**
   ```bash
   docker-compose up -d
   ```

3. **Verify services are running:**
   ```bash
   # Check all services
   docker-compose ps

   # Check agent health
   curl http://localhost:8080/health
   ```

4. **Run a test prediction:**
   ```bash
   curl -X POST http://localhost:8080/predict \
     -H "Content-Type: application/json" \
     -d '{"job_id":"test-1","input":{"text":"hello world","lang":"en"}}'
   ```

## 🏗️ Architecture Overview

### Core Components

- **AI Agents**: Rust-based microservices using Candle for quantized inference
- **Message Queue**: Redis Streams for job distribution and results
- **Training Pipeline**: QLoRA/LoRA training for adapter generation
- **Observability**: Prometheus, Grafana, Jaeger for monitoring and tracing
- **Storage**: MinIO for artifact management
- **Orchestration**: Crew AI for high-level task planning

### Data Flow

```
Crew AI → Router → Redis Streams → Agent Workers → Results → Golden Samples → Training → Adapters → Deployment
```

## 📊 Monitoring Dashboard

Access the monitoring dashboard at: http://localhost:3000
- **Username**: admin
- **Password**: admin

Key dashboards:
- **Agent Performance**: Request latency, throughput, error rates
- **GPU Utilization**: Memory usage, temperature, utilization
- **Training Progress**: LoRA training metrics and validation scores

## 🔧 Development Workflow

### 1. Agent Development

```bash
# Build the agent
cargo build --release

# Run locally for development
cargo run --bin agent

# Run tests
cargo test
```

### 2. Training New Adapters

```bash
# Prepare training data (JSONL format)
# Format: {"prompt": "user input", "response": "expected output"}

# Train QLoRA adapter
python tools/scripts/train_qlora.py \
  --base_model /path/to/gemma-3-270m \
  --output_dir ./artifacts/intent_parser_lora_v1 \
  --train_file ./data/train.jsonl \
  --validation_file ./data/val.jsonl \
  --per_device_train_batch_size 1 \
  --gradient_accumulation_steps 8 \
  --num_train_epochs 2
```

### 3. Testing and Validation

```bash
# Run unit tests
cargo test

# Run sandbox tests (gated deployment)
cd tests/sandbox && npm install && npx playwright test

# Load testing
# Use the load test endpoints in the sandbox tests
```

### 4. Deployment

```bash
# Deploy to staging
./tools/scripts/deploy.sh staging

# Deploy to production (requires approval)
./tools/scripts/deploy.sh production
```

## 🔒 Security Features

- **Gated Deployments**: All changes require sandbox testing
- **Artifact Signing**: All model artifacts are checksummed and signed
- **RBAC**: Role-based access control for production deployments
- **Network Policies**: Restricted egress for agent containers
- **Secret Management**: Kubernetes secrets for sensitive configuration

## 📈 Scaling

### Horizontal Scaling
```bash
# Scale agent replicas
kubectl scale deployment chimera-agent --replicas=10
```

### GPU Optimization
- Single-process batching for memory efficiency
- Quantized models (Q4) for reduced memory footprint
- Gradient accumulation for training on limited VRAM

## 🚨 Troubleshooting

### Common Issues

1. **GPU OOM Errors**:
   ```bash
   # Reduce batch size or increase accumulation steps
   # Check GPU memory usage
   nvidia-smi
   ```

2. **Training Failures**:
   ```bash
   # Check training logs
   docker-compose logs trainer

   # Verify data format
   head -n 5 data/train.jsonl
   ```

3. **Agent Not Responding**:
   ```bash
   # Check agent logs
   docker-compose logs agent

   # Verify Redis connectivity
   docker-compose exec redis redis-cli ping
   ```

### Health Checks

```bash
# Agent health
curl http://localhost:8080/health

# Redis health
docker-compose exec redis redis-cli ping

# Full system health
./tools/scripts/health_check.sh
```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `REDIS_URL` | Redis connection string | redis://localhost:6379 |
| `CUDA_VISIBLE_DEVICES` | GPU device selection | 0 |
| `BATCH_WINDOW_MS` | Request batching window | 15 |
| `MAX_TOKENS` | Maximum inference tokens | 512 |

### Model Configuration

Models are stored in `/models/` with the following structure:
```
/models/
├── gemma-3-270m-q4.safetensors  # Base quantized model
├── adapter_v1.safetensors       # LoRA adapter
└── merged_model_q4.safetensors  # Pre-merged model
```

</details>

---

## 📚 Documentation

### Current Status Documentation
- ⭐ [**MASTER COMPLETION PLAN**](./docs/MASTER_COMPLETION_PLAN.md) - Complete 14-18 week roadmap
- ⭐ [**PROJECT STATUS**](./PROJECT_STATUS.md) - Detailed current state analysis
- ⭐ [**Layer 4 Final Report**](./docs/LAYER4_FINAL_REPORT.md) - What's actually working

### Layer 4 (Only Implemented Layer)
- [Layer 4 README](./src/layer4/README.md) - Execution fabric documentation
- [Layer 4 Status](./docs/LAYER4_STATUS.md) - Quick reference
- [Layer 4 Test Results](./docs/LAYER4_TEST_RESULTS.md) - 63 tests passing

### Planned Documentation (For Future Layers)
- [API Documentation](./docs/api/) - Not yet implemented
- [Architecture Guide](./docs/architecture/) - Design only
- [Deployment Guide](./docs/deployment/) - Partial configs
- [Training Guide](./docs/training/) - Not yet implemented

## 🤝 Contributing

**Current Focus**: We are in early development implementing Layers 1-3 and 5-8.

### How to Help

1. **Review the [Master Completion Plan](./docs/MASTER_COMPLETION_PLAN.md)**
2. **Pick a layer to implement** (see Phase 2-8 in the plan)
3. **Follow the architecture specs** provided in the plan
4. **Write comprehensive tests** (>90% coverage required)
5. **Submit PR with:**
   - Implementation code
   - Test suite
   - Documentation updates
   - Integration test results

### Development Setup (Layer 4 Only)

```bash
# Currently only Layer 4 can be tested
cd src/layer4

# Run tests (WSL required for Windows)
wsl ~/.cargo/bin/cargo test

# Run benchmarks
wsl ~/.cargo/bin/cargo bench
```

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

For support and questions:
- Check the troubleshooting guide above
- Review the logs in `/app/logs/`
- Open an issue with detailed error information

---

## 🎯 Project Timeline

**Started**: January 2025  
**Current Status**: 15% Complete (Layer 4: 85% complete, all other layers: 0%)  
**Estimated Completion**: 14-18 weeks with proper team (3.5 FTE)  
**Total Effort**: 1,888 engineering hours remaining  

### Major Milestones

- [x] **Week 0**: Layer 4 implementation and testing (COMPLETE)
- [ ] **Week 1**: Emergency security fixes (NEXT)
- [ ] **Weeks 2-7**: Implement Layers 1-3 (API, Queue, Validation)
- [ ] **Weeks 8-14**: Implement Layers 5-8 (Analytics, Training, Evolution, Orchestration)
- [ ] **Weeks 15-16**: Integration testing and infrastructure
- [ ] **Weeks 17-18**: Production deployment

**See [MASTER_COMPLETION_PLAN.md](./docs/MASTER_COMPLETION_PLAN.md) for detailed breakdown.**

---

**Project Status**: 🚧 **UNDER ACTIVE DEVELOPMENT** 🚧  
**Last Updated**: October 21, 2025  
**Next Update**: After Phase 0 (Security Fixes) completion
