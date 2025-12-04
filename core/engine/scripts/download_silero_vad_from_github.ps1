# Silero VAD 模型下载脚本（从 GitHub）
# 
# 从 GitHub 仓库下载最新的 Silero VAD ONNX 模型

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Silero VAD 模型下载（GitHub）" -ForegroundColor Cyan
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

$modelPath = Join-Path $modelDir "silero_vad_github.onnx"
$backupPath = Join-Path $modelDir "silero_vad.onnx.backup"

# 备份现有模型（如果存在）
if (Test-Path (Join-Path $modelDir "silero_vad.onnx")) {
    $existingModel = Join-Path $modelDir "silero_vad.onnx"
    Write-Host "[备份] 备份现有模型..." -ForegroundColor Yellow
    Copy-Item $existingModel $backupPath -Force
    Write-Host "[备份] 已备份到: $backupPath" -ForegroundColor Green
}

# 检查模型是否已存在
if (Test-Path $modelPath) {
    Write-Host "[警告] 模型文件已存在: $modelPath" -ForegroundColor Yellow
    $overwrite = Read-Host "是否覆盖? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "[取消] 下载已取消" -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "[下载] Silero VAD ONNX 模型（从 GitHub）..." -ForegroundColor Yellow
Write-Host "  模型路径: $modelPath" -ForegroundColor Gray
Write-Host ""

# GitHub 原始文件链接（多个备用地址）
$downloadUrls = @(
    "https://github.com/snakers4/silero-vad/raw/master/src/silero_vad/data/silero_vad.onnx",
    "https://raw.githubusercontent.com/snakers4/silero-vad/master/src/silero_vad/data/silero_vad.onnx",
    "https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx",
    "https://models.silero.ai/vad_models/silero_vad.onnx"
)

$downloadSuccess = $false

foreach ($url in $downloadUrls) {
    try {
        Write-Host "[尝试] 从以下地址下载:" -ForegroundColor Yellow
        Write-Host "  $url" -ForegroundColor Gray
        Write-Host ""
        
        $ProgressPreference = 'SilentlyContinue'
        $webClient = New-Object System.Net.WebClient
        $webClient.Headers.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        
        # 下载文件
        $webClient.DownloadFile($url, $modelPath)
        $webClient.Dispose()
        
        # 验证文件大小
        $fileInfo = Get-Item $modelPath
        $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
        
        Write-Host "[成功] 模型下载完成!" -ForegroundColor Green
        Write-Host "[验证] 文件大小: $fileSizeMB MB" -ForegroundColor Green
        
        if ($fileSizeMB -lt 1) {
            Write-Host "[警告] 文件大小异常小，可能下载失败" -ForegroundColor Yellow
            Remove-Item $modelPath -Force
            continue
        }
        
        if ($fileSizeMB -gt 10) {
            Write-Host "[警告] 文件大小异常大，可能下载了错误文件" -ForegroundColor Yellow
            Remove-Item $modelPath -Force
            continue
        }
        
        $downloadSuccess = $true
        break
        
    } catch {
        Write-Host "[失败] 下载失败: $_" -ForegroundColor Red
        Write-Host ""
        if (Test-Path $modelPath) {
            Remove-Item $modelPath -Force
        }
        continue
    }
}

if (-not $downloadSuccess) {
    Write-Host ""
    Write-Host "[错误] 所有下载源都失败了" -ForegroundColor Red
    Write-Host ""
    Write-Host "请尝试手动下载:" -ForegroundColor Yellow
    Write-Host "  1. 访问: https://github.com/snakers4/silero-vad" -ForegroundColor Yellow
    Write-Host "  2. 进入: src/silero_vad/data/" -ForegroundColor Yellow
    Write-Host "  3. 下载: silero_vad.onnx" -ForegroundColor Yellow
    Write-Host "  4. 保存到: $modelPath" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "或者使用 Git LFS:" -ForegroundColor Yellow
    Write-Host "  git clone https://github.com/snakers4/silero-vad.git" -ForegroundColor Cyan
    Write-Host "  cd silero-vad" -ForegroundColor Cyan
    Write-Host "  git lfs pull" -ForegroundColor Cyan
    Write-Host "  copy src\silero_vad\data\silero_vad.onnx $modelPath" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "  下载完成！" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "模型文件位置: $modelPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "提示: 如果这是新下载的模型，请更新配置文件中的模型路径" -ForegroundColor Yellow
Write-Host "  或者将文件重命名为: silero_vad.onnx" -ForegroundColor Yellow
Write-Host ""

