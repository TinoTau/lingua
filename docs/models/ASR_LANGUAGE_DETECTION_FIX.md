# ASR 语言检测修复说明

**日期**: 2025-11-21  
**状态**: ✅ **已修复，支持自动语言检测**

---

## ✅ 修复内容

### 1. ASR 语言检测支持

**问题**: `infer` 方法没有使用 `language_hint` 参数

**修复**: 
- 在 `infer` 方法中使用 `language_hint` 参数
- 如果 `language_hint` 为 `None`，则启用自动语言检测
- 如果 `language_hint` 有值，则使用指定的语言

**代码修改**:
```rust
// 在 core/engine/src/asr_whisper/streaming.rs 的 infer 方法中
// 6. 设置语言（如果提供了 language_hint）
if let Some(ref lang_hint) = request.language_hint {
    let mut engine = self.engine.lock()?;
    engine.set_language(Some(lang_hint.clone()));
} else {
    // 自动检测语言：设置为 None
    let mut engine = self.engine.lock()?;
    engine.set_language(None);
}
```

### 2. 测试代码更新

**修改**: 使用 `None` 进行自动语言检测

```rust
// 在 core/engine/examples/test_s2s_full_simple.rs 中
let asr_request = AsrRequest {
    frame: frame.clone(),
    language_hint: None,  // None 表示自动检测语言
};
```

---

## 📊 Whisper 自动语言检测能力

### 支持的语言

Whisper 支持 99 种语言的自动检测，包括：
- ✅ 英文 (en)
- ✅ 中文 (zh)
- ✅ 日文 (ja)
- ✅ 韩文 (ko)
- ✅ 法文 (fr)
- ✅ 德文 (de)
- ✅ 西班牙文 (es)
- ✅ 俄文 (ru)
- ... 等 99 种语言

### 自动检测机制

1. **语言检测**: Whisper 会在推理前自动检测音频的语言
2. **语言 token**: 检测到的语言会作为语言 token 添加到 prompt 中（如 `[_LANG_en]` 或 `[_LANG_zh]`）
3. **转录**: 使用检测到的语言进行转录

### 使用方式

**自动检测**（推荐）:
```rust
let asr_request = AsrRequest {
    frame: frame.clone(),
    language_hint: None,  // 自动检测
};
```

**指定语言**:
```rust
let asr_request = AsrRequest {
    frame: frame.clone(),
    language_hint: Some("zh".to_string()),  // 强制使用中文
};
```

---

## ⚠️ 当前问题

### 1. ASR 识别结果

**现象**: 中文音频被识别为英文

**可能原因**:
1. 音频文件本身可能是英文的
2. Whisper 的语言检测可能不准确
3. 需要验证音频文件的实际内容

### 2. NMT 翻译质量问题

**现象**: 出现重复 token 和乱码

**可能原因**:
1. ASR 输出错误（英文而不是中文）
2. NMT 模型本身的问题
3. Tokenizer 解码问题

---

## 🎯 产品需求实现

### 需求

**产品需求**: 能够自动识别英文或者中文，并且翻译成对应的中文或者英文

### 实现方案

1. **ASR 自动语言检测** ✅
   - 使用 `language_hint: None` 启用自动检测
   - Whisper 会自动检测音频的语言（支持 99 种语言）

2. **动态翻译方向选择** ⚠️ **需要实现**
   - 根据 ASR 检测到的语言，动态选择翻译方向
   - 如果检测到中文，翻译成英文
   - 如果检测到英文，翻译成中文

3. **NMT 模型选择** ⚠️ **需要实现**
   - 根据翻译方向选择对应的 NMT 模型
   - 中文→英文: `m2m100-zh-en`
   - 英文→中文: `m2m100-en-zh`

---

## 📋 下一步行动

### 优先级 1: 实现动态翻译方向选择

1. **获取 ASR 检测到的语言**
   - 从 `AsrResult` 中获取检测到的语言
   - `StableTranscript` 包含 `language` 字段

2. **根据语言选择翻译方向**
   - 如果语言是中文，选择 `zh-en` 翻译方向
   - 如果语言是英文，选择 `en-zh` 翻译方向

3. **动态加载 NMT 模型**
   - 根据翻译方向加载对应的 NMT 模型

### 优先级 2: 验证音频文件

1. **检查音频文件内容**
   - 确认 `test_output/s2s_flow_test.wav` 是否真的是中文音频
   - 如果音频文件本身是英文，需要准备正确的中文音频文件

2. **测试英文音频**
   - 使用 `test_output/mms_tts_onnx_test.wav` 测试英文→中文翻译
   - 验证 ASR 是否能正确识别英文

---

## 📚 参考

- Whisper 语言检测: https://github.com/openai/whisper
- ASR 实现: `core/engine/src/asr_whisper/`
- 测试代码: `core/engine/examples/test_s2s_full_simple.rs`

