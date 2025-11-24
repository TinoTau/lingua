# M2M100 实时翻译系统 - 一键启动脚本（Windows PowerShell）
# 
# 用途：启动所有必需的服务（NMT、TTS）并检查健康状态
# 
# 使用方法：
#   .\scripts\start_all.ps1

Write-Host "=== M2M100 实时翻译系统 - 一键启动 ===" -ForegroundColor Green
Write-Host ""

$ErrorActionPreference = "Stop"

# 配置
$NMT_SERVICE_URL = "http://127.0.0.1:5008"
$TTS_SERVICE_URL = "http://127.0.0.1:5005"
$NMT_SERVICE_DIR = "services\nmt_m2m100"
$TTS_SERVICE_SCRIPT = "scripts\wsl2_piper\start_piper_service.sh"

# 检查服务是否已运行
function Test-ServiceHealth {
    param(
        [string]$Url,
        [string]$ServiceName
    )
    
    try {
        $response = Invoke-RestMethod -Uri "$Url/health" -Method GET -TimeoutSec 2 -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

# 启动 NMT 服务
Write-Host "[1/3] 检查 NMT 服务..." -ForegroundColor Cyan
if (Test-ServiceHealth -Url $NMT_SERVICE_URL -ServiceName "NMT") {
    Write-Host "  ✅ NMT 服务已在运行" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  NMT 服务未运行，正在启动..." -ForegroundColor Yellow
    Write-Host "  请在另一个终端运行以下命令启动 NMT 服务：" -ForegroundColor White
    Write-Host "    cd $NMT_SERVICE_DIR" -ForegroundColor Gray
    Write-Host "    uvicorn nmt_service:app --host 127.0.0.1 --port 5008" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  或者按 Enter 继续（稍后手动启动）..." -ForegroundColor Yellow
    Read-Host
}

# 启动 TTS 服务
Write-Host ""
Write-Host "[2/3] 检查 TTS 服务..." -ForegroundColor Cyan
if (Test-ServiceHealth -Url $TTS_SERVICE_URL -ServiceName "TTS") {
    Write-Host "  ✅ TTS 服务已在运行" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  TTS 服务未运行，正在启动..." -ForegroundColor Yellow
    Write-Host "  请在 WSL2 终端运行以下命令启动 TTS 服务：" -ForegroundColor White
    Write-Host "    wsl bash $TTS_SERVICE_SCRIPT" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  或者按 Enter 继续（稍后手动启动）..." -ForegroundColor Yellow
    Read-Host
}

# 最终健康检查
Write-Host ""
Write-Host "[3/3] 最终健康检查..." -ForegroundColor Cyan
$nmtOk = Test-ServiceHealth -Url $NMT_SERVICE_URL -ServiceName "NMT"
$ttsOk = Test-ServiceHealth -Url $TTS_SERVICE_URL -ServiceName "TTS"

if ($nmtOk -and $ttsOk) {
    Write-Host "  ✅ 所有服务运行正常！" -ForegroundColor Green
    Write-Host ""
    Write-Host "服务状态：" -ForegroundColor Cyan
    Write-Host "  NMT: $NMT_SERVICE_URL - ✅ 正常" -ForegroundColor Green
    Write-Host "  TTS: $TTS_SERVICE_URL - ✅ 正常" -ForegroundColor Green
    Write-Host ""
    Write-Host "现在可以运行集成测试或启动主程序了！" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  部分服务未就绪：" -ForegroundColor Yellow
    if (-not $nmtOk) {
        Write-Host "    NMT: ❌ 未运行" -ForegroundColor Red
    } else {
        Write-Host "    NMT: ✅ 正常" -ForegroundColor Green
    }
    if (-not $ttsOk) {
        Write-Host "    TTS: ❌ 未运行" -ForegroundColor Red
    } else {
        Write-Host "    TTS: ✅ 正常" -ForegroundColor Green
    }
    Write-Host ""
    Write-Host "请确保所有服务都已启动后再运行测试。" -ForegroundColor Yellow
}

Write-Host ""

