# Restore marian-zh-en original IR 10 models from backup
# This script restores the original models before the IR version downgrade attempt

$ErrorActionPreference = "Continue"

Write-Host "=== Restore marian-zh-en Original Models ===" -ForegroundColor Cyan
Write-Host ""

$projectRoot = Split-Path -Parent $PSScriptRoot
$modelDir = Join-Path $projectRoot "core\engine\models\nmt\marian-zh-en"

$backupEncoder = Join-Path $modelDir "encoder_model.onnx.ir10.backup"
$backupDecoder = Join-Path $modelDir "model.onnx.ir10.backup"
$encoderModel = Join-Path $modelDir "encoder_model.onnx"
$decoderModel = Join-Path $modelDir "model.onnx"

# Check if backup files exist
if (-not (Test-Path $backupEncoder)) {
    Write-Host "[ERROR] Backup file not found: $backupEncoder" -ForegroundColor Red
    Write-Host "[INFO] Original model may not have been backed up" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $backupDecoder)) {
    Write-Host "[ERROR] Backup file not found: $backupDecoder" -ForegroundColor Red
    Write-Host "[INFO] Original model may not have been backed up" -ForegroundColor Yellow
    exit 1
}

# Restore encoder model
Write-Host "[1/2] Restoring encoder model..." -ForegroundColor Yellow
Copy-Item $backupEncoder $encoderModel -Force
if ($LASTEXITCODE -eq 0 -or $?) {
    Write-Host "[OK] Encoder model restored" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Failed to restore encoder model" -ForegroundColor Red
    exit 1
}

# Restore decoder model
Write-Host "[2/2] Restoring decoder model..." -ForegroundColor Yellow
Copy-Item $backupDecoder $decoderModel -Force
if ($LASTEXITCODE -eq 0 -or $?) {
    Write-Host "[OK] Decoder model restored" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Failed to restore decoder model" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Verification ===" -ForegroundColor Cyan
Write-Host "Checking model IR versions..." -ForegroundColor Gray

# Verify encoder model
try {
    $encoderInfo = python -c "import onnx; m = onnx.load('$encoderModel'); print(f'IR {m.ir_version}, Opset {m.opset_import[0].version}')" 2>&1
    Write-Host "Encoder: $encoderInfo" -ForegroundColor Green
} catch {
    Write-Host "[WARNING] Could not verify encoder model" -ForegroundColor Yellow
}

# Verify decoder model
try {
    $decoderInfo = python -c "import onnx; m = onnx.load('$decoderModel'); print(f'IR {m.ir_version}, Opset {m.opset_import[0].version}')" 2>&1
    Write-Host "Decoder: $decoderInfo" -ForegroundColor Green
} catch {
    Write-Host "[WARNING] Could not verify decoder model" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Restoration completed ===" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Upgrade ONNX Runtime to support IR 10" -ForegroundColor White
Write-Host "  2. Test all functionality (including previous features)" -ForegroundColor White
Write-Host "  3. Run S2S full test: cargo run --example test_s2s_full_simple" -ForegroundColor White

