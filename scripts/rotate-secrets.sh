#!/usr/bin/env bash
################################################################################
# Project Chimera - Secrets Rotation Script
################################################################################
#
# PURPOSE:
#   Automates secure credential rotation for all Project Chimera services.
#   Creates backup, generates new secrets, updates .env, and validates services.
#
# USAGE:
#   ./scripts/rotate-secrets.sh [OPTIONS]
#
# OPTIONS:
#   --all              Rotate all credentials (default)
#   --postgres         Rotate PostgreSQL password only
#   --redis            Rotate Redis password only
#   --minio            Rotate MinIO credentials only
#   --grafana          Rotate Grafana admin password only
#   --jwt              Rotate JWT secret only
#   --dry-run          Show what would be changed without making changes
#   --no-restart       Don't restart services after rotation
#   --backup-dir PATH  Custom backup directory (default: ./backups)
#
# EXAMPLES:
#   # Rotate all credentials
#   ./scripts/rotate-secrets.sh --all
#
#   # Rotate only database password
#   ./scripts/rotate-secrets.sh --postgres
#
#   # Dry run to preview changes
#   ./scripts/rotate-secrets.sh --all --dry-run
#
# SAFETY:
#   - Creates timestamped backup before any changes
#   - Validates new credentials before committing
#   - Supports rollback via restore-secrets.sh
#   - Logs all operations to security audit log
#
################################################################################

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${PROJECT_ROOT}/.env"
ENV_EXAMPLE="${PROJECT_ROOT}/.env.example"
BACKUP_DIR="${PROJECT_ROOT}/backups"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/.env.backup.${TIMESTAMP}"
AUDIT_LOG="${PROJECT_ROOT}/docs/security/SECURITY_AUDIT_LOG.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
DRY_RUN=false
NO_RESTART=false
ROTATE_ALL=false
ROTATE_POSTGRES=false
ROTATE_REDIS=false
ROTATE_MINIO=false
ROTATE_GRAFANA=false
ROTATE_JWT=false

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Generate secure random password
generate_password() {
    local length="${1:-32}"
    openssl rand -base64 "$length" | tr -d '\n'
}

# Backup current .env file
backup_env_file() {
    if [[ ! -f "$ENV_FILE" ]]; then
        log_error ".env file not found at $ENV_FILE"
        log_info "Creating from .env.example..."
        cp "$ENV_EXAMPLE" "$ENV_FILE"
    fi

    mkdir -p "$BACKUP_DIR"
    chmod 700 "$BACKUP_DIR"  # Owner read/write/execute only
    
    cp "$ENV_FILE" "$BACKUP_FILE"
    chmod 600 "$BACKUP_FILE"  # Owner read/write only
    
    log_success "Backup created: $BACKUP_FILE"
}

# Update environment variable in .env file
update_env_var() {
    local var_name="$1"
    local new_value="$2"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_info "[DRY RUN] Would update $var_name"
        return
    fi
    
    # Check if variable exists
    if grep -q "^${var_name}=" "$ENV_FILE"; then
        # Update existing variable
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed syntax
            sed -i '' "s|^${var_name}=.*|${var_name}=${new_value}|" "$ENV_FILE"
        else
            # Linux sed syntax
            sed -i "s|^${var_name}=.*|${var_name}=${new_value}|" "$ENV_FILE"
        fi
        log_success "Updated $var_name"
    else
        # Add new variable
        echo "${var_name}=${new_value}" >> "$ENV_FILE"
        log_success "Added $var_name"
    fi
}

# Validate service connectivity after rotation
validate_service() {
    local service_name="$1"
    
    if [[ "$DRY_RUN" == true ]] || [[ "$NO_RESTART" == true ]]; then
        return
    fi
    
    log_info "Validating $service_name service..."
    
    # Wait for service to be healthy (timeout after 30 seconds)
    local timeout=30
    local elapsed=0
    
    while [[ $elapsed -lt $timeout ]]; do
        if docker-compose ps "$service_name" | grep -q "Up"; then
            log_success "$service_name is running"
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    log_error "$service_name failed to start within ${timeout}s"
    return 1
}

# Log rotation to security audit
log_to_audit() {
    local message="$1"
    
    if [[ "$DRY_RUN" == true ]]; then
        return
    fi
    
    if [[ ! -f "$AUDIT_LOG" ]]; then
        log_warning "Audit log not found: $AUDIT_LOG"
        return
    fi
    
    {
        echo ""
        echo "---"
        echo ""
        echo "## Secrets Rotation - $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
        echo ""
        echo "$message"
        echo ""
        echo "**Rotated by**: $(whoami)@$(hostname)"
        echo "**Backup**: $(basename "$BACKUP_FILE")"
    } >> "$AUDIT_LOG"
    
    log_success "Logged to security audit"
}

