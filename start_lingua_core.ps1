# Lingua Core Runtime - One-click startup script (Windows PowerShell)
# 
# Purpose: Start all required services (NMT, TTS, CoreEngine)
# 
# Usage:
#   .\start_lingua_core.ps1

Write-Host "=== Lingua Core Runtime - One-click Startup ===" -ForegroundColor Green
Write-Host ""

$ErrorActionPreference = "Continue"  # Change to Continue to allow script to continue on errors

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# Configuration
$NMT_SERVICE_URL = "http://127.0.0.1:5008"
$TTS_SERVICE_URL = "http://127.0.0.1:5005"
$CORE_ENGINE_PORT = 9000
$CONFIG_FILE = "lingua_core_config.toml"
$NMT_SERVICE_DIR = "services\nmt_m2m100"
$CORE_ENGINE_DIR = "core\engine"
$WEB_ROOT = "clients\web_pwa"
$WEB_PORT = 8080

# Process tracking for cleanup
$script:StartedProcesses = @()
$script:WslPids = @()

# Cleanup function
function Stop-AllServices {
    Write-Host ""
    Write-Host "=== Stopping all services ===" -ForegroundColor Yellow
    
    # Stop CoreEngine
    $coreEngineProcesses = Get-Process -Name "core_engine" -ErrorAction SilentlyContinue
    if ($coreEngineProcesses) {
        Write-Host "  Stopping CoreEngine..." -ForegroundColor Gray
        $coreEngineProcesses | Stop-Process -Force -ErrorAction SilentlyContinue
    }
    
    # Stop NMT service (Python uvicorn) - find by port
    $nmtConnections = Get-NetTCPConnection -LocalPort 5008 -ErrorAction SilentlyContinue
    foreach ($conn in $nmtConnections) {
        if ($conn.OwningProcess) {
            $proc = Get-Process -Id $conn.OwningProcess -ErrorAction SilentlyContinue
            if ($proc -and ($proc.ProcessName -eq "python" -or $proc.ProcessName -eq "pythonw")) {
                Write-Host "  Stopping NMT service (PID: $($proc.Id))..." -ForegroundColor Gray
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            }
        }
    }
    
    # Stop processes started by this script
    foreach ($proc in $script:StartedProcesses) {
        if (-not $proc.HasExited) {
            Write-Host "  Stopping process: $($proc.ProcessName) (PID: $($proc.Id))" -ForegroundColor Gray
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }
    
    # Stop WSL processes (TTS service)
    Write-Host "  Stopping TTS service in WSL..." -ForegroundColor Gray
    # Try to kill piper_http_server processes in WSL
    wsl bash -c "pkill -f piper_http_server || true" 2>$null
    # Also try to kill uvicorn processes on port 5005
    wsl bash -c "lsof -ti:5005 | xargs kill -9 2>/dev/null || true" 2>$null
    
    # Also try to kill by port
    Write-Host "  Cleaning up processes on ports..." -ForegroundColor Gray
    $ports = @(5005, 5008, 9000, $WEB_PORT)
    foreach ($port in $ports) {
        $connections = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
        foreach ($conn in $connections) {
            if ($conn.OwningProcess) {
                Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
            }
        }
    }
    
    Write-Host "  All services stopped." -ForegroundColor Green
}

# Configure port forwarding for TTS service
function Configure-TtsPortForwarding {
    Write-Host "[Port Forward] Configuring port forwarding for TTS service..." -ForegroundColor Gray
    
    # Get WSL IP address (suppress all output to avoid formatting issues)
    $wslIp = (wsl bash -c "hostname -I | awk '{print `$1}'" 2>$null | Out-String).Trim()
    if (-not $wslIp) {
        Write-Host "[Port Forward] WARNING: Could not get WSL IP address" -ForegroundColor Yellow
        return $false
    }
    
    Write-Host "[Port Forward] WSL IP: $wslIp" -ForegroundColor Gray
    
    # Check if running as administrator
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    
    if (-not $isAdmin) {
        Write-Host "[Port Forward] WARNING: Port forwarding requires administrator privileges" -ForegroundColor Yellow
        Write-Host "[Port Forward] Please run this script as Administrator, or configure manually:" -ForegroundColor Yellow
        Write-Host "[Port Forward]   netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1" -ForegroundColor Gray
        Write-Host "[Port Forward]   netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp" -ForegroundColor Gray
        return $false
    }
    
    # Remove existing port forwarding rule (if any)
    try {
        $null = netsh interface portproxy delete v4tov4 listenport=5005 listenaddress=127.0.0.1 2>&1
    } catch {
        # Ignore errors if rule doesn't exist
    }
    
    # Add new port forwarding rule
    try {
        $result = netsh interface portproxy add v4tov4 listenport=5005 listenaddress=127.0.0.1 connectport=5005 connectaddress=$wslIp 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[Port Forward] Port forwarding configured successfully" -ForegroundColor Green
            Start-Sleep -Seconds 1
            return $true
        } else {
            Write-Host "[Port Forward] WARNING: Failed to configure port forwarding: $result" -ForegroundColor Yellow
            return $false
        }
    } catch {
        Write-Host "[Port Forward] WARNING: Failed to configure port forwarding: $_" -ForegroundColor Yellow
        return $false
    }
}

