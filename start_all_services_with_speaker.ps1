# 一键启动所有服务（包含音色识别和分配功能）
# 服务列表：
# - Speaker Embedding (Windows, 端口 5003)
# - YourTTS (WSL2, 端口 5004)
# - NMT (Windows, 端口 5008)
# - ASR Service (Windows, 端口 6006) - Faster-Whisper ASR 服务
# - Piper TTS (WSL2, 端口 5005) - 可选，如果使用 YourTTS 可能不需要
# - CoreEngine (Windows, 端口 9000) - 包含 VAD（ASR 通过 HTTP 调用）
# - Web Frontend (Windows, 端口 8080) - Web 前端界面

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Lingua All Services Startup (With Speaker Recognition)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# 检查必要的文件
$coreEnginePath = Join-Path $scriptDir "core\engine\target\release\core_engine.exe"
$configPath = Join-Path $scriptDir "lingua_core_config.toml"

if (-not (Test-Path $coreEnginePath)) {
    Write-Host "[ERROR] CoreEngine executable not found" -ForegroundColor Red
    Write-Host "[INFO] Please build CoreEngine first:" -ForegroundColor Yellow
    Write-Host "  cd core\engine && cargo build --release --bin core_engine" -ForegroundColor Yellow
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

# ============================================================
# 1. 启动 Speaker Embedding 服务（Windows, 端口 5003）
# ============================================================
Write-Host "[1/6] Starting Speaker Embedding service..." -ForegroundColor Cyan
$pythonPath = "D:\Program Files\Anaconda\envs\lingua-py310\python.exe"
$speakerEmbeddingScript = Join-Path $scriptDir "core\engine\scripts\speaker_embedding_service.py"

if ((Test-Path $pythonPath) -and (Test-Path $speakerEmbeddingScript)) {
    $speakerEmbeddingCommand = @"
`$Host.UI.RawUI.WindowTitle = 'Speaker Embedding Service (Port 5003)'
cd '$scriptDir'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  Speaker Embedding Service (GPU)' -ForegroundColor Green
Write-Host '  Port: 5003' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
& '$pythonPath' '$speakerEmbeddingScript' --gpu
"@
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $speakerEmbeddingCommand
    Write-Host "  ✓ Speaker Embedding service starting in new window (port 5003)" -ForegroundColor Green
    Start-Sleep -Seconds 5  # 增加等待时间，确保服务完全启动
}
else {
    Write-Host "  ⚠ Speaker Embedding service script not found, skipping..." -ForegroundColor Yellow
}

# ============================================================
# 2. 启动 YourTTS 服务（WSL2, 端口 5004）
# ============================================================
Write-Host "[2/6] Starting YourTTS service (WSL)..." -ForegroundColor Cyan
$yourttsScript = Join-Path $scriptDir "core\engine\scripts\start_yourtts_wsl.ps1"
if (Test-Path $yourttsScript) {
    try {
        # 转换为 WSL 路径
        $fullPath = (Resolve-Path $scriptDir).Path
        if ($fullPath -match '^([A-Z]):\\(.*)$') {
            $drive = $matches[1].ToLower()
            $pathPart = $matches[2] -replace '\\', '/'
            $wslPath = "/mnt/$drive/$pathPart"
        }
        
        # 检查 GPU（在 WSL 中）
        Write-Host "  Checking GPU availability in WSL..." -ForegroundColor Gray
        # 使用 bash -c 抑制 systemd 警告
        $gpuCheck = wsl bash -c "nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null" 2>$null
        if ($LASTEXITCODE -eq 0 -and $gpuCheck -and $gpuCheck -notmatch "error|not found|Failed to start") {
            # 过滤掉 systemd 警告信息
            $gpuName = ($gpuCheck -split "`n" | Where-Object { $_ -notmatch "systemd|Failed to start" } | Select-Object -First 1).Trim()
            if ($gpuName) {
                Write-Host "  ✅ GPU available: $gpuName" -ForegroundColor Green
                $useGpu = "--gpu"
            } else {
                Write-Host "  ⚠️  GPU check returned unexpected output, using CPU" -ForegroundColor Yellow
                $useGpu = ""
            }
        } else {
            # 尝试另一种方法检查 GPU
            $gpuCheck2 = wsl bash -c "command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L 2>/dev/null | head -1" 2>$null
            if ($gpuCheck2 -and $gpuCheck2 -match "GPU") {
                Write-Host "  ✅ GPU detected: $gpuCheck2" -ForegroundColor Green
                $useGpu = "--gpu"
            } else {
                Write-Host "  ⚠️  GPU not available, using CPU" -ForegroundColor Yellow
                Write-Host "     Note: systemd warnings can be ignored if GPU is actually available" -ForegroundColor Gray
                $useGpu = "--gpu"  # 即使检测失败，也尝试使用 GPU（服务内部会fallback到CPU）
            }
        }
        
        # 构建 bash 命令（使用转义避免 PowerShell 解析问题）
        # 使用 Python 3.10 环境 (venv-wsl-py310) 以确保 librosa 兼容性
        $bashCmd = "cd $wslPath && source venv-wsl-py310/bin/activate && python3 core/engine/scripts/yourtts_service.py $useGpu --port 5004 --host 0.0.0.0"
        
        # 在 PowerShell 窗口中启动 WSL 服务，设置窗口标题
        # 注意：使用转义的变量引用，避免在 here-string 中立即解析
        # 使用 Python 3.10 环境以确保 librosa 兼容性
        $yourttsCommand = @"
`$Host.UI.RawUI.WindowTitle = 'YourTTS Service (WSL - Port 5004 - Python 3.10)'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  YourTTS Service (WSL - Zero-shot TTS)' -ForegroundColor Green
Write-Host '  Port: 5004' -ForegroundColor Yellow
Write-Host '  Environment: Python 3.10 (venv-wsl-py310)' -ForegroundColor Cyan
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
`$bashCmd = '$bashCmd'
wsl bash -c `$bashCmd
Write-Host ''
Write-Host 'Service stopped. Press any key to close...' -ForegroundColor Yellow
`$null = `$Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
"@
        Start-Process powershell -ArgumentList "-NoExit", "-Command", $yourttsCommand
        Write-Host "  ✓ YourTTS service starting in new window (WSL, port 5004)" -ForegroundColor Green
        
        # 等待服务启动，然后配置端口转发
        Start-Sleep -Seconds 8  # 增加等待时间，确保 WSL 服务完全启动
        try {
            $wslIp = (wsl -d "Ubuntu-22.04" hostname -I).Trim().Split()[0]
            if ($wslIp) {
                netsh interface portproxy delete v4tov4 listenport=5004 listenaddress=127.0.0.1 2>&1 | Out-Null
                netsh interface portproxy add v4tov4 listenport=5004 listenaddress=127.0.0.1 connectport=5004 connectaddress=$wslIp 2>&1 | Out-Null
                Write-Host "  ✓ Port forwarding configured: 127.0.0.1:5004 -> $wslIp:5004" -ForegroundColor Green
            }
        }
        catch {
            Write-Host "  ⚠ Port forwarding may need manual configuration" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Host "  ⚠ Failed to start YourTTS service: $_" -ForegroundColor Yellow
    }
}
else {
    Write-Host "  ⚠ YourTTS service script not found, skipping..." -ForegroundColor Yellow
}

Start-Sleep -Seconds 2

# ============================================================
# 3. 启动 NMT 服务（Windows, 端口 5008）
# ============================================================
Write-Host "[3/6] Starting NMT service..." -ForegroundColor Cyan
$nmtServiceDir = Join-Path $scriptDir "services\nmt_m2m100"
if (Test-Path $nmtServiceDir) {
    $nmtPythonPath = Join-Path $nmtServiceDir "venv\Scripts\python.exe"
    if (-not (Test-Path $nmtPythonPath)) {
        $nmtPythonPath = "python"
    }
    
    $nmtCommand = @"
`$Host.UI.RawUI.WindowTitle = 'NMT Service (Port 5008)'
cd '$nmtServiceDir'
if (Test-Path 'venv\Scripts\Activate.ps1') { .\venv\Scripts\Activate.ps1 }
`$env:HF_LOCAL_FILES_ONLY='true'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  NMT Service (M2M100)' -ForegroundColor Green
Write-Host '  Port: 5008' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
& '$nmtPythonPath' -m uvicorn nmt_service:app --host 127.0.0.1 --port 5008
"@
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $nmtCommand
    Write-Host "  ✓ NMT service starting in new window (port 5008)" -ForegroundColor Green
    Start-Sleep -Seconds 5  # 增加等待时间，确保服务完全启动
}
else {
    Write-Host "  ⚠ NMT service directory not found, skipping..." -ForegroundColor Yellow
}

# ============================================================
# 4. 启动 Piper TTS 服务（WSL2, 端口 5005）- 可选
# ============================================================
Write-Host "[4/6] Starting Piper TTS service (WSL, optional)..." -ForegroundColor Cyan
$piperScript = Join-Path $scriptDir "scripts\wsl2_piper\start_piper_service.sh"
if (Test-Path $piperScript) {
    try {
        $fullPath = (Resolve-Path $piperScript).Path
        if ($fullPath -match '^([A-Z]):\\(.*)$') {
            $drive = $matches[1].ToLower()
            $pathPart = $matches[2] -replace '\\', '/'
            $wslPath = "/mnt/$drive/$pathPart"
        }
        $piperCommand = @"
`$Host.UI.RawUI.WindowTitle = 'Piper TTS Service (WSL - Port 5005)'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  Piper TTS Service (WSL)' -ForegroundColor Green
Write-Host '  Port: 5005' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
wsl bash $wslPath
Write-Host ''
Write-Host 'Service stopped. Press any key to close...' -ForegroundColor Yellow
`$null = `$Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
"@
        Start-Process powershell -ArgumentList "-NoExit", "-Command", $piperCommand
        Write-Host "  ✓ Piper TTS service starting in new window (WSL, port 5005)" -ForegroundColor Green
        
        Start-Sleep -Seconds 3
        try {
            $wslIp = (wsl hostname -I).Trim().Split()[0]
            if ($wslIp) {
                netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1 2>&1 | Out-Null
                netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp 2>&1 | Out-Null
                Write-Host "  ✓ Port forwarding configured: 127.0.0.1:5005 -> $wslIp:5005" -ForegroundColor Green
            }
        }
        catch {
            Write-Host "  ⚠ Port forwarding may need manual configuration" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Host "  ⚠ Failed to start Piper TTS service: $_" -ForegroundColor Yellow
    }
}
else {
    Write-Host "  ⚠ Piper TTS service script not found, skipping..." -ForegroundColor Yellow
}

