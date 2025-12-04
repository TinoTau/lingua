# Silero VAD 使用指南

## 一、模型文件

### 模型位置
- **路径**：`core/engine/models/vad/silero/silero_vad.onnx`
- **大小**：约 2.2 MB
- **格式**：ONNX Runtime 格式

### 模型信息
- **类型**：Silero VAD (Voice Activity Detection)
- **采样率**：16 kHz
- **帧大小**：512 samples（32ms @ 16kHz）
- **输出**：语音概率（0.0-1.0）

## 二、使用方法

### 1. 基本使用

```rust
use core_engine::vad::{SileroVad, VoiceActivityDetector};
use core_engine::types::AudioFrame;

// 创建 SileroVad 实例
let vad = SileroVad::new("models/vad/silero/silero_vad.onnx")?;

// 创建音频帧（16kHz，单声道）
let frame = AudioFrame {
    sample_rate: 16000,
    channels: 1,
    data: vec![0.0f32; 512],  // 512 samples = 32ms @ 16kHz
    timestamp_ms: 0,
};

// 检测语音活动
let result = vad.detect(frame).await?;

if result.is_boundary {
    println!("检测到自然停顿！置信度: {:.2}", result.confidence);
}
```

### 2. 自定义配置

```rust
use core_engine::vad::{SileroVad, SileroVadConfig};

let config = SileroVadConfig {
    model_path: "models/vad/silero/silero_vad.onnx".to_string(),
    sample_rate: 16000,
    frame_size: 512,
    silence_threshold: 0.5,           // 静音阈值（0.0-1.0）
    min_silence_duration_ms: 600,     // 最小静音时长（毫秒）
};

let vad = SileroVad::with_config(config)?;
```

### 3. 在 CoreEngine 中使用

```rust
use core_engine::vad::{SileroVad, VoiceActivityDetector};
use core_engine::bootstrap::CoreEngineBuilder;
use std::sync::Arc;

// 创建 SileroVad
let vad = Arc::new(SileroVad::new("models/vad/silero/silero_vad.onnx")?);

// 在 CoreEngineBuilder 中使用
let engine = CoreEngineBuilder::new()
    .vad(vad)
    // ... 其他配置
    .build()?;
```

## 三、配置参数说明

### SileroVadConfig

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `model_path` | `String` | `"models/vad/silero/silero_vad.onnx"` | 模型文件路径 |
| `sample_rate` | `u32` | `16000` | 采样率（Hz） |
| `frame_size` | `usize` | `512` | 帧大小（samples） |
| `silence_threshold` | `f32` | `0.5` | 静音阈值（0.0-1.0），低于此值认为是静音 |
| `min_silence_duration_ms` | `u64` | `600` | 最小静音时长（毫秒），超过此时长才判定为自然停顿 |

### 参数调优建议

1. **`silence_threshold`**（静音阈值）
   - **范围**：0.0 - 1.0
   - **默认**：0.5
   - **调优**：
     - 如果误检太多（将语音误判为静音），**降低**阈值（如 0.3-0.4）
     - 如果漏检太多（将静音误判为语音），**提高**阈值（如 0.6-0.7）

2. **`min_silence_duration_ms`**（最小静音时长）
   - **范围**：建议 300-1000 ms
   - **默认**：600 ms（符合第二阶段目标：0.6-0.8秒）
   - **调优**：
     - 如果检测太敏感（短停顿也被识别），**增加**时长（如 800-1000 ms）
     - 如果检测太迟钝（长停顿才识别），**减少**时长（如 300-500 ms）

## 四、工作原理

### 1. 检测流程

```
音频帧 (512 samples @ 16kHz)
    ↓
归一化到 [-1, 1]
    ↓
ONNX 推理
    ↓
获取语音概率 (0.0-1.0)
    ↓
判断是否静音 (概率 < threshold)
    ↓
累积静音帧数
    ↓
判断是否达到最小静音时长
    ↓
返回 DetectionOutcome
```

### 2. 边界类型

SileroVad 检测到的边界类型为 `BoundaryType::NaturalPause`（自然停顿），区别于：
- `BoundaryType::TimeBased`：基于固定时间间隔的边界（TimeBasedVad）
- `BoundaryType::Forced`：强制边界

### 3. 输出信息

`DetectionOutcome` 包含：
- `is_boundary: bool` - 是否为边界
- `confidence: f32` - 语音概率（0.0-1.0）
- `frame: AudioFrame` - 原始音频帧
- `boundary_type: Option<BoundaryType>` - 边界类型（`Some(BoundaryType::NaturalPause)` 或 `None`）

## 五、与 TimeBasedVad 对比

| 特性 | SileroVad | TimeBasedVad |
|------|-----------|--------------|
| **检测方式** | 基于语音活动检测 | 基于固定时间间隔 |
| **准确性** | 高（能识别自然停顿） | 低（固定间隔，可能切分句子） |
| **性能** | 中等（需要 ONNX 推理） | 高（仅时间计算） |
| **适用场景** | 自然对话、连续输入 | 简单场景、测试 |
| **配置复杂度** | 中等（需要调参） | 低（仅需设置间隔） |

## 六、性能优化建议

1. **批量处理**：如果可能，批量处理多个音频帧以减少 ONNX 推理次数
2. **缓存 Session**：SileroVad 内部已使用 `Arc<Mutex<Session>>` 缓存 ONNX Session
3. **异步处理**：使用 `async/await` 避免阻塞主线程

## 七、常见问题

### Q1: 模型文件找不到？
**A**: 确保模型文件位于 `core/engine/models/vad/silero/silero_vad.onnx`，或使用绝对路径。

### Q2: 采样率不匹配？
**A**: SileroVad 要求 16kHz 采样率。如果音频是其他采样率，需要先重采样。

### Q3: 检测不准确？
**A**: 尝试调整 `silence_threshold` 和 `min_silence_duration_ms` 参数。

### Q4: 性能问题？
**A**: 
- 确保使用 GPU 版本的 ONNX Runtime（如果可用）
- 考虑使用 TimeBasedVad 作为备选方案

## 八、示例代码

### 完整示例

```rust
use core_engine::vad::{SileroVad, VoiceActivityDetector};
use core_engine::types::AudioFrame;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 VAD
    let vad = Arc::new(SileroVad::new("models/vad/silero/silero_vad.onnx")?);
    
    // 2. 模拟音频流
    for i in 0..100 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0f32; 512],  // 实际应用中从音频流读取
            timestamp_ms: i * 32,     // 32ms per frame
        };
        
        // 3. 检测
        let result = vad.detect(frame).await?;
        
        if result.is_boundary {
            println!("Frame {}: 检测到自然停顿！置信度: {:.2}", i, result.confidence);
        }
    }
    
    Ok(())
}
```

## 九、参考资源

- [Silero VAD GitHub](https://github.com/snakers4/silero-vad)
- [ONNX Runtime 文档](https://onnxruntime.ai/docs/)
- [第二阶段目标文档](../product/第二阶段目标.md)

