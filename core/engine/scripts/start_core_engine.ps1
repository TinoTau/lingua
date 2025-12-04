# 启动 CoreEngine（包含 ASR 服务）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting CoreEngine (with ASR/Whisper)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本目录和项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 脚本在 core\engine\scripts\，需要向上三级到项目根目录
$projectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptDir))

# CoreEngine 可执行文件路径
$coreEnginePath = Join-Path $projectRoot "core\engine\target\release\core_engine.exe"

# 检查 CoreEngine 是否已编译
if (-not (Test-Path $coreEnginePath)) {
    Write-Host "[ERROR] CoreEngine executable not found: $coreEnginePath" -ForegroundColor Red
    Write-Host "[INFO] Please build CoreEngine first:" -ForegroundColor Yellow
    Write-Host "  cd core\engine" -ForegroundColor Cyan
    Write-Host "  cargo build --release --bin core_engine" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "[INFO] Note: CUDA support is enabled by default in Cargo.toml" -ForegroundColor Cyan
    exit 1
}

# 配置文件路径
$configPath = Join-Path $projectRoot "lingua_core_config.toml"

# 检查配置文件是否存在
if (-not (Test-Path $configPath)) {
    Write-Host "[WARN] Config file not found: $configPath" -ForegroundColor Yellow
    Write-Host "[INFO] Using default config or command line arguments" -ForegroundColor Cyan
}

# 设置 CUDA 环境变量（如果 CUDA 已安装）
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
    Write-Host "CUDA environment configured: $cudaPath" -ForegroundColor Green
} else {
    Write-Host "[WARN] CUDA not found at: $cudaPath" -ForegroundColor Yellow
    Write-Host "[INFO] CoreEngine will use CPU mode" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "CoreEngine: $coreEnginePath" -ForegroundColor Yellow
Write-Host "Config: $configPath" -ForegroundColor Yellow
Write-Host "Port: 9000" -ForegroundColor Yellow
Write-Host ""

# 检查端口是否被占用
$portInUse = Get-NetTCPConnection -LocalPort 9000 -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "[WARN] Port 9000 is already in use" -ForegroundColor Yellow
    Write-Host "[INFO] You may need to stop the existing service first" -ForegroundColor Cyan
    Write-Host ""
}

# 切换到项目根目录
Set-Location $projectRoot

# 启动 CoreEngine
Write-Host "Starting CoreEngine..." -ForegroundColor Green
Write-Host "Service endpoint: http://127.0.0.1:9000" -ForegroundColor Cyan
Write-Host "Health check: http://127.0.0.1:9000/health" -ForegroundColor Cyan
Write-Host "S2S API: http://127.0.0.1:9000/s2s" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# 启动服务
& $coreEnginePath --config $configPath

