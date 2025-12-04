#!/usr/bin/env python3
"""
YourTTS 模型导出为 ONNX 格式

使用方法：
    python export_yourtts_to_onnx.py [--output-dir OUTPUT_DIR] [--model-path MODEL_PATH]

参数：
    --output-dir: ONNX 模型输出目录（默认：core/engine/models/tts/your_tts_onnx）
    --model-path: YourTTS 模型路径（默认：core/engine/models/tts/your_tts）
"""

import sys
import os
import argparse
from pathlib import Path

# 检查并安装必要的依赖
def check_and_install_dependencies():
    """检查并安装必要的依赖"""
    missing_deps = []
    
    # 检查 torch
    try:
        import torch
    except ImportError:
        missing_deps.append("torch")
    
    # 检查 onnx
    try:
        import onnx
    except ImportError:
        missing_deps.append("onnx")
    
    # 检查 onnxruntime
    try:
        import onnxruntime
    except ImportError:
        missing_deps.append("onnxruntime")
    
    if missing_deps:
        print("⚠️  缺少以下依赖:", ", ".join(missing_deps))
        print("正在尝试安装...")
        import subprocess
        for dep in missing_deps:
            try:
                subprocess.check_call([sys.executable, "-m", "pip", "install", dep])
                print(f"✅ {dep} 安装成功")
            except Exception as e:
                print(f"❌ {dep} 安装失败: {e}")
                print(f"   请手动安装: pip install {dep}")
                return False
        print("✅ 所有依赖安装完成")
        print()
    
    return True

# 在导入之前检查依赖
if not check_and_install_dependencies():
    print("❌ 依赖安装失败，请手动安装:")
    print("   pip install torch onnx onnxruntime")
    sys.exit(1)

import torch
import torch.onnx

# 添加项目路径
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

def export_yourtts_to_onnx(model_path, output_dir, verbose=True):
    """
    将 YourTTS 模型导出为 ONNX 格式
    
    Args:
        model_path: YourTTS 模型路径
        output_dir: ONNX 模型输出目录
        verbose: 是否显示详细信息
    """
    try:
        from TTS.api import TTS
        import torch
        import numpy as np
    except ImportError as e:
        print(f"❌ 缺少依赖: {e}")
        print("请安装: pip install TTS torch onnx")
        return False
    
    model_path = Path(model_path)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    if verbose:
        print("=" * 60)
        print("  YourTTS 模型导出为 ONNX")
        print("=" * 60)
        print(f"模型路径: {model_path}")
        print(f"输出目录: {output_dir}")
        print()
    
    try:
        # 加载 YourTTS 模型
        if verbose:
            print("📦 加载 YourTTS 模型...")
        
        # 使用 TTS API 加载模型
        # 如果 model_path 存在，尝试从路径加载；否则使用模型名称
        if model_path.exists():
            try:
                tts = TTS(model_path=str(model_path), progress_bar=False)
                if verbose:
                    print("✅ 从路径加载模型成功")
            except:
                # 如果路径加载失败，尝试使用模型名称
                tts = TTS(model_name="tts_models/multilingual/multi-dataset/your_tts", 
                          progress_bar=False)
                if verbose:
                    print("✅ 使用模型名称加载成功")
        else:
            tts = TTS(model_name="tts_models/multilingual/multi-dataset/your_tts", 
                      progress_bar=False)
            if verbose:
                print("✅ 使用模型名称加载成功")
        
        if verbose:
            print()
        
        # 获取模型对象
        model = tts.tts_model
        if model is None:
            print("❌ 无法获取模型对象")
            print("   尝试访问 tts.model...")
            if hasattr(tts, 'model'):
                model = tts.model
            else:
                return False
        
        # 设置模型为评估模式
        model.eval()
        
        if verbose:
            print("🔧 分析模型结构...")
            print(f"   模型类型: {type(model)}")
            print(f"   模型属性: {[attr for attr in dir(model) if not attr.startswith('_')]}")
            print()
        
        # 准备示例输入
        if verbose:
            print("🔧 准备示例输入...")
        
        # YourTTS 的输入通常是文本序列
        # 创建一个示例文本
        example_text = "Hello, this is a test."
        
        # 将文本转换为序列（如果 TTS 对象有这个方法）
        if hasattr(tts, 'text_to_sequence'):
            try:
                example_inputs = tts.text_to_sequence(example_text)
                if not isinstance(example_inputs, torch.Tensor):
                    example_inputs = torch.tensor(example_inputs)
            except:
                # 如果转换失败，使用默认输入
                example_inputs = torch.randint(0, 100, (1, 50))  # batch_size=1, sequence_length=50
        else:
            # 使用默认输入
            example_inputs = torch.randint(0, 100, (1, 50))
        
        if verbose:
            print(f"   示例输入形状: {example_inputs.shape}")
            print()
        
        # 导出为 ONNX
        output_path = output_dir / "yourtts.onnx"
        
        if verbose:
            print(f"📤 导出模型到: {output_path}")
        
        try:
            torch.onnx.export(
                model,
                example_inputs,
                str(output_path),
                export_params=True,
                opset_version=13,  # 使用 ONNX opset 13
                do_constant_folding=True,
                input_names=['input'],
                output_names=['output'],
                dynamic_axes={
                    'input': {0: 'batch_size', 1: 'sequence_length'},
                    'output': {0: 'batch_size', 1: 'sequence_length'}
                } if len(example_inputs.shape) > 1 else {
                    'input': {0: 'batch_size'},
                    'output': {0: 'batch_size'}
                },
                verbose=verbose
            )
            
            if verbose:
                print(f"✅ 模型导出成功: {output_path}")
            
            # 验证导出的模型
            if verbose:
                print()
                print("🔍 验证导出的模型...")
            
            try:
                import onnx
                onnx_model = onnx.load(str(output_path))
                onnx.checker.check_model(onnx_model)
                if verbose:
                    print("✅ ONNX 模型验证通过")
                return True
            except ImportError:
                if verbose:
                    print("⚠️  无法验证模型（缺少 onnx 库）")
                return True
            except Exception as e:
                if verbose:
                    print(f"⚠️  模型验证失败: {e}")
                return False
                
        except Exception as e:
            if verbose:
                print(f"❌ 导出失败: {e}")
                print()
                print("💡 提示:")
                print("   1. YourTTS 模型可能包含多个组件，需要分别导出")
                print("   2. 尝试使用 export_yourtts_to_onnx_advanced.py 脚本")
                print("   3. 检查模型输入格式是否正确")
            import traceback
            traceback.print_exc()
            return False
        
    except Exception as e:
        print(f"❌ 导出过程出错: {e}")
        import traceback
        traceback.print_exc()
        return False

