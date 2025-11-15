# 逐个运行单元测试，找出卡住的地方

Write-Host "========== 运行单元测试 ==========" -ForegroundColor Green

Write-Host ""
Write-Host "1. 测试音频预处理..." -ForegroundColor Yellow
cargo test --test asr_whisper_audio_preprocessing_test -- --nocapture 2>&1 | Select-Object -First 30

Write-Host ""
Write-Host "2. 测试 ASR 语言设置..." -ForegroundColor Yellow
cargo test --test asr_whisper_language_test -- --nocapture 2>&1 | Select-Object -First 30

Write-Host ""
Write-Host "3. 测试 NMT Tokenizer..." -ForegroundColor Yellow
cargo test --test nmt_tokenizer_multi_lang -- --nocapture 2>&1 | Select-Object -First 30

Write-Host ""
Write-Host "4. 测试 NMT 模型加载..." -ForegroundColor Yellow
cargo test --test nmt_onnx_model_load -- --nocapture 2>&1 | Select-Object -First 30

Write-Host ""
Write-Host "5. 测试 NMT 快速测试..." -ForegroundColor Yellow
cargo test --test nmt_quick_test -- --nocapture 2>&1 | Select-Object -First 30

Write-Host ""
Write-Host "========== 单元测试完成 ==========" -ForegroundColor Green