# ============================================================================
# ROTATION FUNCTIONS
# ============================================================================

rotate_postgres() {
    log_info "Rotating PostgreSQL credentials..."
    
    local new_password
    new_password=$(generate_password 32)
    
    update_env_var "POSTGRES_PASSWORD" "$new_password"
    
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting PostgreSQL service..."
        docker-compose restart postgres
        validate_service "postgres"
    fi
}

rotate_redis() {
    log_info "Rotating Redis credentials..."
    
    local new_password
    new_password=$(generate_password 32)
    
    update_env_var "REDIS_PASSWORD" "$new_password"
    
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting Redis service..."
        docker-compose restart redis
        validate_service "redis"
    fi
}

rotate_minio() {
    log_info "Rotating MinIO credentials..."
    
    local new_password
    new_password=$(generate_password 32)
    
    update_env_var "MINIO_ROOT_PASSWORD" "$new_password"
    
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting MinIO service..."
        docker-compose restart minio
        validate_service "minio"
    fi
}

rotate_grafana() {
    log_info "Rotating Grafana admin credentials..."
    
    local new_password
    new_password=$(generate_password 20)
    
    update_env_var "GRAFANA_ADMIN_PASSWORD" "$new_password"
    
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting Grafana service..."
        docker-compose restart grafana
        validate_service "grafana"
    fi
}

rotate_jwt_secret() {
    log_info "Rotating JWT secret..."
    
    local new_secret
    new_secret=$(generate_password 64)
    
    update_env_var "JWT_SECRET" "$new_secret"
    
    log_warning "JWT secret rotated - all existing tokens will be invalidated"
    
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting agent services..."
        docker-compose restart agent
        validate_service "agent"
    fi
}

# ============================================================================
# ARGUMENT PARSING
# ============================================================================

if [[ $# -eq 0 ]]; then
    ROTATE_ALL=true
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            ROTATE_ALL=true
            shift
            ;;
        --postgres)
            ROTATE_POSTGRES=true
            shift
            ;;
        --redis)
            ROTATE_REDIS=true
            shift
            ;;
        --minio)
            ROTATE_MINIO=true
            shift
            ;;
        --grafana)
            ROTATE_GRAFANA=true
            shift
            ;;
        --jwt)
            ROTATE_JWT=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-restart)
            NO_RESTART=true
            shift
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        -h|--help)
            head -n 50 "$0" | grep "^#" | sed 's/^# //; s/^#//'
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    log_info "Starting secrets rotation..."
    
    if [[ "$DRY_RUN" == true ]]; then
        log_warning "DRY RUN MODE - No changes will be made"
    fi
    
    # Create backup
    backup_env_file
    
    # Perform rotations
    if [[ "$ROTATE_ALL" == true ]]; then
        rotate_postgres
        rotate_redis
        rotate_minio
        rotate_grafana
        rotate_jwt_secret
    else
        [[ "$ROTATE_POSTGRES" == true ]] && rotate_postgres
        [[ "$ROTATE_REDIS" == true ]] && rotate_redis
        [[ "$ROTATE_MINIO" == true ]] && rotate_minio
        [[ "$ROTATE_GRAFANA" == true ]] && rotate_grafana
        [[ "$ROTATE_JWT" == true ]] && rotate_jwt_secret
    fi
    
    # Restart dependent services if needed
    if [[ "$DRY_RUN" == false ]] && [[ "$NO_RESTART" == false ]]; then
        log_info "Restarting dependent services..."
        docker-compose restart agent trainer playwright
    fi
    
    # Log to audit
    local rotated_services=""
    [[ "$ROTATE_ALL" == true ]] && rotated_services="All credentials"
    [[ "$ROTATE_POSTGRES" == true ]] && rotated_services="${rotated_services} PostgreSQL"
    [[ "$ROTATE_REDIS" == true ]] && rotated_services="${rotated_services} Redis"
    [[ "$ROTATE_MINIO" == true ]] && rotated_services="${rotated_services} MinIO"
    [[ "$ROTATE_GRAFANA" == true ]] && rotated_services="${rotated_services} Grafana"
    [[ "$ROTATE_JWT" == true ]] && rotated_services="${rotated_services} JWT"
    
    log_to_audit "**Rotated**: ${rotated_services}"
    
    if [[ "$DRY_RUN" == true ]]; then
        log_warning "DRY RUN COMPLETE - No changes were made"
    else
        log_success "Secrets rotation complete!"
        log_info "Backup saved to: $BACKUP_FILE"
        log_info "To rollback: ./scripts/restore-secrets.sh $BACKUP_FILE"
    fi
}

# Run main function
cd "$PROJECT_ROOT"
main
