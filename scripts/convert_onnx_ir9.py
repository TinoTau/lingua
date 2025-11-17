#!/usr/bin/env python3
"""
将 ONNX 模型从 IR version 10 降级到 IR version 9

使用方法:
    python scripts/convert_onnx_ir9.py \
        --input_model core/engine/models/emotion/xlm-r/model.onnx \
        --output_model core/engine/models/emotion/xlm-r/model_ir9.onnx
"""

import argparse
import onnx
from onnx import version_converter, helper


def convert_to_ir9(input_model_path: str, output_model_path: str):
    """将 ONNX 模型降级到 IR version 9"""
    print(f"=== Loading Model ===")
    print(f"Input: {input_model_path}")
    
    # 加载模型
    model = onnx.load(input_model_path)
    print(f"Current IR Version: {model.ir_version}")
    print(f"Current Opset Version: {model.opset_import[0].version}")
    
    # 尝试降级到 IR version 9
    print(f"\n=== Converting to IR Version 9 ===")
    try:
        # 方法1: 直接修改 IR version（可能不兼容）
        # 方法2: 使用 version_converter（可能失败）
        converted_model = version_converter.convert_version(model, 9)
        print(f"✅ Conversion successful")
        print(f"New IR Version: {converted_model.ir_version}")
        print(f"New Opset Version: {converted_model.opset_import[0].version}")
        
        # 保存模型
        onnx.save(converted_model, output_model_path)
        print(f"\n✅ Model saved to: {output_model_path}")
        
    except Exception as e:
        print(f"❌ Version conversion failed: {e}")
        print(f"\n=== Trying Manual IR Version Downgrade ===")
        
        # 方法3: 手动修改 IR version（不推荐，但可以尝试）
        # 注意：这可能导致运行时错误，因为某些操作可能不兼容
        model.ir_version = 9
        
        # 尝试降级 opset version
        if model.opset_import[0].version > 12:
            print(f"⚠️  Warning: Opset version {model.opset_import[0].version} > 12")
            print(f"   Attempting to set opset version to 12...")
            # 创建新的 opset import
            new_opset = helper.make_opsetid("", 12)
            model.opset_import[0].CopyFrom(new_opset)
        
        # 保存模型
        onnx.save(model, output_model_path)
        print(f"⚠️  Model saved with manual IR version downgrade: {output_model_path}")
        print(f"   IR Version: {model.ir_version}")
        print(f"   Opset Version: {model.opset_import[0].version}")
        print(f"   ⚠️  WARNING: This may cause runtime errors. Test carefully!")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Convert ONNX model from IR 10 to IR 9")
    parser.add_argument(
        "--input_model",
        type=str,
        required=True,
        help="Input ONNX model path",
    )
    parser.add_argument(
        "--output_model",
        type=str,
        required=True,
        help="Output ONNX model path",
    )
    
    args = parser.parse_args()
    convert_to_ir9(args.input_model, args.output_model)

