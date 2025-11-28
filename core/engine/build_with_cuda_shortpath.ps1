# CUDA 编译脚本 - 使用短路径名
# 此脚本自动设置 CUDA 环境变量（使用短路径名）并编译 CoreEngine

Write-Host "=== CoreEngine CUDA 编译脚本（短路径名版本）===" -ForegroundColor Cyan

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

# 获取短路径名
Write-Host "`n获取短路径名..." -ForegroundColor Cyan
$fso = New-Object -ComObject Scripting.FileSystemObject
try {
    $shortPath = $fso.GetFolder($cudaPath).ShortPath
    Write-Host "原始路径: $cudaPath" -ForegroundColor Yellow
    Write-Host "短路径: $shortPath" -ForegroundColor Green
} catch {
    Write-Host "警告: 无法获取短路径名，使用原始路径" -ForegroundColor Yellow
    $shortPath = $cudaPath
}

# 验证 CUDA 路径
if (-not (Test-Path "$shortPath\bin\nvcc.exe")) {
    Write-Host "错误: 在 $shortPath 中未找到 nvcc.exe" -ForegroundColor Red
    exit 1
}

# 设置环境变量（使用短路径）
Write-Host "`n设置 CUDA 环境变量（使用短路径）..." -ForegroundColor Cyan
$env:CUDA_PATH = $shortPath
$env:CUDAToolkit_ROOT = $shortPath
$env:CUDA_ROOT = $shortPath
$env:CUDA_HOME = $shortPath
$env:CMAKE_CUDA_COMPILER = "$shortPath\bin\nvcc.exe"
$env:PATH = "$shortPath\bin;$shortPath\libnvvp;$env:PATH"

# 验证 nvcc
Write-Host "验证 CUDA 编译器..." -ForegroundColor Cyan
$nvccVersion = & "$shortPath\bin\nvcc.exe" --version 2>&1 | Select-String -Pattern "release"
if ($nvccVersion) {
    Write-Host "✓ $nvccVersion" -ForegroundColor Green
} else {
    Write-Host "警告: 无法获取 nvcc 版本信息" -ForegroundColor Yellow
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

# 显示重要提示
Write-Host "`n=== 重要提示 ===" -ForegroundColor Yellow
Write-Host "如果编译失败并提示 'No CUDA toolset found'，这通常是因为：" -ForegroundColor Yellow
Write-Host "1. Visual Studio Build Tools 缺少 CUDA 工具集支持" -ForegroundColor Yellow
Write-Host "2. 需要在 Visual Studio Installer 中安装 CUDA 工具集组件" -ForegroundColor Yellow
Write-Host "`n如果无法安装 CUDA 工具集，可能需要：" -ForegroundColor Yellow
Write-Host "- 安装完整的 Visual Studio 2022 Community 版本" -ForegroundColor Yellow
Write-Host "- 或者使用其他编译方法（如 WSL2 + Linux）" -ForegroundColor Yellow
Write-Host ""

# 开始编译
Write-Host "开始编译 CoreEngine (Release 模式)..." -ForegroundColor Cyan
Write-Host "注意: 首次编译可能需要 10-30 分钟" -ForegroundColor Yellow
Write-Host ""

cargo build --release --bin core_engine

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✓ 编译成功！" -ForegroundColor Green
    Write-Host "可执行文件位置: $scriptDir\target\release\core_engine.exe" -ForegroundColor Cyan
} else {
    Write-Host "`n✗ 编译失败" -ForegroundColor Red
    Write-Host "`n如果错误信息包含 'No CUDA toolset found'，请参考以下文档：" -ForegroundColor Yellow
    Write-Host "- docs/operational/ASR_GPU_编译故障排查.md" -ForegroundColor Cyan
    Write-Host "- docs/operational/ASR_GPU_配置完成.md" -ForegroundColor Cyan
    exit 1
}