# Start web server helper
function Start-WebServer {
    param(
        [string]$RootPath,
        [int]$Port
    )

    $resolvedRoot = (Resolve-Path $RootPath -ErrorAction SilentlyContinue)
    if (-not $resolvedRoot) {
        Write-Host "  WARNING: Web root not found: $RootPath" -ForegroundColor Yellow
        return $null
    }

    Write-Host ""
    Write-Host "[4/4] Starting Web UI server..." -ForegroundColor Cyan

    $pythonCandidates = @("python", "python3")
    foreach ($candidate in $pythonCandidates) {
        $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
        if ($cmd) {
            try {
                Write-Host "  Using $candidate HTTP server (port $Port)" -ForegroundColor Gray
                $proc = Start-Process -NoNewWindow -PassThru `
                    -FilePath $cmd.Source `
                    -ArgumentList "-m", "http.server", $Port, "--directory", $resolvedRoot.Path
                Write-Host "  OK Web UI available at http://localhost:$Port" -ForegroundColor Green
                return $proc
            } catch {
                Write-Host "  WARNING: Failed to start Python HTTP server: $_" -ForegroundColor Yellow
            }
        }
    }

    $npx = Get-Command npx -ErrorAction SilentlyContinue
    if ($npx) {
        try {
            Write-Host "  Using npx http-server (port $Port)" -ForegroundColor Gray
            $proc = Start-Process -NoNewWindow -PassThru `
                -FilePath $npx.Source `
                -ArgumentList "http-server", $resolvedRoot.Path, "-p", $Port
            Write-Host "  OK Web UI available at http://localhost:$Port" -ForegroundColor Green
            return $proc
        } catch {
            Write-Host "  WARNING: Failed to start npx http-server: $_" -ForegroundColor Yellow
        }
    }

    Write-Host "  WARNING: Could not auto-start Web UI server." -ForegroundColor Yellow
    Write-Host "           Please run one of the following commands manually:" -ForegroundColor Yellow
    Write-Host "             python -m http.server $Port --directory `"$($resolvedRoot.Path)`"" -ForegroundColor Gray
    Write-Host "             npx http-server `"$($resolvedRoot.Path)`" -p $Port" -ForegroundColor Gray
    return $null
}

# Register cleanup on script exit
Register-EngineEvent PowerShell.Exiting -Action { Stop-AllServices } | Out-Null

# Handle Ctrl+C
[Console]::TreatControlCAsInput = $false
$null = Register-ObjectEvent -InputObject ([System.Console]) -EventName "CancelKeyPress" -Action {
    $_.Cancel = $true
    Stop-AllServices
    exit 0
}