Start-Sleep -Seconds 2

# ============================================================
# 5. 启动 ASR 服务 (Faster-Whisper) (Windows, 端口 6006)
# ============================================================
Write-Host "[5/7] Starting ASR service (Faster-Whisper)..." -ForegroundColor Cyan
$asrScript = Join-Path $scriptDir "core\engine\scripts\start_asr_service.ps1"
if (Test-Path $asrScript) {
    Write-Host "  Found ASR script: $asrScript" -ForegroundColor Gray
    $asrServiceDir = Join-Path $scriptDir "core\engine\scripts"
    $asrCommand = @"
`$ErrorActionPreference = 'Continue'
`$Host.UI.RawUI.WindowTitle = 'ASR Service (Faster-Whisper - Port 6006)'
cd '$asrServiceDir'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  ASR Service (Faster-Whisper)' -ForegroundColor Green
Write-Host '  Port: 6006' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
try {
    & '$asrScript'
} catch {
    Write-Host "Error: `$_" -ForegroundColor Red
    Write-Host "Press any key to close..." -ForegroundColor Yellow
    `$null = `$Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
}
"@
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $asrCommand
    Write-Host "  ✓ ASR service starting in new window (port 6006)" -ForegroundColor Green
    Write-Host "  Note: Check the ASR service window for startup status" -ForegroundColor Gray
    Start-Sleep -Seconds 10  # 增加等待时间，确保服务完全启动（模型加载需要时间）
} else {
    Write-Host "  ⚠ ASR service script not found at: $asrScript" -ForegroundColor Yellow
    Write-Host "  ⚠ Please ensure ASR service is started manually on port 6006" -ForegroundColor Yellow
}

