# 一键停止所有服务脚本
# 停止 CoreEngine、NMT 服务和 TTS 服务

Write-Host "=== Lingua 服务一键停止 ===" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# 停止端口上的服务
$ports = @(9000, 5008, 5005, 8080)
$stopped = @()

foreach ($port in $ports) {
    $connections = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($connections) {
        $processes = $connections | Select-Object -ExpandProperty OwningProcess -Unique
        foreach ($pid in $processes) {
            try {
                $process = Get-Process -Id $pid -ErrorAction SilentlyContinue
                if ($process) {
                    $processName = $process.ProcessName
                    Write-Host "停止端口 $port 上的进程: $processName (PID: $pid)..." -ForegroundColor Yellow
                    Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
                    $stopped += "端口 $port: $processName (PID: $pid)"
                }
            } catch {
                Write-Host "  警告: 无法停止进程 $pid" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "端口 $port: 无活动服务" -ForegroundColor Gray
    }
}

# 停止 PowerShell 后台作业（如果有）
$jobs = Get-Job -ErrorAction SilentlyContinue
if ($jobs) {
    Write-Host ""
    Write-Host "停止后台作业..." -ForegroundColor Yellow
    $jobs | Stop-Job -ErrorAction SilentlyContinue
    $jobs | Remove-Job -ErrorAction SilentlyContinue
    Write-Host "✓ 后台作业已停止" -ForegroundColor Green
}

# 停止 WSL 中的 TTS 服务（如果存在）
Write-Host ""
Write-Host "检查 WSL 中的 TTS 服务..." -ForegroundColor Yellow
try {
    $wslProcess = wsl bash -c "lsof -ti:5005 2>/dev/null || echo ''" 2>$null
    if ($wslProcess -and $wslProcess.Trim()) {
        Write-Host "停止 WSL 中的 TTS 服务..." -ForegroundColor Yellow
        wsl bash -c "lsof -ti:5005 | xargs kill -9 2>/dev/null || true" 2>$null
        wsl bash -c "pkill -f piper_http_server 2>/dev/null || true" 2>$null
        Write-Host "✓ WSL TTS 服务已停止" -ForegroundColor Green
    } else {
        Write-Host "WSL 中无 TTS 服务运行" -ForegroundColor Gray
    }
} catch {
    Write-Host "  无法检查 WSL 服务（可能 WSL 未运行）" -ForegroundColor Gray
}

# 等待一下，确保进程完全停止
Start-Sleep -Seconds 1

# 验证端口是否已释放
Write-Host ""
Write-Host "验证端口状态..." -ForegroundColor Cyan
$allStopped = $true
foreach ($port in $ports) {
    $connections = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($connections) {
        Write-Host "  ⚠ 端口 $port 仍被占用" -ForegroundColor Yellow
        $allStopped = $false
    } else {
        Write-Host "  ✓ 端口 $port 已释放" -ForegroundColor Green
    }
}

Write-Host ""
if ($stopped.Count -gt 0) {
    Write-Host "已停止的服务:" -ForegroundColor Green
    foreach ($item in $stopped) {
        Write-Host "  - $item" -ForegroundColor White
    }
} else {
    Write-Host "没有发现运行中的服务" -ForegroundColor Gray
}

if ($allStopped) {
    Write-Host ""
    Write-Host "✓ 所有服务已停止" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "⚠ 部分端口可能仍被占用，请手动检查" -ForegroundColor Yellow
    Write-Host "  可以使用以下命令查看: Get-NetTCPConnection -LocalPort 9000,5008,5005,8080" -ForegroundColor Gray
}

