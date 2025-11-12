param(
  [string]$ProjectRoot = "."
)

$ErrorActionPreference = "Stop"
function Info($m){ Write-Host "[INFO]  $m" -ForegroundColor Cyan }
function Ok($m){ Write-Host   "[OK]   $m" -ForegroundColor Green }
function Warn($m){ Write-Host "[WARN]  $m" -ForegroundColor Yellow }
function Fail($m){ Write-Host "[FAIL]  $m" -ForegroundColor Red }

# -------- Paths (recreate) --------
$Core = Join-Path $ProjectRoot "core/engine/models"
if (Test-Path $Core) { Info "Removing existing: $Core"; Remove-Item $Core -Recurse -Force }
$ASR  = Join-Path $Core "asr/whisper-base"
$NMT  = Join-Path $Core "nmt"
$EMO  = Join-Path $Core "emotion/xlm-r"
$VAD  = Join-Path $Core "vad/silero"
$EMB  = Join-Path $Core "persona/embedding-default"
$FS2  = Join-Path $Core "tts/fastspeech2-lite"
$HFG  = Join-Path $Core "tts/hifigan-lite"
$TMP  = Join-Path $ProjectRoot "_tmp_models_clean"
New-Item -ItemType Directory -Force -Path $ASR,$NMT,$EMO,$VAD,$EMB,$FS2,$HFG,$TMP | Out-Null

# -------- Utils --------
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

function Invoke-Download([string]$Url,[string]$OutPath){
  $cmd = Get-Command curl.exe -ErrorAction SilentlyContinue
  if ($null -ne $cmd) {
    & $cmd.Source -L --retry 5 --retry-delay 2 -o $OutPath $Url
  } else {
    Invoke-WebRequest -Uri $Url -OutFile $OutPath -UseBasicParsing -MaximumRedirection 10
  }
}

function Download {
  param([string]$Url,[string]$OutPath)
  if (-not ($Url -like 'http*')) { throw ("Invalid URL: " + $Url) }
  New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutPath) | Out-Null
  Info ("Downloading: " + $Url)
  try {
    Invoke-Download $Url $OutPath
  } catch {
    Fail ("Download failed: " + $Url)
    throw
  }
  if (-not (Test-Path $OutPath)) { throw ("Download failed: " + $Url) }
  Ok   ("Saved: " + $OutPath)
}

function DownloadOptional {
  param([string]$Url,[string]$OutPath)
  if (-not ($Url -like 'http*')) { return $false }
  New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutPath) | Out-Null
  Info ("(opt) Downloading: " + $Url)
  try {
    Invoke-Download $Url $OutPath
    if (-not (Test-Path $OutPath)) { throw "not saved" }
    Ok ("(opt) Saved: " + $OutPath)
    return $true
  } catch {
    Warn "(opt) Missing upstream or failed: $Url"
    return $false
  }
}

function ExtractZip { param([string]$ZipPath,[string]$Dest)
  Info ("Extracting: " + (Split-Path -Leaf $ZipPath))
  Expand-Archive -Path $ZipPath -DestinationPath $Dest -Force
}
function CopyFirst { param([string]$Pattern,[string]$From,[string]$To,[string]$Rename)
  $files = Get-ChildItem -Path $From -Recurse -Filter $Pattern -File -ErrorAction SilentlyContinue
  if ($files.Count -gt 0){
    $target = (Join-Path $To $Rename); Copy-Item -Path $files[0].FullName -Destination $target -Force
    Ok ("Placed: " + $target)
  } else { Warn ("Missing pattern '" + $Pattern + "' under " + $From) }
}
function CopyAll { param([string[]]$Patterns,[string]$From,[string]$To)
  foreach($p in $Patterns){
    Get-ChildItem -Path $From -Recurse -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -like $p } |
    ForEach-Object { $dest = Join-Path $To $_.Name; Copy-Item $_.FullName -Destination $dest -Force; Info ("Copied " + $_.Name + " -> " + $To) }
  }
}

# -------- Persona / Embedding (all-MiniLM-L6-v2 ONNX) --------
$EmbUrls = @{
  "model.onnx"              = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/model.onnx?download=true';
  "tokenizer.json"          = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/tokenizer.json?download=true';
  "config.json"             = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/config.json?download=true';
  "special_tokens_map.json" = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/special_tokens_map.json?download=true';
  "tokenizer_config.json"   = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/tokenizer_config.json?download=true';
  "vocab.txt"               = 'https://huggingface.co/onnx-models/all-MiniLM-L6-v2-onnx/resolve/main/vocab.txt?download=true'
}
foreach($name in $EmbUrls.Keys){ Download -Url $EmbUrls[$name] -OutPath (Join-Path $EMB $name) }
Set-Content -Path (Join-Path $EMB 'README.txt') -Value @'
Default persona embedding model (ONNX): all-MiniLM-L6-v2 (384-d).
Files: model.onnx, tokenizer.json, config.json, tokenizer_config.json, special_tokens_map.json, vocab.txt
'@ -Encoding UTF8

