# 服务间通信架构说明

## 问题

现在各个服务之间的通信都是通过内存吗？

## 答案

**不是全部通过内存。系统采用混合架构：**

1. **CoreEngine 内部组件**：通过内存中的函数调用和 `Arc` 共享（同步/异步）
2. **外部 Python 服务**：通过 HTTP 网络通信（NMT、TTS、Speaker Embedding）
3. **可选的内置实现**：某些组件（如 NMT）支持 ONNX 本地推理，通过内存调用

## 详细架构

### 1. CoreEngine 内部组件（内存通信）

这些组件都在同一个 Rust 进程中，通过内存中的函数调用和 `Arc` 共享状态：

```
┌─────────────────────────────────────────────────────────┐
│              CoreEngine (Rust 进程)                      │
│                                                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐          │
│  │   VAD    │───>│   ASR    │───>│   NMT    │          │
│  │          │    │          │    │ (Adapter)│          │
│  └──────────┘    └──────────┘    └────┬─────┘          │
│                                        │                │
│                                        │ (函数调用)      │
│                                        ↓                │
│                              ┌─────────────────┐        │
│                              │  NMT Client     │        │
│                              │  (HTTP Client)  │        │
│                              └────────┬────────┘        │
│                                       │                 │
│                                       │ (HTTP)          │
└───────────────────────────────────────┼─────────────────┘
                                        │
                                        ↓
                              ┌─────────────────┐
                              │  NMT Service    │
                              │  (Python)       │
                              └─────────────────┘
```

**通信方式**：
- **VAD → ASR**：函数调用，通过返回值传递 `AsrResult`
- **ASR → NMT**：函数调用，通过参数传递 `TranslationRequest`
- **NMT Adapter → NMT Client**：函数调用，通过参数和返回值传递数据
- **状态共享**：使用 `Arc<Mutex<>>` 或 `Arc<RwLock<>>` 共享状态

**示例代码**：
```rust
// VAD → ASR（内存函数调用）
let vad_result = self.vad.detect(frame).await?;  // 返回 DetectionOutcome
if vad_result.is_boundary {
    let asr_result = whisper_asr.infer_on_boundary().await?;  // 返回 AsrResult
}

// ASR → NMT（内存函数调用）
let translation_result = self.translate_and_publish(&transcript, timestamp).await?;
// translate_and_publish 内部调用：
// self.nmt.translate(translation_request).await?  // 返回 TranslationResponse
```

### 2. 外部 Python 服务（HTTP 网络通信）

这些服务运行在独立的 Python 进程中，通过 HTTP 协议通信：

#### 2.1 NMT 服务（M2M100）

**服务端点**：`http://127.0.0.1:5008/v1/translate`

**通信方式**：HTTP POST 请求

**客户端实现**：`core/engine/src/nmt_client/local_m2m100.rs`

```rust
pub struct LocalM2m100HttpClient {
    base_url: String,  // "http://127.0.0.1:5008"
    http: Client,      // reqwest::Client
}

impl NmtClient for LocalM2m100HttpClient {
    async fn translate(&self, req: &NmtTranslateRequest) -> Result<NmtTranslateResponse> {
        let url = format!("{}/v1/translate", self.base_url);
        let response = self.http.post(&url).json(req).send().await?;
        let body: NmtTranslateResponse = response.json().await?;
        Ok(body)
    }
}
```

**数据流**：
```
Rust NMT Client
  ↓ (HTTP POST, JSON序列化)
Python NMT Service (FastAPI)
  ↓ (计算翻译 + 质量指标)
  ↓ (HTTP Response, JSON序列化)
Rust NMT Client
  ↓ (反序列化)
TranslationResponse (包含 quality_metrics)
```

#### 2.2 TTS 服务（YourTTS / Piper）

**YourTTS 端点**：`http://127.0.0.1:5004/synthesize`  
**Piper 端点**：`http://127.0.0.1:5005/tts`

**通信方式**：HTTP POST 请求

**客户端实现**：
- `core/engine/src/tts_streaming/yourtts_http.rs`
- `core/engine/src/tts_streaming/piper_http.rs`

```rust
pub struct YourTtsHttp {
    client: reqwest::Client,
    config: YourTtsHttpConfig,  // endpoint: "http://127.0.0.1:5004"
}

impl TtsStreaming for YourTtsHttp {
    async fn synthesize(&self, request: &TtsRequest) -> EngineResult<TtsStreamChunk> {
        let url = format!("{}/synthesize", self.config.endpoint);
        let response = self.client.post(&url).json(&request).send().await?;
        let audio_data = response.bytes().await?;
        Ok(TtsStreamChunk { audio_data, ... })
    }
}
```

**数据流**：
```
Rust TTS Client
  ↓ (HTTP POST, JSON序列化)
Python TTS Service (Flask/FastAPI)
  ↓ (TTS合成)
  ↓ (HTTP Response, WAV/PCM音频数据)
Rust TTS Client
  ↓ (音频数据)
TtsStreamChunk
```

#### 2.3 Speaker Embedding 服务

**服务端点**：`http://127.0.0.1:5003/extract_embedding`

**通信方式**：HTTP POST 请求

**客户端实现**：`core/engine/src/speaker_identifier/speaker_embedding_client.rs`

