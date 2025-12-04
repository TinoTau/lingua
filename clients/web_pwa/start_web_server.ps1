# 启动 Web 前端服务器（独立脚本）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting Web Frontend Server" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 默认端口
$port = 8080

# 检查命令行参数
if ($args.Count -gt 0) {
    if ($args[0] -match '^-p|--port$') {
        if ($args.Count -gt 1) {
            $port = [int]$args[1]
        }
    } elseif ($args[0] -match '^\d+$') {
        $port = [int]$args[0]
    }
}

Write-Host "Server directory: $scriptDir" -ForegroundColor Yellow
Write-Host "Port: $port" -ForegroundColor Yellow
Write-Host "Access URL: http://localhost:$port" -ForegroundColor Cyan
Write-Host ""

# 切换到脚本目录
Set-Location $scriptDir

# 尝试使用 Python 启动
$pythonCandidates = @("python", "python3")
foreach ($cmd in $pythonCandidates) {
    if (Get-Command $cmd -ErrorAction SilentlyContinue) {
        Write-Host "Using $cmd to start HTTP server..." -ForegroundColor Green
        Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
        Write-Host ""
        & $cmd -m http.server $port --directory $scriptDir
        exit 0
    }
}

# 尝试使用 npx http-server
if (Get-Command npx -ErrorAction SilentlyContinue) {
    Write-Host "Using npx http-server..." -ForegroundColor Green
    Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
    Write-Host ""
    & npx http-server $scriptDir -p $port
    exit 0
}

# 如果都不可用，显示手动命令
Write-Host "[ERROR] No HTTP server found" -ForegroundColor Red
Write-Host "[INFO] Please install one of the following:" -ForegroundColor Yellow
Write-Host "  - Python: https://www.python.org/downloads/" -ForegroundColor Cyan
Write-Host "  - Node.js: https://nodejs.org/" -ForegroundColor Cyan
Write-Host ""
Write-Host "[INFO] Or run manually:" -ForegroundColor Yellow
Write-Host "  python -m http.server $port --directory $scriptDir" -ForegroundColor Cyan
Write-Host "  npx http-server $scriptDir -p $port" -ForegroundColor Cyan
exit 1

