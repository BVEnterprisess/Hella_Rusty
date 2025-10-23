#!/bin/bash

# Layer5 Production Deployment Script
# This script handles the complete production deployment of Layer5

set -euo pipefail

# Configuration
NAMESPACE="project-chimera"
DEPLOYMENT_NAME="layer5-refinement"
IMAGE_TAG="${1:-latest}"
BACKUP_DIR="/tmp/layer5-backup"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Pre-deployment checks
pre_deployment_checks() {
    log "Running pre-deployment checks..."

    # Check if deployment exists
    if ! kubectl get deployment $DEPLOYMENT_NAME -n $NAMESPACE >/dev/null 2>&1; then
        error "Deployment $DEPLOYMENT_NAME not found in namespace $NAMESPACE"
    fi

    # Check cluster resources
    AVAILABLE_CPU=$(kubectl describe nodes | grep "cpu:" | head -1 | awk '{print $2}' | tr -d 'm')
    AVAILABLE_MEMORY=$(kubectl describe nodes | grep "memory:" | head -1 | awk '{print $2}' | tr -d 'Mi')

    if [ "$AVAILABLE_CPU" -lt 1000 ] || [ "$AVAILABLE_MEMORY" -lt 2048 ]; then
        warn "Insufficient cluster resources. Available: CPU=${AVAILABLE_CPU}m, Memory=${AVAILABLE_MEMORY}Mi"
    fi

    # Check dependencies
    log "Checking Layer4 integration..."
    if ! kubectl get service layer4-service -n $NAMESPACE >/dev/null 2>&1; then
        error "Layer4 service not available"
    fi

    log "Checking Redis availability..."
    if ! kubectl exec -n $NAMESPACE deployment/redis -- redis-cli ping | grep -q PONG; then
        error "Redis not available"
    fi

    log "Pre-deployment checks completed"
}

# Create backup
create_backup() {
    log "Creating backup before deployment..."

    mkdir -p $BACKUP_DIR
    BACKUP_FILE="$BACKUP_DIR/layer5-backup-$(date +%Y%m%d-%H%M%S).tar.gz"

    # Backup current configuration
    kubectl get deployment $DEPLOYMENT_NAME -n $NAMESPACE -o yaml > "$BACKUP_DIR/deployment-backup.yaml"
    kubectl get configmap layer5-config -n $NAMESPACE -o yaml > "$BACKUP_DIR/configmap-backup.yaml"
    kubectl get secret layer5-secrets -n $NAMESPACE -o yaml > "$BACKUP_DIR/secret-backup.yaml"

    log "Backup created: $BACKUP_FILE"
}

# Deploy to staging first (blue-green)
deploy_to_staging() {
    log "Deploying to staging environment..."

    # Update blue deployment with new image
    kubectl set image deployment/${DEPLOYMENT_NAME}-blue layer5=project-chimera/layer5:$IMAGE_TAG -n $NAMESPACE

    # Scale up blue deployment
    kubectl scale deployment/${DEPLOYMENT_NAME}-blue --replicas=3 -n $NAMESPACE

    # Wait for rollout
    log "Waiting for blue deployment rollout..."
    kubectl rollout status deployment/${DEPLOYMENT_NAME}-blue -n $NAMESPACE --timeout=300s

    # Validate staging deployment
    validate_deployment "${DEPLOYMENT_NAME}-blue"

    log "Staging deployment completed successfully"
}

# Validate deployment
validate_deployment() {
    local deployment_name=$1

    log "Validating deployment $deployment_name..."

    # Check pod health
    kubectl wait --for=condition=ready pod -l app=layer5,deployment=$deployment_name -n $NAMESPACE --timeout=60s

    # Check health endpoints
    local pod_name=$(kubectl get pods -l app=layer5,deployment=$deployment_name -n $NAMESPACE -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n $NAMESPACE $pod_name -- curl -f http://localhost:8080/health
    kubectl exec -n $NAMESPACE $pod_name -- curl -f http://localhost:8080/ready

    # Check metrics endpoint
    kubectl exec -n $NAMESPACE $pod_name -- curl -f http://localhost:9090/metrics

    # Validate integration with Layer4
    kubectl exec -n $NAMESPACE $pod_name -- curl -f http://layer4-service:8080/health

    log "Deployment validation completed"
}

# Switch traffic to new version
switch_traffic() {
    log "Switching traffic to new version..."

    # Update active service to point to blue
    kubectl patch service layer5-refinement-active -n $NAMESPACE -p '{"spec":{"selector":{"deployment":"blue"}}}'

    # Wait for traffic switch
    sleep 10

    # Validate active service
    kubectl exec -n $NAMESPACE deployment/layer4-integration -- curl -f http://layer5-refinement-active:8080/health

    log "Traffic switched successfully"
}

# Scale down old version
scale_down_old() {
    log "Scaling down old version..."

    # Scale down green deployment
    kubectl scale deployment/${DEPLOYMENT_NAME}-green --replicas=0 -n $NAMESPACE

    # Wait for scale down
    kubectl wait --for=delete pod -l app=layer5,deployment=green -n $NAMESPACE --timeout=60s

    log "Old version scaled down"
}

# Post-deployment validation
post_deployment_validation() {
    log "Running post-deployment validation..."

    # Check overall system health
    validate_deployment "${DEPLOYMENT_NAME}-blue"

    # Monitor metrics for 5 minutes
    log "Monitoring metrics for 5 minutes..."
    sleep 300

    # Check error rates
    local error_rate=$(kubectl exec -n $NAMESPACE deployment/${DEPLOYMENT_NAME}-blue -- curl -s http://localhost:9090/metrics | grep 'layer5_errors_total' | tail -1 | awk '{print $2}')
    if [ "${error_rate:-0}" -gt 0 ]; then
        warn "Error rate detected: $error_rate"
    fi

    # Check optimization accuracy
    local accuracy=$(kubectl exec -n $NAMESPACE deployment/${DEPLOYMENT_NAME}-blue -- curl -s http://localhost:9090/metrics | grep 'layer5_optimization_accuracy' | tail -1 | awk '{print $2}')
    if [ "${accuracy:-0}" -lt 0.95 ]; then
        warn "Optimization accuracy below threshold: $accuracy"
    fi

    log "Post-deployment validation completed"
}

# Rollback procedure
rollback() {
    log "Starting rollback procedure..."

    # Switch traffic back to green
    kubectl patch service layer5-refinement-active -n $NAMESPACE -p '{"spec":{"selector":{"deployment":"green"}}}'

    # Scale up green deployment
    kubectl scale deployment/${DEPLOYMENT_NAME}-green --replicas=3 -n $NAMESPACE

    # Scale down blue deployment
    kubectl scale deployment/${DEPLOYMENT_NAME}-blue --replicas=0 -n $NAMESPACE

    log "Rollback completed"
    exit 1
}

# Main deployment flow
main() {
    log "Starting Layer5 production deployment..."

    # Trap for rollback on error
    trap rollback ERR

    pre_deployment_checks
    create_backup
    deploy_to_staging
    switch_traffic
    scale_down_old
    post_deployment_validation

    log "Layer5 production deployment completed successfully!"
    log "New version: $IMAGE_TAG"
    log "Active deployment: blue"
    log "Backup location: $BACKUP_DIR"
}

# Run main function
main "$@"