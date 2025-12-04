# Speaker Identifier 测试脚本
# 用于快速测试说话者识别功能

Write-Host "=== Speaker Identifier 测试 ===" -ForegroundColor Green
Write-Host ""

# 1. 运行所有单元测试
Write-Host "1. 运行所有单元测试..." -ForegroundColor Yellow
cargo test --lib speaker_identifier -- --nocapture

Write-Host ""
Write-Host "2. 运行 VAD 模式测试..." -ForegroundColor Yellow
cargo test --lib speaker_identifier::vad_based::tests -- --nocapture

Write-Host ""
Write-Host "3. 运行 Embedding 模式测试..." -ForegroundColor Yellow
cargo test --lib speaker_identifier::embedding_based::tests -- --nocapture

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green

