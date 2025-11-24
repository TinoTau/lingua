# Lingua Core Runtime - 一键启动脚本（Windows PowerShell）
# 
# 用途：启动所有必需的服务（NMT、TTS、CoreEngine）
# 
# 使用方法：
#   .\start_lingua_core.ps1

Write-Host "=== Lingua Core Runtime - 一键启动 ===" -ForegroundColor Green
Write-Host ""

$ErrorActionPreference = "Stop"

# 配置
$NMT_SERVICE_URL = "http://127.0.0.1:9001"
$TTS_SERVICE_URL = "http://127.0.0.1:9002"
$CORE_ENGINE_PORT = 9000
$CONFIG_FILE = "lingua_core_config.toml"

# 检查配置文件
if (-not (Test-Path $CONFIG_FILE)) {
    Write-Host "⚠️  配置文件不存在: $CONFIG_FILE" -ForegroundColor Yellow
    Write-Host "请确保配置文件存在后再运行此脚本。" -ForegroundColor Yellow
    exit 1
}

# 启动 Piper TTS 服务
Write-Host "[1/3] 启动 Piper TTS 服务..." -ForegroundColor Cyan
try {
    Start-Process -NoNewWindow -FilePath "piper" -ArgumentList "--server", "--port", "9002" -ErrorAction Stop
    Write-Host "  ✅ Piper TTS 服务已启动（端口 9002）" -ForegroundColor Green
} catch {
    Write-Host "  ⚠️  无法启动 Piper TTS 服务: $_" -ForegroundColor Yellow
    Write-Host "  请确保 piper 命令在 PATH 中，或手动启动 TTS 服务" -ForegroundColor Yellow
}

# 启动 Python NMT 服务
Write-Host ""
Write-Host "[2/3] 启动 Python NMT 服务..." -ForegroundColor Cyan
try {
    # 检查虚拟环境
    if (-not (Test-Path "venv")) {
        Write-Host "  创建虚拟环境..." -ForegroundColor Gray
        python -m venv venv
    }
    
    # 激活虚拟环境并启动服务
    & "venv\Scripts\activate"
    Start-Process -NoNewWindow -FilePath "python" -ArgumentList "-m", "uvicorn", "nmt_service:app", "--host", "127.0.0.1", "--port", "9001" -ErrorAction Stop
    Write-Host "  ✅ Python NMT 服务已启动（端口 9001）" -ForegroundColor Green
} catch {
    Write-Host "  ⚠️  无法启动 Python NMT 服务: $_" -ForegroundColor Yellow
    Write-Host "  请确保 Python 和 uvicorn 已安装，或手动启动 NMT 服务" -ForegroundColor Yellow
}

# 等待服务启动
Write-Host ""
Write-Host "等待服务启动..." -ForegroundColor Cyan
Start-Sleep -Seconds 3

# 启动 CoreEngine
Write-Host ""
Write-Host "[3/3] 启动 CoreEngine..." -ForegroundColor Cyan
try {
    # 构建 CoreEngine（如果尚未构建）
    if (-not (Test-Path "target\release\core_engine.exe")) {
        Write-Host "  构建 CoreEngine..." -ForegroundColor Gray
        cargo build --release --bin core_engine
    }
    
    Start-Process -NoNewWindow -FilePath "target\release\core_engine.exe" -ArgumentList "--config", $CONFIG_FILE -ErrorAction Stop
    Write-Host "  ✅ CoreEngine 已启动（端口 $CORE_ENGINE_PORT）" -ForegroundColor Green
} catch {
    Write-Host "  ⚠️  无法启动 CoreEngine: $_" -ForegroundColor Yellow
    Write-Host "  请确保已构建 CoreEngine，或手动启动" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== 启动完成 ===" -ForegroundColor Green
Write-Host ""
Write-Host "服务状态：" -ForegroundColor Cyan
Write-Host "  NMT: $NMT_SERVICE_URL - 检查 /health" -ForegroundColor White
Write-Host "  TTS: $TTS_SERVICE_URL - 检查 /health" -ForegroundColor White
Write-Host "  CoreEngine: http://127.0.0.1:$CORE_ENGINE_PORT - 检查 /health" -ForegroundColor White
Write-Host ""
Write-Host "API 端点：" -ForegroundColor Cyan
Write-Host "  POST http://127.0.0.1:$CORE_ENGINE_PORT/s2s - 整句翻译" -ForegroundColor White
Write-Host "  WS   ws://127.0.0.1:$CORE_ENGINE_PORT/stream - 流式翻译" -ForegroundColor White
Write-Host "  GET  http://127.0.0.1:$CORE_ENGINE_PORT/health - 健康检查" -ForegroundColor White
Write-Host ""

