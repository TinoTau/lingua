# Speaker Embedding 到 TTS 的完整数据流

## 问题

从 Speaker Embedding 到 CoreEngine，再到 NMT，再到 YourTTS 这些服务之间的通信方式是什么？

## 答案

**混合通信方式**：
1. **Speaker Embedding → CoreEngine**：HTTP 网络通信
2. **CoreEngine 内部**：内存传递（函数参数）
3. **CoreEngine → NMT**：只传递 `speaker_id`（内存），不传递 `voice_embedding` 和 `reference_audio`
4. **CoreEngine → YourTTS**：HTTP 网络通信，传递 `speaker_id`、`reference_audio`、`voice_embedding`

## 完整数据流

### 1. Speaker Embedding Service → CoreEngine

**通信方式**：HTTP POST 请求

**端点**：`http://127.0.0.1:5003/extract_embedding`

**客户端**：`core/engine/src/speaker_identifier/speaker_embedding_client.rs`

**数据流**：
```
CoreEngine::process_audio_segment()
  ↓ (调用)
SpeakerIdentifier::identify_speaker()
  ↓ (HTTP POST, JSON序列化)
Speaker Embedding Service (Python)
  ↓ (提取音色特征)
  ↓ (HTTP Response, JSON序列化)
SpeakerIdentificationResult {
    speaker_id: String,
    is_new_speaker: bool,
    confidence: f32,
    voice_embedding: Option<Vec<f32>>,      // ← 音色特征向量
    reference_audio: Option<Vec<f32>>,      // ← 参考音频
}
```

**代码位置**：`core/engine/src/bootstrap/engine.rs:197-319`

```rust
// 1. 调用 Speaker Identifier
let speaker_result = self.speaker_identifier
    .identify_speaker(&audio_frames, vad_result.frame.timestamp_ms)
    .await?;

// 2. 提取数据
let speaker_id = speaker_result.speaker_id.clone();
let voice_embedding = speaker_result.voice_embedding.clone();
let reference_audio = speaker_result.reference_audio.clone();
```

### 2. CoreEngine 内部传递（内存）

**通信方式**：内存中的函数参数传递

**数据传递**：
- `speaker_id`：通过 `StableTranscript` 传递
- `voice_embedding`：通过函数参数传递
- `reference_audio`：通过函数参数传递

**代码位置**：`core/engine/src/bootstrap/engine.rs:316-319`

```rust
// 提取数据（内存中）
let speaker_id = speaker_result.as_ref().map(|r| r.speaker_id.clone());
let voice_embedding = speaker_result.as_ref().and_then(|r| r.voice_embedding.clone());
let reference_audio = speaker_result.as_ref().and_then(|r| r.reference_audio.clone());

// 添加到 ASR 结果中
final_transcript.speaker_id = speaker_id.clone();
```

### 3. CoreEngine → NMT

**通信方式**：内存函数调用

**传递的数据**：**只传递 `speaker_id`**，不传递 `voice_embedding` 和 `reference_audio`

**原因**：NMT 只需要知道说话者ID（用于日志和追踪），不需要音色信息

**代码位置**：`core/engine/src/bootstrap/engine.rs:1493-1503`

```rust
// 构造翻译请求
let translation_request = TranslationRequest {
    transcript: PartialTranscript {
        text: transcript.text.clone(),
        confidence: 1.0,
        is_final: true,
    },
    target_language: target_language.clone(),
    wait_k: None,
    speaker_id: transcript.speaker_id.clone(),  // ← 只传递 speaker_id
    // voice_embedding 和 reference_audio 不传递给 NMT
};

// 执行翻译
let translation_response = self.nmt.translate(translation_request).await?;
```

**数据流**：
```
CoreEngine::translate_and_publish()
  ↓ (函数调用，内存传递)
NMT::translate(TranslationRequest)
  ↓ (如果使用 HTTP 客户端)
NMT Client → NMT Service (HTTP)
  ↓ (返回)
TranslationResponse {
    translated_text: String,
    speaker_id: Option<String>,  // ← 从请求中传递过来
    // 没有 voice_embedding 和 reference_audio
}
```

### 4. CoreEngine → YourTTS

**通信方式**：HTTP POST 请求

**端点**：`http://127.0.0.1:5004/synthesize`

**客户端**：`core/engine/src/tts_streaming/yourtts_http.rs`

**传递的数据**：
- `speaker_id`：用于从缓存中查找 `reference_audio`
- `reference_audio`：如果 `speaker_id` 不在缓存中，使用提供的 `reference_audio`
- `voice_embedding`：**不再传递**（已移除，只用于注册时）
- `speech_rate`：语速信息（用于调整TTS语速）

