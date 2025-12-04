# 单独启动 CoreEngine 服务
# 服务：CoreEngine (Windows, 端口 9000) - 包含 VAD、ASR、NMT、TTS 等核心功能

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Lingua CoreEngine Service Startup" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 检查必要的文件
$coreEnginePath = Join-Path $scriptDir "core\engine\target\release\core_engine.exe"
$configPath = Join-Path $scriptDir "lingua_core_config.toml"

if (-not (Test-Path $coreEnginePath)) {
    Write-Host "[ERROR] CoreEngine executable not found: $coreEnginePath" -ForegroundColor Red
    Write-Host "[INFO] Please build CoreEngine first:" -ForegroundColor Yellow
    Write-Host "  cd core\engine" -ForegroundColor Yellow
    Write-Host "  cargo build --release --bin core_engine" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $configPath)) {
    Write-Host "[WARNING] Config file not found: $configPath" -ForegroundColor Yellow
    Write-Host "[INFO] Using default configuration" -ForegroundColor Yellow
}

# 设置 CUDA 环境变量（如果需要）
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
if (Test-Path $cudaPath) {
    $env:CUDA_PATH = $cudaPath
    $env:CUDAToolkit_ROOT = $cudaPath
    $env:CUDA_ROOT = $cudaPath
    $env:CUDA_HOME = $cudaPath
    $cudaBin = Join-Path $cudaPath "bin"
    $cudaLibnvvp = Join-Path $cudaPath "libnvvp"
    $cudaNvcc = Join-Path $cudaBin "nvcc.exe"
    $env:CMAKE_CUDA_COMPILER = $cudaNvcc
    $env:PATH = "$cudaBin;$cudaLibnvvp;$env:PATH"
    Write-Host "[INFO] CUDA environment variables set" -ForegroundColor Green
}

# 检查端口是否被占用
$port = 9000
$portInUse = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "[WARNING] Port $port is already in use" -ForegroundColor Yellow
    Write-Host "[INFO] Please stop the service using port $port first" -ForegroundColor Yellow
    Write-Host "[INFO] Or modify the port in lingua_core_config.toml" -ForegroundColor Yellow
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        exit 1
    }
}

Write-Host "[INFO] Starting CoreEngine service..." -ForegroundColor Cyan
Write-Host "[INFO] Service URL: http://127.0.0.1:$port" -ForegroundColor Cyan
Write-Host "[INFO] WebSocket URL: ws://127.0.0.1:$port/stream" -ForegroundColor Cyan
Write-Host "[INFO] Health Check: http://127.0.0.1:$port/health" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop the service" -ForegroundColor Yellow
Write-Host ""

# 启动 CoreEngine
try {
    & $coreEnginePath
} catch {
    Write-Host "[ERROR] Failed to start CoreEngine: $_" -ForegroundColor Red
    exit 1
}

