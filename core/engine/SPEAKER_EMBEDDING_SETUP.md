# Speaker Embedding 和 YourTTS 服务设置指南

## ✅ 模型确认

模型已成功下载到：
- **Speaker Embedding**: `D:\Programs\github\lingua\core\engine\models\speaker_embedding\cache`
- **YourTTS**: `D:\Programs\github\lingua\core\engine\models\tts\your_tts`

## 🚀 启动服务

### 1. 启动 Speaker Embedding 服务（端口 5003）

```bash
# CPU 模式
python core/engine/scripts/speaker_embedding_service.py

# GPU 模式（如果可用）
python core/engine/scripts/speaker_embedding_service.py --gpu

# 自定义端口和地址
python core/engine/scripts/speaker_embedding_service.py --gpu --port 5003 --host 127.0.0.1
```

**服务端点**：
- `GET /health` - 健康检查
- `POST /extract` - 提取说话者特征向量

### 2. 启动 YourTTS 服务（端口 5004）

```bash
# CPU 模式
python core/engine/scripts/yourtts_service.py

# GPU 模式（推荐，如果可用）
python core/engine/scripts/yourtts_service.py --gpu

# 自定义端口和地址
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 127.0.0.1
```

**服务端点**：
- `GET /health` - 健康检查
- `POST /synthesize` - 语音合成（支持 zero-shot）

## 📝 配置使用

### 在 Rust 代码中配置

```rust
use core_engine::*;

// 创建引擎，使用 Embedding 模式
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
```

### 使用 YourTTS（Zero-shot TTS）

```rust
use core_engine::*;

// 创建 YourTTS 客户端
let yourtts = YourTtsHttp::new(YourTtsHttpConfig {
    endpoint: "http://127.0.0.1:5004".to_string(),
    timeout_ms: 10000,
})?;

// 在 CoreEngineBuilder 中使用
let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_tts(Arc::new(yourtts))
    .build()?;
```

## 🔧 功能说明

### Speaker Embedding（音色提取）

- **输入**：16kHz 单声道音频（f32）
- **输出**：192 维特征向量
- **用途**：识别说话者，提取音色特征

### YourTTS（音色分配）

- **输入**：文本 + 参考音频（可选）
- **输出**：22050Hz 音频数据
- **用途**：根据参考音频生成相似音色的语音（zero-shot TTS）

## ⚠️ 注意事项

1. **服务必须运行**：Rust 代码通过 HTTP 调用 Python 服务，服务必须先启动
2. **GPU 模式**：如果系统有 GPU，使用 `--gpu` 参数可以显著提升性能
3. **端口冲突**：确保端口 5003 和 5004 未被占用
4. **模型路径**：确保模型文件在正确的位置

## 🧪 测试

### 测试 Speaker Embedding 服务

```bash
# 健康检查
curl http://127.0.0.1:5003/health

# 提取 embedding（需要提供音频数据）
curl -X POST http://127.0.0.1:5003/extract \
  -H "Content-Type: application/json" \
  -d '{"audio": [0.1, 0.2, ...]}'
```

### 测试 YourTTS 服务

```bash
# 健康检查
curl http://127.0.0.1:5004/health

# 语音合成
curl -X POST http://127.0.0.1:5004/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "你好，世界", "language": "zh"}'
```

## 📊 性能优化

1. **使用 GPU**：两个服务都支持 GPU 模式，可以显著提升性能
2. **批量处理**：Speaker Embedding 服务支持批量处理
3. **缓存**：说话者 embedding 会被缓存，避免重复计算

## 🔄 工作流程

1. **音频输入** → VAD 检测边界
2. **Speaker Embedding 服务** → 提取特征向量（192 维）
3. **说话者识别** → 判断是否为新说话者
4. **保存参考音频** → 用于 zero-shot TTS
5. **YourTTS 服务** → 使用参考音频生成相似音色的语音

## ✅ 完成状态

- ✅ Speaker Embedding HTTP 服务（支持 GPU）
- ✅ YourTTS HTTP 服务（支持 GPU）
- ✅ Rust HTTP 客户端（Speaker Embedding）
- ✅ Rust HTTP 客户端（YourTTS）
- ✅ 集成到 EmbeddingBasedSpeakerIdentifier
- ⚠️ TTS 合成逻辑支持 reference_audio（部分完成，需要传递 reference_audio）

