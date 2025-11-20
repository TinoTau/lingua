#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
检查 TTS 模型的输入/输出规范

使用方法:
    python scripts/check_tts_model_io.py
"""

import onnx
from pathlib import Path
import sys
import io

# 设置 UTF-8 编码输出（Windows 兼容）
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

def check_model_io(model_path: Path, model_name: str):
    """检查模型的输入/输出规范"""
    print(f"\n=== {model_name} ===")
    print(f"模型路径: {model_path}")
    
    if not model_path.exists():
        print(f"❌ 模型文件不存在")
        return
    
    try:
        # 使用 ONNX 库检查模型结构
        onnx_model = onnx.load(str(model_path))
        print("\n输入:")
        for inp in onnx_model.graph.input:
            shape = []
            for dim in inp.type.tensor_type.shape.dim:
                if dim.dim_value > 0:
                    shape.append(dim.dim_value)
                elif dim.dim_param:
                    shape.append(dim.dim_param)
                else:
                    shape.append("?")
            print(f"  名称: {inp.name}")
            print(f"  形状: {shape}")
            try:
                type_name = onnx.mapping.TENSOR_TYPE_TO_NP_TYPE[inp.type.tensor_type.elem_type]
                print(f"  类型: {type_name}")
            except:
                print(f"  类型: {inp.type.tensor_type.elem_type}")
        
        print("\n输出:")
        for out in onnx_model.graph.output:
            shape = []
            for dim in out.type.tensor_type.shape.dim:
                if dim.dim_value > 0:
                    shape.append(dim.dim_value)
                elif dim.dim_param:
                    shape.append(dim.dim_param)
                else:
                    shape.append("?")
            print(f"  名称: {out.name}")
            print(f"  形状: {shape}")
            try:
                type_name = onnx.mapping.TENSOR_TYPE_TO_NP_TYPE[out.type.tensor_type.elem_type]
                print(f"  类型: {type_name}")
            except:
                print(f"  类型: {out.type.tensor_type.elem_type}")
        
        # 尝试使用 ONNX Runtime 检查（如果可用）
        try:
            import onnxruntime as ort
            print("\nONNX Runtime 会话信息:")
            session = ort.InferenceSession(str(model_path))
            print("输入:")
            for inp in session.get_inputs():
                print(f"  名称: {inp.name}")
                print(f"  形状: {inp.shape}")
                print(f"  类型: {inp.type}")
            
            print("输出:")
            for out in session.get_outputs():
                print(f"  名称: {out.name}")
                print(f"  形状: {out.shape}")
                print(f"  类型: {out.type}")
        except ImportError:
            print("\n⚠️  ONNX Runtime 未安装，跳过运行时检查")
        except Exception as e:
            print(f"\n⚠️  ONNX Runtime 检查失败: {e}")
            
    except Exception as e:
        print(f"❌ 检查失败: {e}")
        import traceback
        traceback.print_exc()

def main():
    # 自动检测脚本所在目录和项目根目录
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    current_dir = Path.cwd()
    
    # 尝试多个可能的路径（按优先级）
    possible_paths = [
        current_dir / "models" / "tts",  # 从 core/engine 目录运行
        current_dir / "core" / "engine" / "models" / "tts",  # 从项目根目录运行
        repo_root / "core" / "engine" / "models" / "tts",  # 从 scripts 目录运行
        Path("core/engine/models/tts"),  # 相对路径
        Path("models/tts"),  # 相对路径
    ]
    
    model_dir = None
    for path in possible_paths:
        abs_path = path.resolve()
        if abs_path.exists():
            model_dir = abs_path
            break
    
    if model_dir is None:
        print("❌ 无法找到 TTS 模型目录")
        print(f"当前工作目录: {current_dir.absolute()}")
        print("尝试过的路径:")
        for path in possible_paths:
            abs_path = path.resolve()
            exists = "✅" if abs_path.exists() else "❌"
            print(f"  {exists} {abs_path}")
        return
    
    print(f"✅ 找到模型目录: {model_dir}\n")
    
    # FastSpeech2 模型
    fastspeech2_zh = model_dir / "fastspeech2-lite" / "fastspeech2_csmsc_streaming.onnx"
    fastspeech2_en = model_dir / "fastspeech2-lite" / "fastspeech2_ljspeech.onnx"
    
    # HiFiGAN 模型
    hifigan_zh = model_dir / "hifigan-lite" / "hifigan_csmsc.onnx"
    hifigan_en = model_dir / "hifigan-lite" / "hifigan_ljspeech.onnx"
    
    check_model_io(fastspeech2_zh, "FastSpeech2 中文")
    check_model_io(fastspeech2_en, "FastSpeech2 英文")
    check_model_io(hifigan_zh, "HiFiGAN 中文")
    check_model_io(hifigan_en, "HiFiGAN 英文")

if __name__ == "__main__":
    main()

