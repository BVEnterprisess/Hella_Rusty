#!/usr/bin/env bash
################################################################################
# Project Chimera - Network Security Validation Script
################################################################################
#
# PURPOSE:
#   Validates that critical services are NOT externally accessible
#   Tests that only monitoring services have exposed ports
#
# USAGE:
#   ./scripts/test-network-security.sh
#
# REQUIREMENTS:
#   - docker-compose running
#   - nc (netcat) or nmap installed
#
################################################################################

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
log_fail() { echo -e "${RED}[FAIL]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }

TESTS_PASSED=0
TESTS_FAILED=0

# Test if a port is NOT accessible
test_port_closed() {
    local service_name="$1"
    local port="$2"
    local description="$3"
    
    log_info "Testing: $description (port $port should be CLOSED)"
    
    # Try to connect with 2-second timeout
    if timeout 2 bash -c "echo > /dev/tcp/localhost/$port" 2>/dev/null; then
        log_fail "❌ $service_name port $port is EXPOSED (security risk!)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    else
        log_pass "✅ $service_name port $port is NOT accessible (secure)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    fi
}

# Test if a port IS accessible (for monitoring services)
test_port_open() {
    local service_name="$1"
    local port="$2"
    local description="$3"
    
    log_info "Testing: $description (port $port should be OPEN)"
    
    if timeout 2 bash -c "echo > /dev/tcp/localhost/$port" 2>/dev/null; then
        log_pass "✅ $service_name port $port is accessible (expected)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        log_fail "❌ $service_name port $port is NOT accessible (unexpected)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Banner
echo "======================================================================"
echo "  Project Chimera - Network Security Validation"
echo "======================================================================"
echo ""

# Check if Docker Compose is running
log_info "Checking if services are running..."
if ! docker-compose ps | grep -q "Up"; then
    log_warn "No services are running. Starting services..."
    docker-compose up -d
    sleep 10
fi

echo ""
log_info "Testing critical services (should NOT be externally accessible)..."
echo ""

# Critical Services - Should NOT be accessible externally
test_port_closed "PostgreSQL" 5432 "Database should not be exposed"
test_port_closed "Redis" 6379 "Cache should not be exposed"
test_port_closed "MinIO S3" 9000 "Object storage API should not be exposed"
test_port_closed "MinIO Console" 9001 "Object storage console should not be exposed"
test_port_closed "NATS Client" 4222 "Message queue should not be exposed"
test_port_closed "NATS HTTP" 8222 "Message queue HTTP should not be exposed"

echo ""
log_info "Testing monitoring services (should be accessible)..."
echo ""

# Monitoring Services - Should BE accessible
test_port_open "Prometheus" 9090 "Metrics collection should be accessible"
test_port_open "Grafana" 3000 "Dashboard should be accessible"
test_port_open "Jaeger UI" 16686 "Tracing UI should be accessible"
test_port_open "Jaeger Collector" 14268 "Tracing collector should be accessible"

echo ""
echo "======================================================================"
echo "  Test Summary"
echo "======================================================================"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))

echo "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [[ $TESTS_FAILED -eq 0 ]]; then
    log_pass "🎉 All network security tests passed!"
    echo ""
    echo "✅ Critical services are NOT externally accessible"
    echo "✅ Monitoring services ARE accessible as expected"
    echo "✅ Network segmentation is working correctly"
    echo ""
    exit 0
else
    log_fail "❌ Some network security tests failed!"
    echo ""
    echo "SECURITY RISK: Exposed services detected"
    echo "Action Required: Review docker-compose.yml port mappings"
    echo ""
    exit 1
fi
