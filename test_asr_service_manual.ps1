# 手动测试 ASR 服务启动
# 这个脚本会显示详细的启动信息，帮助诊断问题

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  ASR Service Manual Test" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = "D:\Programs\github\lingua\core\engine\scripts"
Set-Location $scriptDir

Write-Host "1. Checking script directory..." -ForegroundColor Yellow
Write-Host "   Directory: $scriptDir" -ForegroundColor Gray
if (Test-Path $scriptDir) {
    Write-Host "   ✓ Directory exists" -ForegroundColor Green
} else {
    Write-Host "   ✗ Directory not found!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "2. Checking asr_service.py..." -ForegroundColor Yellow
$asrServicePy = Join-Path $scriptDir "asr_service.py"
if (Test-Path $asrServicePy) {
    Write-Host "   ✓ asr_service.py found" -ForegroundColor Green
} else {
    Write-Host "   ✗ asr_service.py not found at: $asrServicePy" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "3. Checking Python environment..." -ForegroundColor Yellow
$pythonPath = "D:\Program Files\Anaconda\envs\lingua-py310\python.exe"
if (Test-Path $pythonPath) {
    Write-Host "   ✓ Python found: $pythonPath" -ForegroundColor Green
    $pythonVersion = & $pythonPath --version 2>&1
    Write-Host "   Version: $pythonVersion" -ForegroundColor Gray
} else {
    Write-Host "   ✗ Python not found at: $pythonPath" -ForegroundColor Red
    Write-Host "   Trying system Python..." -ForegroundColor Yellow
    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($python) {
        $pythonPath = $python.Source
        Write-Host "   ✓ Using system Python: $pythonPath" -ForegroundColor Green
    } else {
        Write-Host "   ✗ No Python found!" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "4. Checking faster-whisper..." -ForegroundColor Yellow
$checkResult = & $pythonPath -c "import faster_whisper; print('OK')" 2>&1
if ($LASTEXITCODE -eq 0 -and $checkResult -match "OK") {
    Write-Host "   ✓ faster-whisper is installed" -ForegroundColor Green
} else {
    Write-Host "   ✗ faster-whisper not installed or import failed" -ForegroundColor Red
    Write-Host "   Error: $checkResult" -ForegroundColor Red
    Write-Host ""
    Write-Host "   Installing faster-whisper..." -ForegroundColor Yellow
    & $pythonPath -m pip install faster-whisper fastapi uvicorn soundfile numpy
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✓ Installation successful" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Installation failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "5. Starting ASR service..." -ForegroundColor Yellow
Write-Host "   This will start the service in this window." -ForegroundColor Gray
Write-Host "   Press Ctrl+C to stop the service." -ForegroundColor Gray
Write-Host ""

# Set environment variables
$env:ASR_MODEL_PATH = "model/whisper-large-v3"
$env:ASR_DEVICE = "cpu"
$env:ASR_COMPUTE_TYPE = "float32"
$env:ASR_SERVICE_PORT = "6006"

Write-Host "Configuration:" -ForegroundColor Cyan
Write-Host "  Model Path: $env:ASR_MODEL_PATH" -ForegroundColor White
Write-Host "  Device: $env:ASR_DEVICE" -ForegroundColor White
Write-Host "  Compute Type: $env:ASR_COMPUTE_TYPE" -ForegroundColor White
Write-Host "  Port: $env:ASR_SERVICE_PORT" -ForegroundColor White
Write-Host ""

# Start service
& $pythonPath $asrServicePy

