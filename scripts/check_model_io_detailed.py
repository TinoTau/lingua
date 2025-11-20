#!/usr/bin/env python3
"""
详细检查模型的输入输出，看看是否有问题
"""

import numpy as np
import onnxruntime as ort
from pathlib import Path

model_dir = Path(__file__).parent.parent / "core" / "engine" / "models" / "tts" / "vits-zh-aishell3"

# 加载模型
model_path = model_dir / "vits-aishell3.onnx"
if not model_path.exists():
    model_path = model_dir / "vits-aishell3.int8.onnx"

print(f"加载模型: {model_path}")
session = ort.InferenceSession(str(model_path), providers=["CPUExecutionProvider"])

print("\n" + "=" * 80)
print("模型输入信息")
print("=" * 80)
for i, input_meta in enumerate(session.get_inputs()):
    print(f"\n输入 {i}: {input_meta.name}")
    print(f"  类型: {input_meta.type}")
    print(f"  形状: {input_meta.shape}")
    if input_meta.shape:
        print(f"  动态维度: {[dim for dim in input_meta.shape if isinstance(dim, str)]}")

print("\n" + "=" * 80)
print("模型输出信息")
print("=" * 80)
for i, output_meta in enumerate(session.get_outputs()):
    print(f"\n输出 {i}: {output_meta.name}")
    print(f"  类型: {output_meta.type}")
    print(f"  形状: {output_meta.shape}")
    if output_meta.shape:
        print(f"  动态维度: {[dim for dim in output_meta.shape if isinstance(dim, str)]}")

# 测试一个简单的输入
print("\n" + "=" * 80)
print("测试简单输入")
print("=" * 80)

# 最简单的输入：只有一个音节
simple_tokens = [0, 19, 81, 1]  # sil + n + i3 + eos
print(f"\n简单 token 序列: {simple_tokens}")

x = np.array([simple_tokens], dtype=np.int64)
x_length = np.array([len(simple_tokens)], dtype=np.int64)
noise_scale = np.array([0.667], dtype=np.float32)
length_scale = np.array([1.0], dtype=np.float32)
noise_scale_w = np.array([0.8], dtype=np.float32)
sid = np.array([0], dtype=np.int64)

inputs = {
    'x': x,
    'x_length': x_length,
    'noise_scale': noise_scale,
    'length_scale': length_scale,
    'noise_scale_w': noise_scale_w,
    'sid': sid,
}

print(f"\n输入形状:")
for name, arr in inputs.items():
    print(f"  {name}: {arr.shape}, dtype={arr.dtype}")

try:
    outputs = session.run(None, inputs)
    print(f"\n✅ 推理成功")
    print(f"输出数量: {len(outputs)}")
    for i, output in enumerate(outputs):
        print(f"  输出 {i}: shape={output.shape}, dtype={output.dtype}")
        print(f"    范围: min={output.min():.6f}, max={output.max():.6f}, mean={output.mean():.6f}")
        if output.size > 0:
            print(f"    前10个值: {output.flatten()[:10]}")
except Exception as e:
    print(f"\n❌ 推理失败: {e}")
    import traceback
    traceback.print_exc()

# 检查模型元数据
print("\n" + "=" * 80)
print("模型元数据")
print("=" * 80)
try:
    metadata = session.get_modelmeta()
    print(f"模型描述: {metadata.description}")
    print(f"域: {metadata.domain}")
    print(f"图形名称: {metadata.graph_name}")
    print(f"生产者名称: {metadata.producer_name}")
    print(f"版本: {metadata.version}")
except Exception as e:
    print(f"无法获取元数据: {e}")
