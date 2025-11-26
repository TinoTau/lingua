# Quick TTS Service Check

Write-Host "=== Checking TTS Service in WSL ===" -ForegroundColor Cyan
Write-Host ""

# Check if TTS service is running
Write-Host "[1] Checking TTS process..." -ForegroundColor Yellow
wsl bash -c "ps aux | grep -E 'piper_http_server|uvicorn.*5005' | grep -v grep || echo 'NOT_FOUND'"

Write-Host ""
Write-Host "[2] Checking port 5005..." -ForegroundColor Yellow
wsl bash -c "netstat -tuln 2>/dev/null | grep ':5005' || ss -tuln 2>/dev/null | grep ':5005' || echo 'NOT_FOUND'"

Write-Host ""
Write-Host "[3] Testing from WSL..." -ForegroundColor Yellow
wsl bash -c "curl -s http://127.0.0.1:5005/health || echo 'FAILED'"

Write-Host ""
Write-Host "[4] Testing port forwarding (127.0.0.1:5005)..." -ForegroundColor Yellow
try {
    $result = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -TimeoutSec 3 -ErrorAction Stop
    Write-Host "SUCCESS: Port forwarding works! Status: $($result.StatusCode)" -ForegroundColor Green
    Write-Host "Response: $($result.Content)" -ForegroundColor Gray
}
catch {
    Write-Host "FAILED: Port forwarding not working: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting steps:" -ForegroundColor Yellow
    Write-Host "1. Check port forwarding: netsh interface portproxy show all" -ForegroundColor White
    Write-Host "2. Verify WSL IP: wsl hostname -I" -ForegroundColor White
    Write-Host "3. Reconfigure port forwarding (as admin):" -ForegroundColor White
    $wslIp = (wsl bash -c "hostname -I | awk '{print `$1}'").Trim()
    Write-Host "   netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1" -ForegroundColor Gray
    Write-Host "   netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Cyan

