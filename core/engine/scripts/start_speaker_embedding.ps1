# 启动 Speaker Embedding 服务（绕过 conda 激活问题）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting Speaker Embedding Service" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# Python 完整路径
$pythonPath = "D:\Program Files\Anaconda\envs\lingua-py310\python.exe"

# 检查 Python 是否存在
if (-not (Test-Path $pythonPath)) {
    Write-Host "[ERROR] Python not found at: $pythonPath" -ForegroundColor Red
    Write-Host "[INFO] Please update the pythonPath in this script" -ForegroundColor Yellow
    exit 1
}

# 获取脚本目录和项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 脚本在 core\engine\scripts\，需要向上三级到项目根目录
$projectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptDir))
$serviceScript = Join-Path $projectRoot "core\engine\scripts\speaker_embedding_service.py"

# 检查服务脚本是否存在
if (-not (Test-Path $serviceScript)) {
    Write-Host "[ERROR] Service script not found: $serviceScript" -ForegroundColor Red
    exit 1
}

Write-Host "Python: $pythonPath" -ForegroundColor Yellow
Write-Host "Service script: $serviceScript" -ForegroundColor Yellow
Write-Host ""

# 检查 GPU
Write-Host "Checking GPU availability..." -ForegroundColor Cyan
$gpuCheck = & $pythonPath -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU name:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')" 2>&1
Write-Host $gpuCheck -ForegroundColor Cyan
Write-Host ""

# 启动服务
Write-Host "Starting Speaker Embedding service (GPU mode)..." -ForegroundColor Green
Write-Host "Service endpoint: http://127.0.0.1:5003" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# 切换到项目根目录
Set-Location $projectRoot

# 启动服务
& $pythonPath $serviceScript --gpu

