# 完整流程集成测试脚本（包含音色识别和分配）
# 测试流程：语音输入 → VAD → ASR → 音色识别 → 翻译 → TTS（音色分配）→ 语音输出
#
# 使用方法：
#   1. 确保所有服务已启动（运行 start_all_services_with_speaker.ps1）
#   2. 运行此脚本进行测试
#
# 测试方式：
#   - 通过 Web 前端进行实时测试（推荐）
#   - 或通过 API 发送音频文件进行测试

$ErrorActionPreference = "Continue"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  完整流程集成测试（音色识别 + 音色分配）" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 检查服务状态
Write-Host "[1/5] 检查服务状态..." -ForegroundColor Yellow
Write-Host ""

$services = @(
    @{Name="Speaker Embedding"; Url="http://127.0.0.1:5003/health"; Port=5003},
    @{Name="YourTTS"; Url="http://127.0.0.1:5004/health"; Port=5004},
    @{Name="NMT Service"; Url="http://127.0.0.1:5008/health"; Port=5008},
    @{Name="CoreEngine"; Url="http://127.0.0.1:9000/health"; Port=9000},
    @{Name="Web Frontend"; Url="http://localhost:8080"; Port=8080}
)

$allServicesRunning = $true

foreach ($service in $services) {
    try {
        $response = Invoke-WebRequest -Uri $service.Url -Method GET -TimeoutSec 2 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "  ✓ $($service.Name) (端口 $($service.Port)) - 运行中" -ForegroundColor Green
        } else {
            Write-Host "  ✗ $($service.Name) (端口 $($service.Port)) - 状态码: $($response.StatusCode)" -ForegroundColor Red
            $allServicesRunning = $false
        }
    } catch {
        Write-Host "  ✗ $($service.Name) (端口 $($service.Port)) - 未运行" -ForegroundColor Red
        $allServicesRunning = $false
    }
}

Write-Host ""

if (-not $allServicesRunning) {
    Write-Host "[错误] 部分服务未运行，请先启动所有服务：" -ForegroundColor Red
    Write-Host "  .\start_all_services_with_speaker.ps1" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "或者手动启动以下服务：" -ForegroundColor Yellow
    Write-Host "  1. Speaker Embedding Service (端口 5003)" -ForegroundColor White
    Write-Host "  2. YourTTS Service (端口 5004)" -ForegroundColor White
    Write-Host "  3. NMT Service (端口 5008)" -ForegroundColor White
    Write-Host "  4. CoreEngine (端口 9000)" -ForegroundColor White
    Write-Host "  5. Web Frontend (端口 8080)" -ForegroundColor White
    exit 1
}

Write-Host "[2/5] 检查配置文件..." -ForegroundColor Yellow
$configPath = "lingua_core_config.toml"
if (Test-Path $configPath) {
    Write-Host "  ✓ 配置文件存在: $configPath" -ForegroundColor Green
    
    # 检查说话者识别配置
    $configContent = Get-Content $configPath -Raw
    if ($configContent -match '\[speaker_identification\]') {
        Write-Host "  ✓ 说话者识别配置已启用" -ForegroundColor Green
        
        if ($configContent -match 'mode\s*=\s*"embedding_based"') {
            Write-Host "  ✓ 使用 embedding_based 模式（音色特征识别）" -ForegroundColor Green
        } elseif ($configContent -match 'mode\s*=\s*"vad_based"') {
            Write-Host "  ⚠ 使用 vad_based 模式（基于时间间隔，无音色识别）" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ⚠ 未找到说话者识别配置" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ✗ 配置文件不存在: $configPath" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[3/5] 测试流程说明..." -ForegroundColor Yellow
Write-Host ""
Write-Host "  完整流程：" -ForegroundColor Cyan
Write-Host "    1. 语音输入（麦克风或音频文件）" -ForegroundColor White
Write-Host "    2. VAD（语音活动检测）- Silero VAD" -ForegroundColor White
Write-Host "    3. ASR（语音识别）- Whisper" -ForegroundColor White
Write-Host "    4. 音色识别 - Speaker Embedding Service" -ForegroundColor White
Write-Host "    5. 翻译 - NMT Service (M2M100)" -ForegroundColor White
Write-Host "    6. TTS（语音合成）- YourTTS（使用参考音频进行音色克隆）" -ForegroundColor White
Write-Host "    7. 语音输出（保持原说话者音色）" -ForegroundColor White
Write-Host ""

Write-Host "[4/5] 测试方式..." -ForegroundColor Yellow
Write-Host ""
Write-Host "  方式 1：Web 前端测试（推荐）" -ForegroundColor Cyan
Write-Host "    1. 打开浏览器访问: http://localhost:8080" -ForegroundColor White
Write-Host "    2. 点击 '开始录音' 按钮" -ForegroundColor White
Write-Host "    3. 对着麦克风说话" -ForegroundColor White
Write-Host "    4. 观察以下内容：" -ForegroundColor White
Write-Host "       - 识别的文本（ASR 结果）" -ForegroundColor Gray
Write-Host "       - 翻译的文本（NMT 结果）" -ForegroundColor Gray
Write-Host "       - 说话者 ID（Speaker ID）" -ForegroundColor Gray
Write-Host "       - 播放的语音（TTS 输出，应保持原音色）" -ForegroundColor Gray
Write-Host ""

Write-Host "  方式 2：API 测试（使用音频文件）" -ForegroundColor Cyan
Write-Host "    使用 curl 或 Postman 发送 POST 请求到：" -ForegroundColor White
Write-Host "      URL: http://127.0.0.1:9000/api/s2s" -ForegroundColor Gray
Write-Host "      Method: POST" -ForegroundColor Gray
Write-Host "      Content-Type: application/json" -ForegroundColor Gray
Write-Host "      Body: { \"audio\": \"<base64_encoded_wav>\", \"src_lang\": \"zh\", \"tgt_lang\": \"en\" }" -ForegroundColor Gray
Write-Host ""

Write-Host "[5/5] 打开 Web 前端..." -ForegroundColor Yellow
Write-Host ""

# 尝试打开浏览器
$webUrl = "http://localhost:8080"
try {
    Start-Process $webUrl
    Write-Host "  ✓ 已打开浏览器: $webUrl" -ForegroundColor Green
} catch {
    Write-Host "  ⚠ 无法自动打开浏览器，请手动访问: $webUrl" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "  测试准备完成！" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "测试检查清单：" -ForegroundColor Cyan
Write-Host "  □ VAD 能正确检测语音边界" -ForegroundColor White
Write-Host "  □ ASR 能正确识别语音文本" -ForegroundColor White
Write-Host "  □ 说话者识别能区分不同说话者" -ForegroundColor White
Write-Host "  □ 翻译结果正确" -ForegroundColor White
Write-Host "  □ TTS 输出保持原说话者音色" -ForegroundColor White
Write-Host "  □ 整个流程延迟可接受（< 3秒）" -ForegroundColor White
Write-Host ""
Write-Host "💡 提示：" -ForegroundColor Cyan
Write-Host "  - 如果说话者识别不工作，检查 Speaker Embedding 服务是否正常运行" -ForegroundColor Gray
Write-Host "  - 如果音色分配不工作，检查 YourTTS 服务是否正常运行" -ForegroundColor Gray
Write-Host "  - 查看各个服务的日志窗口以获取详细调试信息" -ForegroundColor Gray
Write-Host ""
Write-Host "按任意键退出..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

