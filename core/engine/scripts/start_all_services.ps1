# 启动所有服务（Speaker Embedding + YourTTS，Windows 本地 Python + 可选 GPU）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting All Speaker Services (Windows)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

# 检查 Python
try {
    $pythonVersion = python --version 2>&1
    Write-Host "Python: $pythonVersion" -ForegroundColor Cyan
} catch {
    Write-Host "Error: Python not found. Please install Python first." -ForegroundColor Red
    exit 1
}

# 可选：检查 CUDA
try {
    $cudaCheck = python -c "import torch; print('CUDA available:', torch.cuda.is_available())" 2>&1
    Write-Host $cudaCheck -ForegroundColor Cyan
} catch {
    Write-Host "Warning: Could not check CUDA availability (torch not installed?)" -ForegroundColor Yellow
}

# 1. Speaker Embedding 服务（Windows，GPU）
Write-Host "[1/2] Starting Speaker Embedding service (Windows, GPU)..." -ForegroundColor Yellow
$speakerEmbeddingScript = Join-Path $projectRoot "core\engine\scripts\speaker_embedding_service.py"
Start-Process python -ArgumentList $speakerEmbeddingScript, "--gpu" -WindowStyle Normal

# 等待服务启动
Write-Host "Waiting for Speaker Embedding service to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# 2. YourTTS 服务（Windows，GPU）
Write-Host "[2/2] Starting YourTTS service (Windows, GPU)..." -ForegroundColor Yellow
$yourttsScript = Join-Path $projectRoot "core\engine\scripts\yourtts_service.py"
Start-Process python -ArgumentList $yourttsScript, "--gpu" -WindowStyle Normal

Write-Host ""
Write-Host "✅ All services started!" -ForegroundColor Green
Write-Host ""
Write-Host "Service endpoints:" -ForegroundColor Cyan
Write-Host "  Speaker Embedding: http://127.0.0.1:5003" -ForegroundColor Yellow
Write-Host "  YourTTS: http://127.0.0.1:5004" -ForegroundColor Yellow
Write-Host ""
Write-Host "Check the service windows for status." -ForegroundColor Yellow
Write-Host "Press Ctrl+C in each service window to stop." -ForegroundColor Yellow

