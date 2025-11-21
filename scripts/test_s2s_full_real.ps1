# Complete S2S Flow Test Script
# Tests: Real ASR (Whisper) + Real NMT (Marian) + Piper TTS

$ErrorActionPreference = "Continue"

Write-Host "=== Complete S2S Flow Test ===" -ForegroundColor Cyan
Write-Host ""

# Check if input file is provided
if ($args.Count -eq 0) {
    Write-Host "[ERROR] Please provide input WAV file path" -ForegroundColor Red
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\scripts\test_s2s_full_real.ps1 <input_wav_file>"
    Write-Host ""
    Write-Host "Example:" -ForegroundColor Yellow
    Write-Host "  .\scripts\test_s2s_full_real.ps1 test_input\chinese_audio.wav"
    exit 1
}

$inputFile = $args[0]

# Check if file exists
if (-not (Test-Path $inputFile)) {
    Write-Host "[ERROR] Input file not found: $inputFile" -ForegroundColor Red
    exit 1
}

Write-Host "[INFO] Input file: $inputFile" -ForegroundColor Green
Write-Host ""

# Check Piper HTTP service
Write-Host "[1/3] Checking Piper HTTP service..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -Method GET -TimeoutSec 2 -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "[OK] Piper HTTP service is running" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Service returned status: $($response.StatusCode)" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "[ERROR] Cannot connect to Piper HTTP service" -ForegroundColor Red
    Write-Host "[INFO] Please start the service in WSL2:" -ForegroundColor Yellow
    Write-Host "  bash scripts/wsl2_piper/start_piper_service.sh" -ForegroundColor Yellow
    exit 1
}

# Check model directories
Write-Host ""
Write-Host "[2/3] Checking model directories..." -ForegroundColor Yellow

$whisperModelDir = "core\engine\models\asr\whisper-base"
$nmtModelDir = "core\engine\models\nmt\marian-zh-en"

if (-not (Test-Path $whisperModelDir)) {
    Write-Host "[ERROR] Whisper ASR model directory not found: $whisperModelDir" -ForegroundColor Red
    Write-Host "[INFO] Please download Whisper model" -ForegroundColor Yellow
    exit 1
}
Write-Host "[OK] Whisper ASR model found" -ForegroundColor Green

if (-not (Test-Path $nmtModelDir)) {
    Write-Host "[ERROR] Marian NMT model directory not found: $nmtModelDir" -ForegroundColor Red
    Write-Host "[INFO] Please export Marian NMT model" -ForegroundColor Yellow
    exit 1
}
Write-Host "[OK] Marian NMT model found" -ForegroundColor Green

# Run the test
Write-Host ""
Write-Host "[3/3] Running S2S test..." -ForegroundColor Yellow
Write-Host ""

$engineDir = "core\engine"
Push-Location $engineDir
try {
    # Convert Windows path to relative path if needed
    $relativePath = $inputFile
    if ([System.IO.Path]::IsPathRooted($inputFile)) {
        $repoRoot = (Get-Location).Path
        $relativePath = Resolve-Path $inputFile -Relative
    }
    
    # Run the test (use simple version to avoid linker issues)
    cargo run --example test_s2s_full_simple -- $relativePath
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "[OK] Test completed successfully!" -ForegroundColor Green
        Write-Host "[INFO] Output file: test_output\s2s_full_real_test.wav" -ForegroundColor Cyan
    } else {
        Write-Host ""
        Write-Host "[ERROR] Test failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

