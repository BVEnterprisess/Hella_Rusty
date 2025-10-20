#!/bin/bash

# Database Backup Automation Script for Project Chimera
# Handles PostgreSQL and Redis backups with encryption and cloud storage

set -euo pipefail

# Configuration
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=7
ENCRYPTION_KEY_FILE="/etc/backup.key"
LOG_FILE="/var/log/backup.log"

# Cloud storage configuration
S3_BUCKET="project-chimera-backups"
SUPABASE_PROJECT="zgbhitjnhzheaqyptzms"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# PostgreSQL backup function
backup_postgresql() {
    local backup_name="postgres_backup_$DATE.sql"
    local backup_path="$BACKUP_DIR/$backup_name"

    log "Starting PostgreSQL backup..."

    # Create database dump
    pg_dumpall -h localhost -U postgres > "$backup_path" 2>> "$LOG_FILE"

    if [ $? -eq 0 ]; then
        log "PostgreSQL backup created: $backup_path"

        # Compress the backup
        gzip "$backup_path"
        backup_path="$backup_path.gz"

        # Encrypt the backup
        if [ -f "$ENCRYPTION_KEY_FILE" ]; then
            openssl enc -aes-256-cbc -salt -in "$backup_path" -out "$backup_path.enc" -pass file:"$ENCRYPTION_KEY_FILE" 2>> "$LOG_FILE"
            rm "$backup_path"
            backup_path="$backup_path.enc"
            log "PostgreSQL backup encrypted"
        fi

        return 0
    else
        log "ERROR: PostgreSQL backup failed"
        return 1
    fi
}

# Redis backup function
backup_redis() {
    local backup_name="redis_backup_$DATE.rdb"
    local backup_path="$BACKUP_DIR/$backup_name"

    log "Starting Redis backup..."

    # Use redis-cli to create backup
    redis-cli SAVE 2>> "$LOG_FILE"

    if [ $? -eq 0 ]; then
        # Copy the Redis dump file
        cp /data/dump.rdb "$backup_path" 2>> "$LOG_FILE"

        # Compress the backup
        gzip "$backup_path"
        backup_path="$backup_path.gz"

        # Encrypt the backup
        if [ -f "$ENCRYPTION_KEY_FILE" ]; then
            openssl enc -aes-256-cbc -salt -in "$backup_path" -out "$backup_path.enc" -pass file:"$ENCRYPTION_KEY_FILE" 2>> "$LOG_FILE"
            rm "$backup_path"
            backup_path="$backup_path.enc"
            log "Redis backup encrypted"
        fi

        log "Redis backup created: $backup_path"
        return 0
    else
        log "ERROR: Redis backup failed"
        return 1
    fi
}

# Upload to cloud storage
upload_to_cloud() {
    local file_path="$1"
    local file_name=$(basename "$file_path")

    log "Uploading $file_name to cloud storage..."

    # Upload to S3 (if AWS CLI is configured)
    if command -v aws &> /dev/null; then
        aws s3 cp "$file_path" "s3://$S3_BUCKET/$file_name" 2>> "$LOG_FILE"
        if [ $? -eq 0 ]; then
            log "Successfully uploaded to S3"
        fi
    fi

    # Upload to Supabase Storage (if configured)
    if command -v supabase &> /dev/null; then
        supabase storage cp "$file_path" "backups/$file_name" --project-ref "$SUPABASE_PROJECT" 2>> "$LOG_FILE"
        if [ $? -eq 0 ]; then
            log "Successfully uploaded to Supabase Storage"
        fi
    fi
}

# Cleanup old backups
cleanup_old_backups() {
    log "Cleaning up backups older than $RETENTION_DAYS days..."

    find "$BACKUP_DIR" -type f -name "*.enc" -o -name "*.gz" -o -name "*.sql" -mtime +$RETENTION_DAYS -delete 2>> "$LOG_FILE"

    if [ $? -eq 0 ]; then
        log "Old backups cleaned up"
    else
        log "WARNING: Some old backups could not be deleted"
    fi
}

# Health check function
health_check() {
    log "Performing health checks..."

    # Check PostgreSQL
    if pg_isready -h localhost -U postgres 2>> "$LOG_FILE"; then
        log "PostgreSQL is healthy"
    else
        log "ERROR: PostgreSQL is not responding"
        return 1
    fi

    # Check Redis
    if redis-cli ping 2>> "$LOG_FILE" | grep -q PONG; then
        log "Redis is healthy"
    else
        log "ERROR: Redis is not responding"
        return 1
    fi

    return 0
}

# Main execution
main() {
    log "=== Starting database backup process ==="

    # Perform health check first
    if ! health_check; then
        log "ERROR: Health check failed, aborting backup"
        exit 1
    fi

    # Create backups
    local backup_success=true

    if ! backup_postgresql; then
        backup_success=false
    fi

    if ! backup_redis; then
        backup_success=false
    fi

    # Upload to cloud if backups were successful
    if [ "$backup_success" = true ]; then
        for backup_file in "$BACKUP_DIR"/*; do
            if [ -f "$backup_file" ]; then
                upload_to_cloud "$backup_file"
            fi
        done
    fi

    # Cleanup old backups
    cleanup_old_backups

    if [ "$backup_success" = true ]; then
        log "=== Database backup process completed successfully ==="
        exit 0
    else
        log "=== Database backup process completed with errors ==="
        exit 1
    fi
}

# Handle script interruption
trap 'log "Backup process interrupted"; exit 1' INT TERM

# Run main function
main "$@"