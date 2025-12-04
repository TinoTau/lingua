# Silero VAD Python vs Rust 对比测试指南

## 概述

此对比测试使用相同的官方模型和测试用例，对比 Python 和 Rust 实现的输出差异，以验证 Rust 实现的正确性。

## 运行步骤

### 1. 运行 Rust 测试

```powershell
cd core\engine
cargo run --example test_silero_vad_comparison
```

### 2. 运行 Python 测试

确保已安装 `onnxruntime`：

```powershell
pip install onnxruntime numpy
```

然后运行：

```powershell
cd core\engine\scripts
python test_silero_vad_python_vs_rust.py
```

或者从项目根目录：

```powershell
python core\engine\scripts\test_silero_vad_python_vs_rust.py
```

## 对比要点

### 1. 模型输出格式

- **Python**: 检查 `output.shape` 和原始输出值
- **Rust**: 检查日志中的 `Model output debug` 和 `Output array shape`

### 2. Speech Probability 计算

- **Python**: 根据输出形状解析 `speech_prob` 和 `silence_prob`
- **Rust**: 检查 `speech_prob` 的计算逻辑是否正确

### 3. 阈值判断

- 两者都应该使用 `speech_prob > 0.5` 来判断是否为语音
- 如果静音帧的 `speech_prob` 也 > 0.5，说明输出解析有问题

## 预期结果

### 语音帧（440Hz 正弦波）
- **Python**: `speech_prob` 应该 > 0.5（例如 0.8-0.9）
- **Rust**: `speech_prob` 应该与 Python 一致

### 静音帧（全零）
- **Python**: `speech_prob` 应该 < 0.5（例如 0.1-0.2）
- **Rust**: `speech_prob` 应该与 Python 一致

## 当前发现的问题

从 Rust 测试输出可以看到：

1. **模型输出形状**: `[1, 1]`（单一值）
2. **原始输出值**: 
   - 语音帧: `0.12431821` → 反转后 `0.87568176`
   - 静音帧: `0.18367606` → 反转后 `0.81632394`

3. **问题**: 静音帧的反转后值仍然 > 0.5，说明：
   - 要么反转逻辑不正确
   - 要么模型输出本身就是语音概率（不需要反转）

## 下一步

运行 Python 测试后，对比两者的输出：

1. 如果 Python 的输出也是 `[1, 1]` 格式，检查它的值是什么
2. 如果 Python 的输出是 `[1, 2]` 格式，说明 Rust 需要调整输出解析逻辑
3. 根据 Python 的正确输出，修正 Rust 的实现

## 参考

- Silero VAD 官方仓库: https://github.com/snakers4/silero-vad
- ONNX Runtime Python API: https://onnxruntime.ai/docs/api/python/

