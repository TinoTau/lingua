# Fix Port Forwarding for TTS Service
# Run this script as Administrator

Write-Host "=== Fixing Port Forwarding for TTS Service ===" -ForegroundColor Cyan
Write-Host ""

# Check if running as admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "WARNING: This script should be run as Administrator" -ForegroundColor Yellow
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    Write-Host ""
}

# Get current WSL IP
Write-Host "[1] Getting WSL IP address..." -ForegroundColor Yellow
$wslIp = (wsl bash -c "hostname -I | awk '{print `$1}'").Trim()
if (-not $wslIp) {
    Write-Host "ERROR: Could not get WSL IP address" -ForegroundColor Red
    exit 1
}
Write-Host "  WSL IP: $wslIp" -ForegroundColor Green

# Check existing port forwarding
Write-Host ""
Write-Host "[2] Checking existing port forwarding..." -ForegroundColor Yellow
$existing = netsh interface portproxy show all | Select-String "5005"
if ($existing) {
    Write-Host "  Found existing rule:" -ForegroundColor Gray
    $existing | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    
    Write-Host ""
    Write-Host "[3] Removing existing rule..." -ForegroundColor Yellow
    netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1 2>&1 | Out-Null
    Write-Host "  Removed" -ForegroundColor Green
} else {
    Write-Host "  No existing rule found" -ForegroundColor Gray
    Write-Host ""
    Write-Host "[3] Adding new port forwarding rule..." -ForegroundColor Yellow
}

# Add new port forwarding rule
Write-Host ""
Write-Host "[4] Adding port forwarding rule..." -ForegroundColor Yellow
$result = netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Successfully added port forwarding rule" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Failed to add port forwarding rule" -ForegroundColor Red
    Write-Host "  Error: $result" -ForegroundColor Red
    exit 1
}

# Verify the rule
Write-Host ""
Write-Host "[5] Verifying port forwarding rule..." -ForegroundColor Yellow
$verify = netsh interface portproxy show all | Select-String "5005"
if ($verify) {
    Write-Host "  Port forwarding rule:" -ForegroundColor Green
    $verify | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
} else {
    Write-Host "  WARNING: Could not verify port forwarding rule" -ForegroundColor Yellow
}

# Test connection
Write-Host ""
Write-Host "[6] Testing connection..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -TimeoutSec 3 -ErrorAction Stop
    Write-Host "  SUCCESS: Port forwarding is working!" -ForegroundColor Green
    Write-Host "  Response: $($response.Content)" -ForegroundColor Gray
} catch {
    Write-Host "  FAILED: Still cannot connect: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Additional troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Check Windows Firewall settings" -ForegroundColor White
    Write-Host "  2. Verify TTS service is running: wsl bash -c 'curl http://127.0.0.1:5005/health'" -ForegroundColor White
    Write-Host "  3. Try restarting WSL: wsl --shutdown (then restart services)" -ForegroundColor White
}

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Cyan

