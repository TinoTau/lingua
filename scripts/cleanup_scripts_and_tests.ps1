# Cleanup scripts and test files
# Date: 2024-12-19

$rootDir = "D:\Programs\github\lingua"
$scriptsDir = "$rootDir\scripts"

Set-Location $rootDir

# Files to delete with reasons
$filesToDelete = @(
    # Documentation cleanup scripts (completed tasks)
    @{
        File = "scripts\cleanup_docs.ps1"
        Reason = "Documentation cleanup completed"
    },
    @{
        File = "scripts\cleanup_docs_manual.md"
        Reason = "Documentation cleanup completed"
    },
    @{
        File = "scripts\add_dates_to_docs.ps1"
        Reason = "Date addition completed"
    },
    @{
        File = "scripts\delete_redundant_docs.ps1"
        Reason = "Redundant docs deletion completed"
    },
    @{
        File = "scripts\delete_additional_redundant_docs.ps1"
        Reason = "Additional redundant docs deletion completed"
    },
    
    # Diagnostic scripts (no longer needed)
    @{
        File = "scripts\diagnose_cargo_stall.ps1"
        Reason = "Compilation stall issue resolved"
    },
    @{
        File = "scripts\diagnose_cargo_stall_simple.ps1"
        Reason = "Compilation stall issue resolved"
    },
    @{
        File = "scripts\quick_check.ps1"
        Reason = "Quick check completed, no longer needed"
    },
    
    # Download scripts (models already downloaded)
    @{
        File = "scripts\download_mms_tts_zho.ps1"
        Reason = "Model already downloaded, manual guide available if needed"
    },
    @{
        File = "scripts\download_mms_tts_zho_manual.md"
        Reason = "Model already downloaded, manual guide no longer needed"
    },
    @{
        File = "scripts\download_vits_zh_aishell3.ps1"
        Reason = "Model already downloaded, manual guide available if needed"
    },
    @{
        File = "scripts\download_vits_zh_aishell3_manual.md"
        Reason = "Model already downloaded, manual guide no longer needed"
    },
    @{
        File = "scripts\download_sherpa_onnx_vits_zh.ps1"
        Reason = "Model already downloaded"
    },
    @{
        File = "scripts\download_sherpa_onnx_vits_zh.sh"
        Reason = "Model already downloaded"
    },
    @{
        File = "scripts\download_whisper_ggml.ps1"
        Reason = "Model already downloaded"
    },
    
    # Test scripts for failed models (no longer needed)
    @{
        File = "scripts\test_breeze2_vits_detailed.py"
        Reason = "Breeze2-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\test_breeze2_vits_inference.py"
        Reason = "Breeze2-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\test_breeze2_vits_long_sentences.py"
        Reason = "Breeze2-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\check_breeze2_vits_model.py"
        Reason = "Breeze2-VITS model checking completed, model not usable"
    },
    @{
        File = "scripts\test_sherpa_onnx_vits_final.py"
        Reason = "Sherpa-ONNX-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\test_sherpa_onnx_vits_inference.py"
        Reason = "Sherpa-ONNX-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\test_sherpa_onnx_vits_multiple_formats.py"
        Reason = "Sherpa-ONNX-VITS model testing completed, model not usable"
    },
    @{
        File = "scripts\check_sherpa_onnx_vits_model.py"
        Reason = "Sherpa-ONNX-VITS model checking completed, model not usable"
    },
    @{
        File = "scripts\test_vits_zh_aishell3_correct_inference.py"
        Reason = "vits-zh-aishell3 model testing completed, model not usable"
    },
    @{
        File = "scripts\test_vits_zh_aishell3_parameter_tuning.py"
        Reason = "vits-zh-aishell3 model testing completed, model not usable"
    },
    @{
        File = "scripts\test_vits_zh_aishell3_with_different_speaker.py"
        Reason = "vits-zh-aishell3 model testing completed, model not usable"
    },
    @{
        File = "scripts\test_vits_zh_tokenizer.py"
        Reason = "vits-zh-aishell3 tokenizer testing completed"
    },
    @{
        File = "scripts\check_vits_zh_aishell3_model.py"
        Reason = "vits-zh-aishell3 model checking completed, model not usable"
    },
    @{
        File = "scripts\check_vits_zh_aishell3_manual.md"
        Reason = "vits-zh-aishell3 manual guide no longer needed"
    },
    @{
        File = "scripts\test_hifigan_model.py"
        Reason = "HiFiGAN model testing completed, model not usable"
    },
    @{
        File = "scripts\test_different_tokenizer_formats.py"
        Reason = "Tokenizer format testing completed"
    },
    
    # Debug scripts (debugging completed)
    @{
        File = "scripts\debug_vits_tokenizer.py"
        Reason = "VITS tokenizer debugging completed"
    },
    @{
        File = "scripts\debug_vits_zh_tokenizer_detailed.py"
        Reason = "VITS Chinese tokenizer debugging completed"
    },
    @{
        File = "scripts\compare_tokenizer_with_original.py"
        Reason = "Tokenizer comparison completed"
    },
    @{
        File = "scripts\compare_symbols_with_tokens.py"
        Reason = "Symbols comparison completed"
    },
    
    # Check scripts (checking completed)
    @{
        File = "scripts\check_original_vits_implementation.md"
        Reason = "Original VITS implementation check completed"
    },
    @{
        File = "scripts\check_original_vits_manual.md"
        Reason = "Original VITS manual check completed"
    },
    @{
        File = "scripts\fetch_original_vits_tokenizer.py"
        Reason = "Original VITS tokenizer fetch completed"
    },
    @{
        File = "scripts\fetch_more_vits_files.py"
        Reason = "VITS files fetch completed"
    },
    
    # Other completed tasks
    @{
        File = "scripts\create_phone_id_map.py"
        Reason = "Phone ID map creation completed (not needed for VITS)"
    },
    @{
        File = "scripts\test_emotion_ir9.py"
        Reason = "Emotion IR9 testing completed"
    },
    @{
        File = "scripts\test_marian_decoder_kv_cache.py"
        Reason = "Marian decoder KV cache testing completed"
    },
    @{
        File = "scripts\verify_plan1_code_issues.py"
        Reason = "Plan1 code verification completed"
    },
    @{
        File = "scripts\fix_linker_error.ps1"
        Reason = "Linker error fix completed, issue resolved"
    },
    @{
        File = "scripts\check_tts_files_simple.ps1"
        Reason = "Checks FastSpeech2 model which is no longer used"
    },
    
    # Duplicate files
    @{
        File = "setup_whisper_env.ps1"
        Reason = "Duplicate of power_shell_scripts\setup_whisper_env.ps1"
    }
)