Start-Sleep -Seconds 2

# ============================================================
# 6. 启动 CoreEngine（Windows, 端口 9000）- 包含 VAD（ASR 通过 HTTP 调用）
# ============================================================
Write-Host "[6/7] Starting CoreEngine (with VAD, ASR via HTTP)..." -ForegroundColor Cyan
$coreCommand = @"
`$Host.UI.RawUI.WindowTitle = 'CoreEngine (Port 9000) - VAD + ASR + Speaker Recognition'
cd '$scriptDir'
`$env:CUDA_PATH = '$cudaPath'
`$env:CUDAToolkit_ROOT = '$cudaPath'
`$env:CUDA_ROOT = '$cudaPath'
`$env:CUDA_HOME = '$cudaPath'
`$env:CMAKE_CUDA_COMPILER = '$cudaNvcc'
`$cudaBinPath = '$cudaBin'
`$cudaLibPath = '$cudaLibnvvp'
`$env:PATH = `"`$cudaBinPath;`$cudaLibPath;`$env:PATH`"
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  CoreEngine' -ForegroundColor Green
Write-Host '  Port: 9000' -ForegroundColor Yellow
Write-Host '  Features: VAD + ASR + Speaker Recognition' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
.\core\engine\target\release\core_engine.exe --config lingua_core_config.toml
"@
Start-Process powershell -ArgumentList "-NoExit", "-Command", $coreCommand
Write-Host "  ✓ CoreEngine starting in new window (port 9000)" -ForegroundColor Green

