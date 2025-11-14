#!/usr/bin/env python3
"""
简化版：导出 Marian NMT 模型为 ONNX 格式
使用 transformers 的内置导出功能

这个脚本更简单，但可能导出的模型格式与当前代码不完全匹配。
如果上面的脚本失败，可以尝试这个。
"""

import argparse
import sys
from pathlib import Path

try:
    from transformers import MarianMTModel, MarianTokenizer
    from transformers.onnx import export, FeaturesManager
except ImportError as e:
    print(f"Error: Missing required package: {e}")
    print("\nPlease install required packages:")
    print("  pip install torch transformers onnx onnxruntime")
    sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description="Export Marian NMT model to ONNX (simple version)")
    parser.add_argument(
        "--model_name",
        type=str,
        help="HuggingFace model name (e.g., Helsinki-NLP/opus-mt-en-zh)",
    )
    parser.add_argument(
        "--model_path",
        type=str,
        help="Local path to model directory",
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        required=True,
        help="Output directory for ONNX models",
    )
    
    args = parser.parse_args()
    
    if not args.model_name and not args.model_path:
        parser.error("Either --model_name or --model_path must be provided")
    
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Loading model...")
    if args.model_path:
        model = MarianMTModel.from_pretrained(args.model_path)
        tokenizer = MarianTokenizer.from_pretrained(args.model_path)
    else:
        model = MarianMTModel.from_pretrained(args.model_name)
        tokenizer = MarianTokenizer.from_pretrained(args.model_name)
    
    print(f"Model loaded: {model.config.model_type}")
    
    # 使用 transformers 的导出功能
    try:
        model_kind, model_onnx_config = FeaturesManager.check_supported_model_or_raise(model)
        onnx_config = model_onnx_config(model.config)
        
        print(f"\nExporting model (kind: {model_kind})...")
        
        # 导出模型
        onnx_path = output_dir / "model.onnx"
        export(
            preprocessor=tokenizer,
            model=model,
            config=onnx_config,
            opset=14,
            output=onnx_path,
        )
        
        print(f"✓ Model exported to: {onnx_path}")
        print("\nNote: This exports the full model. You may need to:")
        print("1. Extract encoder and decoder separately")
        print("2. Or modify your code to use the full model")
        
    except Exception as e:
        print(f"✗ Export failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()