Write-Host "Files to be deleted:"
Write-Host ""
$totalSize = 0
foreach ($item in $filesToDelete) {
    $filePath = $item.File
    $reason = $item.Reason
    $fullPath = Join-Path $rootDir $filePath
    if (Test-Path $fullPath) {
        $size = (Get-Item $fullPath).Length
        $totalSize += $size
        Write-Host "  - $filePath ($size bytes)"
        Write-Host "    Reason: $reason"
    } else {
        Write-Host "  - $filePath (file not found, skipping)"
    }
}

Write-Host ""
Write-Host "Total size: $([math]::Round($totalSize / 1KB, 2)) KB"
Write-Host ""
Write-Host "Note: The following files are KEPT because they may still be useful:"
Write-Host "  - export_*.py: Model export scripts (may be needed for future models)"
Write-Host "  - convert_*.py: Model conversion scripts (may be needed)"
Write-Host "  - test_mms_tts_onnx.py: Working TTS model test (reference)"
Write-Host "  - test_tts_models.py: TTS model testing (may be needed)"
Write-Host "  - test_tts_manual.ps1: Manual TTS testing (may be needed)"
Write-Host "  - check_tts_model_io.py: TTS model I/O checking (may be needed)"
Write-Host "  - install_onnxruntime.*: Installation scripts (may be needed)"
Write-Host "  - check_onnx_model.py: General ONNX model checking (useful)"
Write-Host "  - check_model_io_detailed.py: Model I/O checking (useful)"
Write-Host "  - test_tts_manual.ps1: Manual TTS testing (may be needed)"
Write-Host ""

$confirm = Read-Host "Confirm deletion? (y/N)"
if ($confirm -eq "y" -or $confirm -eq "Y") {
    $deletedCount = 0
    foreach ($item in $filesToDelete) {
        $filePath = $item.File
        $fullPath = Join-Path $rootDir $filePath
        if (Test-Path $fullPath) {
            Remove-Item $fullPath -Force
            Write-Host "Deleted: $filePath"
            $deletedCount++
        }
    }
    Write-Host ""
    Write-Host "Deletion complete: $deletedCount files deleted"
} else {
    Write-Host "Deletion cancelled"
}

