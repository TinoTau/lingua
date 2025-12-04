# YourTTS ONNX 导出脚本使用说明

## ⚠️ 重要：在 WSL 环境中操作

**导出操作必须在 WSL 环境中进行**，因为 TTS 库和模型都在 WSL 中。

## 快速开始

### 1. 在 WSL 中安装依赖

```bash
# 在 WSL 中安装（如果尚未安装）
wsl python3 -m pip install TTS torch onnx onnxruntime
```

### 2. 在 WSL 中运行基础导出

```bash
# 方式 1：从 Windows 在 WSL 中运行
wsl python3 core/engine/scripts/export_yourtts_to_onnx.py

# 方式 2：进入 WSL 后运行
wsl
cd /mnt/d/Programs/github/lingua
python3 core/engine/scripts/export_yourtts_to_onnx.py
```

## 脚本说明

### export_yourtts_to_onnx.py

基础导出脚本，尝试导出完整的 YourTTS 模型。

**参数**：
- `--output-dir`: ONNX 模型输出目录（默认：`core/engine/models/tts/your_tts_onnx`）
- `--model-path`: YourTTS 模型路径（默认：`core/engine/models/tts/your_tts`）
- `--check-only`: 仅检查依赖，不执行导出

**示例**：
```bash
# 在 WSL 中运行
wsl python3 export_yourtts_to_onnx.py --output-dir ./onnx_models --model-path ./models/your_tts
```

### export_yourtts_to_onnx_advanced.py

高级导出脚本，尝试分别导出模型的各个组件（编码器、解码器、声码器等）。

**参数**：
- `--output-dir`: ONNX 模型输出目录
- `--model-path`: YourTTS 模型路径
- `--export-full`: 尝试导出完整模型（而不是各个组件）

**示例**：
```bash
python export_yourtts_to_onnx_advanced.py --export-full
```

## 注意事项

1. **模型结构复杂**：YourTTS 包含多个组件，可能需要分别导出
2. **输入格式**：需要正确准备示例输入（文本序列）
3. **动态形状**：需要正确设置动态轴以支持不同长度的输入
4. **版本兼容**：确保 PyTorch 和 ONNX 版本兼容

## 故障排除

### 问题 1：无法获取模型对象

**解决**：
- 检查模型路径是否正确
- 尝试使用模型名称而不是路径
- 查看 TTS 库的文档了解正确的加载方式

### 问题 2：导出失败，提示不支持的操作

**解决**：
- 检查 PyTorch 和 ONNX 版本
- 尝试使用不同的 ONNX opset 版本
- 考虑使用 `onnx-simplifier` 简化模型

### 问题 3：导出的模型推理结果不正确

**解决**：
- 确保模型处于评估模式
- 检查输入数据的格式和范围
- 验证动态形状设置是否正确

## 参考文档

详细说明请参考：`core/engine/YOURTTS_ONNX_EXPORT_GUIDE.md`

