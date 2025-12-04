# 从 Windows 在 WSL 中运行 YourTTS ONNX 导出脚本

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  YourTTS ONNX 导出工具（WSL 环境）" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

# 转换为 WSL 路径
$wslPath = $projectRoot -replace '^([A-Z]):', '/mnt/$1' -replace '\\', '/'
$wslPath = $wslPath.ToLower()

Write-Host "项目根目录 (Windows): $projectRoot" -ForegroundColor Yellow
Write-Host "项目根目录 (WSL): $wslPath" -ForegroundColor Yellow
Write-Host ""

# 检查 WSL 是否可用
try {
    $wslVersion = wsl --version 2>&1
    Write-Host "✅ WSL 可用" -ForegroundColor Green
} catch {
    Write-Host "❌ 错误: WSL 不可用" -ForegroundColor Red
    Write-Host "请先安装 WSL2" -ForegroundColor Red
    exit 1
}

# 检查依赖
Write-Host "📌 检查依赖..." -ForegroundColor Cyan
$ttsCheck = wsl python3 -c "import TTS" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  TTS 库未安装，尝试安装..." -ForegroundColor Yellow
    wsl python3 -m pip install TTS
}

$torchCheck = wsl python3 -c "import torch" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  torch 未安装，尝试安装..." -ForegroundColor Yellow
    wsl python3 -m pip install torch
}

$onnxCheck = wsl python3 -c "import onnx" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  onnx 未安装，尝试安装..." -ForegroundColor Yellow
    wsl python3 -m pip install onnx
}

$onnxruntimeCheck = wsl python3 -c "import onnxruntime" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  onnxruntime 未安装，尝试安装..." -ForegroundColor Yellow
    wsl python3 -m pip install onnxruntime
}

Write-Host "✅ 依赖检查完成" -ForegroundColor Green
Write-Host ""

# 运行导出脚本
Write-Host "🚀 开始导出 YourTTS 模型为 ONNX..." -ForegroundColor Cyan
Write-Host ""

$wslCommand = "cd $wslPath && python3 core/engine/scripts/export_yourtts_to_onnx.py"

# 传递所有参数到 WSL
$argsString = $args -join ' '
if ($argsString) {
    $wslCommand += " $argsString"
}

wsl bash -c $wslCommand

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Green
    Write-Host "✅ 导出完成！" -ForegroundColor Green
    Write-Host "============================================================" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Red
    Write-Host "❌ 导出失败" -ForegroundColor Red
    Write-Host "============================================================" -ForegroundColor Red
}

exit $LASTEXITCODE

