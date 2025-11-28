# 简化版一键启动脚本
# 在单独的窗口中启动每个服务

Write-Host "=== Lingua Service Startup (Multi-Window Mode) ===" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 检查必要的文件
$coreEnginePath = Join-Path $scriptDir "core\engine\target\release\core_engine.exe"
$configPath = Join-Path $scriptDir "lingua_core_config.toml"
$nmtServicePath = Join-Path $scriptDir "services\nmt_m2m100"

if (-not (Test-Path $coreEnginePath)) {
    Write-Host "Error: CoreEngine executable not found" -ForegroundColor Red
    Write-Host "Please build CoreEngine first: cd core\engine && cargo build --release --bin core_engine" -ForegroundColor Yellow
    exit 1
}

# 设置 CUDA 环境变量
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
if (Test-Path $cudaPath) {
    $env:CUDA_PATH = $cudaPath
    $env:CUDAToolkit_ROOT = $cudaPath
    $env:CUDA_ROOT = $cudaPath
    $env:CUDA_HOME = $cudaPath
    $cudaBin = Join-Path $cudaPath "bin"
    $cudaLibnvvp = Join-Path $cudaPath "libnvvp"
    $cudaNvcc = Join-Path $cudaBin "nvcc.exe"
    $env:CMAKE_CUDA_COMPILER = $cudaNvcc
    $env:PATH = "$cudaBin;$cudaLibnvvp;$env:PATH"
}

# 启动 TTS 服务（WSL 中的 Piper，新窗口）
Write-Host "Starting TTS service (new window, WSL)..." -ForegroundColor Cyan
$ttsServiceScript = Join-Path $scriptDir "scripts\wsl2_piper\start_piper_service.sh"
if (Test-Path $ttsServiceScript) {
    try {
        # 转换 Windows 路径为 WSL 路径
        $fullPath = (Resolve-Path $ttsServiceScript).Path
        if ($fullPath -match '^([A-Z]):\\(.*)$') {
            $drive = $matches[1].ToLower()
            $pathPart = $matches[2] -replace '\\', '/'
            $wslPath = "/mnt/$drive/$pathPart"
        }
        # 在新窗口中启动 WSL 中的 TTS 服务
        $ttsCommand = "Write-Host '=== TTS Service (Piper in WSL) ===' -ForegroundColor Green; wsl bash $wslPath"
        Start-Process powershell -ArgumentList "-NoExit", "-Command", $ttsCommand
        Write-Host "  TTS service starting in new window (WSL, port 5005)" -ForegroundColor Green
        
        # 等待服务启动，然后配置端口转发
        Start-Sleep -Seconds 3
        try {
            # 获取 WSL IP 地址
            $wslIp = (wsl hostname -I).Trim().Split()[0]
            if ($wslIp) {
                # 删除旧的端口转发规则
                netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1 2>&1 | Out-Null
                # 添加新的端口转发规则
                $result = netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp 2>&1
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "  Port forwarding configured: 127.0.0.1:5005 -> $wslIp:5005" -ForegroundColor Green
                } else {
                    Write-Host "  Warning: Port forwarding may require administrator privileges" -ForegroundColor Yellow
                }
            }
        } catch {
            Write-Host "  Warning: Port forwarding may need manual configuration" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "  Warning: Failed to start TTS service: $_" -ForegroundColor Yellow
        Write-Host "  TTS service will need to be started manually" -ForegroundColor Yellow
    }
} else {
    Write-Host "  Warning: TTS service script not found: $ttsServiceScript" -ForegroundColor Yellow
    Write-Host "  TTS service will need to be started manually" -ForegroundColor Yellow
}

Start-Sleep -Seconds 2

# 启动 NMT 服务
Write-Host "Starting NMT service (new window)..." -ForegroundColor Cyan
# 优先尝试使用本地文件（完全禁用网络验证）
# 如果失败，会自动从配置文件读取 token
# 同时清除过期的 token 缓存
$nmtCommand = "cd '$nmtServicePath'; .\venv\Scripts\Activate.ps1; Remove-Item -Path `"`$env:USERPROFILE\.cache\huggingface\token`" -Force -ErrorAction SilentlyContinue; `$env:HF_LOCAL_FILES_ONLY='true'; Write-Host '=== NMT Service (GPU) ===' -ForegroundColor Green; uvicorn nmt_service:app --host 127.0.0.1 --port 5008"
Start-Process powershell -ArgumentList "-NoExit", "-Command", $nmtCommand

Start-Sleep -Seconds 3

# 启动 CoreEngine
Write-Host "Starting CoreEngine (new window)..." -ForegroundColor Cyan
# 使用 here-string 正确转义包含空格的路径
$coreCommand = @"
cd '$scriptDir'
`$env:CUDA_PATH = '$cudaPath'
`$env:CUDAToolkit_ROOT = '$cudaPath'
`$env:CUDA_ROOT = '$cudaPath'
`$env:CUDA_HOME = '$cudaPath'
`$env:CMAKE_CUDA_COMPILER = '$cudaNvcc'
`$cudaBinPath = '$cudaBin'
`$cudaLibPath = '$cudaLibnvvp'
`$env:PATH = `"`$cudaBinPath;`$cudaLibPath;`$env:PATH`"
Write-Host '=== CoreEngine (ASR GPU) ===' -ForegroundColor Green
.\core\engine\target\release\core_engine.exe --config lingua_core_config.toml
"@
Start-Process powershell -ArgumentList "-NoExit", "-Command", $coreCommand

Start-Sleep -Seconds 2

# 启动 Web 前端服务器（新窗口，端口 8080）
Write-Host "Starting Web Frontend (new window)..." -ForegroundColor Cyan
$webPwaPath = Join-Path $scriptDir "clients\web_pwa"
$webServerScript = Join-Path $webPwaPath "start_server.ps1"
if (Test-Path $webServerScript) {
    $webCommand = "cd '$webPwaPath'; Write-Host '=== Web Frontend (PWA) ===' -ForegroundColor Green; .\start_server.ps1 -Port 8080"
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $webCommand
    Write-Host "  Web frontend starting in new window (port 8080)" -ForegroundColor Green
} else {
    Write-Host "  Warning: Web frontend script not found at $webServerScript" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "All services started in new windows" -ForegroundColor Green
Write-Host ""
Write-Host "Service URLs:" -ForegroundColor Cyan
Write-Host "  - TTS Service: http://127.0.0.1:5005" -ForegroundColor White
Write-Host "  - NMT Service: http://127.0.0.1:5008" -ForegroundColor White
Write-Host "  - CoreEngine: http://0.0.0.0:9000" -ForegroundColor White
Write-Host "  - Web Frontend: http://localhost:8080" -ForegroundColor White
Write-Host ""
Write-Host "To stop services:" -ForegroundColor Cyan
Write-Host "  - Method 1: Close the corresponding PowerShell windows" -ForegroundColor White
Write-Host "  - Method 2: Run .\stop_all_services.ps1" -ForegroundColor White
