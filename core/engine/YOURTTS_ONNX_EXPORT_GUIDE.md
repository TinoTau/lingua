# YourTTS 模型导出为 ONNX 指南

## 概述

将 YourTTS 模型导出为 ONNX 格式，以便在 Rust 中使用 ONNX Runtime 运行。

## ⚠️ 重要：在 WSL 环境中操作

**导出操作必须在 WSL 环境中进行**，因为：
- ✅ TTS 库安装在 WSL 中
- ✅ YourTTS 模型在 WSL 中（或可通过 WSL 访问）
- ✅ 导出脚本需要访问 TTS 库和模型文件

**不要在 Windows PowerShell 中直接运行导出脚本！**

## 挑战

YourTTS 是一个复杂的模型，包含多个组件：
- **文本编码器**：将文本转换为特征
- **说话者编码器**：提取说话者特征
- **解码器**：生成语音特征
- **声码器**：将特征转换为音频

导出为 ONNX 可能需要分别处理这些组件。

## 方法 1：使用 TTS 库的导出功能（如果支持）

### 在 WSL 中检查 TTS 库版本

```bash
# 在 WSL 中运行
wsl python3 -m pip show TTS

# 或进入 WSL 后运行
wsl
python3 -m pip show TTS
```

### 查看是否有导出功能

```python
from TTS.api import TTS
tts = TTS(model_name="tts_models/multilingual/multi-dataset/your_tts")

# 检查是否有导出方法
if hasattr(tts, 'export_onnx'):
    tts.export_onnx(output_path="yourtts.onnx")
```

**注意**：TTS 库可能不直接支持 ONNX 导出。

## 方法 2：手动导出（推荐）

### 步骤 1：在 WSL 中安装依赖

```bash
# 在 WSL 中安装（如果尚未安装）
wsl python3 -m pip install TTS torch onnx onnxruntime

# 或进入 WSL 后安装
wsl
pip install TTS torch onnx onnxruntime
```

### 步骤 2：在 WSL 中分析模型结构

```bash
# 在 WSL 中运行 Python 脚本
wsl python3 -c "
from TTS.api import TTS
tts = TTS(model_name='tts_models/multilingual/multi-dataset/your_tts')
model = tts.tts_model
print('模型类型:', type(model))
print('模型属性:', [attr for attr in dir(model) if not attr.startswith('_')])
"
```

或创建临时脚本：

```bash
# 在 WSL 中
wsl
cd /mnt/d/Programs/github/lingua
python3 << 'EOF'
from TTS.api import TTS
tts = TTS(model_name="tts_models/multilingual/multi-dataset/your_tts")
model = tts.tts_model
print(type(model))
print(dir(model))
if hasattr(model, 'encoder'):
    print("找到编码器")
if hasattr(model, 'decoder'):
    print("找到解码器")
if hasattr(model, 'vocoder'):
    print("找到声码器")
if hasattr(model, 'speaker_encoder'):
    print("找到说话者编码器")
EOF
```

### 步骤 3：在 WSL 中导出各个组件

使用提供的脚本（**必须在 WSL 中运行**）：

```bash
# 方式 1：从 Windows 在 WSL 中运行
wsl python3 core/engine/scripts/export_yourtts_to_onnx.py

# 方式 2：进入 WSL 后运行
wsl
cd /mnt/d/Programs/github/lingua
python3 core/engine/scripts/export_yourtts_to_onnx.py
```

### 步骤 4：验证导出的模型

```python
import onnx
import onnxruntime as ort

# 加载 ONNX 模型
onnx_model = onnx.load("yourtts.onnx")
onnx.checker.check_model(onnx_model)

# 创建推理会话
session = ort.InferenceSession("yourtts.onnx")

# 查看输入输出
for input in session.get_inputs():
    print(f"Input: {input.name}, Shape: {input.shape}, Type: {input.type}")
for output in session.get_outputs():
    print(f"Output: {output.name}, Shape: {output.shape}, Type: {output.type}")
```

## 方法 3：使用 TTS 库的源代码

### 查找模型定义

TTS 库的模型定义通常在：
- `TTS/tts/models/your_tts.py`
- `TTS/vocoder/models/your_tts_vocoder.py`

### 修改模型以支持导出

可能需要：
1. 修改模型的 `forward` 方法以支持 ONNX 导出
2. 处理动态形状
3. 处理条件输入（如说话者嵌入）

## 方法 4：使用第三方工具

### 使用 torch2trt（转换为 TensorRT）

```bash
pip install torch2trt
```

### 使用 onnx-simplifier

