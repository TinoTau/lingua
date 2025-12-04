# 启动 Speaker Embedding 和 YourTTS 服务（Windows PowerShell）

Write-Host "Starting Speaker Embedding and YourTTS services..." -ForegroundColor Green

# 检查 Python 是否可用
try {
    $pythonVersion = python --version 2>&1
    Write-Host "Python: $pythonVersion" -ForegroundColor Cyan
} catch {
    Write-Host "Error: Python not found. Please install Python first." -ForegroundColor Red
    exit 1
}

# 检查 CUDA 是否可用（可选）
try {
    $cudaCheck = python -c "import torch; print('CUDA available:', torch.cuda.is_available())" 2>&1
    Write-Host $cudaCheck -ForegroundColor Cyan
} catch {
    Write-Host "Warning: Could not check CUDA availability" -ForegroundColor Yellow
}

# 获取脚本目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

# 启动 Speaker Embedding 服务（GPU 模式）
Write-Host "`nStarting Speaker Embedding service (GPU mode)..." -ForegroundColor Yellow
$speakerEmbeddingScript = Join-Path $projectRoot "core\engine\scripts\speaker_embedding_service.py"
Start-Process python -ArgumentList $speakerEmbeddingScript, "--gpu" -WindowStyle Normal

# 等待服务启动
Write-Host "Waiting for Speaker Embedding service to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# 启动 YourTTS 服务（GPU 模式）
Write-Host "Starting YourTTS service (GPU mode)..." -ForegroundColor Yellow
$yourttsScript = Join-Path $projectRoot "core\engine\scripts\yourtts_service.py"
Start-Process python -ArgumentList $yourttsScript, "--gpu" -WindowStyle Normal

Write-Host "`n✅ Services started!" -ForegroundColor Green
Write-Host "   Speaker Embedding: http://127.0.0.1:5003" -ForegroundColor Cyan
Write-Host "   YourTTS: http://127.0.0.1:5004" -ForegroundColor Cyan
Write-Host "`nCheck the service windows for status." -ForegroundColor Yellow
Write-Host "Press Ctrl+C in each service window to stop." -ForegroundColor Yellow

