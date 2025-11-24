# M2M100 实时翻译系统 - 一键停止脚本（Windows PowerShell）
# 
# 用途：停止所有运行中的服务
# 
# 使用方法：
#   .\scripts\stop_all.ps1

Write-Host "=== M2M100 实时翻译系统 - 停止服务 ===" -ForegroundColor Yellow
Write-Host ""

# 停止 NMT 服务（Python uvicorn）
Write-Host "[1/2] 停止 NMT 服务..." -ForegroundColor Cyan
$nmtProcesses = Get-Process -Name "python" -ErrorAction SilentlyContinue | Where-Object {
    $_.CommandLine -like "*uvicorn*nmt_service*" -or $_.CommandLine -like "*nmt_service*"
}
if ($nmtProcesses) {
    $nmtProcesses | ForEach-Object {
        Write-Host "  停止进程: $($_.Id) - $($_.ProcessName)" -ForegroundColor White
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }
    Write-Host "  ✅ NMT 服务已停止" -ForegroundColor Green
} else {
    Write-Host "  ℹ️  NMT 服务未运行" -ForegroundColor Gray
}

# 停止 TTS 服务（WSL2 中的 Piper 服务）
Write-Host ""
Write-Host "[2/2] 停止 TTS 服务..." -ForegroundColor Cyan
Write-Host "  请在 WSL2 终端中按 Ctrl+C 停止 Piper 服务" -ForegroundColor Yellow
Write-Host "  或运行: wsl bash -c 'pkill -f piper_http_server'" -ForegroundColor Gray

Write-Host ""
Write-Host "✅ 停止脚本执行完成" -ForegroundColor Green
Write-Host ""

