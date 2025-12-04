# Start ASR Service (faster-whisper)
# Usage: .\start_asr_service.ps1

$ErrorActionPreference = "Continue"

# Get script directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Starting ASR Service (faster-whisper)..." -ForegroundColor Green
Write-Host "Script directory: $scriptDir" -ForegroundColor Gray

# Check if asr_service.py exists
$asrServicePy = Join-Path $scriptDir "asr_service.py"
if (-not (Test-Path $asrServicePy)) {
    Write-Host "Error: asr_service.py not found at $asrServicePy" -ForegroundColor Red
    Write-Host "Press any key to close..." -ForegroundColor Yellow
    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
    exit 1
}

# Try to use existing Python environment (lingua-py310)
$pythonPath = "D:\Program Files\Anaconda\envs\lingua-py310\python.exe"
if (-not (Test-Path $pythonPath)) {
    # Fallback to system Python
    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($python) {
        $pythonPath = $python.Source
        Write-Host "Using system Python: $pythonPath" -ForegroundColor Yellow
    } else {
        Write-Host "Error: Python not found. Please install Python 3.10+" -ForegroundColor Red
        Write-Host "Press any key to close..." -ForegroundColor Yellow
        $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
        exit 1
    }
} else {
    Write-Host "Using Python environment: $pythonPath" -ForegroundColor Green
}

# Check if requirements file exists
$requirementsFile = Join-Path $scriptDir "asr_service_requirements.txt"
if (Test-Path $requirementsFile) {
    Write-Host "Checking dependencies..." -ForegroundColor Yellow
    # Try to import faster-whisper to check if it's installed
    $checkResult = & $pythonPath -c "import faster_whisper; print('OK')" 2>&1
    if ($LASTEXITCODE -ne 0 -or $checkResult -notmatch "OK") {
        Write-Host "Installing dependencies from $requirementsFile..." -ForegroundColor Yellow
        & $pythonPath -m pip install -r $requirementsFile
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Warning: Failed to install some dependencies. Continuing anyway..." -ForegroundColor Yellow
        }
    } else {
        Write-Host "Dependencies already installed." -ForegroundColor Green
    }
} else {
    Write-Host "Warning: asr_service_requirements.txt not found. Assuming dependencies are installed." -ForegroundColor Yellow
}

# Set environment variables
# Model path options:
# - "Systran/faster-whisper-large-v3" (recommended, optimized for faster-whisper)
# - "openai/whisper-large-v3" (original OpenAI model)
# - Local path like "models/whisper-large-v3" (if you have downloaded the model locally)
$env:ASR_MODEL_PATH = "Systran/faster-whisper-large-v3"

# GPU Configuration
# Check if CUDA is available
$cudaAvailable = $false
try {
    $nvidiaSmi = Get-Command nvidia-smi -ErrorAction SilentlyContinue
    if ($nvidiaSmi) {
        $gpuCheck = nvidia-smi --query-gpu=name --format=csv,noheader 2>&1
        if ($LASTEXITCODE -eq 0 -and $gpuCheck -and $gpuCheck -notmatch "error|not found") {
            $cudaAvailable = $true
            Write-Host "  ✓ GPU detected, using CUDA" -ForegroundColor Green
        }
    }
} catch {
    # GPU not available, use CPU
}

if ($cudaAvailable) {
    $env:ASR_DEVICE = "cuda"
    $env:ASR_COMPUTE_TYPE = "float16"  # Use float16 for GPU (faster and uses less memory)
    Write-Host "  Device: CUDA (GPU)" -ForegroundColor Green
    Write-Host "  Compute Type: float16" -ForegroundColor Green
} else {
    $env:ASR_DEVICE = "cpu"
    $env:ASR_COMPUTE_TYPE = "float32"  # Use float32 for CPU
    Write-Host "  Device: CPU" -ForegroundColor Yellow
    Write-Host "  Compute Type: float32" -ForegroundColor Yellow
    Write-Host "  Note: To use GPU, ensure CUDA is installed and nvidia-smi works" -ForegroundColor Gray
}

$env:ASR_SERVICE_PORT = "6006"

Write-Host ""
Write-Host "Configuration:" -ForegroundColor Cyan
Write-Host "  Model Path: $env:ASR_MODEL_PATH" -ForegroundColor White
Write-Host "  Device: $env:ASR_DEVICE" -ForegroundColor White
Write-Host "  Compute Type: $env:ASR_COMPUTE_TYPE" -ForegroundColor White
Write-Host "  Port: $env:ASR_SERVICE_PORT" -ForegroundColor White
Write-Host ""

# Start service
Write-Host "Starting ASR service on port $env:ASR_SERVICE_PORT..." -ForegroundColor Green
Write-Host "Note: Model loading may take a while on first start..." -ForegroundColor Yellow
Write-Host ""

try {
    & $pythonPath $asrServicePy
} catch {
    Write-Host "Error starting ASR service: $_" -ForegroundColor Red
    Write-Host "Press any key to close..." -ForegroundColor Yellow
    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
    exit 1
}

