# 启动 Whisper HTTP 服务器（whisper.cpp server）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting Whisper HTTP Server (whisper.cpp)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本目录和项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 脚本在 core\engine\scripts\，需要向上三级到项目根目录
$projectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptDir))

# Whisper server 可执行文件路径
# 注意：需要先编译 whisper.cpp
$whisperServerPath = Join-Path $projectRoot "third_party\whisper.cpp\build\bin\whisper-server.exe"

# 如果 Windows 版本不存在，尝试 Linux 版本（WSL）
if (-not (Test-Path $whisperServerPath)) {
    Write-Host "[WARN] Whisper server not found at: $whisperServerPath" -ForegroundColor Yellow
    Write-Host "[INFO] Checking if whisper.cpp is built..." -ForegroundColor Cyan
    
    # 检查是否需要编译
    $whisperCppDir = Join-Path $projectRoot "third_party\whisper.cpp"
    if (Test-Path $whisperCppDir) {
        Write-Host "[INFO] whisper.cpp directory found, but server may not be built yet" -ForegroundColor Yellow
        Write-Host "[INFO] You may need to build whisper.cpp first:" -ForegroundColor Yellow
        Write-Host "  cd third_party\whisper.cpp" -ForegroundColor Cyan
        Write-Host "  mkdir build" -ForegroundColor Cyan
        Write-Host "  cd build" -ForegroundColor Cyan
        Write-Host "  cmake .." -ForegroundColor Cyan
        Write-Host "  cmake --build . --config Release" -ForegroundColor Cyan
        Write-Host ""
    }
    
    # 尝试使用 WSL 中的 whisper-server
    Write-Host "[INFO] Whisper server not found in Windows" -ForegroundColor Yellow
    Write-Host "[INFO] You can:" -ForegroundColor Cyan
    Write-Host "  1. Build whisper.cpp in Windows (see instructions above)" -ForegroundColor Cyan
    Write-Host "  2. Or use WSL to run whisper-server:" -ForegroundColor Cyan
    Write-Host "     wsl -d `"Ubuntu-22.04`" -- bash -c `"cd /mnt/d/Programs/github/lingua/third_party/whisper.cpp && ./build/bin/whisper-server --model models/ggml-base.bin --host 0.0.0.0 --port 8080`"" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "[INFO] Note: If using Rust WhisperAsrStreaming, you don't need this HTTP server" -ForegroundColor Cyan
    exit 1
}

# 模型路径
$modelPath = Join-Path $projectRoot "third_party\whisper.cpp\models\ggml-base.bin"

# 如果模型不存在，提示下载
if (-not (Test-Path $modelPath)) {
    Write-Host "[WARN] Model not found at: $modelPath" -ForegroundColor Yellow
    Write-Host "[INFO] You may need to download a Whisper model first" -ForegroundColor Yellow
    Write-Host "[INFO] Available models: ggml-tiny.bin, ggml-base.bin, ggml-small.bin, etc." -ForegroundColor Cyan
    Write-Host ""
}

# 默认端口
$port = 8080

# 检查端口是否被占用
$portInUse = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "[WARN] Port $port is already in use" -ForegroundColor Yellow
    Write-Host "[INFO] You can change the port with --port option" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "Whisper server: $whisperServerPath" -ForegroundColor Yellow
Write-Host "Model: $modelPath" -ForegroundColor Yellow
Write-Host "Port: $port" -ForegroundColor Yellow
Write-Host "Host: 127.0.0.1" -ForegroundColor Yellow
Write-Host ""

# 启动服务器
Write-Host "Starting Whisper HTTP server..." -ForegroundColor Green
Write-Host "Service endpoint: http://127.0.0.1:$port" -ForegroundColor Cyan
Write-Host "Inference endpoint: http://127.0.0.1:$port/inference" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# 切换到项目根目录
Set-Location $projectRoot

# 启动服务器
& $whisperServerPath `
    --model $modelPath `
    --host 127.0.0.1 `
    --port $port `
    --language auto

