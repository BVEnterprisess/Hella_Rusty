# Project Chimera Resource Cleanup Script
# Automatically frees up system resources

Write-Host "Starting Project Chimera Resource Cleanup..." -ForegroundColor Green

# Function to log actions
function Write-Log {
    param($Message, $Color = "White")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

# 1. Clean up Docker resources
Write-Log "Cleaning up Docker resources..." "Yellow"
try {
    docker system prune -f
    Write-Log "Docker cleanup completed" "Green"
} catch {
    Write-Log "Docker cleanup failed: $_" "Red"
}

# 2. Stop unnecessary Docker containers
Write-Log "Stopping unnecessary Docker containers..." "Yellow"
try {
    docker-compose down 2>$null
    Write-Log "Docker containers stopped" "Green"
} catch {
    Write-Log "Failed to stop containers: $_" "Red"
}

# 3. Clear Windows temp files
Write-Log "Clearing Windows temp files..." "Yellow"
try {
    Remove-Item -Path "$env:TEMP\*" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "C:\Windows\Temp\*" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Log "Temp files cleared" "Green"
} catch {
    Write-Log "Failed to clear temp files: $_" "Red"
}

# 4. Clear Rust/Cargo cache (if exists)
Write-Log "Clearing Rust/Cargo cache..." "Yellow"
try {
    if (Test-Path "$env:USERPROFILE\.cargo") {
        cargo cache --autoclean 2>$null
        Write-Log "Cargo cache cleared" "Green"
    } else {
        Write-Log "Cargo not found" "Yellow"
    }
} catch {
    Write-Log "Failed to clear Cargo cache: $_" "Red"
}

# 5. Kill high resource processes (optional - be careful)
Write-Log "Checking for high resource processes..." "Yellow"
try {
    $highCpuProcesses = Get-Process | Where-Object {$_.CPU -gt 50} | Where-Object {$_.ProcessName -notlike "System" -and $_.ProcessName -notlike "svchost" -and $_.ProcessName -notlike "csrss"}
    if ($highCpuProcesses) {
        Write-Log "High CPU processes found:" "Red"
        $highCpuProcesses | ForEach-Object {
            Write-Log "  $($_.ProcessName) (PID: $($_.Id), CPU: $($_.CPU)%)" "Red"
        }
        # Uncomment the next line if you want to automatically kill them (not recommended for production)
        # $highCpuProcesses | Stop-Process -Force
    } else {
        Write-Log "No high CPU processes found" "Green"
    }
} catch {
    Write-Log "Failed to check processes: $_" "Red"
}

# 6. Run Windows Disk Cleanup
Write-Log "Running Windows Disk Cleanup..." "Yellow"
try {
    Start-Process -FilePath "cleanmgr.exe" -ArgumentList "/sagerun:1" -Wait
    Write-Log "Disk cleanup completed" "Green"
} catch {
    Write-Log "Disk cleanup failed: $_" "Red"
}

# 7. Optimize WSL2 (if running)
Write-Log "Optimizing WSL2..." "Yellow"
try {
    wsl --shutdown 2>$null
    Write-Log "WSL2 shutdown for optimization" "Green"
} catch {
    Write-Log "WSL2 optimization failed: $_" "Red"
}

# 8. Clear browser caches (optional)
Write-Log "Clearing browser caches..." "Yellow"
try {
    # Clear Chrome cache
    if (Test-Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache") {
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache\*" -Recurse -Force -ErrorAction SilentlyContinue
    }
    # Clear Firefox cache
    if (Test-Path "$env:LOCALAPPDATA\Mozilla\Firefox\Profiles") {
        Get-ChildItem "$env:LOCALAPPDATA\Mozilla\Firefox\Profiles" | ForEach-Object {
            Remove-Item -Path "$($_.FullName)\cache2\*" -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    Write-Log "Browser caches cleared" "Green"
} catch {
    Write-Log "Failed to clear browser caches: $_" "Red"
}

Write-Log "Resource cleanup completed!" "Green"
Write-Log "Run 'system_monitor.ps1' to verify improvements" "Cyan"