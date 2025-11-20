# Manual TTS test script with timeout protection

$ErrorActionPreference = "Continue"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$engineDir = Join-Path $repoRoot "core\engine"

Write-Host "=== TTS Manual Test Script ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: TtsStub (no model files needed)
Write-Host "[Test 1] Testing TtsStub (no model files needed)..." -ForegroundColor Yellow
Push-Location $engineDir
try {
    $job = Start-Job -ScriptBlock { 
        param($dir) 
        Set-Location $dir
        cargo test --lib tts_stub -- --nocapture 2>&1 | Select-Object -First 30
    } -ArgumentList $engineDir
    
    if (Wait-Job $job -Timeout 30) {
        $result = Receive-Job $job
        Write-Host "[OK] Test output:" -ForegroundColor Green
        $result | ForEach-Object { Write-Host $_ }
    } else {
        Write-Host "[FAIL] Test timeout (30 seconds)" -ForegroundColor Red
        Stop-Job $job
    }
    Remove-Job $job
} catch {
    Write-Host "[ERROR] $_" -ForegroundColor Red
}
Pop-Location
Write-Host ""

# Test 2: Text Processor Load
Write-Host "[Test 2] Testing Text Processor Load..." -ForegroundColor Yellow
Push-Location $engineDir
try {
    $job = Start-Job -ScriptBlock { 
        param($dir) 
        Set-Location $dir
        cargo test --lib test_text_processor_load -- --nocapture 2>&1 | Select-Object -First 30
    } -ArgumentList $engineDir
    
    if (Wait-Job $job -Timeout 30) {
        $result = Receive-Job $job
        Write-Host "[OK] Test output:" -ForegroundColor Green
        $result | ForEach-Object { Write-Host $_ }
    } else {
        Write-Host "[FAIL] Test timeout (30 seconds)" -ForegroundColor Red
        Stop-Job $job
    }
    Remove-Job $job
} catch {
    Write-Host "[ERROR] $_" -ForegroundColor Red
}
Pop-Location
Write-Host ""

# Test 3: Model Load (requires model files)
Write-Host "[Test 3] Testing Model Load (requires model files)..." -ForegroundColor Yellow
Push-Location $engineDir
try {
    $job = Start-Job -ScriptBlock { 
        param($dir) 
        Set-Location $dir
        cargo test --lib test_tts_model_load -- --nocapture 2>&1 | Select-Object -First 30
    } -ArgumentList $engineDir
    
    if (Wait-Job $job -Timeout 60) {
        $result = Receive-Job $job
        Write-Host "[OK] Test output:" -ForegroundColor Green
        $result | ForEach-Object { Write-Host $_ }
    } else {
        Write-Host "[FAIL] Test timeout (60 seconds)" -ForegroundColor Red
        Stop-Job $job
    }
    Remove-Job $job
} catch {
    Write-Host "[ERROR] $_" -ForegroundColor Red
}
Pop-Location
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
Write-Host "For more tests, see TTS_MANUAL_TEST_COMMANDS.md" -ForegroundColor Yellow

