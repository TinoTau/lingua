# Safe Git Add Script
# Skips inaccessible directories like WSL virtual environments

Write-Host "=== Safe Git Add ===" -ForegroundColor Green
Write-Host ""

# Step 1: Add config files
Write-Host "Step 1: Adding config files..." -ForegroundColor Yellow
git add .gitignore .gitattributes 2>&1 | Out-Null

# Step 2: Add other files with error ignoring
Write-Host "Step 2: Adding other files (skipping inaccessible directories)..." -ForegroundColor Yellow
git add --ignore-errors . 2>&1 | Out-Null

# Step 3: Check status
Write-Host ""
Write-Host "Step 3: Checking staged files..." -ForegroundColor Yellow
$staged = git diff --cached --name-only
if ($staged) {
    Write-Host "[OK] Files added to staging area:" -ForegroundColor Green
    $staged | Select-Object -First 10 | ForEach-Object { Write-Host "  $_" -ForegroundColor White }
    if ($staged.Count -gt 10) {
        Write-Host "  ... and $($staged.Count - 10) more files" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "You can now run: git commit -m 'your commit message'" -ForegroundColor Cyan
} else {
    Write-Host "[WARN] No files were added to staging area" -ForegroundColor Yellow
    Write-Host "Please check if there are new files to commit" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Note: CRLF warnings are normal, Git will auto-convert line endings" -ForegroundColor Gray
Write-Host "Note: venv-wsl errors have been ignored" -ForegroundColor Gray
