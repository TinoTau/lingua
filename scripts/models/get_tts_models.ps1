param(
  [string]$BaseRoot = "D:\work\Lingua\tts311"  
)

$ErrorActionPreference = "Stop"

function Info($m){ Write-Host "[INFO]  $m" -ForegroundColor Cyan }
function Ok($m){ Write-Host   "[OK]   $m" -ForegroundColor Green }
function Warn($m){ Write-Host "[WARN]  $m" -ForegroundColor Yellow }
function Fail($m){ Write-Host "[FAIL]  $m" -ForegroundColor Red }

# ---- Paths ----
$TtsRoot   = Join-Path $BaseRoot "tts"
$FS2Dir    = Join-Path $TtsRoot  "fastspeech2-lite"
$HFGDir    = Join-Path $TtsRoot  "hifigan-lite"
$TmpDir    = Join-Path $TtsRoot  "_tmp"

New-Item -ItemType Directory -Force -Path $FS2Dir | Out-Null
New-Item -ItemType Directory -Force -Path $HFGDir | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

# ---- Model URLs (official PaddleSpeech Released Models) ----
$urls = @{
  # FastSpeech2 (Chinese, CSMSC) — lite/streaming ONNX
  "fs2_csmsc"   = "https://paddlespeech.bj.bcebos.com/Parakeet/released_models/fastspeech2/fastspeech2_cnndecoder_csmsc_streaming_onnx_1.0.0.zip";
  # FastSpeech2 (English, LJSpeech) — ONNX
  "fs2_ljs"     = "https://paddlespeech.bj.bcebos.com/Parakeet/released_models/fastspeech2/fastspeech2_ljspeech_onnx_1.1.0.zip";
  # HiFiGAN (Chinese, CSMSC) — ONNX
  "hfg_csmsc"   = "https://paddlespeech.bj.bcebos.com/Parakeet/released_models/hifigan/hifigan_csmsc_onnx_0.2.0.zip";
  # HiFiGAN (English, LJSpeech) — ONNX
  "hfg_ljs"     = "https://paddlespeech.bj.bcebos.com/Parakeet/released_models/hifigan/hifigan_ljspeech_onnx_1.1.0.zip";
}

function DownloadFile($url, $outPath) {
  Info "Downloading: $url"
  try {
    Invoke-WebRequest -Uri $url -OutFile $outPath -UseBasicParsing
  } catch {
    Warn "Invoke-WebRequest failed, retrying with BitsTransfer..."
    Start-BitsTransfer -Source $url -Destination $outPath
  }
  if (-not (Test-Path $outPath)) { throw "Download failed: $url" }
}

function DownloadAndExtract($url, $destDir) {
  $zipName = Split-Path -Leaf $url
  $zipPath = Join-Path $TmpDir $zipName
  DownloadFile -url $url -outPath $zipPath
  Info "Extracting: $zipName"
  Expand-Archive -Path $zipPath -DestinationPath $destDir -Force
  Remove-Item $zipPath -Force
}

# ---- Download & extract ----
$work = @{
  "fs2_csmsc" = @{ url = $urls["fs2_csmsc"]; out = (Join-Path $TmpDir "fs2_csmsc") };
  "fs2_ljs"   = @{ url = $urls["fs2_ljs"];   out = (Join-Path $TmpDir "fs2_ljs") };
  "hfg_csmsc" = @{ url = $urls["hfg_csmsc"]; out = (Join-Path $TmpDir "hfg_csmsc") };
  "hfg_ljs"   = @{ url = $urls["hfg_ljs"];   out = (Join-Path $TmpDir "hfg_ljs") };
}

foreach ($k in $work.Keys) {
  $item = $work[$k]
  New-Item -ItemType Directory -Force -Path $item.out | Out-Null
  DownloadAndExtract -url $item.url -destDir $item.out
}

# ---- Collect ONNX & configs into final folders ----
function CollectTo($sourceDir, $targetDir, $baseName) {
  $onnx = Get-ChildItem -Path $sourceDir -Recurse -Filter *.onnx -ErrorAction SilentlyContinue
  if ($onnx.Count -eq 0) { Warn "No ONNX found under $sourceDir" } else {
    $destOnnx = Join-Path $targetDir ("{0}.onnx" -f $baseName)
    Copy-Item -Path $onnx[0].FullName -Destination $destOnnx -Force
    Ok "ONNX -> $destOnnx"
  }
  $cfgs = Get-ChildItem -Path $sourceDir -Recurse -Include *.json,*.yaml,*.yml -ErrorAction SilentlyContinue
  foreach ($c in $cfgs) {
    Copy-Item -Path $c.FullName -Destination (Join-Path $targetDir $c.Name) -Force
  }
}

# FastSpeech2 (put into tts/fastspeech2-lite)
CollectTo -sourceDir $work["fs2_csmsc"].out -targetDir $FS2Dir -baseName "fastspeech2_csmsc_streaming"
CollectTo -sourceDir $work["fs2_ljs"].out   -targetDir $FS2Dir -baseName "fastspeech2_ljspeech"

# HiFiGAN (put into tts/hifigan-lite)
CollectTo -sourceDir $work["hfg_csmsc"].out -targetDir $HFGDir -baseName "hifigan_csmsc"
CollectTo -sourceDir $work["hfg_ljs"].out   -targetDir $HFGDir -baseName "hifigan_ljspeech"

# ---- Cleanup temp ----
Remove-Item $TmpDir -Recurse -Force

Ok "All models prepared."
Write-Host "Placed under: $TtsRoot" -ForegroundColor Magenta
Write-Host " - FastSpeech2-lite: $FS2Dir" -ForegroundColor Magenta
Write-Host " - HiFiGAN-lite    : $HFGDir" -ForegroundColor Magenta
