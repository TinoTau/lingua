#!/usr/bin/env python3
"""
使用旧版本 PyTorch（支持 opset 12）重新导出 Emotion XLM-R 模型为 ONNX IR version 9

**重要**: 此脚本需要 PyTorch 1.13 或更早版本（支持 opset 12）

使用方法:
    # 1. 安装旧版本 PyTorch（在虚拟环境中）
    pip install torch==1.13.1 transformers onnx
    
    # 2. 运行导出脚本
    python scripts/export_emotion_model_ir9_old_pytorch.py \
        --model_name cardiffnlp/twitter-xlm-roberta-base-sentiment \
        --output_dir core/engine/models/emotion/xlm-r \
        --opset_version 12
"""

import argparse
import os
import sys
from pathlib import Path
import torch
import onnx

# 检查 PyTorch 版本
if torch.__version__ >= "2.0.0":
    print("⚠️  Warning: PyTorch version is too new (>= 2.0.0)")
    print("   This script requires PyTorch 1.13 or earlier for opset 12 support")
    print("   Current version: {}".format(torch.__version__))
    print("   Please install PyTorch 1.13.1: pip install torch==1.13.1")
    sys.exit(1)

from transformers import AutoTokenizer, AutoModelForSequenceClassification


def export_model(model_name: str, output_dir: str, opset_version: int = 12):
    """使用旧版本 PyTorch 导出模型为 ONNX IR version 9"""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    print(f"=== Loading Model ===")
    print(f"Model: {model_name}")
    print(f"Output directory: {output_dir}")
    print(f"PyTorch version: {torch.__version__}")
    print(f"Target opset version: {opset_version}")
    
    # 加载模型和 tokenizer
    print("\n=== Loading Model and Tokenizer ===")
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForSequenceClassification.from_pretrained(model_name)
    model.eval()
    
    # 保存 tokenizer 文件
    tokenizer.save_pretrained(output_dir)
    
    # 准备示例输入
    print("\n=== Preparing Example Input ===")
    sample_text = "I love this product!"
    inputs = tokenizer(sample_text, return_tensors="pt", padding=True, truncation=True, max_length=128)
    
    # 导出 ONNX 模型（使用旧版本 API，确保 opset 12）
    print(f"\n=== Exporting ONNX Model (opset_version={opset_version}) ===")
    onnx_path = output_path / "model_ir9_pytorch13.onnx"
    
    # 使用旧版本的 torch.onnx.export API
    # 注意：旧版本可能不支持 dynamic_shapes，使用 dynamic_axes
    torch.onnx.export(
        model,
        (inputs["input_ids"], inputs["attention_mask"]),
        str(onnx_path),
        input_names=["input_ids", "attention_mask"],
        output_names=["logits"],
        dynamic_axes={
            "input_ids": {0: "batch_size", 1: "sequence_length"},
            "attention_mask": {0: "batch_size", 1: "sequence_length"},
            "logits": {0: "batch_size"},
        },
        opset_version=opset_version,
        do_constant_folding=True,
        export_params=True,
    )
    
    print(f"✅ Model exported to: {onnx_path}")
    
    # 验证 ONNX 模型
    print("\n=== Verifying ONNX Model ===")
    onnx_model = onnx.load(str(onnx_path))
    print(f"IR Version: {onnx_model.ir_version}")
    print(f"Opset Version: {onnx_model.opset_import[0].version}")
    
    if onnx_model.ir_version > 9:
        print(f"⚠️  Warning: IR version {onnx_model.ir_version} > 9")
        print("   This may still not be compatible with ort 1.16.3")
    else:
        print("✅ IR version is compatible with ort 1.16.3")
    
    if onnx_model.opset_import[0].version > 12:
        print(f"⚠️  Warning: Opset version {onnx_model.opset_import[0].version} > 12")
        print("   Some operations may not be compatible")
    else:
        print("✅ Opset version is compatible")
    
    print("\n=== Export Complete ===")
    print(f"Model files:")
    print(f"  - {onnx_path}")
    print(f"  - {output_path / 'tokenizer.json'}")
    print(f"  - {output_path / 'config.json'}")
    print(f"\n⚠️  Next step: Test if this model can be loaded by ort 1.16.3")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Export Emotion XLM-R model to ONNX IR version 9 using old PyTorch")
    parser.add_argument(
        "--model_name",
        type=str,
        default="cardiffnlp/twitter-xlm-roberta-base-sentiment",
        help="HuggingFace model name",
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        default="core/engine/models/emotion/xlm-r",
        help="Output directory",
    )
    parser.add_argument(
        "--opset_version",
        type=int,
        default=12,
        help="ONNX opset version (12 for IR version 9)",
    )
    
    args = parser.parse_args()
    export_model(args.model_name, args.output_dir, args.opset_version)

