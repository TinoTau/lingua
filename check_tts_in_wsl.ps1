# 检查 WSL 中 TTS 服务状态

Write-Host "=== 检查 WSL 中 TTS 服务 ===" -ForegroundColor Cyan
Write-Host ""

# 1. 检查进程
Write-Host "[1] 检查 TTS 服务进程..." -ForegroundColor Yellow
$processes = wsl bash -c "ps aux | grep -E 'piper_http_server|uvicorn.*5005' | grep -v grep"
if ($processes) {
    Write-Host "  ✓ 找到 TTS 服务进程:" -ForegroundColor Green
    $processes | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
} else {
    Write-Host "  ✗ 未找到 TTS 服务进程" -ForegroundColor Red
    Write-Host "    请运行: wsl bash scripts/wsl2_piper/start_piper_service.sh" -ForegroundColor Yellow
}

Write-Host ""

# 2. 检查端口监听
Write-Host "[2] 检查端口 5005 监听状态..." -ForegroundColor Yellow
$portInfo = wsl bash -c "netstat -tuln 2>/dev/null | grep ':5005' || ss -tuln 2>/dev/null | grep ':5005' || echo 'NOT_FOUND'"
if ($portInfo -and $portInfo -ne "NOT_FOUND") {
    Write-Host "  ✓ 端口 5005 正在监听:" -ForegroundColor Green
    Write-Host "    $portInfo" -ForegroundColor Gray
    
    # 检查是否监听在 0.0.0.0
    if ($portInfo -match '0\.0\.0\.0:5005' -or $portInfo -match '\*:5005') {
        Write-Host "  ✓ 服务监听在 0.0.0.0:5005（可以从外部访问）" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ 服务可能只监听在 127.0.0.1:5005（无法从外部访问）" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✗ 端口 5005 未被监听" -ForegroundColor Red
}

Write-Host ""

# 3. 从 WSL 内部测试
Write-Host "[3] 从 WSL 内部测试 TTS 服务..." -ForegroundColor Yellow
$wslTest = wsl bash -c "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:5005/health 2>/dev/null || echo 'ERROR'"
if ($wslTest -eq "200") {
    Write-Host "  ✓ WSL 内部访问成功 (HTTP $wslTest)" -ForegroundColor Green
} else {
    Write-Host "  ✗ WSL 内部访问失败: $wslTest" -ForegroundColor Red
}

Write-Host ""

# 4. 测试从 Windows 通过 WSL IP 直接访问
Write-Host "[4] 测试从 Windows 通过 WSL IP 直接访问..." -ForegroundColor Yellow
$wslIp = wsl bash -c "hostname -I | awk '{print `$1}'"
Write-Host "  WSL IP: $wslIp" -ForegroundColor Gray
try {
    $directTest = Invoke-WebRequest -Uri "http://$wslIp:5005/health" -TimeoutSec 2 -ErrorAction Stop
    if ($directTest.StatusCode -eq 200) {
        Write-Host "  ✓ 直接访问 WSL IP 成功" -ForegroundColor Green
        Write-Host "    这意味着服务运行正常，但端口转发可能有问题" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ✗ 直接访问 WSL IP 失败: $_" -ForegroundColor Red
    Write-Host "    这可能是因为 WSL 防火墙或网络配置问题" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Check Complete ===" -ForegroundColor Cyan