# Check config file
if (-not (Test-Path $CONFIG_FILE)) {
    Write-Host "WARNING: Config file not found: $CONFIG_FILE" -ForegroundColor Yellow
    Write-Host "Please ensure the config file exists before running this script." -ForegroundColor Yellow
    Write-Host "You can create it from the template in the project root." -ForegroundColor Yellow
}

# Start Piper TTS service
Write-Host "[1/3] Starting Piper TTS service..." -ForegroundColor Cyan
Write-Host ""
$TTS_SERVICE_SCRIPT = "scripts\wsl2_piper\start_piper_service.sh"
if (Test-Path $TTS_SERVICE_SCRIPT) {
    try {
        # Convert Windows path to WSL path
        $fullPath = (Resolve-Path $TTS_SERVICE_SCRIPT).Path
        # Convert D:\path\to\file to /mnt/d/path/to/file
        if ($fullPath -match '^([A-Z]):\\(.*)$') {
            $drive = $matches[1].ToLower()
            $pathPart = $matches[2] -replace '\\', '/'
            $wslPath = "/mnt/$drive/$pathPart"
        } else {
            throw "Invalid path format: $fullPath"
        }
        
        # Start via WSL2 in background and track the process
        $wslProcess = Start-Process -NoNewWindow -PassThru wsl -ArgumentList "bash", $wslPath
        $script:StartedProcesses += $wslProcess
        Write-Host "  OK Piper TTS service starting via WSL2 (port 5005)" -ForegroundColor Green
        Write-Host "  Note: TTS service runs in WSL, use 'wsl pkill -f piper_http_server' to stop manually" -ForegroundColor Gray
        
        # Wait a bit for service to start, then configure port forwarding
        Start-Sleep -Seconds 3
        $portForwardConfigured = Configure-TtsPortForwarding
        if (-not $portForwardConfigured) {
            Write-Host "[Port Forward] Note: Port forwarding will be attempted during service readiness check" -ForegroundColor Gray
        }
    } catch {
        Write-Host "  WARNING: Failed to start Piper TTS service via WSL2: $_" -ForegroundColor Yellow
        Write-Host "  Please start TTS service manually:" -ForegroundColor Yellow
        Write-Host "    wsl bash $TTS_SERVICE_SCRIPT" -ForegroundColor Gray
    }
} else {
    # Try direct piper command
    try {
        Start-Process -NoNewWindow -FilePath "piper" -ArgumentList "--server", "--port", "9002" -ErrorAction Stop
        Write-Host "  OK Piper TTS service started (port 9002)" -ForegroundColor Green
    } catch {
        Write-Host "  WARNING: Failed to start Piper TTS service: $_" -ForegroundColor Yellow
        Write-Host "  Please start TTS service manually:" -ForegroundColor Yellow
        Write-Host "    wsl bash $TTS_SERVICE_SCRIPT" -ForegroundColor Gray
        Write-Host "    OR: piper --server --port 9002" -ForegroundColor Gray
    }
}

# Start Python NMT service
Write-Host ""
Write-Host ""
Write-Host "[2/3] Starting Python NMT service..." -ForegroundColor Cyan
if (-not (Test-Path $NMT_SERVICE_DIR)) {
    Write-Host "  ERROR: NMT service directory not found: $NMT_SERVICE_DIR" -ForegroundColor Red
    Write-Host "  Please ensure the NMT service is available." -ForegroundColor Yellow
} else {
    try {
        Push-Location $NMT_SERVICE_DIR
        
        # Check virtual environment
        if (-not (Test-Path "venv")) {
            Write-Host "  Creating virtual environment..." -ForegroundColor Gray
            python -m venv venv
        }
        
        # Install dependencies
        Write-Host "[NMT] Installing dependencies..." -ForegroundColor Gray
        & "venv\Scripts\python.exe" -m pip install -q -r requirements.txt
        
        # Start service and track the process
        $pythonExe = Join-Path (Get-Location) "venv\Scripts\python.exe"
        $nmtProcess = Start-Process -NoNewWindow -PassThru -FilePath $pythonExe -ArgumentList "-m", "uvicorn", "nmt_service:app", "--host", "127.0.0.1", "--port", "5008" -ErrorAction Stop
        $script:StartedProcesses += $nmtProcess
        Write-Host "  OK Python NMT service started (port 5008, PID: $($nmtProcess.Id))" -ForegroundColor Green
        
        Pop-Location
    } catch {
        Write-Host "  WARNING: Failed to start Python NMT service: $_" -ForegroundColor Yellow
        Write-Host "  Please start NMT service manually:" -ForegroundColor Yellow
        Write-Host "    cd $NMT_SERVICE_DIR" -ForegroundColor Gray
        Write-Host "    venv\Scripts\python.exe -m uvicorn nmt_service:app --host 127.0.0.1 --port 5008" -ForegroundColor Gray
        Pop-Location -ErrorAction SilentlyContinue
    }
}

