param(
  [string]$EnvName = "tts_env"
)

$ErrorActionPreference = "Stop"
function Info($m){ Write-Host "[INFO]  $m" -ForegroundColor Cyan }
function Ok($m){ Write-Host   "[OK]   $m" -ForegroundColor Green }


conda config --set solver libmamba 2>$null | Out-Null
conda config --set channel_priority strict 2>$null | Out-Null
conda config --prepend channels conda-forge 2>$null | Out-Null

$envs = conda env list --json | ConvertFrom-Json
$exists = $false
foreach ($p in $envs.envs) {
    if ([IO.Path]::GetFileName($p) -eq $EnvName) { $exists = $true; break }
}
if (-not $exists) {
    throw "Conda env '$EnvName' not found. Please create it first."
}


Info "Installing minimal runtime deps to '$EnvName' (pure conda) ..."
conda install -y -n $EnvName onnxruntime
if ($LASTEXITCODE -ne 0) {
    throw "Failed to install onnxruntime"
}
Ok "onnxruntime installed successfully."
