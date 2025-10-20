# Project Chimera - AI Agent DevOps Environment

A comprehensive DevOps setup for Project Chimera, a multi-agent AI orchestration platform with single-GPU deployment, self-evolving capabilities, and enterprise-grade observability.

## 🚀 Quick Start

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

## 📚 Documentation

- [API Documentation](./docs/api/)
- [Architecture Guide](./docs/architecture/)
- [Deployment Guide](./docs/deployment/)
- [Training Guide](./docs/training/)

## 🤝 Contributing

1. Create a feature branch
2. Make your changes
3. Add tests in the appropriate test directory
4. Run the full test suite
5. Submit a pull request with sandbox test results

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

For support and questions:
- Check the troubleshooting guide above
- Review the logs in `/app/logs/`
- Open an issue with detailed error information

---

**Built with ❤️ for the future of AI orchestration**