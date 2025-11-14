#!/usr/bin/env python3
"""
检查 ONNX 模型的输入输出信息
用于验证导出的模型是否正确
"""

import argparse
import sys
from pathlib import Path

try:
    import onnx
except ImportError:
    print("Error: Please install onnx: pip install onnx")
    sys.exit(1)


def check_model(model_path: Path):
    """检查 ONNX 模型的输入输出"""
    print(f"\n=== Checking model: {model_path} ===\n")
    
    if not model_path.exists():
        print(f"Error: Model file not found: {model_path}")
        return
    
    try:
        model = onnx.load(str(model_path))
        
        print("Inputs:")
        for i, inp in enumerate(model.graph.input):
            print(f"  [{i}] {inp.name}")
            if inp.type.tensor_type.shape.dim:
                shape = [dim.dim_value if dim.dim_value > 0 else f"dim_{i}" 
                        for i, dim in enumerate(inp.type.tensor_type.shape.dim)]
                print(f"      Shape: {shape}")
            print(f"      Type: {inp.type.tensor_type.elem_type}")
        
        print("\nOutputs:")
        for i, out in enumerate(model.graph.output):
            print(f"  [{i}] {out.name}")
            if out.type.tensor_type.shape.dim:
                shape = [dim.dim_value if dim.dim_value > 0 else f"dim_{i}" 
                        for i, dim in enumerate(out.type.tensor_type.shape.dim)]
                print(f"      Shape: {shape}")
            print(f"      Type: {out.type.tensor_type.elem_type}")
        
        print(f"\nModel size: {model_path.stat().st_size / (1024*1024):.2f} MB")
        
    except Exception as e:
        print(f"Error loading model: {e}")
        import traceback
        traceback.print_exc()


def main():
    parser = argparse.ArgumentParser(description="Check ONNX model inputs and outputs")
    parser.add_argument("model_path", type=str, help="Path to ONNX model file")
    
    args = parser.parse_args()
    check_model(Path(args.model_path))


if __name__ == "__main__":
    main()

