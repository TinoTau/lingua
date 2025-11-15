# PowerShell 脚本：下载预转换的 Whisper GGML 模型
# 使用方法: .\scripts\download_whisper_ggml.ps1 -ModelSize base -OutputDir core\engine\models\asr\whisper-base

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("tiny", "base", "small", "medium", "large")]
    [string]$ModelSize = "base",
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = "core\engine\models\asr\whisper-base"
)

$ErrorActionPreference = "Stop"

Write-Host "=== 下载 Whisper GGML 模型 ===" -ForegroundColor Cyan
Write-Host "模型大小: $ModelSize" -ForegroundColor Yellow
Write-Host "输出目录: $OutputDir" -ForegroundColor Yellow

# 创建输出目录
$outputPath = Join-Path $PSScriptRoot ".." $OutputDir
$outputPath = [System.IO.Path]::GetFullPath($outputPath)
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null

# GGML 模型下载地址
$baseUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main"
$modelFile = "ggml-$ModelSize.bin"
$url = "$baseUrl/$modelFile"
$outputFile = Join-Path $outputPath $modelFile

# 检查文件是否已存在
if (Test-Path $outputFile) {
    $fileSize = (Get-Item $outputFile).Length / 1MB
    Write-Host "✓ 模型文件已存在: $outputFile" -ForegroundColor Green
    Write-Host "  文件大小: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Gray
    exit 0
}

Write-Host "`n下载地址: $url" -ForegroundColor Gray
Write-Host "保存到: $outputFile" -ForegroundColor Gray
Write-Host "`n正在下载..." -ForegroundColor Yellow

try {
    # 使用 Invoke-WebRequest 下载
    $ProgressPreference = 'SilentlyContinue'  # 禁用进度条以提高速度
    Invoke-WebRequest -Uri $url -OutFile $outputFile -UseBasicParsing
    
    if (Test-Path $outputFile) {
        $fileSize = (Get-Item $outputFile).Length / 1MB
        Write-Host "✓ 下载成功!" -ForegroundColor Green
        Write-Host "  文件: $outputFile" -ForegroundColor Gray
        Write-Host "  大小: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Gray
    } else {
        Write-Host "✗ 下载失败: 文件不存在" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "✗ 下载失败: $_" -ForegroundColor Red
    Write-Host "`n提示: 可以手动下载模型文件:" -ForegroundColor Yellow
    Write-Host "  1. 访问: https://huggingface.co/ggerganov/whisper.cpp/tree/main" -ForegroundColor Gray
    Write-Host "  2. 下载: ggml-$ModelSize.bin" -ForegroundColor Gray
    Write-Host "  3. 保存到: $outputFile" -ForegroundColor Gray
    exit 1
}

Write-Host "`n✓ 模型准备完成!" -ForegroundColor Green

