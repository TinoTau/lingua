# Ensure we fail fast on errors
$ErrorActionPreference = "Stop"

Write-Host "=== STEP 0: Ensure LLVM/clang is installed and in PATH ==="

# Try common install locations for LLVM
$llvmCandidates = @(
    "C:\Program Files\LLVM\bin",
    "C:\Program Files (x86)\LLVM\bin"
)

$llvmBin = $null
foreach ($p in $llvmCandidates) {
    if (Test-Path (Join-Path $p "clang.exe")) {
        $llvmBin = $p
        break
    }
}

if (-not $llvmBin) {
    Write-Warning "clang.exe not found in common locations. Trying to install LLVM via winget..."
    try {
        winget install -e --id LLVM.LLVM -h 0
    } catch {
        Write-Warning "Failed to install LLVM via winget. Please install LLVM manually from https://github.com/llvm/llvm-project/releases"
    }

    foreach ($p in $llvmCandidates) {
        if (Test-Path (Join-Path $p "clang.exe")) {
            $llvmBin = $p
            break
        }
    }
}

if ($llvmBin) {
    Write-Host "Found LLVM bin directory: $llvmBin"

    # Add to current session PATH
    if (-not $env:PATH.ToLower().Contains($llvmBin.ToLower())) {
        $env:PATH += ";$llvmBin"
        Write-Host "Added LLVM bin to current session PATH."
    }

    # Add to user PATH (permanent for new shells)
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath.ToLower().Contains($llvmBin.ToLower())) {
        if ([string]::IsNullOrEmpty($userPath)) {
            $newPath = $llvmBin
        } else {
            $newPath = "$userPath;$llvmBin"
        }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Added LLVM bin to USER PATH (will take effect in new terminals)."
    }
} else {
    Write-Warning "Could not find clang.exe. Later steps may fail. Please check LLVM installation."
}

Write-Host ""
Write-Host "=== STEP 1: Check clang and ninja ==="

try {
    clang --version
} catch {
    Write-Warning "clang is still not available. Please verify LLVM installation and PATH."
}

try {
    ninja --version
    Write-Host "Ninja is already installed."
} catch {
    Write-Host "Ninja is not installed. Installing Ninja via winget..."
    try {
        winget install -e --id Ninja-build.Ninja -h 0
    } catch {
        Write-Warning "Failed to install Ninja via winget. Please install Ninja manually from https://github.com/ninja-build/ninja/releases"
    }
}

Write-Host ""
Write-Host "=== STEP 2: Clone or update whisper.cpp ==="

$root = Get-Location
$thirdPartyDir = Join-Path $root "third_party"
if (-not (Test-Path $thirdPartyDir)) {
    New-Item -ItemType Directory -Path $thirdPartyDir | Out-Null
}

$whisperDir = Join-Path $thirdPartyDir "whisper.cpp"
if (-not (Test-Path $whisperDir)) {
    git clone https://github.com/ggerganov/whisper.cpp.git $whisperDir
} else {
    Write-Host "whisper.cpp already exists. Updating with git pull..."
    Set-Location $whisperDir
    git pull
    Set-Location $root
}

Write-Host ""
Write-Host "=== STEP 3: Build whisper.cpp with CMake + Ninja (Release) ==="

$whisperBuild = Join-Path $whisperDir "build"

cmake -S $whisperDir -B $whisperBuild `
      -G Ninja `
      -DWHISPER_BUILD_EXAMPLES=ON

cmake --build $whisperBuild --config Release

Write-Host ""
Write-Host "=== DONE ==="
Write-Host "whisper.cpp build directory: $whisperBuild"
Write-Host "You can now start adding whisper_binding and Rust FFI."
