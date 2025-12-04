# 启动 NMT 服务（绕过 conda 激活问题）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting NMT Service (M2M100)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本目录和项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 脚本在 core\engine\scripts\，需要向上三级到项目根目录
$projectRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptDir))
$nmtServiceDir = Join-Path $projectRoot "services\nmt_m2m100"

# 检查服务目录是否存在
if (-not (Test-Path $nmtServiceDir)) {
    Write-Host "[ERROR] NMT service directory not found: $nmtServiceDir" -ForegroundColor Red
    exit 1
}

# Python 路径（使用 venv 中的 Python）
$pythonPath = Join-Path $nmtServiceDir "venv\Scripts\python.exe"

# 如果 venv 不存在，尝试使用系统 Python
if (-not (Test-Path $pythonPath)) {
    Write-Host "[WARN] Virtual environment not found, using system Python" -ForegroundColor Yellow
    $pythonPath = "python"
}

Write-Host "Python: $pythonPath" -ForegroundColor Yellow
Write-Host "Service directory: $nmtServiceDir" -ForegroundColor Yellow
Write-Host ""

# 切换到服务目录
Set-Location $nmtServiceDir

# 检查 uvicorn 是否可用
Write-Host "Checking uvicorn..." -ForegroundColor Cyan
$uvicornCheck = & $pythonPath -m uvicorn --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] uvicorn not found. Please install dependencies:" -ForegroundColor Red
    Write-Host "  pip install -r requirements.txt" -ForegroundColor Yellow
    exit 1
}
Write-Host $uvicornCheck -ForegroundColor Cyan
Write-Host ""

# 设置环境变量：使用本地文件模式（跳过 token 验证）
$env:HF_LOCAL_FILES_ONLY = "true"
Write-Host "Using local files only (no token verification needed)" -ForegroundColor Cyan
Write-Host ""

# 启动服务
Write-Host "Starting NMT service..." -ForegroundColor Green
Write-Host "Service endpoint: http://127.0.0.1:5008" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# 启动服务
& $pythonPath -m uvicorn nmt_service:app --host 127.0.0.1 --port 5008

