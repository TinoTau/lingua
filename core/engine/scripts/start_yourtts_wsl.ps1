# 从 Windows 在 WSL 中启动 YourTTS 服务

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Starting YourTTS Service in WSL" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 获取项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)

# 转换为 WSL 路径（假设项目在 D:\Programs\github\lingua）
# 注意：需要根据实际路径调整
$wslPath = $projectRoot -replace '^([A-Z]):', '/mnt/$1' -replace '\\', '/'
$wslPath = $wslPath.ToLower()

Write-Host "Project root (Windows): $projectRoot" -ForegroundColor Yellow
Write-Host "Project root (WSL): $wslPath" -ForegroundColor Yellow
Write-Host ""

# 检查 WSL 是否可用
try {
    $wslVersion = wsl --version 2>&1
    Write-Host "WSL is available" -ForegroundColor Green
} catch {
    Write-Host "Error: WSL is not available" -ForegroundColor Red
    Write-Host "Please install WSL2 first" -ForegroundColor Red
    exit 1
}

# 检查 GPU 是否可用（在 WSL 中）
Write-Host "Checking GPU availability in WSL..." -ForegroundColor Cyan
# 使用 bash -c 执行命令，抑制 systemd 警告
$gpuCheck = wsl bash -c "nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null" 2>$null
if ($LASTEXITCODE -eq 0 -and $gpuCheck -and $gpuCheck -notmatch "error|not found|Failed to start") {
    # 过滤掉 systemd 警告信息
    $gpuName = ($gpuCheck -split "`n" | Where-Object { $_ -notmatch "systemd|Failed to start" } | Select-Object -First 1).Trim()
    if ($gpuName) {
        Write-Host "✅ GPU available: $gpuName" -ForegroundColor Green
        $useGpu = "--gpu"
    } else {
        Write-Host "⚠️  GPU check returned unexpected output, trying CPU mode" -ForegroundColor Yellow
        $useGpu = ""
    }
} else {
    # 尝试另一种方法：直接在 WSL 中检查
    $gpuCheck2 = wsl bash -c "command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L 2>/dev/null | head -1" 2>$null
    if ($gpuCheck2 -and $gpuCheck2 -match "GPU") {
        Write-Host "✅ GPU detected: $gpuCheck2" -ForegroundColor Green
        $useGpu = "--gpu"
    } else {
        Write-Host "⚠️  GPU not available or nvidia-smi failed, using CPU" -ForegroundColor Yellow
        Write-Host "   Note: systemd warnings can be ignored if GPU is actually available" -ForegroundColor Gray
        $useGpu = "--gpu"  # 即使检测失败，也尝试使用 GPU（服务内部会fallback到CPU）
    }
}

Write-Host ""
Write-Host "Starting YourTTS service in WSL..." -ForegroundColor Cyan
Write-Host "  Port: 5004" -ForegroundColor Yellow
Write-Host "  Host: 0.0.0.0 (accessible from Windows)" -ForegroundColor Yellow
Write-Host "  GPU: $([string]::IsNullOrEmpty($useGpu) ? 'No' : 'Yes')" -ForegroundColor Yellow
Write-Host ""

# 在 WSL 中启动服务（使用 Python 3.10 环境）
# 注意：使用 Start-Process 以在新窗口中运行
$wslCommand = "cd $wslPath && source venv-wsl-py310/bin/activate && python core/engine/scripts/yourtts_service.py $useGpu --port 5004 --host 0.0.0.0"

Start-Process wsl -ArgumentList "bash", "-c", $wslCommand -WindowStyle Normal

Write-Host ""
Write-Host "✅ YourTTS service started in WSL" -ForegroundColor Green
Write-Host "   Service endpoint: http://127.0.0.1:5004" -ForegroundColor Cyan
Write-Host "   Check the WSL window for status" -ForegroundColor Yellow