**代码位置**：`core/engine/src/bootstrap/engine.rs:1621-1750`

```rust
// 构造 TTS 请求
let tts_request = TtsRequest {
    text: processed_text,
    voice: voice,
    locale: target_language,
    speaker_id: translation.speaker_id.clone(),      // ← 传递 speaker_id
    reference_audio: reference_audio.clone(),        // ← 传递 reference_audio
    speech_rate: speech_rate,                        // ← 传递语速
    // voice_embedding 不再传递（只用于注册时）
};

// 调用 TTS
let tts_chunk = self.tts.synthesize(&tts_request).await?;
```

**数据流**：
```
CoreEngine::synthesize_and_publish()
  ↓ (构造 TtsRequest)
TtsRequest {
    text: String,
    speaker_id: Option<String>,        // ← 用于缓存查找
    reference_audio: Option<Vec<f32>>, // ← 如果缓存未命中，使用此音频
    speech_rate: Option<f32>,          // ← 语速信息
}
  ↓ (HTTP POST, JSON序列化)
YourTTS Service (Python)
  ↓ (TTS合成)
  ↓ (HTTP Response, WAV/PCM音频数据)
TtsStreamChunk { audio_data: Vec<u8> }
```

**YourTTS 服务端处理**：`services/yourtts_service.py`

```python
# 1. 优先从缓存中查找
if speaker_id and speaker_id in speaker_cache:
    reference_audio = speaker_cache[speaker_id]['reference_audio']
    # 使用缓存的音频
else:
    # 2. 使用请求中提供的 reference_audio
    if reference_audio:
        reference_audio = request.reference_audio
    else:
        # 3. 使用默认音色
        reference_audio = default_voice
```

### 5. 异步 Speaker 注册（可选）

**通信方式**：HTTP POST 请求（异步，非阻塞）

**端点**：`http://127.0.0.1:5004/register_speaker`

**触发条件**：当识别到新说话者时（`is_new_speaker = true`）

**代码位置**：`core/engine/src/bootstrap/engine.rs:323-357`

```rust
// 如果是新说话者，异步注册到 YourTTS
if is_new_speaker {
    if let (Some(sid), Some(ref_audio)) = (speaker_id.clone(), reference_audio.clone()) {
        tokio::spawn(async move {
            // 异步调用 YourTTS 注册接口
            client.register_speaker(
                sid,
                ref_audio,
                16000,
                voice_embedding,  // ← 注册时传递 voice_embedding
            ).await;
        });
    }
}
```

**数据流**：
```
CoreEngine (检测到新说话者)
  ↓ (异步任务，非阻塞)
YourTTS::register_speaker()
  ↓ (HTTP POST, JSON序列化)
YourTTS Service /register_speaker
  ↓ (保存到缓存)
speaker_cache[speaker_id] = {
    'reference_audio': reference_audio,
    'sample_rate': sample_rate,
}
```

## 完整数据流图

