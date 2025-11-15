#!/bin/bash
# 逐个运行单元测试，找出卡住的地方

echo "========== 运行单元测试 =========="

echo ""
echo "1. 测试音频预处理..."
cargo test --test asr_whisper_audio_preprocessing_test -- --nocapture 2>&1 | head -20

echo ""
echo "2. 测试 ASR 语言设置..."
cargo test --test asr_whisper_language_test -- --nocapture 2>&1 | head -20

echo ""
echo "3. 测试 NMT Tokenizer..."
cargo test --test nmt_tokenizer_multi_lang -- --nocapture 2>&1 | head -20

echo ""
echo "4. 测试 NMT 模型加载..."
cargo test --test nmt_onnx_model_load -- --nocapture 2>&1 | head -20

echo ""
echo "5. 测试 NMT 快速测试..."
cargo test --test nmt_quick_test -- --nocapture 2>&1 | head -20

echo ""
echo "========== 单元测试完成 =========="

