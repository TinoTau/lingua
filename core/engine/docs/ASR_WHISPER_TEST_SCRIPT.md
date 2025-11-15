# ASR Whisper 测试脚本说明

## 测试文件

### `asr_whisper_simple_test.rs`
完整的 Whisper 转录测试脚本，使用 `third_party/whisper.cpp/samples/jfk.wav` 音频文件。

## 使用方法

```bash
# 运行完整测试
cargo test --test asr_whisper_simple_test -- --nocapture

# 只运行转录测试
cargo test --test asr_whisper_simple_test test_whisper_simple_transcribe -- --nocapture
```

## 测试内容

1. **模型加载**: 从 `core/engine/models/asr/whisper-base/ggml-base.bin` 加载模型
2. **音频加载**: 从 `third_party/whisper.cpp/samples/jfk.wav` 加载音频
3. **音频预处理**: 
   - 读取 WAV 文件
   - 转换为单声道（如果需要）
   - 重采样到 16kHz（如果需要）
4. **推理**: 使用 Whisper 模型进行转录
5. **结果验证**: 检查是否包含 JFK 演讲的经典台词

## 预期输出

测试应该输出：
- 模型加载成功
- 音频加载成功（包含采样率、声道等信息）
- 推理完成（包含耗时）
- 转录结果（包含时间戳和文本）
- 验证结果（是否找到预期短语）

## 示例输出

```
========== Whisper 简单转录测试 ==========
加载模型: D:\Programs\github\lingua\core\engine\models\asr\whisper-base\ggml-base.bin
✓ 模型加载成功
加载音频: D:\Programs\github\lingua\third_party\whisper.cpp\samples\jfk.wav
  采样率: 16000 Hz
  声道: 1
✓ 音频加载成功 (109900 样本, 6.87 秒)

运行推理...
✓ 推理完成

找到 2 个片段

========== 转录结果 ==========
片段 0: And so my fellow Americans, ask not what your country can do for you,
片段 1: ask what you can do for your country.

========== 完整转录 ==========
And so my fellow Americans, ask not what your country can do for you, ask what you can do for your country.

========== 验证结果 ==========
✓ 找到: 'ask not what your country can do for you'
✓ 找到: 'what you can do for your country'

✓ 所有预期短语都找到了！
```

## 注意事项

1. **API 限制**: `whisper-rs` 0.15.1 的 `WhisperSegment` 字段可能是私有的，当前使用 Debug 输出解析作为临时方案
2. **音频格式**: 音频必须是 WAV 格式，支持 16kHz 单声道或立体声
3. **模型文件**: 确保 `ggml-base.bin` 文件存在且完整

## 下一步

- 实现正式的音频预处理模块（步骤 2.1）
- 封装为 `AsrStreaming` trait 实现（步骤 2.3）

