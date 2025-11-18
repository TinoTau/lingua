# 简单的 TTS 文件检查脚本（PowerShell）
# 不读取文件内容，只检查是否存在

Write-Host "=== TTS 模型文件检查 ===" -ForegroundColor Cyan
Write-Host ""

$modelDir = "core\engine\models\tts"

if (-not (Test-Path $modelDir)) {
    Write-Host "❌ TTS 模型目录不存在: $modelDir" -ForegroundColor Red
    exit 1
}

Write-Host "✅ TTS 模型目录存在: $modelDir" -ForegroundColor Green
Write-Host ""

# 检查 FastSpeech2
Write-Host "=== FastSpeech2 模型 ===" -ForegroundColor Yellow
$fs2Dir = Join-Path $modelDir "fastspeech2-lite"

$fs2Files = @(
    "fastspeech2_csmsc_streaming.onnx",
    "fastspeech2_ljspeech.onnx",
    "phone_id_map.txt",
    "speech_stats.npy"
)

foreach ($file in $fs2Files) {
    $path = Join-Path $fs2Dir $file
    if (Test-Path $path) {
        $size = (Get-Item $path).Length / 1MB
        Write-Host "  ✅ $file ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file (不存在)" -ForegroundColor Red
    }
}

# 检查 HiFiGAN
Write-Host ""
Write-Host "=== HiFiGAN 模型 ===" -ForegroundColor Yellow
$hifiganDir = Join-Path $modelDir "hifigan-lite"

$hifiganFiles = @(
    "hifigan_csmsc.onnx",
    "hifigan_ljspeech.onnx"
)

foreach ($file in $hifiganFiles) {
    $path = Join-Path $hifiganDir $file
    if (Test-Path $path) {
        $size = (Get-Item $path).Length / 1MB
        Write-Host "  ✅ $file ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file (不存在)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== 检查完成 ===" -ForegroundColor Cyan