Start-Sleep -Seconds 2

# ============================================================
# 7. 启动 Web 前端服务器（Windows, 端口 8080）
# ============================================================
Write-Host "[7/7] Starting Web Frontend..." -ForegroundColor Cyan
$webPwaPath = Join-Path $scriptDir "clients\web_pwa"
$webServerScript = Join-Path $webPwaPath "start_server.ps1"
if (Test-Path $webServerScript) {
    $webCommand = @"
`$Host.UI.RawUI.WindowTitle = 'Web Frontend (Port 8080)'
cd '$webPwaPath'
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host '  Web Frontend (PWA)' -ForegroundColor Green
Write-Host '  Port: 8080' -ForegroundColor Yellow
Write-Host '  URL: http://localhost:8080' -ForegroundColor Yellow
Write-Host '============================================================' -ForegroundColor Cyan
Write-Host ''
.\start_server.ps1 -Port 8080
"@
    Start-Process powershell -ArgumentList "-NoExit", "-Command", $webCommand
    Write-Host "  ✓ Web Frontend starting in new window (port 8080)" -ForegroundColor Green
}
else {
    Write-Host "  ⚠ Web frontend script not found, skipping..." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "  All services started successfully!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Each service is running in a separate window:" -ForegroundColor Cyan
Write-Host "  📋 Window titles:" -ForegroundColor Yellow
Write-Host "     - Speaker Embedding Service (Port 5003)" -ForegroundColor White
Write-Host "     - YourTTS Service (WSL - Port 5004)" -ForegroundColor White
Write-Host "     - NMT Service (Port 5008)" -ForegroundColor White
Write-Host "     - ASR Service - Faster-Whisper (Port 6006)" -ForegroundColor White
Write-Host "     - Piper TTS Service (WSL - Port 5005)" -ForegroundColor White
Write-Host "     - CoreEngine (Port 9000) - VAD + ASR (HTTP) + Speaker Recognition" -ForegroundColor White
Write-Host "     - Web Frontend (Port 8080)" -ForegroundColor White
Write-Host ""
Write-Host "Service URLs:" -ForegroundColor Cyan
Write-Host "  - Speaker Embedding: http://127.0.0.1:5003" -ForegroundColor White
Write-Host "  - YourTTS:           http://127.0.0.1:5004" -ForegroundColor White
Write-Host "  - NMT Service:       http://127.0.0.1:5008" -ForegroundColor White
Write-Host "  - ASR Service:       http://127.0.0.1:6006" -ForegroundColor White
Write-Host "  - Piper TTS:         http://127.0.0.1:5005" -ForegroundColor White
Write-Host "  - CoreEngine:        http://127.0.0.1:9000" -ForegroundColor White
Write-Host "  - Web Frontend:      http://localhost:8080" -ForegroundColor Cyan
Write-Host ""
Write-Host "Features enabled:" -ForegroundColor Cyan
Write-Host "  ✓ VAD (Voice Activity Detection) - Built-in" -ForegroundColor Green
Write-Host "  ✓ ASR (Automatic Speech Recognition) - Faster-Whisper (HTTP, Port 6006)" -ForegroundColor Green
Write-Host "  ✓ Speaker Recognition (Embedding-based)" -ForegroundColor Green
Write-Host "  ✓ Voice Assignment (YourTTS zero-shot)" -ForegroundColor Green
Write-Host ""
Write-Host "💡 Tip: Each service window shows its own logs for easy debugging" -ForegroundColor Cyan
Write-Host ""
Write-Host "To stop services:" -ForegroundColor Cyan
Write-Host "  - Close the corresponding PowerShell windows" -ForegroundColor White
Write-Host "  - Or run: .\stop_all_services.ps1" -ForegroundColor White
Write-Host ""

