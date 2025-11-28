# CUDA 编译脚本
# 此脚本自动设置 CUDA 环境变量并编译 CoreEngine

Write-Host "=== CoreEngine CUDA 编译脚本 ===" -ForegroundColor Cyan

# 检测 CUDA 路径
$cudaPath = $env:CUDA_PATH
if (-not $cudaPath) {
    # 尝试常见的 CUDA 安装路径
    $possiblePaths = @(
        "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4",
        "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1",
        "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0"
    )
    
    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            $cudaPath = $path
            Write-Host "找到 CUDA 安装路径: $cudaPath" -ForegroundColor Green
            break
        }
    }
    
    if (-not $cudaPath) {
        Write-Host "错误: 未找到 CUDA 安装路径" -ForegroundColor Red
        Write-Host "请手动设置 CUDA_PATH 环境变量，或修改此脚本中的路径" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "使用环境变量中的 CUDA 路径: $cudaPath" -ForegroundColor Green
}

# 验证 CUDA 路径
if (-not (Test-Path "$cudaPath\bin\nvcc.exe")) {
    Write-Host "错误: 在 $cudaPath 中未找到 nvcc.exe" -ForegroundColor Red
    exit 1
}

# 设置环境变量
Write-Host "`n设置 CUDA 环境变量..." -ForegroundColor Cyan
$env:CUDA_PATH = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 验证 nvcc
Write-Host "验证 CUDA 编译器..." -ForegroundColor Cyan
$nvccVersion = & "$cudaPath\bin\nvcc.exe" --version 2>&1 | Select-String -Pattern "release"
if ($nvccVersion) {
    Write-Host "✓ $nvccVersion" -ForegroundColor Green
} else {
    Write-Host "警告: 无法获取 nvcc 版本信息" -ForegroundColor Yellow
}

# 验证 CMake 是否能找到 CUDA（可选）
Write-Host "`n验证 CMake 是否能检测到 CUDA..." -ForegroundColor Cyan
$cmakeTest = cmake --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ CMake 已安装" -ForegroundColor Green
} else {
    Write-Host "警告: CMake 未安装或不在 PATH 中" -ForegroundColor Yellow
}

# 切换到项目目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 清理旧的编译产物（可选）
$clean = Read-Host "`n是否清理旧的编译产物? (y/N)"
if ($clean -eq 'y' -or $clean -eq 'Y') {
    Write-Host "清理编译产物..." -ForegroundColor Cyan
    cargo clean
}

# 开始编译
Write-Host "`n开始编译 CoreEngine (Release 模式)..." -ForegroundColor Cyan
Write-Host "注意: 首次编译可能需要 10-30 分钟" -ForegroundColor Yellow
Write-Host ""

cargo build --release --bin core_engine

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✓ 编译成功！" -ForegroundColor Green
    Write-Host "可执行文件位置: $scriptDir\target\release\core_engine.exe" -ForegroundColor Cyan
} else {
    Write-Host "`n✗ 编译失败" -ForegroundColor Red
    Write-Host "请检查错误信息，或参考文档进行故障排查" -ForegroundColor Yellow
    exit 1
}