# -------- VAD (Silero ONNX) --------
$SileroUrl = 'https://huggingface.co/onnx-community/silero-vad/resolve/main/onnx/model.onnx?download=true'
Download -Url $SileroUrl -OutPath (Join-Path $VAD 'silero_vad.onnx')
Set-Content -Path (Join-Path $VAD 'README.txt') -Value @'
Silero VAD (ONNX). Run with ONNX Runtime.
'@ -Encoding UTF8

# -------- ASR / Whisper (INT8) --------
$whisperBase = 'https://huggingface.co/onnx-community/whisper-base-int8/resolve/main'
$asrRequired = @{
  "encoder_model_int8.onnx"           = "$whisperBase/encoder_model_int8.onnx?download=true";
  "decoder_with_past_model_int8.onnx" = "$whisperBase/decoder_with_past_model_int8.onnx?download=true";
  "decoder_model_int8.onnx"           = "$whisperBase/decoder_model_int8.onnx?download=true";
  "config.json"                       = "$whisperBase/config.json?download=true";
  "tokenizer.json"                    = "$whisperBase/tokenizer.json?download=true";
  "vocab.json"                        = "$whisperBase/vocab.json?download=true";
  "merges.txt"                        = "$whisperBase/merges.txt?download=true"
}
$asrOptional = @{
  "preprocessor_config.json"          = "$whisperBase/preprocessor_config.json?download=true";
  "special_tokens_map.json"           = "$whisperBase/special_tokens_map.json?download=true";
  "tokenizer_config.json"             = "$whisperBase/tokenizer_config.json?download=true"
}
foreach($f in $asrRequired.Keys){ Download -Url $asrRequired[$f] -OutPath (Join-Path $ASR $f) }
foreach($f in $asrOptional.Keys){  DownloadOptional -Url $asrOptional[$f] -OutPath (Join-Path $ASR $f) }

# -------- NMT / Marian (6 directions) --------
$marianPairs = @(
  @{ name = "marian-en-zh"; repo = "https://huggingface.co/Xenova/opus-mt-en-zh/resolve/main" },
  @{ name = "marian-zh-en"; repo = "https://huggingface.co/Xenova/opus-mt-zh-en/resolve/main" },
  @{ name = "marian-en-ja"; repo = "https://huggingface.co/Xenova/opus-mt-en-ja/resolve/main" },
  @{ name = "marian-ja-en"; repo = "https://huggingface.co/Xenova/opus-mt-ja-en/resolve/main" },
  @{ name = "marian-en-es"; repo = "https://huggingface.co/Xenova/opus-mt-en-es/resolve/main" },
  @{ name = "marian-es-en"; repo = "https://huggingface.co/Xenova/opus-mt-es-en/resolve/main" }
)
foreach($m in $marianPairs){
  $dst = Join-Path $NMT $m.name
  New-Item -ItemType Directory -Force -Path $dst | Out-Null
  # required
  $req = @{
    "model.onnx"            = "$($m.repo)/model.onnx?download=true";
    "model.onnx_data"       = "$($m.repo)/model.onnx_data?download=true";
    "source.spm"            = "$($m.repo)/source.spm?download=true";
    "target.spm"            = "$($m.repo)/target.spm?download=true";
    "tokenizer_config.json" = "$($m.repo)/tokenizer_config.json?download=true"
  }
  foreach($f in $req.Keys){ Download -Url $req[$f] -OutPath (Join-Path $dst $f) }
  # optional
  $opt = @{
    "config.json"             = "$($m.repo)/config.json?download=true";
    "special_tokens_map.json" = "$($m.repo)/special_tokens_map.json?download=true";
    "vocab.json"              = "$($m.repo)/vocab.json?download=true"
  }
  foreach($f in $opt.Keys){ DownloadOptional -Url $opt[$f] -OutPath (Join-Path $dst $f) }
}

