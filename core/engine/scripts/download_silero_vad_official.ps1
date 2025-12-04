# 下载官方 Silero VAD 模型
# 使用方法：在 PowerShell 中运行此脚本

$ErrorActionPreference = "Stop"

# 获取脚本所在目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$coreEngineDir = Split-Path -Parent $scriptDir
$modelDir = Join-Path $coreEngineDir "models\vad\silero"

# 创建模型目录（如果不存在）
if (-not (Test-Path $modelDir)) {
    New-Item -ItemType Directory -Path $modelDir -Force | Out-Null
    Write-Host "创建模型目录: $modelDir" -ForegroundColor Green
}

# 模型文件路径
$modelPath = Join-Path $modelDir "silero_vad_official.onnx"
$backupPath = Join-Path $modelDir "silero_vad.onnx.backup"

# 备份现有模型（如果存在）
if (Test-Path (Join-Path $modelDir "silero_vad.onnx")) {
    $existingModel = Join-Path $modelDir "silero_vad.onnx"
    Write-Host "备份现有模型..." -ForegroundColor Yellow
    Copy-Item $existingModel $backupPath -Force
    Write-Host "已备份到: $backupPath" -ForegroundColor Green
}

# 官方模型下载地址（使用 Hugging Face）
# 主地址（可能需要认证）：
# - https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx
# 备用地址（已验证可用）：
# - https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx
$modelUrl = "https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  下载官方 Silero VAD 模型" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "下载地址: $modelUrl" -ForegroundColor Yellow
Write-Host "保存路径: $modelPath" -ForegroundColor Yellow
Write-Host ""

try {
    # 使用 PowerShell 的 Invoke-WebRequest 下载
    Write-Host "开始下载..." -ForegroundColor Green
    $ProgressPreference = 'Continue'
    
    # 尝试使用不同的方法下载
    try {
        # 方法1：使用 Invoke-WebRequest（添加 User-Agent 头）
        $headers = @{
            "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        }
        Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing -TimeoutSec 60 -Headers $headers
    } catch {
        Write-Host "方法1失败，尝试方法2..." -ForegroundColor Yellow
        # 方法2：使用 System.Net.WebClient（添加 User-Agent 头）
        $webClient = New-Object System.Net.WebClient
        $webClient.Headers.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        $webClient.DownloadFile($modelUrl, $modelPath)
        $webClient.Dispose()
    }
    
    # 验证文件
    if (Test-Path $modelPath) {
        $fileInfo = Get-Item $modelPath
        $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
        Write-Host ""
        Write-Host "下载成功！" -ForegroundColor Green
        Write-Host "  文件大小: $fileSizeMB MB" -ForegroundColor Green
        Write-Host "  文件路径: $modelPath" -ForegroundColor Green
        Write-Host ""
        Write-Host "提示: 请更新配置文件中的模型路径为: models/vad/silero/silero_vad_official.onnx" -ForegroundColor Yellow
    } else {
        Write-Host "下载失败：文件不存在" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host ""
    Write-Host "下载失败: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "如果下载失败，可以尝试以下方法：" -ForegroundColor Yellow
    Write-Host "1. 使用浏览器直接下载: $modelUrl" -ForegroundColor Yellow
    Write-Host "2. 在 WSL 中使用 wget: wget $modelUrl -O silero_vad_official.onnx" -ForegroundColor Yellow
    Write-Host "3. 使用 Python 下载脚本" -ForegroundColor Yellow
    exit 1
}

Write-Host "============================================================" -ForegroundColor Cyan
