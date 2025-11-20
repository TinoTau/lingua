# Archive manual/documentation files script
# Purpose: Move outdated manual/documentation files to archived directory

$ErrorActionPreference = "Continue"

Write-Host "=== Archiving manual/documentation files ===" -ForegroundColor Cyan
Write-Host ""

# Create archived directory if it doesn't exist
$archivedDir = Join-Path $PSScriptRoot "archived"
if (-not (Test-Path $archivedDir)) {
    New-Item -ItemType Directory -Path $archivedDir -Force | Out-Null
    Write-Host "[INFO] Created archived directory: $archivedDir" -ForegroundColor Yellow
}

# Files to archive (outdated or redundant documentation)
$filesToArchive = @(
    # Outdated Piper manual files
    "scripts\manual_download_piper.md",
    
    # Other manual/documentation files that may be outdated
    "scripts\install_onnxruntime_simple.md",
    "scripts\README_export_models.md",
    "scripts\original_vits_code\README.md"
)

$archivedCount = 0
$notFoundCount = 0
$skippedCount = 0

foreach ($file in $filesToArchive) {
    # Build full path: scripts directory + relative file path
    $filePath = $file -replace "^scripts\\", ""
    $fullPath = Join-Path $PSScriptRoot $filePath
    
    if (Test-Path $fullPath) {
        try {
            # Get relative path for archived location
            $relativePath = $filePath
            $archivedPath = Join-Path $archivedDir $relativePath
            
            # Create subdirectory if needed
            $archivedSubDir = Split-Path $archivedPath -Parent
            if (-not (Test-Path $archivedSubDir)) {
                New-Item -ItemType Directory -Path $archivedSubDir -Force | Out-Null
            }
            
            Move-Item $fullPath $archivedPath -Force
            Write-Host "[ARCHIVED] $file -> archived/$relativePath" -ForegroundColor Green
            $archivedCount++
        } catch {
            Write-Host "[ERROR] Failed to archive $file : $_" -ForegroundColor Red
        }
    } else {
        Write-Host "[SKIP] Not found: $file" -ForegroundColor Gray
        $notFoundCount++
    }
}

Write-Host ""
Write-Host "=== Archive completed ===" -ForegroundColor Cyan
Write-Host "Archived: $archivedCount files"
Write-Host "Not found: $notFoundCount files"
Write-Host ""
Write-Host "Active documentation files (kept):"
Write-Host "  - scripts\wsl2_piper\README.md (active - WSL2 Piper guide)"