# -------- Emotion / XLM-R --------
$emoRepo = "https://huggingface.co/Xenova/twitter-xlm-roberta-base-sentiment/resolve/main"
$emoRequired = @{
  "model.onnx"               = "$emoRepo/model.onnx?download=true";
  "model.onnx_data"          = "$emoRepo/model.onnx_data?download=true";
  "tokenizer.json"           = "$emoRepo/tokenizer.json?download=true";
  "config.json"              = "$emoRepo/config.json?download=true";
  "sentencepiece.bpe.model"  = "$emoRepo/sentencepiece.bpe.model?download=true"
}
$emoOptional = @{
  "special_tokens_map.json"  = "$emoRepo/special_tokens_map.json?download=true";
  "tokenizer_config.json"    = "$emoRepo/tokenizer_config.json?download=true"
}
foreach($f in $emoRequired.Keys){ Download -Url $emoRequired[$f] -OutPath (Join-Path $EMO $f) }
foreach($f in $emoOptional.Keys){ DownloadOptional -Url $emoOptional[$f] -OutPath (Join-Path $EMO $f) }

# -------- TTS (FastSpeech2-lite + HiFiGAN-lite) --------
# FastSpeech2 CN streaming lite
$fs2_cn_zip = Join-Path $TMP 'fastspeech2_cnndecoder_csmsc_streaming_onnx_1.0.0.zip'
Download    -Url 'https://paddlespeech.bj.bcebos.com/Parakeet/released_models/fastspeech2/fastspeech2_cnndecoder_csmsc_streaming_onnx_1.0.0.zip' -OutPath $fs2_cn_zip
ExtractZip  -ZipPath $fs2_cn_zip -Dest $TMP
CopyFirst   -Pattern '*.onnx' -From $TMP -To $FS2 -Rename 'fastspeech2_csmsc_streaming.onnx'
CopyAll     -Patterns @('*.json','*.yaml','*.yml','*.npy','*stats*.npy','*phone*id*.txt','phone*map*.txt','phones.txt','*.cmvn') -From $TMP -To $FS2

# FastSpeech2 EN LJSpeech
$fs2_en_zip = Join-Path $TMP 'fastspeech2_ljspeech_onnx_1.1.0.zip'
Download    -Url 'https://paddlespeech.bj.bcebos.com/Parakeet/released_models/fastspeech2/fastspeech2_ljspeech_onnx_1.1.0.zip' -OutPath $fs2_en_zip
ExtractZip  -ZipPath $fs2_en_zip -Dest $TMP
CopyFirst   -Pattern '*.onnx' -From $TMP -To $FS2 -Rename 'fastspeech2_ljspeech.onnx'
CopyAll     -Patterns @('*.json','*.yaml','*.yml','*.npy','*stats*.npy','*phone*id*.txt','phone*map*.txt','phones.txt','*.cmvn') -From $TMP -To $FS2

# HiFiGAN CN CSMSC
$hfg_cn_zip = Join-Path $TMP 'hifigan_csmsc_onnx_0.2.0.zip'
Download    -Url 'https://paddlespeech.bj.bcebos.com/Parakeet/released_models/hifigan/hifigan_csmsc_onnx_0.2.0.zip' -OutPath $hfg_cn_zip
ExtractZip  -ZipPath $hfg_cn_zip -Dest $TMP
CopyFirst   -Pattern '*.onnx' -From $TMP -To $HFG -Rename 'hifigan_csmsc.onnx'
CopyAll     -Patterns @('*.json','*.yaml','*.yml') -From $TMP -To $HFG

# HiFiGAN EN LJSpeech
$hfg_en_zip = Join-Path $TMP 'hifigan_ljspeech_onnx_1.1.0.zip'
Download    -Url 'https://paddlespeech.bj.bcebos.com/Parakeet/released_models/hifigan/hifigan_ljspeech_onnx_1.1.0.zip' -OutPath $hfg_en_zip
ExtractZip  -ZipPath $hfg_en_zip -Dest $TMP
CopyFirst   -Pattern '*.onnx' -From $TMP -To $HFG -Rename 'hifigan_ljspeech.onnx'
CopyAll     -Patterns @('*.json','*.yaml','*.yml') -From $TMP -To $HFG

# -------- Cleanup temp --------
try { Remove-Item $TMP -Recurse -Force } catch { Warn ("Temp cleanup failed; remove manually: " + $TMP) }

# -------- Summary --------
Write-Host "`n[SUMMARY]" -ForegroundColor Yellow
Write-Host ("ASR whisper-base          : " + $ASR)
Write-Host ("NMT marian-* (6 dirs)     : " + $NMT)
Write-Host ("Emotion xlm-r             : " + $EMO)
Write-Host ("VAD silero                : " + $VAD)
Write-Host ("Persona embedding-default : " + $EMB)
Write-Host ("TTS FastSpeech2-lite      : " + $FS2)
Write-Host ("TTS HiFiGAN-lite          : " + $HFG)
Ok "All required models freshly downloaded and organized (incl. tokenizer/config extras)."
