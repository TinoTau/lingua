# WSL2 Environment Setup Script
# Purpose: Check and install WSL2 + Ubuntu if not already installed

$ErrorActionPreference = "Continue"

Write-Host "=== WSL2 Environment Check and Installation ===" -ForegroundColor Cyan
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[ERROR] This script requires administrator privileges." -ForegroundColor Red
    Write-Host "Please right-click PowerShell and select Run as administrator." -ForegroundColor Red
    exit 1
}

# Check if WSL is installed
Write-Host "[1/4] Checking WSL status..." -ForegroundColor Yellow
$wslCheckResult = $null
$wslExitCode = 0
try {
    $wslCheckResult = wsl -l -v 2>&1
    $wslExitCode = $LASTEXITCODE
} catch {
    $wslExitCode = 1
}

# Check if WSL is installed
$wslInstalled = $true
if ($wslExitCode -ne 0) {
    $wslInstalled = $false
} else {
    $wslOutputStr = $wslCheckResult | Out-String
    if ($wslOutputStr -match "has no installed" -or $wslOutputStr.Trim() -eq "") {
        $wslInstalled = $false
    }
}

if (-not $wslInstalled) {
    Write-Host "[INFO] WSL is not installed, starting installation..." -ForegroundColor Yellow
    Write-Host "[INFO] Executing: wsl --install" -ForegroundColor Gray
    wsl --install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] WSL installation failed, exit code: $LASTEXITCODE" -ForegroundColor Red
        exit 1
    }
    Write-Host "[OK] WSL installation completed." -ForegroundColor Green
    Write-Host "Please restart your computer and run this script again." -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "[OK] WSL is installed" -ForegroundColor Green
    if ($wslCheckResult) {
        Write-Host $wslCheckResult
    }
}

# Check WSL version
Write-Host ""
Write-Host "[2/4] Checking WSL version..." -ForegroundColor Yellow
$wslList = $null
try {
    $wslList = wsl -l -v 2>&1 | Out-String
} catch {
    $wslList = ""
}

# Check for WSL2
$hasWsl2 = $false
if ($wslList) {
    $hasWsl2 = $wslList -match "VERSION\s+2" -or $wslList -match "\s+2\s+" -or $wslList -match "2\s+Running"
}

if (-not $hasWsl2) {
    Write-Host "[WARN] WSL1 detected or version unclear, attempting to upgrade to WSL2..." -ForegroundColor Yellow
    $distroName = $null
    if ($wslList) {
        $lines = $wslList -split "`n"
        foreach ($line in $lines) {
            if ($line -match "^\s*(\w+)" -and $line -notmatch "NAME" -and $line -notmatch "---") {
                $distroName = $matches[1]
                break
            }
        }
    }
    
    if ($distroName) {
        Write-Host "[INFO] Upgrading distribution to WSL2..." -ForegroundColor Gray
        wsl --set-version $distroName 2
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[WARN] WSL2 upgrade may have failed, but continuing..." -ForegroundColor Yellow
        } else {
            Write-Host "[OK] Upgraded to WSL2" -ForegroundColor Green
        }
    } else {
        Write-Host "[WARN] Cannot identify distribution name, skipping upgrade" -ForegroundColor Yellow
        Write-Host "[INFO] You may need to manually upgrade to WSL2" -ForegroundColor Gray
    }
} else {
    Write-Host "[OK] Using WSL2" -ForegroundColor Green
}

# Check if Ubuntu is installed
Write-Host ""
Write-Host "[3/4] Checking Ubuntu distribution..." -ForegroundColor Yellow
$hasUbuntu = $false
if ($wslList) {
    $hasUbuntu = $wslList -match "Ubuntu"
}

if (-not $hasUbuntu) {
    Write-Host "[INFO] Ubuntu not detected, starting installation..." -ForegroundColor Yellow
    Write-Host "[INFO] Executing: wsl --install -d Ubuntu" -ForegroundColor Gray
    wsl --install -d Ubuntu
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Ubuntu installation failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "[OK] Ubuntu installation completed." -ForegroundColor Green
    Write-Host "Please start Ubuntu for the first time and set up username and password." -ForegroundColor Yellow
    Write-Host "After setup, run this script again to continue." -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "[OK] Ubuntu is installed" -ForegroundColor Green
}

# Verify WSL2 running status
Write-Host ""
Write-Host "[4/4] Verifying WSL2 running status..." -ForegroundColor Yellow
$wslState = $null
try {
    $wslState = wsl -l -v 2>&1 | Out-String
} catch {
    $wslState = ""
}

$isRunning = $false
if ($wslState) {
    $isRunning = $wslState -match "Running"
}

if (-not $isRunning) {
    Write-Host "[INFO] WSL2 is not running, attempting to start..." -ForegroundColor Yellow
    try {
        wsl -d Ubuntu -- echo "WSL2 is ready" 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] WSL2 started successfully" -ForegroundColor Green
        } else {
            Write-Host "[WARN] WSL2 may not be running, but continuing..." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "[WARN] Could not start WSL2, but continuing..." -ForegroundColor Yellow
    }
} else {
    Write-Host "[OK] WSL2 is running normally" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== WSL2 Environment Setup Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Run: wsl bash scripts/wsl2_piper/install_piper_in_wsl.sh"
Write-Host "  2. Or manually execute installation commands in WSL2 terminal"
Write-Host ""
