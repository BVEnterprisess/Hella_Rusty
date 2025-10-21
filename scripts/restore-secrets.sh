#!/usr/bin/env bash
################################################################################
# Project Chimera - Secrets Restore Script
################################################################################
#
# PURPOSE:
#   Restores secrets from a backup file created by rotate-secrets.sh
#   Used for rollback after failed rotation or emergency recovery
#
# USAGE:
#   ./scripts/restore-secrets.sh BACKUP_FILE
#
# EXAMPLES:
#   # Restore from specific backup
#   ./scripts/restore-secrets.sh backups/.env.backup.20251021-120000
#
#   # List available backups
#   ./scripts/restore-secrets.sh --list
#
#   # Restore latest backup
#   ./scripts/restore-secrets.sh --latest
#
################################################################################

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${PROJECT_ROOT}/.env"
BACKUP_DIR="${PROJECT_ROOT}/backups"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# List available backups
list_backups() {
    log_info "Available backups in $BACKUP_DIR:"
    if [[ ! -d "$BACKUP_DIR" ]] || [[ -z "$(ls -A "$BACKUP_DIR")" ]]; then
        log_warning "No backups found"
        exit 0
    fi
    ls -lh "$BACKUP_DIR"/.env.backup.* | awk '{print $9, "(" $5 ")", $6, $7, $8}'
}

# Get latest backup
get_latest_backup() {
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log_error "Backup directory not found: $BACKUP_DIR"
        exit 1
    fi
    
    local latest
    latest=$(ls -t "$BACKUP_DIR"/.env.backup.* 2>/dev/null | head -n 1)
    
    if [[ -z "$latest" ]]; then
        log_error "No backups found in $BACKUP_DIR"
        exit 1
    fi
    
    echo "$latest"
}

# Restore from backup
restore_backup() {
    local backup_file="$1"
    
    if [[ ! -f "$backup_file" ]]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi
    
    log_warning "This will replace current .env file with backup"
    log_info "Current .env will be backed up to .env.before-restore"
    
    # Backup current state before restoring
    if [[ -f "$ENV_FILE" ]]; then
        cp "$ENV_FILE" "${ENV_FILE}.before-restore"
        log_success "Backed up current .env to .env.before-restore"
    fi
    
    # Restore from backup
    cp "$backup_file" "$ENV_FILE"
    chmod 600 "$ENV_FILE"
    
    log_success "Restored from: $backup_file"
    log_info "Restarting services..."
    
    docker-compose restart
    
    log_success "Restoration complete!"
}

# Main
case "${1:-}" in
    --list)
        list_backups
        ;;
    --latest)
        backup_file=$(get_latest_backup)
        log_info "Restoring from latest backup: $backup_file"
        restore_backup "$backup_file"
        ;;
    --help|-h)
        head -n 20 "$0" | grep "^#" | sed 's/^# //; s/^#//'
        exit 0
        ;;
    "")
        log_error "No backup file specified"
        log_info "Usage: $0 BACKUP_FILE"
        log_info "       $0 --list"
        log_info "       $0 --latest"
        exit 1
        ;;
    *)
        restore_backup "$1"
        ;;
esac
