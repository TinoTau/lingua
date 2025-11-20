# 快速修复：临时禁用 TTS 模块

如果编译卡住，可以临时禁用 TTS 模块来验证问题是否出在 TTS 模块。

## 步骤 1: 注释掉 TTS 模块声明

**文件**: `core/engine/src/lib.rs`

```rust
pub mod bootstrap;
pub mod asr_whisper;
pub mod cache_manager;
pub mod config_manager;
pub mod emotion_adapter;
pub mod error;
pub mod event_bus;
pub mod nmt_incremental;
pub mod persona_adapter;
pub mod telemetry;
// pub mod tts_streaming;  // 临时注释掉
pub mod types;
pub mod vad;
pub mod asr_streaming;
pub mod onnx_utils;
```

## 步骤 2: 注释掉 TTS 的 pub use

**文件**: `core/engine/src/lib.rs`

```rust
// pub use tts_streaming::{TtsRequest, TtsStreamChunk, TtsStreaming, FastSpeech2TtsEngine, TtsStub};  // 临时注释掉
```

## 步骤 3: 注释掉 bootstrap.rs 中的 TTS 相关代码

**文件**: `core/engine/src/bootstrap.rs`

### 3.1 注释掉 use 语句

```rust
// use crate::tts_streaming::TtsStreaming;  // 临时注释掉
```

### 3.2 注释掉 CoreEngine 中的 tts 字段

```rust
pub struct CoreEngine {
    event_bus: Arc<dyn EventBus>,
    vad: Arc<dyn VoiceActivityDetector>,
    asr: Arc<dyn AsrStreaming>,
    nmt: Arc<dyn NmtIncremental>,
    emotion: Arc<dyn EmotionAdapter>,
    persona: Arc<dyn PersonaAdapter>,
    // tts: Arc<dyn TtsStreaming>,  // 临时注释掉
    config: Arc<dyn ConfigManager>,
    cache: Arc<dyn CacheManager>,
    telemetry: Arc<dyn TelemetrySink>,
}
```

### 3.3 注释掉 CoreEngineBuilder 中的 tts 字段

```rust
pub struct CoreEngineBuilder {
    event_bus: Option<Arc<dyn EventBus>>,
    vad: Option<Arc<dyn VoiceActivityDetector>>,
    asr: Option<Arc<dyn AsrStreaming>>,
    nmt: Option<Arc<dyn NmtIncremental>>,
    emotion: Option<Arc<dyn EmotionAdapter>>,
    persona: Option<Arc<dyn PersonaAdapter>>,
    // tts: Option<Arc<dyn TtsStreaming>>,  // 临时注释掉
    config: Option<Arc<dyn ConfigManager>>,
    cache: Option<Arc<dyn CacheManager>>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
}
```

### 3.4 注释掉 CoreEngineBuilder::new() 中的 tts

```rust
impl CoreEngineBuilder {
    pub fn new() -> Self {
        Self {
            event_bus: None,
            vad: None,
            asr: None,
            nmt: None,
            emotion: None,
            persona: None,
            // tts: None,  // 临时注释掉
            config: None,
            cache: None,
            telemetry: None,
        }
    }
    // ...
    // pub fn tts(mut self, tts: Arc<dyn TtsStreaming>) -> Self { ... }  // 临时注释掉
}
```

### 3.5 注释掉 CoreEngineBuilder::build() 中的 tts

```rust
// tts: self.tts.ok_or_else(|| EngineError::new("tts is missing"))?,  // 临时注释掉
```

### 3.6 注释掉 CoreEngine::shutdown() 中的 tts.close()

```rust
pub async fn shutdown(&self) -> EngineResult<()> {
    self.asr.finalize().await?;
    self.nmt.finalize().await?;
    // self.tts.close().await?;  // 临时注释掉
    self.cache.purge().await?;
    // ...
}
```

## 步骤 4: 尝试编译

```powershell
cd core\engine
cargo check --lib
```

## 结果分析

- **如果编译成功**: 说明问题出在 TTS 模块，需要检查 TTS 模块的代码
- **如果仍然卡住**: 说明问题不在 TTS 模块，可能是系统环境或其他模块的问题

## 恢复 TTS 模块

如果确认问题不在 TTS 模块，可以恢复所有注释掉的代码。

