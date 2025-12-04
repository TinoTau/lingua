# 检查 CoreEngine 服务状态

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  CoreEngine 服务诊断工具" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$port = 9000

# 1. 检查端口是否被占用
Write-Host "[1/4] 检查端口 $port 状态..." -ForegroundColor Cyan
$portInUse = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "  ✅ 端口 $port 已被占用" -ForegroundColor Green
    $processId = $portInUse.OwningProcess | Select-Object -First 1
    $process = Get-Process -Id $processId -ErrorAction SilentlyContinue
    if ($process) {
        Write-Host "  进程名称: $($process.ProcessName)" -ForegroundColor Yellow
        Write-Host "  进程 ID: $processId" -ForegroundColor Yellow
        Write-Host "  进程路径: $($process.Path)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ❌ 端口 $port 未被占用（服务可能未运行）" -ForegroundColor Red
}

Write-Host ""

# 2. 检查可执行文件
Write-Host "[2/4] 检查 CoreEngine 可执行文件..." -ForegroundColor Cyan
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$coreEnginePath = Join-Path $scriptDir "core\engine\target\release\core_engine.exe"
if (Test-Path $coreEnginePath) {
    Write-Host "  ✅ 可执行文件存在: $coreEnginePath" -ForegroundColor Green
    $fileInfo = Get-Item $coreEnginePath
    Write-Host "  文件大小: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor Yellow
    Write-Host "  修改时间: $($fileInfo.LastWriteTime)" -ForegroundColor Yellow
} else {
    Write-Host "  ❌ 可执行文件不存在: $coreEnginePath" -ForegroundColor Red
    Write-Host "  请先编译: cd core\engine && cargo build --release --bin core_engine" -ForegroundColor Yellow
}

Write-Host ""

# 3. 检查配置文件
Write-Host "[3/4] 检查配置文件..." -ForegroundColor Cyan
$configPath = Join-Path $scriptDir "lingua_core_config.toml"
if (Test-Path $configPath) {
    Write-Host "  ✅ 配置文件存在: $configPath" -ForegroundColor Green
    $configContent = Get-Content $configPath -Raw
    if ($configContent -match "port\s*=\s*(\d+)") {
        $configPort = $matches[1]
        Write-Host "  配置的端口: $configPort" -ForegroundColor Yellow
        if ($configPort -ne $port) {
            Write-Host "  ⚠️  警告: 配置端口 ($configPort) 与检查端口 ($port) 不一致" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "  ⚠️  配置文件不存在: $configPath" -ForegroundColor Yellow
    Write-Host "  将使用默认配置" -ForegroundColor Yellow
}

Write-Host ""

# 4. 测试 HTTP 连接
Write-Host "[4/4] 测试 HTTP 连接..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:$port/health" -Method GET -TimeoutSec 2 -ErrorAction Stop
    Write-Host "  ✅ HTTP 连接成功" -ForegroundColor Green
    Write-Host "  状态码: $($response.StatusCode)" -ForegroundColor Yellow
    Write-Host "  响应内容: $($response.Content)" -ForegroundColor Yellow
} catch {
    Write-Host "  ❌ HTTP 连接失败: $_" -ForegroundColor Red
    Write-Host "  可能原因:" -ForegroundColor Yellow
    Write-Host "    1. 服务未运行" -ForegroundColor Yellow
    Write-Host "    2. 端口不正确" -ForegroundColor Yellow
    Write-Host "    3. 防火墙阻止连接" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  诊断完成" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "如果服务未运行，请执行:" -ForegroundColor Yellow
Write-Host "  .\start_core_engine_only.ps1" -ForegroundColor Cyan
Write-Host ""

