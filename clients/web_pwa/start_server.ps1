# Start Lingua Web PWA local server (Windows PowerShell)

param(
    [int]$Port = 8080
)

Write-Host 'Starting Lingua Web PWA local server...' -ForegroundColor Green
Write-Host ""

$serverRoot = $PSScriptRoot
Write-Host "Server directory: $serverRoot" -ForegroundColor Gray
Write-Host "Access URL: http://localhost:$Port" -ForegroundColor Yellow
Write-Host 'Press Ctrl+C to stop the server' -ForegroundColor Gray
Write-Host ""

function Start-PythonServer {
    param(
        [string]$Command,
        [int]$Port,
        [string]$Root
    )

    Write-Host "Using $Command to start Python HTTP Server..." -ForegroundColor Cyan
    & $Command -m http.server $Port --directory $Root
}

function Start-NodeServer {
    param(
        [int]$Port,
        [string]$Root
    )

    Write-Host 'Using npx http-server to start service...' -ForegroundColor Cyan
    & npx http-server $Root -p $Port
}

# 1. Try python / python3 first
$pythonCandidates = @("python", "python3")
foreach ($cmd in $pythonCandidates) {
    if (Get-Command $cmd -ErrorAction SilentlyContinue) {
        Start-PythonServer -Command $cmd -Port $Port -Root $serverRoot
        exit 0
    }
}

# 2. Try npx http-server
if (Get-Command npx -ErrorAction SilentlyContinue) {
    Start-NodeServer -Port $Port -Root $serverRoot
    exit 0
}

# 3. If still not available, show manual commands
Write-Host 'No automatic startup method found. Please run one of the following commands manually:' -ForegroundColor Yellow
$msg1 = '  python -m http.server ' + $Port + ' --directory ' + $serverRoot
$msg2 = '  python3 -m http.server ' + $Port + ' --directory ' + $serverRoot
$msg3 = '  npx http-server ' + $serverRoot + ' -p ' + $Port
$msg4 = '  php -S localhost:' + $Port + ' -t ' + $serverRoot
Write-Host $msg1 -ForegroundColor Gray
Write-Host $msg2 -ForegroundColor Gray
Write-Host $msg3 -ForegroundColor Gray
Write-Host $msg4 -ForegroundColor Gray
exit 1
