# CUDA 编译脚本 - 简化版
# 自动设置 CUDA 环境变量并编译 CoreEngine

Write-Host "=== CoreEngine CUDA 编译 ===" -ForegroundColor Cyan
Write-Host ""

# 设置 CUDA 环境变量
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
Write-Host "设置 CUDA 环境变量..." -ForegroundColor Yellow
$env:CUDA_PATH = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 验证 CUDA
Write-Host "验证 CUDA 安装..." -ForegroundColor Yellow
$nvccVersion = & "$cudaPath\bin\nvcc.exe" --version 2>&1 | Select-String -Pattern "release"
if ($nvccVersion) {
    Write-Host "✓ $nvccVersion" -ForegroundColor Green
} else {
    Write-Host "✗ 无法验证 nvcc" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 切换到项目目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 询问是否清理
$clean = Read-Host "是否清理旧的编译产物? (y/N)"
if ($clean -eq 'y' -or $clean -eq 'Y') {
    Write-Host "清理编译产物..." -ForegroundColor Yellow
    cargo clean
    Write-Host ""
}

# 开始编译
Write-Host "开始编译 CoreEngine (Release 模式)..." -ForegroundColor Cyan
Write-Host "注意: 首次编译可能需要 10-30 分钟" -ForegroundColor Yellow
Write-Host ""

cargo build --release --bin core_engine

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✓ 编译成功！" -ForegroundColor Green
    Write-Host "可执行文件: $scriptDir\target\release\core_engine.exe" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "✗ 编译失败" -ForegroundColor Red
    Write-Host ""
    Write-Host "如果错误信息包含 'No CUDA toolset found'，请确保：" -ForegroundColor Yellow
    Write-Host "1. 已在 Visual Studio Installer 中安装 CUDA 工具集" -ForegroundColor Yellow
    Write-Host "2. 已重启电脑（安装 CUDA 工具集后需要重启）" -ForegroundColor Yellow
    Write-Host "3. 参考文档: docs/operational/ASR_GPU_编译故障排查.md" -ForegroundColor Cyan
    exit 1
}

