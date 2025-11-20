# VITS TTS 中文支持状态

**日期**: 2024-12-19  
**状态**: ⚠️ 代码已实现，但中文模型尚未下载

---

## 当前状态

### ✅ 已完成

1. **多语言代码实现**
   - ✅ `VitsTtsEngine` 支持多语言模型选择
   - ✅ 根据 `TtsRequest.locale` 自动选择英文或中文模型
   - ✅ 英文模型加载（必需）
   - ✅ 中文模型加载（可选，如果存在则加载）

2. **代码修改**
   - ✅ 修改 `VitsTtsEngine` 结构体支持多模型
   - ✅ 实现 `new_from_models_root()` 方法
   - ✅ 实现 `run_inference()` 根据 locale 选择模型
   - ✅ 更新 `bootstrap.rs` 使用新的 API

### ❌ 待完成

1. **中文模型下载**
   - ❌ 尚未下载 `mms-tts-zh-Hans` 模型
   - ✅ 模型位置：`facebook/mms-tts-zh-Hans`（Hugging Face）

---

## 使用方法

### 下载中文模型

**方法 1：使用 Hugging Face（推荐）**

```powershell
# 安装 git-lfs（如果还没有）
git lfs install

# 克隆中文模型仓库
cd D:\Programs\github\lingua\core\engine\models\tts
git clone https://huggingface.co/facebook/mms-tts-zh-Hans mms-tts-zh-Hans
```

**方法 2：使用 huggingface-cli（最简单）**

```powershell
huggingface-cli download facebook/mms-tts-zh-Hans --local-dir D:\Programs\github\lingua\core\engine\models\tts\mms-tts-zh-Hans
```

### 模型目录结构

下载后，目录结构应该是：

```
core/engine/models/tts/
├── mms-tts-eng/          # 英文模型（已下载 ✅）
│   ├── onnx/
│   │   └── model.onnx
│   ├── tokenizer.json
│   └── config.json
└── mms-tts-zh-Hans/      # 中文模型（待下载 ❌）
    ├── onnx/
    │   └── model.onnx
    ├── tokenizer.json
    └── config.json
```

### 使用中文 TTS

```rust
use core_engine::tts_streaming::{VitsTtsEngine, TtsRequest};

let tts_engine = VitsTtsEngine::new_from_models_root(&models_root)?;

// 中文合成
let request = TtsRequest {
    text: "你好，世界。".to_string(),
    voice: "default".to_string(),
    locale: "zh".to_string(),  // 或 "zh-CN", "zh-TW", "cmn"
};
let chunk = tts_engine.synthesize(request).await?;
```

---

## 代码实现细节

### 多语言模型选择

```rust
// 根据 locale 选择模型
let (session, tokenizer) = match locale {
    "zh" | "zh-CN" | "zh-TW" | "cmn" => {
        // 使用中文模型
        if let (Some(ref session_zh), Some(ref tokenizer_zh)) = (&self.session_zh, &self.tokenizer_zh) {
            (session_zh, tokenizer_zh)
        } else {
            return Err(anyhow!("Chinese model not available. Please download mms-tts-zh-Hans model."));
        }
    }
    _ => {
        // 默认使用英文模型
        (&self.session_en, &self.tokenizer_en)
    }
};
```

### 支持的 Locale 值

- **中文**：`"zh"`, `"zh-CN"`, `"zh-TW"`, `"cmn"`
- **英文**：其他所有值（默认）

---

## 测试

### 测试英文（已通过）

```powershell
cargo test test_vits_tts_synthesize_english -- --nocapture
```

### 测试中文（需要先下载模型）

```rust
#[tokio::test]
async fn test_vits_tts_synthesize_chinese() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let models_root = crate_root.join("models/tts");
    
    let tts_engine = VitsTtsEngine::new_from_models_root(&models_root).unwrap();
    
    let request = TtsRequest {
        text: "你好，世界。".to_string(),
        voice: "default".to_string(),
        locale: "zh".to_string(),
    };
    
    let chunk = tts_engine.synthesize(request).await.unwrap();
    assert!(!chunk.audio.is_empty());
}
```

---

## 下一步

1. **下载中文模型**
   - ✅ 模型位置：`facebook/mms-tts-zh-Hans`（Hugging Face）
   - 下载并放置到 `core/engine/models/tts/mms-tts-zh-Hans/`

2. **验证中文模型**
   - 使用 Python 脚本验证模型功能
   - 运行 Rust 测试确认中文 TTS 工作正常

3. **端到端测试**
   - 测试完整流程：ASR（中文）→ NMT（中英）→ TTS（英文）
   - 测试完整流程：ASR（英文）→ NMT（英中）→ TTS（中文）

---

## 相关文档

- `VITS_TTS_IMPLEMENTATION_SUMMARY.md` - VITS TTS 实现总结
- `VITS_TTS_CHINESE_MODEL_GUIDE.md` - 中文模型下载指南

