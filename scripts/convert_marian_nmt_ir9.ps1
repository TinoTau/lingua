# Convert Marian NMT models from IR 10 to IR 9
# This fixes the ONNX Runtime compatibility issue

$ErrorActionPreference = "Continue"

Write-Host "=== Converting Marian NMT Models to IR 9 ===" -ForegroundColor Cyan
Write-Host ""

$projectRoot = Split-Path -Parent $PSScriptRoot
$modelDir = Join-Path $projectRoot "core\engine\models\nmt\marian-zh-en"

if (-not (Test-Path $modelDir)) {
    Write-Host "[ERROR] Model directory not found: $modelDir" -ForegroundColor Red
    exit 1
}

$encoderModel = Join-Path $modelDir "encoder_model.onnx"
$decoderModel = Join-Path $modelDir "model.onnx"

# Check if models exist
if (-not (Test-Path $encoderModel)) {
    Write-Host "[ERROR] Encoder model not found: $encoderModel" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $decoderModel)) {
    Write-Host "[ERROR] Decoder model not found: $decoderModel" -ForegroundColor Red
    exit 1
}

Write-Host "[INFO] Model directory: $modelDir" -ForegroundColor Green
Write-Host ""

# Convert encoder model
Write-Host "[1/2] Converting encoder model..." -ForegroundColor Yellow
$encoderBackup = $encoderModel + ".ir10.backup"
if (-not (Test-Path $encoderBackup)) {
    Copy-Item $encoderModel $encoderBackup
    Write-Host "  [INFO] Backup created: $encoderBackup" -ForegroundColor Gray
}

python scripts\convert_onnx_ir9.py `
    --input_model $encoderModel `
    --output_model $encoderModel

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Failed to convert encoder model" -ForegroundColor Red
    exit 1
}

Write-Host "[OK] Encoder model converted" -ForegroundColor Green
Write-Host ""

# Convert decoder model
Write-Host "[2/2] Converting decoder model..." -ForegroundColor Yellow
$decoderBackup = $decoderModel + ".ir10.backup"
if (-not (Test-Path $decoderBackup)) {
    Copy-Item $decoderModel $decoderBackup
    Write-Host "  [INFO] Backup created: $decoderBackup" -ForegroundColor Gray
}

python scripts\convert_onnx_ir9.py `
    --input_model $decoderModel `
    --output_model $decoderModel

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Failed to convert decoder model" -ForegroundColor Red
    exit 1
}

Write-Host "[OK] Decoder model converted" -ForegroundColor Green
Write-Host ""

Write-Host "=== Conversion completed ===" -ForegroundColor Green
Write-Host ""
Write-Host "Backup files:"
Write-Host "  - $encoderBackup"
Write-Host "  - $decoderBackup"
Write-Host ""
Write-Host "Next step: Run the test again"

