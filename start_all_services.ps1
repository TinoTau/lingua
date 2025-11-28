# 一键启动所有服务脚本
# 启动 CoreEngine (ASR GPU) 和 NMT 服务 (GPU)
# 
# 停止服务：
#   - 方法 1: 按 Ctrl+C（会自动停止后台的 NMT 服务）
#   - 方法 2: 运行 .\stop_all_services.ps1

Write-Host "=== Lingua 服务一键启动 ===" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Stop"

# 获取脚本所在目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 检查必要的文件
$coreEnginePath = "core\engine\target\release\core_engine.exe"
$configPath = "lingua_core_config.toml"
$nmtServicePath = "services\nmt_m2m100"

if (-not (Test-Path $coreEnginePath)) {
    Write-Host "错误: 找不到 CoreEngine 可执行文件: $coreEnginePath" -ForegroundColor Red
    Write-Host "请先编译 CoreEngine: cd core\engine && cargo build --release --bin core_engine" -ForegroundColor Yellow
    exit 1
}

if (-not (Test-Path $configPath)) {
    Write-Host "错误: 找不到配置文件: $configPath" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "$nmtServicePath\venv\Scripts\Activate.ps1")) {
    Write-Host "错误: 找不到 NMT 服务虚拟环境: $nmtServicePath\venv" -ForegroundColor Red
    exit 1
}

# 设置 CUDA 环境变量（用于 CoreEngine）
Write-Host "设置 CUDA 环境变量..." -ForegroundColor Yellow
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
if (Test-Path $cudaPath) {
    $env:CUDA_PATH = $cudaPath
    $env:CUDAToolkit_ROOT = $cudaPath
    $env:CUDA_ROOT = $cudaPath
    $env:CUDA_HOME = $cudaPath
    $env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
    $env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"
    Write-Host "✓ CUDA 环境变量已设置" -ForegroundColor Green
} else {
    Write-Host "⚠ 警告: 未找到 CUDA 路径，使用默认配置" -ForegroundColor Yellow
}

Write-Host ""

# 启动 NMT 服务（后台）
Write-Host "启动 NMT 服务 (端口 5008)..." -ForegroundColor Cyan
$nmtJob = Start-Job -ScriptBlock {
    Set-Location $using:nmtServicePath
    & .\venv\Scripts\Activate.ps1
    uvicorn nmt_service:app --host 127.0.0.1 --port 5008
}

Write-Host "✓ NMT 服务已启动（后台运行）" -ForegroundColor Green
Write-Host "  等待 NMT 服务就绪..." -ForegroundColor Yellow

# 等待 NMT 服务就绪
$nmtReady = $false
for ($i = 1; $i -le 30; $i++) {
    Start-Sleep -Seconds 2
    try {
        $response = Invoke-WebRequest -Uri "http://127.0.0.1:5008/health" -UseBasicParsing -TimeoutSec 2 -ErrorAction SilentlyContinue
        if ($response.StatusCode -eq 200) {
            $health = $response.Content | ConvertFrom-Json
            if ($health.status -eq "ok") {
                $nmtReady = $true
                Write-Host "✓ NMT 服务已就绪 (设备: $($health.device))" -ForegroundColor Green
                break
            }
        }
    } catch {
        # 继续等待
    }
    Write-Host "  等待中... ($i/30)" -ForegroundColor Gray
}

if (-not $nmtReady) {
    Write-Host "⚠ 警告: NMT 服务可能未完全就绪，但继续启动 CoreEngine" -ForegroundColor Yellow
}

Write-Host ""

# 启动 CoreEngine
Write-Host "启动 CoreEngine (端口 9000)..." -ForegroundColor Cyan
Write-Host "  按 Ctrl+C 停止所有服务" -ForegroundColor Yellow
Write-Host ""

# 启动 CoreEngine（前台运行）
try {
    & $coreEnginePath --config $configPath
} finally {
    # 清理：停止 NMT 服务
    Write-Host ""
    Write-Host "正在停止 NMT 服务..." -ForegroundColor Yellow
    Stop-Job $nmtJob -ErrorAction SilentlyContinue
    Remove-Job $nmtJob -ErrorAction SilentlyContinue
    Write-Host "✓ 所有服务已停止" -ForegroundColor Green
}

