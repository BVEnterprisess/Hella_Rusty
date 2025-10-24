# Project Chimera System Monitor
# Monitors CPU, RAM, GPU, and disk usage with alerts

param(
    [int]$IntervalSeconds = 30,
    [int]$CpuThreshold = 80,
    [int]$RamThreshold = 90,
    [int]$GpuTempThreshold = 80,
    [int]$DiskThreshold = 90
)

Write-Host "Starting Project Chimera System Monitor..." -ForegroundColor Green
Write-Host "Monitoring every $IntervalSeconds seconds" -ForegroundColor Yellow
Write-Host "Thresholds: CPU > $CpuThreshold%, RAM > $RamThreshold%, GPU Temp > $GpuTempThreshold°C, Disk > $DiskThreshold%" -ForegroundColor Yellow

while ($true) {
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

    # CPU Usage
    try {
        $cpuUsage = (Get-Counter '\Processor(_Total)\% Processor Time').CounterSamples.CookedValue
        if ($cpuUsage -gt $CpuThreshold) {
            Write-Host "[$timestamp] HIGH CPU USAGE: $cpuUsage%" -ForegroundColor Red
        } else {
            Write-Host "[$timestamp] CPU: $cpuUsage%" -ForegroundColor Green
        }
    } catch {
        Write-Host "[$timestamp] CPU Monitor Error: $_" -ForegroundColor Red
    }

    # RAM Usage
    try {
        $ram = Get-WmiObject -Class Win32_OperatingSystem
        $totalRam = [math]::Round($ram.TotalVisibleMemorySize / 1MB, 2)
        $freeRam = [math]::Round($ram.FreePhysicalMemory / 1MB, 2)
        $usedRam = $totalRam - $freeRam
        $ramPercent = [math]::Round(($usedRam / $totalRam) * 100, 2)

        if ($ramPercent -gt $RamThreshold) {
            Write-Host "[$timestamp] HIGH RAM USAGE: $ramPercent% ($usedRam GB / $totalRam GB)" -ForegroundColor Red
        } else {
            Write-Host "[$timestamp] RAM: $ramPercent% ($usedRam GB / $totalRam GB)" -ForegroundColor Green
        }
    } catch {
        Write-Host "[$timestamp] RAM Monitor Error: $_" -ForegroundColor Red
    }

    # GPU Usage (NVIDIA)
    try {
        $gpuInfo = & nvidia-smi --query-gpu=utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null
        if ($gpuInfo) {
            $gpuUtil, $gpuTemp = $gpuInfo[0].Split(',')
            $gpuUtil = $gpuUtil.Trim()
            $gpuTemp = $gpuTemp.Trim()

            if ([int]$gpuTemp -gt $GpuTempThreshold) {
                Write-Host "[$timestamp] HIGH GPU TEMP: $gpuTemp°C (Util: $gpuUtil%)" -ForegroundColor Red
            } else {
                Write-Host "[$timestamp] GPU: $gpuTemp°C (Util: $gpuUtil%)" -ForegroundColor Green
            }
        } else {
            Write-Host "[$timestamp] GPU: Not available or nvidia-smi not found" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "[$timestamp] GPU Monitor Error: $_" -ForegroundColor Red
    }

    # Disk Usage
    try {
        $disks = Get-WmiObject -Class Win32_LogicalDisk | Where-Object {$_.DriveType -eq 3}
        foreach ($disk in $disks) {
            $totalSize = [math]::Round($disk.Size / 1GB, 2)
            $freeSpace = [math]::Round($disk.FreeSpace / 1GB, 2)
            $usedSpace = $totalSize - $freeSpace
            $diskPercent = [math]::Round(($usedSpace / $totalSize) * 100, 2)

            if ($diskPercent -gt $DiskThreshold) {
                Write-Host "[$timestamp] HIGH DISK USAGE ($($disk.Name)): $diskPercent% ($usedSpace GB / $totalSize GB)" -ForegroundColor Red
            } else {
                Write-Host "[$timestamp] Disk ($($disk.Name)): $diskPercent% ($usedSpace GB / $totalSize GB)" -ForegroundColor Green
            }
        }
    } catch {
        Write-Host "[$timestamp] Disk Monitor Error: $_" -ForegroundColor Red
    }

    # Check for high resource processes
    try {
        $highCpuProcesses = Get-Process | Where-Object {$_.CPU -gt 10} | Sort-Object CPU -Descending | Select-Object Name, CPU, Memory -First 5
        if ($highCpuProcesses) {
            Write-Host "[$timestamp] Top CPU Processes:" -ForegroundColor Yellow
            $highCpuProcesses | ForEach-Object {
                Write-Host "  $($_.Name): CPU $($_.CPU)%, Mem $([math]::Round($_.Memory / 1MB, 2))MB" -ForegroundColor Yellow
            }
        }
    } catch {
        Write-Host "[$timestamp] Process Monitor Error: $_" -ForegroundColor Red
    }

    Write-Host "[$timestamp] --- Next check in $IntervalSeconds seconds ---" -ForegroundColor Cyan
    Start-Sleep -Seconds $IntervalSeconds
}