```rust
pub struct SpeakerEmbeddingClient {
    client: reqwest::Client,
    config: SpeakerEmbeddingClientConfig,  // service_url: "http://127.0.0.1:5003"
}

impl SpeakerEmbeddingClient {
    pub async fn extract_embedding(&self, audio: &[f32], sample_rate: u32) -> Result<Vec<f32>> {
        let url = format!("{}/extract_embedding", self.config.service_url);
        let response = self.client.post(&url).json(&request).send().await?;
        let embedding: Vec<f32> = response.json().await?;
        Ok(embedding)
    }
}
```

**数据流**：
```
Rust Speaker Identifier
  ↓ (HTTP POST, JSON序列化)
Python Speaker Embedding Service
  ↓ (提取音色特征)
  ↓ (HTTP Response, JSON序列化)
Rust Speaker Identifier
  ↓ (反序列化)
voice_embedding: Vec<f32>
```

### 3. 可选的内置实现（内存通信）

某些组件支持本地 ONNX 推理，完全在内存中运行：

#### 3.1 NMT ONNX 实现

**实现**：`core/engine/src/nmt_incremental/m2m100_translation.rs`

```rust
pub struct M2m100NmtOnnx {
    // ONNX 模型在内存中
    encoder_session: ort::Session,
    decoder_session: ort::Session,
    tokenizer: M2M100Tokenizer,
}

impl NmtIncremental for M2m100NmtOnnx {
    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        // 完全在内存中执行，无需网络通信
        let tokens = self.tokenizer.encode(&request.transcript.text)?;
        let encoder_output = self.encoder_session.run(...)?;
        let decoder_output = self.decoder_session.run(...)?;
        let translated_text = self.tokenizer.decode(decoder_output)?;
        Ok(TranslationResponse { translated_text, ... })
    }
}
```

**优势**：
- 零网络延迟
- 不依赖外部服务
- 完全离线运行

**劣势**：
- 需要加载模型到内存（占用较大内存）
- 推理速度可能较慢（取决于硬件）

## 完整通信架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                    CoreEngine (Rust 进程)                        │
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │
│  │   VAD    │───>│   ASR    │───>│   NMT    │───>│   TTS    │ │
│  │          │    │          │    │          │    │          │ │
│  └──────────┘    └──────────┘    └────┬─────┘    └────┬─────┘ │
│         │              │               │               │        │
│         │ (内存)        │ (内存)        │ (内存)        │ (内存) │
│         │              │               │               │        │
│         └──────────────┴───────────────┴───────────────┘        │
│                          │                                       │
│                          │ (Arc共享状态)                          │
│                          │                                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Speaker Identifier                          │  │
│  └───────────────────────┬──────────────────────────────────┘  │
│                          │                                       │
└──────────────────────────┼───────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        │ (HTTP)           │ (HTTP)           │ (HTTP)
        ↓                  ↓                  ↓
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   NMT        │  │   TTS        │  │   Speaker    │
│   Service    │  │   Service    │  │   Embedding  │
│   (Python)   │  │   (Python)   │  │   Service    │
│   :5008      │  │   :5004/5005 │  │   (Python)   │
│              │  │              │  │   :5003      │
└──────────────┘  └──────────────┘  └──────────────┘
```

## 通信方式对比

| 组件 | 通信方式 | 协议 | 延迟 | 数据格式 |
|------|---------|------|------|---------|
| VAD → ASR | 内存函数调用 | - | < 1ms | Rust 结构体 |
| ASR → NMT Adapter | 内存函数调用 | - | < 1ms | Rust 结构体 |
| NMT Adapter → NMT Client | 内存函数调用 | - | < 1ms | Rust 结构体 |
| NMT Client → NMT Service | HTTP 网络 | HTTP/JSON | 10-100ms | JSON |
| NMT Client → NMT ONNX | 内存函数调用 | - | 50-500ms | Rust 结构体 |
| TTS Client → TTS Service | HTTP 网络 | HTTP/JSON+WAV | 100-1000ms | JSON + 二进制音频 |
| Speaker ID → Embedding Service | HTTP 网络 | HTTP/JSON | 50-200ms | JSON |

## 为什么使用混合架构？

### 1. 灵活性

- **开发阶段**：使用 HTTP 服务，便于调试和独立开发
- **生产阶段**：可以选择 ONNX 本地推理，减少依赖

### 2. 性能权衡

- **内存通信**：零延迟，但占用内存
- **HTTP 通信**：有网络延迟，但服务可以独立扩展

### 3. 技术栈

- **Rust**：适合高性能、低延迟的核心逻辑
- **Python**：适合快速开发和模型推理（PyTorch/TensorFlow）

## 总结

### 通信方式分布

1. **CoreEngine 内部**：100% 内存通信（函数调用 + Arc 共享）
2. **外部 Python 服务**：100% HTTP 网络通信
3. **可选 ONNX 实现**：100% 内存通信（本地推理）

### 关键点

- **不是全部通过内存**：外部服务通过 HTTP 通信
- **内部组件通过内存**：VAD、ASR、NMT Adapter 等通过函数调用
- **混合架构**：根据需求选择 HTTP 服务或 ONNX 本地推理

### 性能影响

- **内存通信**：延迟 < 1ms，适合高频调用
- **HTTP 通信**：延迟 10-1000ms，适合低频、重计算任务
- **ONNX 本地推理**：延迟 50-500ms，适合离线场景

这种混合架构在灵活性和性能之间取得了良好的平衡。