# Wait for services to start
Write-Host ""
Write-Host "Waiting for services to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 5
Write-Host "  Checking service status..." -ForegroundColor Gray

# Check if services are ready
$maxWait = 15
$waited = 0
$nmtReady = $false
$ttsReady = $false

while ($waited -lt $maxWait) {
    if (-not $nmtReady) {
        try {
            $nmtResponse = Invoke-WebRequest -Uri "http://127.0.0.1:5008/health" -TimeoutSec 2 -ErrorAction Stop
            if ($nmtResponse.StatusCode -eq 200) { 
                $nmtReady = $true
                Write-Host "  NMT service is ready" -ForegroundColor Green
            }
        } catch { }
    }
    
    if (-not $ttsReady) {
        try {
            # 首先尝试从 Windows 直接访问（如果端口转发已配置）
            $winResponse = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -TimeoutSec 1 -ErrorAction SilentlyContinue
            if ($winResponse.StatusCode -eq 200) {
                $ttsReady = $true
                Write-Host "  TTS service is ready (Windows access)" -ForegroundColor Green
            }
        } catch {
            # 如果 Windows 访问失败，尝试通过 WSL 检查服务是否在运行
            try {
                $wslResult = wsl bash -c "curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:5005/health 2>/dev/null || echo '000'"
                if ($wslResult -eq "200") {
                    Write-Host "  TTS service is running in WSL, but Windows cannot access it" -ForegroundColor Yellow
                    # 尝试自动配置端口转发
                    if (Configure-TtsPortForwarding) {
                        # 再次尝试从 Windows 访问
                        Start-Sleep -Seconds 1
                        try {
                            $winResponse2 = Invoke-WebRequest -Uri "http://127.0.0.1:5005/health" -TimeoutSec 2 -ErrorAction Stop
                            if ($winResponse2.StatusCode -eq 200) {
                                $ttsReady = $true
                                Write-Host "  TTS service is ready (port forwarding configured)" -ForegroundColor Green
                            }
                        } catch {
                            Write-Host "  WARNING: Port forwarding configured but still cannot access from Windows" -ForegroundColor Yellow
                        }
                    }
                }
            } catch { }
        }
    }
    
    if ($nmtReady -and $ttsReady) {
        Write-Host "  All services are ready!" -ForegroundColor Green
        break
    }
    
    Start-Sleep -Seconds 1
    $waited++
}

if (-not ($nmtReady -and $ttsReady)) {
    Write-Host "  Warning: Some services may not be ready yet" -ForegroundColor Yellow
    if (-not $nmtReady) {
        Write-Host "    NMT service (port 5008) is not responding" -ForegroundColor Yellow
    }
    if (-not $ttsReady) {
        Write-Host "    TTS service (port 5005) is not responding" -ForegroundColor Yellow
        Write-Host "    Note: TTS service runs in WSL, check with: wsl bash -c 'curl http://127.0.0.1:5005/health'" -ForegroundColor Gray
        Write-Host "    Run .\check_tts_service.ps1 for detailed diagnostics" -ForegroundColor Gray
    }
}

