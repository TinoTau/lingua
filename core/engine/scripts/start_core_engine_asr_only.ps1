# 启动 CoreEngine 服务（ASR 专用测试）
# 只需要 CoreEngine 服务，不需要 NMT 和 TTS 服务

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting CoreEngine (ASR Only)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
$configPath = Join-Path $projectRoot "lingua_core_config.toml"

# 检查配置文件
if (-not (Test-Path $configPath)) {
    Write-Host "[ERROR] Config file not found: $configPath" -ForegroundColor Red
    Write-Host "[INFO] Please ensure lingua_core_config.toml exists in project root" -ForegroundColor Yellow
    exit 1
}

# 检查可执行文件
$exePath = Join-Path $scriptDir "..\target\debug\core_engine.exe"
if (-not (Test-Path $exePath)) {
    $exePath = Join-Path $scriptDir "..\target\release\core_engine.exe"
    if (-not (Test-Path $exePath)) {
        Write-Host "[ERROR] CoreEngine executable not found" -ForegroundColor Red
        Write-Host "[INFO] Please build CoreEngine first:" -ForegroundColor Yellow
        Write-Host "  cd core\engine && cargo build --bin core_engine" -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "Config file: $configPath" -ForegroundColor Yellow
Write-Host "Executable: $exePath" -ForegroundColor Yellow
Write-Host ""
Write-Host "Starting CoreEngine..." -ForegroundColor Green
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# 切换到项目根目录
Set-Location $projectRoot

# 运行 CoreEngine
& $exePath --config $configPath

