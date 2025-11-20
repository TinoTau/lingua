# Windows side Piper HTTP service test script
# Purpose: Test if Piper HTTP service in WSL2 is working correctly

$ErrorActionPreference = "Stop"

$ENDPOINT = "http://127.0.0.1:5005/tts"
$OUTPUT_FILE = "test_output\piper_wsl2_test.wav"
# Test text in Chinese - construct using byte array to avoid encoding issues
$bytes = [byte[]](0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD, 0xEF, 0xBC, 0x8C, 0xE6, 0xAC, 0xA2, 0xE8, 0xBF, 0x8E, 0xE4, 0xBD, 0xBF, 0xE7, 0x94, 0xA8, 0x20, 0x4C, 0x69, 0x6E, 0x67, 0x75, 0x61, 0x20, 0xE8, 0xAF, 0xAD, 0xE9, 0x9F, 0xB3, 0xE7, 0xBF, 0xBB, 0xE8, 0xAF, 0x91, 0xE7, 0xB3, 0xBB, 0xE7, 0xBB, 0x9F, 0xE3, 0x80, 0x82)
$TEST_TEXT = [System.Text.Encoding]::UTF8.GetString($bytes)

Write-Host "=== Piper HTTP Service Test ===" -ForegroundColor Cyan
Write-Host ""

# Create output directory
New-Item -ItemType Directory -Path "test_output" -Force | Out-Null

# Check if service is running
Write-Host "[1/3] Checking service status..." -ForegroundColor Yellow
try {
    $healthUrl = "http://127.0.0.1:5005/health"
    $response = Invoke-WebRequest -Uri $healthUrl -Method GET -TimeoutSec 2 -ErrorAction SilentlyContinue
    Write-Host "[OK] Service is running" -ForegroundColor Green
} catch {
    Write-Host "[WARN] Cannot connect to service, service may not be started" -ForegroundColor Yellow
    Write-Host "[INFO] Please run start_piper_service.sh in WSL2 to start the service" -ForegroundColor Gray
}

# Build request body
Write-Host ""
Write-Host "[2/3] Sending TTS request..." -ForegroundColor Yellow

# 确保使用 UTF-8 编码
$bodyJson = @{
    text = $TEST_TEXT
    voice = "zh_CN-huayan-medium"
} | ConvertTo-Json -Compress

# 将 JSON 转换为 UTF-8 字节数组，然后转回字符串以确保编码正确
$utf8Bytes = [System.Text.Encoding]::UTF8.GetBytes($bodyJson)
$body = [System.Text.Encoding]::UTF8.GetString($utf8Bytes)

Write-Host "[INFO] Request text: $TEST_TEXT" -ForegroundColor Gray
Write-Host "[INFO] Request URL: $ENDPOINT" -ForegroundColor Gray

try {
    # Send POST request with explicit UTF-8 encoding
    $response = Invoke-WebRequest `
        -Uri $ENDPOINT `
        -Method POST `
        -ContentType "application/json; charset=utf-8" `
        -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) `
        -OutFile $OUTPUT_FILE `
        -TimeoutSec 10

    $statusCode = $response.StatusCode
    Write-Host "[OK] Request successful, status code: $statusCode" -ForegroundColor Green
} catch {
    $errorMsg = $_.Exception.Message
    Write-Host "[ERROR] Request failed: $errorMsg" -ForegroundColor Red
    if ($_.Exception.Response) {
        $statusCode = $_.Exception.Response.StatusCode.value__
        Write-Host "[ERROR] HTTP status code: $statusCode" -ForegroundColor Red
    }
    exit 1
}

# Verify output file
Write-Host ""
Write-Host "[3/3] Verifying output file..." -ForegroundColor Yellow
if (Test-Path $OUTPUT_FILE) {
    $fileInfo = Get-Item $OUTPUT_FILE
    $fileSize = $fileInfo.Length
    if ($fileSize -gt 0) {
        Write-Host "[OK] Audio file generated successfully" -ForegroundColor Green
        Write-Host "[INFO] File size: $fileSize bytes" -ForegroundColor Gray
        Write-Host "[INFO] File location: $OUTPUT_FILE" -ForegroundColor Gray
        Write-Host ""
        Write-Host "Next steps:"
        Write-Host "  1. Play the audio file to verify speech quality"
        Write-Host "  2. If normal, you can continue integration into Rust code"
    } else {
        Write-Host "[ERROR] Audio file is empty" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "[ERROR] Audio file was not generated" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Test completed ===" -ForegroundColor Green
