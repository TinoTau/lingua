param(
  [string]$BaseRoot = "D:\work\Lingua\persona"  # 根目录，脚本会在其下创建 persona/ 和 vad/
)

$ErrorActionPreference = "Stop"
function Info($m){ Write-Host "[INFO]  $m" -ForegroundColor Cyan }
function Ok($m){ Write-Host   "[OK]   $m" -ForegroundColor Green }
function Warn($m){ Write-Host "[WARN]  $m" -ForegroundColor Yellow }

# ---------- 目录结构 ----------
$PersonaRoot = Join-Path $BaseRoot "persona"
$EmbRoot     = Join-Path $PersonaRoot "embeddings"
$EmbMiniLM   = Join-Path $EmbRoot "all-MiniLM-L6-v2"
$VadRoot     = Join-Path $BaseRoot "vad"
$VadSilero   = Join-Path $VadRoot "silero-vad"

New-Item -ItemType Directory -Force -Path $EmbMiniLM | Out-Null
New-Item -ItemType Directory -Force -Path $VadSilero | Out-Null

# ---------- 下载工具 ----------
function Download($Url, $OutPath){
  Info "Downloading: $Url"
  try {
    Invoke-WebRequest -Uri $Url -OutFile $OutPath -UseBasicParsing
  } catch {
    Warn "Invoke-WebRequest failed, retrying with BitsTransfer..."
    Start-BitsTransfer -Source $Url -Destination $OutPath
  }
  if (-not (Test-Path $OutPath)) { throw "Download failed: $Url" }
  Ok   "Saved: $OutPath"
}

# ---------- Persona Embedding ----------
$EmbFiles = @{
  "model.onnx"              = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx";
  "tokenizer.json"          = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/tokenizer.json";
  "config.json"             = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/config.json";
  "special_tokens_map.json" = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/special_tokens_map.json";
  "tokenizer_config.json"   = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/tokenizer_config.json";
  "vocab.txt"               = "https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/vocab.txt"
}
foreach($name in $EmbFiles.Keys){
  $out = Join-Path $EmbMiniLM $name
  Download -Url $EmbFiles[$name] -OutPath $out
}
Set-Content -Path (Join-Path $EmbMiniLM "README.txt") -Value @"
This directory contains the ONNX port of Sentence-Transformers all-MiniLM-L6-v2.
Use ONNX Runtime for inference; embedding size = 384.
"@ -Encoding UTF8

# ---------- VAD ----------
$SileroOnnx = "https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx"
Download -Url $SileroOnnx -OutPath (Join-Path $VadSilero "silero_vad.onnx")
Set-Content -Path (Join-Path $VadSilero "README.txt") -Value @"
This directory contains Silero VAD ONNX model.
Run with ONNX Runtime. For WebRTC VAD, no model file is needed.
"@ -Encoding UTF8

Ok "Persona embeddings and VAD models prepared."
Write-Host "Persona:  $EmbMiniLM" -ForegroundColor Magenta
Write-Host "VAD:      $VadSilero" -ForegroundColor Magenta
