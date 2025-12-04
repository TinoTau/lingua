# Silero VAD 模型下载脚本
# 
# 下载 IR version 9 的 Silero VAD ONNX 模型（兼容 ONNX Runtime 1.16.3）

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Silero VAD 模型下载" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本所在目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$engineDir = Split-Path -Parent $scriptDir
$modelDir = Join-Path $engineDir "models" "vad" "silero"

# 创建模型目录（如果不存在）
if (-not (Test-Path $modelDir)) {
    Write-Host "[创建] 模型目录: $modelDir" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $modelDir -Force | Out-Null
}

$modelPath = Join-Path $modelDir "silero_vad.onnx"

# 检查模型是否已存在
if (Test-Path $modelPath) {
    Write-Host "[警告] 模型文件已存在: $modelPath" -ForegroundColor Yellow
    $overwrite = Read-Host "是否覆盖? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "[取消] 下载已取消" -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "[下载] Silero VAD ONNX 模型..." -ForegroundColor Yellow
Write-Host "  模型路径: $modelPath" -ForegroundColor Gray
Write-Host ""

# Silero VAD 模型下载 URL（IR version 9，兼容 ONNX Runtime 1.16.3）
# 使用 GitHub Releases 中的模型文件
$modelUrl = "https://github.com/snakers4/silero-vad/raw/master/src/silero_vad/data/silero_vad.onnx"

# 备用下载地址（如果上面的链接不可用）
$modelUrlAlt = "https://models.silero.ai/vad_models/silero_vad.onnx"

Write-Host "[信息] 下载地址: $modelUrl" -ForegroundColor Gray
Write-Host ""

try {
    # 尝试使用 Invoke-WebRequest 下载
    Write-Host "[下载] 正在下载模型文件..." -ForegroundColor Yellow
    
    $ProgressPreference = 'SilentlyContinue'  # 禁用进度条以提高速度
    Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -ErrorAction Stop
    
    Write-Host "[成功] 模型下载完成!" -ForegroundColor Green
    Write-Host ""
    
    # 验证文件大小
    $fileInfo = Get-Item $modelPath
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "[验证] 文件大小: $fileSizeMB MB" -ForegroundColor Green
    
    if ($fileSizeMB -lt 1) {
        Write-Host "[警告] 文件大小异常小，可能下载失败" -ForegroundColor Yellow
        Write-Host "       请检查文件内容或手动下载" -ForegroundColor Yellow
    } else {
        Write-Host "[成功] 模型文件验证通过" -ForegroundColor Green
    }
    
} catch {
    Write-Host "[错误] 下载失败: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "请尝试手动下载:" -ForegroundColor Yellow
    Write-Host "  1. 访问: https://github.com/snakers4/silero-vad" -ForegroundColor Yellow
    Write-Host "  2. 下载 ONNX 模型文件" -ForegroundColor Yellow
    Write-Host "  3. 保存到: $modelPath" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "或者使用备用下载地址:" -ForegroundColor Yellow
    Write-Host "  $modelUrlAlt" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "  下载完成！" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "模型文件位置: $modelPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "现在可以运行测试:" -ForegroundColor Yellow
Write-Host "  cargo run --example test_silero_vad_startup" -ForegroundColor Cyan
Write-Host ""

