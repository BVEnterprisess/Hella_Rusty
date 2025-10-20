#!/bin/bash

# Project Chimera Deployment Script
# Handles deployment to different environments with proper validation

set -euo pipefail

# Configuration
ENVIRONMENT="${1:-staging}"
DOCKER_IMAGE="${2:-project-chimera/agent:latest}"
NAMESPACE="${3:-default}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Validation functions
validate_environment() {
    case "$ENVIRONMENT" in
        "development"|"staging"|"production")
            log_info "Deploying to $ENVIRONMENT environment"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            echo "Valid environments: development, staging, production"
            exit 1
            ;;
    esac
}

check_dependencies() {
    local missing_deps=()

    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi

    if ! command -v kubectl &> /dev/null; then
        missing_deps+=("kubectl")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

validate_kubernetes_context() {
    local current_context
    current_context=$(kubectl config current-context)

    case "$ENVIRONMENT" in
        "development")
            if [[ ! "$current_context" =~ dev ]]; then
                log_error "Not connected to development cluster. Current context: $current_context"
                exit 1
            fi
            ;;
        "staging")
            if [[ ! "$current_context" =~ staging ]]; then
                log_error "Not connected to staging cluster. Current context: $current_context"
                exit 1
            fi
            ;;
        "production")
            if [[ ! "$current_context" =~ prod ]]; then
                log_error "Not connected to production cluster. Current context: $current_context"
                exit 1
            fi
            ;;
    esac

    log_info "Connected to correct cluster: $current_context"
}

deploy_to_kubernetes() {
    log_info "Deploying $DOCKER_IMAGE to $ENVIRONMENT"

    # Apply Kubernetes manifests
    kubectl apply -f k8s/namespace.yaml || true
    kubectl apply -f k8s/configmap.yaml
    kubectl apply -f k8s/secrets.yaml
    kubectl apply -f k8s/agent-deployment.yaml

    # Wait for rollout to complete
    log_info "Waiting for deployment to complete..."
    kubectl rollout status deployment/chimera-agent -n "$NAMESPACE" --timeout=300s

    # Verify deployment
    local replicas
    replicas=$(kubectl get deployment chimera-agent -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')

    if [ "$replicas" -gt 0 ]; then
        log_info "Deployment successful! Ready replicas: $replicas"
    else
        log_error "Deployment failed! No ready replicas found."
        exit 1
    fi
}

run_health_checks() {
    log_info "Running health checks..."

    # Get service URL
    local service_url
    service_url=$(kubectl get ingress chimera-agent-ingress -n "$NAMESPACE" -o jsonpath='{.spec.rules[0].host}')

    if [ -z "$service_url" ]; then
        log_warn "No ingress found, using service port-forward"
        kubectl port-forward -n "$NAMESPACE" service/chimera-agent-service 8080:80 &
        local pf_pid=$!
        service_url="localhost:8080"
        sleep 5
    fi

    # Health check
    local health_response
    health_response=$(curl -f -s "http://$service_url/health" || echo "FAILED")

    if [[ "$health_response" == *"healthy"* ]]; then
        log_info "Health check passed!"
    else
        log_error "Health check failed! Response: $health_response"
        exit 1
    fi

    # Test prediction endpoint
    local test_payload='{"job_id":"deploy-test","input":{"text":"test deployment","lang":"en"}}'
    local predict_response
    predict_response=$(curl -f -s -X POST "http://$service_url/predict" \
        -H "Content-Type: application/json" \
        -d "$test_payload" || echo "FAILED")

    if [[ "$predict_response" == *"status"* ]]; then
        log_info "Prediction endpoint test passed!"
    else
        log_error "Prediction endpoint test failed! Response: $predict_response"
        exit 1
    fi

    # Clean up port-forward if used
    if [ ! -z "${pf_pid:-}" ]; then
        kill $pf_pid 2>/dev/null || true
    fi
}

run_smoke_tests() {
    log_info "Running smoke tests..."

    # Basic functionality tests
    local test_cases=(
        "hello world"
        "order a coffee"
        "check inventory"
        "good morning"
    )

    for test_input in "${test_cases[@]}"; do
        local payload="{\"job_id\":\"smoke-$RANDOM\",\"input\":{\"text\":\"$test_input\",\"lang\":\"en\"}}"

        if curl -f -s -X POST "http://$service_url/predict" \
            -H "Content-Type: application/json" \
            -d "$payload" > /dev/null; then
            log_info "✓ Test passed for: '$test_input'"
        else
            log_error "✗ Test failed for: '$test_input'"
            return 1
        fi
    done

    log_info "All smoke tests passed!"
}

main() {
    log_info "Starting Project Chimera deployment to $ENVIRONMENT"

    validate_environment
    check_dependencies
    validate_kubernetes_context
    deploy_to_kubernetes
    run_health_checks
    run_smoke_tests

    log_info "Deployment to $ENVIRONMENT completed successfully! 🚀"
    log_info "Service is ready and accepting requests"
}

# Handle script arguments
if [ $# -eq 0 ]; then
    echo "Usage: $0 <environment> [docker_image] [namespace]"
    echo "Example: $0 staging project-chimera/agent:v1.0.0"
    exit 1
fi

main "$@"