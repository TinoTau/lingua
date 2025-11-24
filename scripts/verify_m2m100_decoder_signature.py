#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""验证 M2M100 decoder 模型的输入签名"""

import sys
from pathlib import Path
import onnxruntime as ort

def verify_decoder_signature(model_path: Path):
    """验证 decoder 模型的输入签名"""
    print(f"\n=== 验证模型: {model_path.name} ===\n")
    
    sess_options = ort.SessionOptions()
    session = ort.InferenceSession(
        model_path.as_posix(),
        sess_options,
        providers=['CPUExecutionProvider']
    )
    
    print("输入签名:")
    for i, inp in enumerate(session.get_inputs()):
        print(f"  Input[{i}] name=\"{inp.name}\"")
        print(f"    shape={inp.shape}")
        print(f"    type={inp.type}")
        
        # 检查 encoder_attention_mask
        if inp.name == "encoder_attention_mask":
            if inp.shape[1] is None or 'src_seq' in str(inp.shape):
                print(f"    ✅ 序列长度是动态的")
            else:
                print(f"    ❌ 序列长度是固定的: {inp.shape[1]}")
        
        # 检查 encoder KV cache
        if "encoder.key" in inp.name or "encoder.value" in inp.name:
            print(f"    ⚠️  Encoder KV cache 输入")
            if len(inp.shape) >= 3:
                print(f"      序列长度维度 (index 2): {inp.shape[2]}")
    
    print("\n输出签名:")
    for i, out in enumerate(session.get_outputs()):
        if i < 5 or i >= len(session.get_outputs()) - 5:
            print(f"  Output[{i}] name=\"{out.name}\"")
            print(f"    shape={out.shape}")
            print(f"    type={out.type}")
        elif i == 5:
            print(f"  ... (省略中间输出) ...")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法: python verify_m2m100_decoder_signature.py <decoder.onnx>")
        sys.exit(1)
    
    model_path = Path(sys.argv[1])
    if not model_path.exists():
        print(f"错误: 模型文件不存在: {model_path}")
        sys.exit(1)
    
    verify_decoder_signature(model_path)

