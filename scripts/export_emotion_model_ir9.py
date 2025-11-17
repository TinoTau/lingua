#!/usr/bin/env python3
"""
重新导出 Emotion XLM-R 模型为 ONNX IR version 9（兼容 ort 1.16.3）

使用方法:
    python scripts/export_emotion_model_ir9.py \
        --model_name cardiffnlp/twitter-xlm-roberta-base-sentiment \
        --output_dir core/engine/models/emotion/xlm-r \
        --opset_version 12
"""

import argparse
import os
from pathlib import Path
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification
import onnx


def export_model(model_name: str, output_dir: str, opset_version: int = 12):
    """导出模型为 ONNX IR version 9"""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    print(f"=== Loading Model ===")
    print(f"Model: {model_name}")
    print(f"Output directory: {output_dir}")
    
    # 加载模型和 tokenizer
    print("\n=== Loading Model and Tokenizer ===")
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForSequenceClassification.from_pretrained(model_name)
    model.eval()
    
    # 保存 tokenizer 文件（如果不存在）
    tokenizer.save_pretrained(output_dir)
    
    # 准备示例输入
    print("\n=== Preparing Example Input ===")
    sample_text = "I love this product!"
    inputs = tokenizer(sample_text, return_tensors="pt", padding=True, truncation=True, max_length=128)
    
    # 导出 ONNX 模型（指定 opset_version 以确保 IR version 9）
    print(f"\n=== Exporting ONNX Model (opset_version={opset_version}) ===")
    onnx_path = output_path / "model.onnx"
    
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
        opset_version=opset_version,  # 使用 opset 12 以确保 IR version 9
        do_constant_folding=True,
    )
    
    print(f"✅ Model exported to: {onnx_path}")
    
    # 验证 ONNX 模型
    print("\n=== Verifying ONNX Model ===")
    onnx_model = onnx.load(str(onnx_path))
    print(f"IR Version: {onnx_model.ir_version}")
    print(f"Opset Version: {onnx_model.opset_import[0].version}")
    
    if onnx_model.ir_version > 9:
        print(f"⚠️  Warning: IR version {onnx_model.ir_version} > 9, may not be compatible with ort 1.16.3")
        print("   Consider using a lower opset_version or upgrading ort")
    else:
        print("✅ IR version is compatible with ort 1.16.3")
    
    # 优化模型（可选，跳过以避免依赖问题）
    print("\n=== Model Optimization ===")
    print("⚠️  Skipping optimization (optional step)")
    
    print("\n=== Export Complete ===")
    print(f"Model files:")
    print(f"  - {onnx_path}")
    print(f"  - {output_path / 'tokenizer.json'}")
    print(f"  - {output_path / 'config.json'}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Export Emotion XLM-R model to ONNX IR version 9")
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

