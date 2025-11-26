# TTS 服务诊断脚本
# 用途：检查 TTS 服务状态并诊断连接问题

Write-Host "=== TTS 服务诊断 ===" -ForegroundColor Cyan
Write-Host ""

# 1. 检查 WSL 中 TTS 服务是否运行
Write-Host "[1/4] 检查 WSL 中的 TTS 服务..." -ForegroundColor Yellow
$wslProcess = wsl bash -c "ps aux | grep -E 'piper_http_server|uvicorn.*5005' | grep -v grep" 2>$null
if ($wslProcess) {
    Write-Host "  ✓ TTS 服务正在 WSL 中运行" -ForegroundColor Green
    Write-Host "    进程信息: $wslProcess" -ForegroundColor Gray
} else {
    Write-Host "  ✗ TTS 服务未在 WSL 中运行" -ForegroundColor Red
    Write-Host "    请运行: wsl bash scripts/wsl2_piper/start_piper_service.sh" -ForegroundColor Yellow
}

Write-Host ""

# 2. 检查 WSL 中端口 5005 是否被监听
Write-Host "[2/4] 检查 WSL 中的端口 5005..." -ForegroundColor Yellow
$wslPort = wsl bash -c "netstat -tuln 2>/dev/null | grep ':5005' || ss -tuln 2>/dev/null | grep ':5005' || echo 'NOT_FOUND'" 2>$null
if ($wslPort -and $wslPort -ne "NOT_FOUND") {
    Write-Host "  ✓ 端口 5005 在 WSL 中被监听" -ForegroundColor Green
    Write-Host "    端口信息: $wslPort" -ForegroundColor Gray
} else {
    Write-Host "  ✗ 端口 5005 未在 WSL 中被监听" -ForegroundColor Red
}

Write-Host ""

# 3. 检查 Windows 中端口 5005 是否可访问
Write-Host "[3/4] 检查 Windows 中的端口 5005..." -ForegroundColor Yellow
$winPort = Get-NetTCPConnection -LocalPort 5005 -ErrorAction SilentlyContinue
if ($winPort) {
    Write-Host "  ✓ 端口 5005 在 Windows 中被监听" -ForegroundColor Green
    Write-Host "    连接信息: $($winPort | Format-Table -AutoSize | Out-String)" -ForegroundColor Gray
} else {
    Write-Host "  ✗ 端口 5005 未在 Windows 中被监听" -ForegroundColor Red
    Write-Host "    这可能是正常的，如果 TTS 服务只在 WSL 中运行" -ForegroundColor Gray
}

Write-Host ""

# 4. 测试从 Windows 访问 TTS 服务
Write-Host "[4/4] 测试从 Windows 访问 TTS 服务..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -TimeoutSec 2 -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "  ✓ TTS 服务健康检查成功" -ForegroundColor Green
        Write-Host "    响应: $($response.Content)" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ TTS 服务返回错误状态码: $($response.StatusCode)" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ 无法连接到 TTS 服务: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "  可能的解决方案:" -ForegroundColor Yellow
    Write-Host "  1. 确保 TTS 服务在 WSL 中运行:" -ForegroundColor White
    Write-Host "     wsl bash scripts/wsl2_piper/start_piper_service.sh" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  2. 检查 WSL IP 地址并尝试直接访问:" -ForegroundColor White
    $wslIp = wsl bash -c "hostname -I | awk '{print `$1}'" 2>$null
    if ($wslIp) {
        Write-Host "     WSL IP: $wslIp" -ForegroundColor Gray
        Write-Host "     尝试访问: http://$wslIp:5005/health" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "  3. 配置端口转发（需要管理员权限）:" -ForegroundColor White
    Write-Host "     netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=== 诊断完成 ===" -ForegroundColor Cyan

