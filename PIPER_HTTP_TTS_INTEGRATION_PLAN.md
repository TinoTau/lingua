# Lingua 项目：WSL TTS 集成技术方案（PiperHttpTts 后端）

**目的：**  
在不破坏现有 ASR / Emotion / NMT / 业务流程的前提下，将已经在 **WSL / Linux 服务** 中跑通的中英 TTS，集成到 Lingua CoreEngine 的 S2S 流程中，使：

> 语音输入 → ASR → NMT → TTS（WSL 服务） → 语音输出

可以真实跑通，并且即使 TTS 出现问题，也不会影响其他功能（至少能返回文本翻译结果）。

---

## 1. 整体思路

1. **保持现有架构不变**：
   - ASR（Whisper）、Emotion（XLM-R）、NMT（Marian）一律不改。
   - CoreEngine 对外接口不改（仍然通过统一的 `TtsStreaming` / `TtsBackend` 抽象调用）。

2. **仅新增一个 TTS 后端实现：`PiperHttpTts`**：
   - 通过 HTTP 调用已经在 WSL 中跑通的 TTS 服务（例如 `http://127.0.0.1:5005/tts`）。
   - TTS 模型和前端逻辑全部在 WSL 服务内部处理，CoreEngine 只管发文本、收 WAV。

3. **通过配置选择 TTS 后端 / 降级行为**：
   - `config.toml` 中增加 `tts.backend = "piper_local"`。
   - 如 TTS 服务不可用，仅记录 warning，不阻塞 ASR / NMT 等功能。

---

## 2. HTTP 接口约定

假设 WSL 内 TTS 服务暴露如下接口（如有差异，可在代码中调整）：

- **URL**：`http://127.0.0.1:5005/tts`
- **Method**：`POST`
- **Request (JSON)**：

  ```json
  {
    "text": "你好，欢迎使用 Lingua 语音翻译系统。",
    "voice": "zh_CN-huayan-medium",
    "language": "zh-CN"
  }
  ```

- **Response**：
  - HTTP 200：Response Body 为 `audio/wav` 二进制字节流
  - HTTP 非 200：表示错误

> 如果当前 WSL 服务使用的字段名或路径不同（例如 `/api/tts`、`lang` 等），只需在 `PiperHttpTts` 的实现中做对应调整即可。

---

## 3. 配置文件扩展（config.toml 示例）

在现有配置基础上，新增 `tts` 配置段：

```toml
[tts]
# 可选值：
#   - "piper_local" : 使用 WSL 内的 Piper/其他 TTS HTTP 服务
#   - "disabled"    : 不启用 TTS，仅返回文本翻译
#   - "legacy"      : 如果需要保留原 FastSpeech2/HiFiGAN，可自定义
backend = "piper_local"

[tts.piper_local]
# WSL TTS 服务的 HTTP endpoint
# 如果服务路径不同（如 /api/tts），在此调整
endpoint = "http://127.0.0.1:5005/tts"

# 默认 voice 名称，需与 WSL TTS 服务中配置保持一致
default_voice = "zh_CN-huayan-medium"

# 语言标识，可传给 TTS 服务（如果服务不需要，可以忽略或固定）
language = "zh-CN"

# HTTP 请求超时时间（毫秒）
timeout_ms = 8000
```

---

## 4. Rust 侧实现：`PiperHttpTts`

### 4.1 依赖建议

在 `core/engine/Cargo.toml` 中确保有：

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream", "gzip", "brotli", "deflate", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bytes = "1"
thiserror = "1"        # 或复用项目已有的错误系统
async-trait = "0.1"    # 如果 Tts trait 是 async 形式
```

> 如项目已有错误类型（例如 `EngineError`），可在实现中复用。

### 4.2 TTS trait 示例（供对照）

假设已有 trait（示意）：

```rust
#[async_trait::async_trait]
pub trait TtsStreaming: Send + Sync {
    async fn synthesize(&self, text: &str, lang: &str) -> Result<TtsResult, TtsError>;
}

pub struct TtsResult {
    pub wav_bytes: bytes::Bytes, // 或者文件路径、流对象等
}
```

实际 trait 名称及返回类型以当前代码为准，仅需在 `PiperHttpTts` 中对齐。

### 4.3 `PiperHttpTts` 结构与构造

新建文件（建议）：

> `core/engine/src/tts_streaming/piper_http_tts.rs`

```rust
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct PiperHttpConfig {
    pub endpoint: String,
    pub default_voice: String,
    pub language: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PiperHttpTts {
    client: Client,
    cfg: PiperHttpConfig,
}

impl PiperHttpTts {
    pub fn new(cfg: PiperHttpConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .build()
            .expect("failed to build reqwest::Client");

        Self { client, cfg }
    }
}

#[derive(Debug, Serialize)]
struct PiperTtsRequest<'a> {
    text: &'a str,
    voice: &'a str,
    #[serde(rename = "language")]
    lang: &'a str,
}

