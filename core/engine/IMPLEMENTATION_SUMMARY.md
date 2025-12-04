# 音色提取和音色分配功能实现总结

## ✅ 已完成功能

### 1. Speaker Embedding HTTP 服务（音色提取）

**文件**：`core/engine/scripts/speaker_embedding_service.py`

**功能**：
- ✅ 支持 GPU 模式（`--gpu` 参数）
- ✅ 提取 192 维说话者特征向量
- ✅ HTTP API：`POST /extract`
- ✅ 健康检查：`GET /health`

**模型位置**：`core/engine/models/speaker_embedding/cache`

### 2. YourTTS HTTP 服务（音色分配）

**文件**：`core/engine/scripts/yourtts_service.py`

**功能**：
- ✅ 支持 GPU 模式（`--gpu` 参数）
- ✅ Zero-shot TTS（使用参考音频生成相似音色）
- ✅ HTTP API：`POST /synthesize`
- ✅ 健康检查：`GET /health`

**模型位置**：`core/engine/models/tts/your_tts`

### 3. Rust HTTP 客户端

**Speaker Embedding 客户端**：
- ✅ `core/engine/src/speaker_identifier/speaker_embedding_client.rs`
- ✅ 支持提取 embedding
- ✅ 健康检查

**YourTTS 客户端**：
- ✅ `core/engine/src/tts_streaming/yourtts_http.rs`
- ✅ 支持 zero-shot TTS
- ✅ 支持 reference_audio

### 4. 代码集成

**Speaker Identifier**：
- ✅ `EmbeddingBasedSpeakerIdentifier` 使用 HTTP 客户端
- ✅ 提取并返回 `voice_embedding` 和 `reference_audio`
- ✅ 支持配置服务 URL

**TTS 合成**：
- ✅ `TtsRequest` 支持 `reference_audio` 字段
- ✅ `YourTtsHttp` 支持 zero-shot TTS
- ✅ 可以传递参考音频进行音色克隆

## 🚀 使用方法

### 启动服务

```bash
# 终端 1：启动 Speaker Embedding 服务（GPU 模式）
python core/engine/scripts/speaker_embedding_service.py --gpu

# 终端 2：启动 YourTTS 服务（GPU 模式）
python core/engine/scripts/yourtts_service.py --gpu
```

### 在代码中使用

```rust
use core_engine::*;

// 1. 创建引擎，使用 Embedding 模式
let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_speaker_identification(
        SpeakerIdentifierMode::EmbeddingBased {
            service_url: Some("http://127.0.0.1:5003".to_string()),
            similarity_threshold: 0.7,
        }
    )?
    .with_speaker_voice_mapping(vec![
        "zh_CN-huayan-medium".to_string(),
        "zh_CN-xiaoyan-medium".to_string(),
    ])
    .with_continuous_mode(true, 5000, 200)
    .build()?;

// 2. 使用 YourTTS（可选，用于 zero-shot TTS）
let yourtts = YourTtsHttp::new(YourTtsHttpConfig {
    endpoint: "http://127.0.0.1:5004".to_string(),
    timeout_ms: 10000,
})?;

let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_tts(Arc::new(yourtts))
    .build()?;
```

## 📊 工作流程

1. **音频输入** → VAD 检测边界
2. **Speaker Embedding 服务** → 提取 192 维特征向量
3. **说话者识别** → 判断是否为新说话者
4. **保存参考音频** → 用于 zero-shot TTS
5. **YourTTS 服务** → 使用参考音频生成相似音色的语音

## ⚠️ 注意事项

1. **服务必须运行**：Rust 代码通过 HTTP 调用 Python 服务
2. **GPU 模式**：使用 `--gpu` 参数可以显著提升性能
3. **端口**：默认端口 5003（Speaker Embedding）和 5004（YourTTS）
4. **模型路径**：确保模型文件在正确位置

## 🔧 配置说明

### Speaker Embedding 配置

```rust
SpeakerIdentifierMode::EmbeddingBased {
    service_url: Some("http://127.0.0.1:5003".to_string()),
    similarity_threshold: 0.7,  // 相似度阈值
}
```

### YourTTS 配置

```rust
YourTtsHttpConfig {
    endpoint: "http://127.0.0.1:5004".to_string(),
    timeout_ms: 10000,  // 超时时间（毫秒）
}
```

## 📝 待完善功能

1. **传递 reference_audio**：当前 `synthesize_and_publish` 中 `reference_audio` 暂时为 `None`，需要从识别结果中获取并传递
2. **音频重采样**：如果输入音频不是 16kHz，需要重采样
3. **错误处理**：增强错误处理和重试机制

## ✅ 测试状态

- ✅ 编译通过
- ⚠️ 需要启动服务进行集成测试
- ⚠️ 需要测试 GPU 模式

## 📚 相关文档

- `SPEAKER_EMBEDDING_SETUP.md` - 服务设置指南
- `SPEAKER_VOICE_CONSISTENCY.md` - 音色一致性说明
- `MODEL_DOWNLOAD_GUIDE.md` - 模型下载指南

