
# ==============================================
# setup_tts_env_conda_v5.ps1
# Pure-Conda setup with resilient per-package install (no pip, no CMake)
# - Installs base packages one-by-one with soft-fail (doesn't abort on missing pkgs)
# - Quotes special version specs automatically
# - Keeps PyTorch CPU logic with graceful fallbacks
# ==============================================

param(
    [string]$EnvName = "tts_env",
    [string]$PyVer   = "3.10",
    [switch]$ForceRecreate
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Info($m){ Write-Host "[INFO]  $m" -ForegroundColor Cyan }
function Ok($m){ Write-Host "[OK]    $m" -ForegroundColor Green }
function Warn($m){ Write-Host "[WARN]  $m" -ForegroundColor Yellow }
function Fail($m){ Write-Host "[FAIL]  $m" -ForegroundColor Red }

$condaCmd = Get-Command conda -ErrorAction SilentlyContinue
if (-not $condaCmd) { Fail "Conda not found in PATH."; throw }
$CondaPath = $condaCmd.Source
Info "Using conda at: $CondaPath"

function RunConda {
    param([Parameter(Mandatory=$true)][string[]]$Args,[switch]$SoftFail)
    Info "conda $($Args -join ' ')"
    & $CondaPath @Args
    if ($LASTEXITCODE -ne 0) {
        if ($SoftFail) {
            Warn "conda command failed (soft): $($Args -join ' ')"
            return $false
        } else {
            throw "conda command failed: $($Args -join ' ')"
        }
    }
    return $true
}

# Speed up & configure channels
try { RunConda @("clean","--lock","-y") | Out-Null } catch { }
try { RunConda @("config","--set","solver","libmamba") | Out-Null } catch { Warn "libmamba not supported; continuing." }
RunConda @("config","--set","channel_priority","strict") | Out-Null
RunConda @("config","--prepend","channels","conda-forge") | Out-Null
RunConda @("config","--set","show_channel_urls","yes") | Out-Null

# Env create / recreate
$envListJson = & $CondaPath env list --json | ConvertFrom-Json
$exists = $false
foreach ($p in $envListJson.envs) { if ([IO.Path]::GetFileName($p) -eq $EnvName) { $exists = $true; break } }
if ($ForceRecreate -and $exists) {
    Warn "Removing env $EnvName ..."
    RunConda @("remove","-y","-n",$EnvName,"--all") | Out-Null
    $exists = $false
}
if (-not $exists) {
    RunConda @("create","-y","-n",$EnvName,"python=$PyVer") | Out-Null
} else {
    Info "Env '$EnvName' exists."
}

# Pre-install PyYAML
RunConda @("install","-y","-n",$EnvName,"pyyaml") | Out-Null

# Resilient per-package install helper
function InstallPkg {
    param([string]$PkgSpec)
    $ok = RunConda @("install","-y","-n",$EnvName,$PkgSpec) -SoftFail
    if ($ok) { Ok "Installed $PkgSpec" } else { Warn "Skipped $PkgSpec (not available or conflict)" }
}

# Base packages (exclude 'pyworld' on Windows to avoid missing binaries)
$basePkgs = @(
    "numpy<2.2",
    "h5py=3.12.1",
    "nltk",
    "librosa=0.10.2.*",
    "sentencepiece",
    # "pyworld",  # often unavailable on win-64 conda-forge; skip to avoid hard failure
    "editdistance",
    "pypinyin",
    "typeguard",
    "configargparse",
    "opt-einsum",
    "hydra-core",
    "kaldiio"
)

foreach ($p in $basePkgs) {
    InstallPkg -PkgSpec $p
}

# PyTorch CPU with graceful fallbacks
$torchOK = $false
if (-not $torchOK) {
    if (RunConda @("install","-y","-n",$EnvName,"pytorch-cpu","torchaudio") -SoftFail) {
        Ok "Installed PyTorch CPU from conda-forge (pytorch-cpu + torchaudio)."
        $torchOK = $true
    } else {
        Warn "conda-forge PyTorch CPU path failed."
    }
}
if (-not $torchOK) {
    if (RunConda @("install","-y","-n",$EnvName,"-c","pytorch","pytorch","torchaudio","cpuonly") -SoftFail) {
        Ok "Installed PyTorch CPU from pytorch channel (pytorch + cpuonly + torchaudio)."
        $torchOK = $true
    } else {
        Warn "pytorch channel path failed."
    }
}
if (-not $torchOK) {
    Warn "Falling back to install PyTorch CPU only (no torchaudio)."
    if (RunConda @("install","-y","-n",$EnvName,"pytorch-cpu") -SoftFail) {
        Ok "Installed pytorch-cpu (conda-forge). torchaudio skipped."
        $torchOK = $true
    } elseif (RunConda @("install","-y","-n",$EnvName,"-c","pytorch","pytorch","cpuonly") -SoftFail) {
        Ok "Installed pytorch (pytorch channel) with cpuonly. torchaudio skipped."
        $torchOK = $true
    } else {
        Fail "Failed to install PyTorch. Please check channels/network."
        throw
    }
}

# Validation
$probe = @"
import importlib, sys
def ver(m):
    try:
        mod = importlib.import_module(m)
        v = getattr(mod, '__version__', 'n/a')
        print(f' - {m}: {v}')
    except Exception as e:
        print(f' - {m}: ERROR -> {e}')

print('Python:', sys.version.replace('\\n', ' '))
mods = ['yaml','numpy','h5py','nltk','torch','librosa','sentencepiece','editdistance','pypinyin','typeguard','configargparse','opt_einsum']
for m in mods:
    ver(m)
try:
    import torchaudio
    print(f' - torchaudio: {getattr(torchaudio, "__version__", "n/a")}')
except Exception:
    print(" - torchaudio: NOT INSTALLED (ok if not needed)")
"@
$probeFile = Join-Path $env:TEMP "probe_tts_env_conda_v5.py"
Set-Content -Path $probeFile -Value $probe -Encoding UTF8
& $CondaPath run -n $EnvName python $probeFile

Ok "Environment '$EnvName' is ready (pure conda, resilient installs)."
Write-Host ""
Write-Host "Activate with:" -ForegroundColor Magenta
Write-Host "  conda activate $EnvName" -ForegroundColor Magenta
