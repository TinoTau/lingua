# Cleanup test files script
# Purpose: Archive or delete outdated test files

$ErrorActionPreference = "Continue"

Write-Host "=== Cleaning up test files ===" -ForegroundColor Cyan
Write-Host ""

# Files to delete (outdated or redundant)
$filesToDelete = @(
    # Outdated Piper test scripts (replaced by wsl2_piper scripts)
    "scripts\test_piper_step1.ps1",
    "scripts\test_piper_step1.sh",
    "scripts\test_piper_step1_manual.md",
    "scripts\download_piper_manual.md",
    "scripts\download_piper_model_manual.md",
    
    # Outdated TTS test scripts
    "scripts\test_tts_manual.ps1",
    "scripts\test_mms_tts_onnx.py",
    "scripts\test_tts_models.py",
    
    # Outdated download scripts (replaced by wsl2_piper scripts)
    "scripts\download_piper.ps1",
    "scripts\download_piper_model.ps1",
    "scripts\download_piper_chinese_model.ps1",
    "scripts\download_piper_simple.ps1",
    "scripts\find_and_download_piper.py",
    
    # Outdated test scripts
    "scripts\test_sherpa_onnx_vits_inference.py",
    "scripts\test_sherpa_onnx_vits_multiple_formats.py"
)

$deletedCount = 0
$notFoundCount = 0

foreach ($file in $filesToDelete) {
    # Build full path: scripts directory + relative file path
    $filePath = $file -replace "^scripts\\", ""
    $fullPath = Join-Path $PSScriptRoot $filePath
    
    if (Test-Path $fullPath) {
        try {
            Remove-Item $fullPath -Force
            Write-Host "[DELETED] $file" -ForegroundColor Green
            $deletedCount++
        } catch {
            Write-Host "[ERROR] Failed to delete $file : $_" -ForegroundColor Red
        }
    } else {
        Write-Host "[SKIP] Not found: $file" -ForegroundColor Gray
        $notFoundCount++
    }
}

Write-Host ""
Write-Host "=== Cleanup completed ===" -ForegroundColor Cyan
Write-Host "Deleted: $deletedCount files"
Write-Host "Not found: $notFoundCount files"
Write-Host ""
Write-Host "Remaining test files:"
Write-Host "  - scripts\wsl2_piper\test_piper_service.ps1 (active)"
Write-Host "  - core\engine\examples\test_piper_http_simple.rs (active)"
Write-Host "  - core\engine\examples\test_s2s_flow_simple.rs (active)"