# Start CoreEngine
Write-Host ""
Write-Host "[3/3] Starting CoreEngine..." -ForegroundColor Cyan
if (-not (Test-Path $CORE_ENGINE_DIR)) {
    Write-Host "  ERROR: CoreEngine directory not found: $CORE_ENGINE_DIR" -ForegroundColor Red
    Write-Host "  Please ensure the CoreEngine is available." -ForegroundColor Yellow
} else {
    try {
        Push-Location $CORE_ENGINE_DIR
        
        # Build CoreEngine if not already built
        $exePath = "target\release\core_engine.exe"
        if (-not (Test-Path $exePath)) {
            Write-Host "  Building CoreEngine..." -ForegroundColor Gray
            cargo build --release --bin core_engine
            if ($LASTEXITCODE -ne 0) {
                throw "Cargo build failed"
            }
        }
        
        # Get absolute path to config file
        $configPath = Join-Path $ScriptDir $CONFIG_FILE
        
        # Start CoreEngine and track the process
        $fullExePath = Join-Path (Get-Location) $exePath
        $coreProcess = Start-Process -NoNewWindow -PassThru -FilePath $fullExePath -ArgumentList "--config", $configPath -ErrorAction Stop
        $script:StartedProcesses += $coreProcess
        Write-Host "  OK CoreEngine started (port $CORE_ENGINE_PORT, PID: $($coreProcess.Id))" -ForegroundColor Green
        
        Pop-Location
    } catch {
        Write-Host "  WARNING: Failed to start CoreEngine: $_" -ForegroundColor Yellow
        Write-Host "  Please build and start CoreEngine manually:" -ForegroundColor Yellow
        Write-Host "    cd $CORE_ENGINE_DIR" -ForegroundColor Gray
        Write-Host "    cargo build --release --bin core_engine" -ForegroundColor Gray
        Write-Host "    .\target\release\core_engine.exe --config ..\..\$CONFIG_FILE" -ForegroundColor Gray
        Pop-Location -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "=== Startup Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Service Status:" -ForegroundColor Cyan
Write-Host "  NMT: $NMT_SERVICE_URL - Check /health" -ForegroundColor White
Write-Host "  TTS: $TTS_SERVICE_URL - Check /health" -ForegroundColor White
Write-Host "  CoreEngine: http://127.0.0.1:$CORE_ENGINE_PORT - Check /health" -ForegroundColor White
Write-Host ""
Write-Host "API Endpoints:" -ForegroundColor Cyan
Write-Host "  POST http://127.0.0.1:$CORE_ENGINE_PORT/s2s - Sentence translation" -ForegroundColor White
Write-Host "  WS   ws://127.0.0.1:$CORE_ENGINE_PORT/stream - Streaming translation" -ForegroundColor White
Write-Host "  GET  http://127.0.0.1:$CORE_ENGINE_PORT/health - Health check" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop all services..." -ForegroundColor Cyan
Write-Host ""

# Start Web UI server
$webRootFull = Join-Path $ScriptDir $WEB_ROOT
if (Test-Path $webRootFull) {
    $webProcess = Start-WebServer -RootPath $webRootFull -Port $WEB_PORT
    if ($webProcess) {
        $script:StartedProcesses += $webProcess
    }
} else {
    Write-Host "  WARNING: Web UI root not found at $webRootFull, skipping web server start." -ForegroundColor Yellow
}

# Wait for user interrupt or process exit
try {
    # Wait for CoreEngine process to exit
    if ($script:StartedProcesses.Count -gt 0) {
        $coreProcess = $script:StartedProcesses | Where-Object { $_.ProcessName -eq "core_engine" } | Select-Object -First 1
        if ($coreProcess) {
            $coreProcess.WaitForExit()
        } else {
            # Wait for any process to exit
            Wait-Process -Id ($script:StartedProcesses[0].Id) -ErrorAction SilentlyContinue
        }
    } else {
        # If no processes tracked, just wait
        Write-Host "Services are running. Press Ctrl+C to stop..." -ForegroundColor Yellow
        while ($true) {
            Start-Sleep -Seconds 1
        }
    }
} catch {
    # User interrupted or error occurred
} finally {
    Stop-AllServices
}