```
┌─────────────────────────────────────────────────────────────────┐
│                    CoreEngine (Rust 进程)                        │
│                                                                  │
│  1. Speaker Identification                                      │
│     └─> SpeakerIdentifier::identify_speaker()                   │
│         │                                                        │
│         │ (HTTP POST)                                           │
│         ↓                                                        │
│  ┌──────────────────────────────────────┐                       │
│  │  Speaker Embedding Service (Python)  │                       │
│  │  :5003                               │                       │
│  └──────────────┬───────────────────────┘                       │
│                 │ (HTTP Response)                                │
│                 ↓                                                │
│  SpeakerIdentificationResult {                                   │
│      speaker_id,                                                 │
│      voice_embedding,  ← 内存中保存                              │
│      reference_audio,  ← 内存中保存                              │
│  }                                                               │
│                                                                  │
│  2. 添加到 ASR 结果（内存传递）                                  │
│     └─> final_transcript.speaker_id = speaker_id                │
│                                                                  │
│  3. 翻译（内存传递，只传递 speaker_id）                          │
│     └─> TranslationRequest {                                    │
│             speaker_id,  ← 只传递 speaker_id                     │
│             // 不传递 voice_embedding 和 reference_audio        │
│         }                                                        │
│         │                                                        │
│         │ (函数调用)                                             │
│         ↓                                                        │
│     NMT::translate()                                            │
│         │                                                        │
│         │ (HTTP POST, 可选)                                      │
│         ↓                                                        │
│     NMT Service (Python) :5008                                  │
│         │                                                        │
│         │ (HTTP Response)                                        │
│         ↓                                                        │
│     TranslationResponse {                                       │
│         translated_text,                                         │
│         speaker_id,  ← 从请求中传递过来                          │
│     }                                                            │
│                                                                  │
│  4. TTS 合成（HTTP，传递 speaker_id + reference_audio）          │
│     └─> TtsRequest {                                            │
│             text,                                                │
│             speaker_id,  ← 用于缓存查找                          │
│             reference_audio,  ← 如果缓存未命中，使用此音频       │
│             speech_rate,  ← 语速信息                             │
│         }                                                        │
│         │                                                        │
│         │ (HTTP POST)                                           │
│         ↓                                                        │
│  ┌──────────────────────────────────────┐                       │
│  │  YourTTS Service (Python)            │                       │
│  │  :5004                               │                       │
│  │  - 检查 speaker_cache[speaker_id]    │                       │
│  │  - 如果未命中，使用 reference_audio   │                       │
│  └──────────────┬───────────────────────┘                       │
│                 │ (HTTP Response, WAV音频)                       │
│                 ↓                                                │
│     TtsStreamChunk { audio_data }                                │
│                                                                  │
│  5. 异步注册（可选，非阻塞）                                     │
│     └─> tokio::spawn(async {                                    │
│             YourTTS::register_speaker(                           │
│                 speaker_id,                                      │
│                 reference_audio,                                 │
│                 voice_embedding,  ← 注册时传递                   │
│             )                                                    │
│         })                                                       │
└─────────────────────────────────────────────────────────────────┘
```

## 数据传递总结

| 阶段 | 通信方式 | 传递的数据 | 不传递的数据 |
|------|---------|-----------|-------------|
| Speaker Embedding → CoreEngine | HTTP | `speaker_id`, `voice_embedding`, `reference_audio` | - |
| CoreEngine 内部 | 内存 | `speaker_id`, `voice_embedding`, `reference_audio` | - |
| CoreEngine → NMT | 内存（函数调用） | `speaker_id` | `voice_embedding`, `reference_audio` |
| CoreEngine → YourTTS | HTTP | `speaker_id`, `reference_audio`, `speech_rate` | `voice_embedding`（只用于注册） |
| CoreEngine → YourTTS (注册) | HTTP (异步) | `speaker_id`, `reference_audio`, `voice_embedding` | - |

## 关键设计决策

### 1. 为什么 NMT 不传递 `voice_embedding` 和 `reference_audio`？

- **NMT 不需要音色信息**：翻译是文本到文本的转换，不需要音频数据
- **减少数据传输**：避免不必要的数据传递，提高性能
- **职责分离**：NMT 只负责翻译，TTS 负责音色

### 2. 为什么 YourTTS 不传递 `voice_embedding`？

- **缓存机制**：YourTTS 使用 `speaker_id` 从缓存中查找 `reference_audio`
- **注册时传递**：`voice_embedding` 只在注册新说话者时传递一次
- **简化请求**：合成请求只需要 `speaker_id` 和 `reference_audio`（如果缓存未命中）

### 3. 为什么使用异步注册？

- **非阻塞**：注册过程可能较慢（需要传输音频数据），不应该阻塞主流程
- **用户体验**：即使注册失败，也可以使用提供的 `reference_audio` 进行合成
- **容错性**：如果注册失败，下次请求时仍然可以使用 `reference_audio`

## 总结

### 通信方式

1. **Speaker Embedding → CoreEngine**：HTTP 网络通信
2. **CoreEngine 内部**：内存传递（函数参数）
3. **CoreEngine → NMT**：内存传递，只传递 `speaker_id`
4. **CoreEngine → YourTTS**：HTTP 网络通信，传递 `speaker_id`、`reference_audio`、`speech_rate`
5. **CoreEngine → YourTTS (注册)**：HTTP 网络通信（异步），传递 `speaker_id`、`reference_audio`、`voice_embedding`

### 数据传递规则

- **`speaker_id`**：全程传递（Speaker Embedding → CoreEngine → NMT → YourTTS）
- **`voice_embedding`**：只在注册时传递给 YourTTS，不传递给 NMT
- **`reference_audio`**：传递给 YourTTS（用于合成），不传递给 NMT
- **`speech_rate`**：从 VAD 获取，传递给 YourTTS（用于调整语速）

这种设计在性能、灵活性和职责分离之间取得了良好的平衡。