def check_onnx_support():
    """检查 ONNX 导出支持"""
    try:
        import onnx
        import onnxruntime
        print("✅ ONNX 相关库已安装")
        return True
    except ImportError as e:
        print(f"❌ 缺少 ONNX 库: {e}")
        print("请安装: pip install onnx onnxruntime")
        return False

def main():
    parser = argparse.ArgumentParser(description="导出 YourTTS 模型为 ONNX 格式")
    parser.add_argument('--output-dir', type=str, 
                       default='core/engine/models/tts/your_tts_onnx',
                       help='ONNX 模型输出目录')
    parser.add_argument('--model-path', type=str,
                       default='core/engine/models/tts/your_tts',
                       help='YourTTS 模型路径')
    parser.add_argument('--check-only', action='store_true',
                       help='仅检查依赖，不执行导出')
    args = parser.parse_args()
    
    print("=" * 60)
    print("  YourTTS ONNX 导出工具")
    print("=" * 60)
    print()
    
    # 检查依赖
    print("📌 检查依赖...")
    if not check_onnx_support():
        return 1
    
    try:
        from TTS.api import TTS
        print("✅ TTS 库已安装")
    except ImportError:
        print("❌ TTS 库未安装")
        print("请安装: pip install TTS")
        return 1
    
    print()
    
    if args.check_only:
        print("✅ 依赖检查完成")
        return 0
    
    # 执行导出
    model_path = project_root / args.model_path
    output_dir = project_root / args.output_dir
    
    success = export_yourtts_to_onnx(model_path, output_dir)
    
    if success:
        print()
        print("=" * 60)
        print("✅ 导出成功！")
        print(f"ONNX 模型保存在: {output_dir}")
        print("=" * 60)
        return 0
    else:
        print()
        print("=" * 60)
        print("❌ 导出失败")
        print("=" * 60)
        print()
        print("💡 提示:")
        print("   1. YourTTS 模型结构复杂，可能需要分别导出不同组件")
        print("   2. 查看 TTS 库文档: https://github.com/coqui-ai/TTS")
        print("   3. 检查模型是否支持 ONNX 导出")
        print("   4. 可能需要修改 TTS 库的源代码以支持导出")
        return 1

if __name__ == '__main__':
    sys.exit(main())

