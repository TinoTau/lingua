# NMT 测试脚本说明

本目录包含多个 NMT（神经机器翻译）功能的测试脚本。

## 测试文件列表

### 1. `nmt_comprehensive_test.rs` - 全面功能测试
**最完整的测试套件**，包含 10 个测试用例：

- `test_01_model_loading`: 测试模型加载和初始化
- `test_02_encoder_inference`: 测试 Encoder 推理（通过完整翻译间接测试）
- `test_03_decoder_single_step`: 测试 Decoder 单步推理（通过完整翻译间接测试）
- `test_04_full_translation_short`: 测试短句翻译
- `test_05_full_translation_medium`: 测试中等长度句子翻译
- `test_06_tokenizer_roundtrip`: 测试 Tokenizer 编码/解码往返
- `test_07_language_pair_support`: 测试多语言对支持
- `test_08_error_handling`: 测试错误处理
- `test_09_performance_benchmark`: 性能基准测试
- `test_10_integration_test`: 完整集成测试

**运行方式：**
```bash
# 运行所有测试
cargo test --test nmt_comprehensive_test -- --nocapture

# 运行特定测试
cargo test --test nmt_comprehensive_test test_01_model_loading -- --nocapture
```

### 2. `nmt_quick_test.rs` - 快速测试
**最简单的测试脚本**，用于快速验证核心功能：

- 初始化 ONNX Runtime
- 加载模型
- 执行基本翻译测试

**运行方式：**
```bash
cargo test --test nmt_quick_test -- --nocapture
```

### 3. `nmt_translate_full.rs` - 完整翻译流程测试
测试完整的翻译流程，包括多个测试用例。

**运行方式：**
```bash
cargo test test_full_translation_pipeline -- --nocapture
```

### 4. `nmt_onnx_model_load.rs` - 模型加载测试
测试 ONNX 模型的加载和 I/O 信息打印。

**运行方式：**
```bash
cargo test test_load_marian_onnx_model -- --nocapture
```

### 5. `nmt_tokenizer_multi_lang.rs` - 多语言 Tokenizer 测试
测试多语言 tokenizer 的自动识别和编码/解码功能。

**运行方式：**
```bash
cargo test --test nmt_tokenizer_multi_lang -- --nocapture
```

## 推荐使用方式

### 快速验证（推荐新手）
```bash
cargo test --test nmt_quick_test -- --nocapture
```

### 完整测试（推荐 CI/CD 或全面验证）
```bash
cargo test --test nmt_comprehensive_test -- --nocapture
```

### 运行所有 NMT 相关测试
```bash
cargo test --test nmt_ -- --nocapture
```

## 测试前提条件

1. **模型文件**：确保 `core/engine/models/nmt/marian-en-zh/` 目录存在，且包含：
   - `encoder_model.onnx` - Encoder 模型
   - `model.onnx` - Decoder 模型
   - `vocab.json` - 词汇表
   - `config.json` - 配置文件（可选）

2. **ONNX Runtime**：确保 `ort` crate 已正确安装（版本 1.16.3）

## 测试输出说明

- `✓` 表示测试通过
- `✗` 表示测试失败
- `⚠` 表示警告或跳过（通常是因为模型文件不存在）

## 注意事项

1. **性能测试**：`test_09_performance_benchmark` 会运行多次翻译，可能需要较长时间。

2. **重复 Token 问题**：当前实现使用 workaround 模式（禁用 KV cache），可能会导致某些情况下生成重复的 token。这是已知问题，不影响功能测试。

3. **模型路径**：所有测试都使用 `CARGO_MANIFEST_DIR` 环境变量来定位模型目录，确保在正确的目录下运行测试。

## 故障排除

### 模型文件未找到
如果看到 `⚠ Skipping test: marian-en-zh model directory not found`，请：
1. 确认模型目录存在
2. 检查路径是否正确
3. 运行模型导出脚本生成模型文件

### 编译错误
如果遇到编译错误，请：
1. 运行 `cargo clean` 清理构建缓存
2. 运行 `cargo build` 重新编译
3. 检查依赖版本是否正确

### 运行时错误
如果测试运行时出错，请：
1. 检查 ONNX Runtime 是否正确安装
2. 确认模型文件格式正确
3. 查看详细错误信息（使用 `--nocapture` 标志）