// 这里的 TtsStreaming / TtsResult / TtsError 请替换为项目实际定义
#[async_trait]
impl TtsStreaming for PiperHttpTts {
    async fn synthesize(&self, text: &str, lang: &str) -> Result<TtsResult, TtsError> {
        // 如果未指定 lang，则使用配置中的默认语言
        let lang = if lang.is_empty() {
            self.cfg.language.as_str()
        } else {
            lang
        };

        let req_body = PiperTtsRequest {
            text,
            voice: &self.cfg.default_voice,
            lang,
        };

        let resp = self
            .client
            .post(&self.cfg.endpoint)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| TtsError::Transport(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let msg = resp
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read error body>".to_string());
            return Err(TtsError::Backend(format!(
                "Piper HTTP error: status={} body={}",
                status, msg
            )));
        }

        // 假设服务直接返回 WAV 字节
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| TtsError::Transport(e.to_string()))?;

        Ok(TtsResult { wav_bytes: bytes })
    }
}
```

> - 如果你的服务返回的是 JSON（例如包含 `audio_base64` 字段），需要在这里增加一层 `serde` 解析与 Base64 解码。
> - 如果项目中 `TtsResult` 使用的是“临时文件路径”，可在这里先把 `bytes` 写入一个临时 WAV 文件，再返回路径。

---

## 5. 在 CoreEngine 中挂接 `PiperHttpTts`

在 CoreEngine 初始化 / Builder 代码中（例如 `core/engine/src/lib.rs` 或相应模块），根据 `config.tts.backend` 选择 TTS 后端：

```rust
fn build_tts_backend(cfg: &Config) -> Box<dyn TtsStreaming> {
    match cfg.tts.backend.as_str() {
        "piper_local" => {
            let p = &cfg.tts.piper_local;
            Box::new(PiperHttpTts::new(PiperHttpConfig {
                endpoint: p.endpoint.clone(),
                default_voice: p.default_voice.clone(),
                language: p.language.clone(),
                timeout_ms: p.timeout_ms,
            }))
        }
        "disabled" => Box::new(NoopTtsBackend::new()),
        "legacy" => Box::new(FastSpeech2TtsBackend::from_config(&cfg.tts.fastspeech2)),
        other => {
            log::warn!("Unknown tts backend '{}', falling back to disabled", other);
            Box::new(NoopTtsBackend::new())
        }
    }
}
```

> 说明：
> - `Config` 结构体中需要增加 `tts` 字段，映射 `config.toml` 的结构；
> - `NoopTtsBackend` 可以简单返回“空音频”或错误，由上层决定降级策略；
> - `FastSpeech2TtsBackend` 若暂未使用，可以保留或注释掉。

---

## 6. S2S 主流程中的降级策略

为避免 TTS 故障影响整个 S2S 流程，在调用 TTS 时建议加一层保护：

```rust
async fn handle_s2s_request(&self, input_audio: AudioStream) -> EngineResult<()> {
    // 1. ASR → 得到原始文本
    let asr_text = self.asr.transcribe(input_audio).await?;

    // 2. Emotion / Persona / NMT → 得到目标文本（如中文）
    let translated = self
        .translator
        .translate_with_emotion(&asr_text)
        .await?;

    // 3. TTS（带降级）
    match self.tts.synthesize(&translated.text, &translated.target_lang).await {
        Ok(tts_result) => {
            // 正常：发布含音频的事件
            self.event_bus.publish(EngineEvent::S2sCompleted {
                text: translated.text,
                audio: Some(tts_result.wav_bytes),
                lang: translated.target_lang,
            });
        }
        Err(err) => {
            // 降级：记录 warn，仅发布文本结果
            log::warn!("TTS failed: {:?}, fallback to text-only", err);
            self.event_bus.publish(EngineEvent::S2sCompleted {
                text: translated.text,
                audio: None,
                lang: translated.target_lang,
            });
        }
    }

    Ok(())
}
```

这样：

- 即使 WSL TTS 服务关闭 / 崩溃 / 配错，ASR + NMT 等功能仍然可用；
- 前端可以根据 `audio` 字段是否为 `Some` 决定是否播放语音。

---

## 7. 开发与测试建议

1. **单元测试（不依赖真实 WSL 服务）**  
   - 使用 `mockito` 或自写 HTTP 测试 server 模拟 TTS 服务：
     - 返回固定的 WAV 头 + 一些伪数据；
     - 验证 `PiperHttpTts` 正确处理 HTTP 状态码、错误信息等。

2. **集成测试（本机 WSL 服务）**  
   - 在 WSL 中启动真实的 TTS 服务（Piper 或其他）；
   - 在 Windows 侧运行 CoreEngine，发起完整的 S2S 请求；
   - 确认能拿到真实的中文 / 英文语音。

3. **压力与性能测试（可选）**  
   - 对同一 WSL TTS 服务并发发起多条短句请求；
   - 观察平均 TTS 延迟、系统 CPU/内存占用；
   - 为未来多用户 / 商业化做数据记录。

---

## 8. 小结

- 当前问题的根源是：
  - 旧的 FastSpeech2 + HiFiGAN 流程在模型层面不能输出真实音频（仅 80 维特征），继续投入成本不划算。
- 本方案通过：
  1. 利用已经在 WSL 内跑通的 TTS 服务（中英均可）；
  2. 新增一个轻量的 `PiperHttpTts` 后端；
  3. 通过配置选择 TTS backend 并提供降级策略；  
  在 **不影响现有 ASR / Emotion / NMT / 业务代码** 的前提下，让 S2S 实现真正的“语音到语音翻译”。

此 MD 文件可直接提交至仓库（例如 `docs/tts/PIPER_HTTP_TTS_INTEGRATION_PLAN.md`），并交付给开发部门按步骤实现与测试。
