# SileroVad 服务启动测试脚本
# 
# 使用方法：
#   .\core\engine\scripts\test_silero_vad.ps1

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  SileroVad 服务启动测试" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本所在目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
$engineDir = Join-Path $projectRoot "core" "engine"

# 检查模型文件
$modelPath = Join-Path $engineDir "models" "vad" "silero" "silero_vad.onnx"
Write-Host "[检查] 模型文件路径: $modelPath" -ForegroundColor Yellow

if (-not (Test-Path $modelPath)) {
    Write-Host "[错误] 模型文件不存在: $modelPath" -ForegroundColor Red
    Write-Host ""
    Write-Host "请确保模型文件位于正确路径，或下载模型文件。" -ForegroundColor Yellow
    Write-Host "模型下载地址: https://github.com/snakers4/silero-vad" -ForegroundColor Yellow
    exit 1
}

Write-Host "[成功] 模型文件存在" -ForegroundColor Green
Write-Host ""

# 切换到 engine 目录
Push-Location $engineDir

try {
    Write-Host "[运行] 执行测试..." -ForegroundColor Yellow
    Write-Host ""
    
    # 运行测试
    cargo run --example test_silero_vad_startup
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Green
        Write-Host "  测试完成！SileroVad 已准备就绪" -ForegroundColor Green
        Write-Host "============================================================" -ForegroundColor Green
    }
    else {
        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Red
        Write-Host "  测试失败，请检查错误信息" -ForegroundColor Red
        Write-Host "============================================================" -ForegroundColor Red
        exit 1
    }
}
finally {
    Pop-Location
}