```bash
pip install onnx-simplifier
python -m onnxsim input.onnx output.onnx
```

## 实际导出示例

### 导出文本编码器

```python
import torch
import torch.onnx

# 获取编码器
encoder = tts.tts_model.encoder
encoder.eval()

# 创建示例输入
dummy_input = torch.randint(0, 100, (1, 50))  # batch_size=1, sequence_length=50

# 导出
torch.onnx.export(
    encoder,
    dummy_input,
    "encoder.onnx",
    export_params=True,
    opset_version=13,
    input_names=['text'],
    output_names=['features'],
    dynamic_axes={
        'text': {0: 'batch_size', 1: 'sequence_length'},
        'features': {0: 'batch_size', 1: 'sequence_length'}
    }
)
```

### 导出说话者编码器

```python
# 获取说话者编码器
speaker_encoder = tts.tts_model.speaker_encoder
speaker_encoder.eval()

# 创建示例输入（音频特征）
dummy_audio = torch.randn(1, 1, 16000)  # batch_size=1, channels=1, samples=16000

# 导出
torch.onnx.export(
    speaker_encoder,
    dummy_audio,
    "speaker_encoder.onnx",
    export_params=True,
    opset_version=13,
    input_names=['audio'],
    output_names=['speaker_embedding'],
    dynamic_axes={
        'audio': {0: 'batch_size', 2: 'audio_length'},
        'speaker_embedding': {0: 'batch_size'}
    }
)
```

## 注意事项

### 1. 动态形状

YourTTS 的输入输出通常是动态的（文本长度、音频长度），需要正确设置 `dynamic_axes`。

### 2. 条件输入

YourTTS 使用说话者嵌入作为条件输入，导出时需要包含这些输入。

### 3. 后处理

某些操作（如音频后处理）可能无法直接导出为 ONNX，需要在 Rust 中重新实现。

### 4. 版本兼容性

确保 PyTorch 和 ONNX 版本兼容：
- PyTorch >= 1.8.0
- ONNX >= 1.9.0

## 验证导出的模型

### 1. 检查模型结构

```python
import onnx

model = onnx.load("yourtts.onnx")
onnx.checker.check_model(model)
```

### 2. 测试推理

```python
import onnxruntime as ort
import numpy as np

session = ort.InferenceSession("yourtts.onnx")

# 准备输入
text_input = np.random.randint(0, 100, (1, 50)).astype(np.int64)
speaker_embedding = np.random.randn(1, 192).astype(np.float32)

# 推理
outputs = session.run(None, {
    'text': text_input,
    'speaker_embedding': speaker_embedding
})

print(f"输出形状: {outputs[0].shape}")
```

## 在 Rust 中使用导出的模型

参考 `VitsTtsEngine` 的实现：

```rust
use ort::{Session, SessionBuilder, Value};

// 加载 ONNX 模型
let session = SessionBuilder::new()?
    .with_model_from_file("yourtts.onnx")?
    .build()?;

// 准备输入
let text_input = /* ... */;
let speaker_embedding = /* ... */;

// 推理
let outputs = session.run(vec![
    Value::from_array(text_input)?,
    Value::from_array(speaker_embedding)?,
])?;
```

## 故障排除

### 问题 1：导出失败，提示不支持的操作

**解决**：
- 检查 PyTorch 和 ONNX 版本
- 某些操作可能需要使用特定版本的 ONNX opset
- 考虑使用 `onnx-simplifier` 简化模型

### 问题 2：导出的模型推理结果不正确

**解决**：
- 确保模型处于评估模式（`model.eval()`）
- 检查输入数据的格式和范围
- 验证动态形状设置是否正确

### 问题 3：模型太大或推理太慢

**解决**：
- 使用模型量化（INT8）
- 使用 ONNX Runtime 的优化选项
- 考虑只导出必要的组件

## 参考资源

- [PyTorch ONNX 导出文档](https://pytorch.org/docs/stable/onnx.html)
- [ONNX Runtime 文档](https://onnxruntime.ai/docs/)
- [TTS 库文档](https://github.com/coqui-ai/TTS)
- [YourTTS 论文](https://arxiv.org/abs/2112.02418)

## 总结

导出 YourTTS 为 ONNX 是一个复杂的过程，可能需要：

1. ✅ 分析模型结构
2. ✅ 分别导出各个组件
3. ✅ 处理动态形状和条件输入
4. ✅ 验证导出的模型
5. ✅ 在 Rust 中实现相应的预处理和后处理

如果导出遇到困难，保持使用 Python HTTP 服务也是一个合理的选择。

