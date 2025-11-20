# TTS 测试命令（修正版）

## 重要说明

测试文件在 `tests/` 目录下，属于**集成测试**，不是单元测试。

**错误用法**:
```powershell
cargo test --lib test_text_processor_load  # ❌ 不会运行 tests/ 目录下的测试
```

**正确用法**:
```powershell
cargo test test_text_processor_load  # ✅ 运行所有匹配的测试
# 或
cargo test --test tts_text_processor_test  # ✅ 运行特定测试文件中的所有测试
```

---

## 正确的测试命令

### 测试 1: 测试文本预处理器加载

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test test_text_processor_load -- --nocapture
```

### 测试 2: 测试文本规范化

```powershell
cargo test test_text_normalization -- --nocapture
```

### 测试 3: 测试音素 ID 映射

```powershell
cargo test test_phoneme_to_id_mapping -- --nocapture
```

### 测试 4: 测试模型加载

```powershell
cargo test test_tts_model_load -- --nocapture
```

### 测试 5: 测试中文 TTS 合成

```powershell
cargo test test_tts_synthesize_chinese -- --nocapture
```

### 测试 6: 测试英文 TTS 合成

```powershell
cargo test test_tts_synthesize_english -- --nocapture
```

### 测试 7: 测试空文本处理

```powershell
cargo test test_tts_empty_text -- --nocapture
```

---

## 运行所有 TTS 测试

```powershell
cargo test tts -- --nocapture
```

这会运行所有名称包含 "tts" 的测试。

---

## 运行特定测试文件的所有测试

```powershell
# 运行 tts_text_processor_test.rs 中的所有测试
cargo test --test tts_text_processor_test -- --nocapture

# 运行 tts_model_load_test.rs 中的所有测试
cargo test --test tts_model_load_test -- --nocapture

# 运行 tts_integration_test.rs 中的所有测试
cargo test --test tts_integration_test -- --nocapture
```

---

## 区别说明

- `cargo test --lib`: 只运行 `src/` 目录下的单元测试（`#[test]` 在模块内部）
- `cargo test`: 运行所有测试（包括 `src/` 和 `tests/` 目录）
- `cargo test --test <file>`: 运行 `tests/<file>.rs` 中的所有测试

