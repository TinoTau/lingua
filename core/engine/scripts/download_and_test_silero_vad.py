#!/usr/bin/env python3
"""
下载并测试 Silero VAD 模型
从 GitHub 下载官方模型，并使用 ONNX Runtime 测试
"""

import os
import sys
import urllib.request
from pathlib import Path

def download_model():
    """从 GitHub 下载 Silero VAD 模型"""
    script_dir = Path(__file__).parent
    core_engine_dir = script_dir.parent
    model_dir = core_engine_dir / "models" / "vad" / "silero"
    model_dir.mkdir(parents=True, exist_ok=True)
    
    model_path = model_dir / "silero_vad_github.onnx"
    
    # GitHub 原始文件链接
    # 注意：GitHub 的 raw 链接可能不稳定，建议使用 HuggingFace
    urls = [
        "https://github.com/snakers4/silero-vad/raw/master/src/silero_vad/data/silero_vad.onnx",
        "https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx",
        "https://models.silero.ai/vad_models/silero_vad.onnx",
    ]
    
    print(f"下载模型到: {model_path}")
    
    for url in urls:
        try:
            print(f"尝试从 {url} 下载...")
            req = urllib.request.Request(url)
            req.add_header('User-Agent', 'Mozilla/5.0')
            urllib.request.urlretrieve(req, str(model_path))
            
            if model_path.exists() and model_path.stat().st_size > 0:
                print(f"✅ 下载成功: {model_path} ({model_path.stat().st_size / 1024 / 1024:.2f} MB)")
                return model_path
        except Exception as e:
            print(f"❌ 下载失败: {e}")
            continue
    
    print("❌ 所有下载源都失败了")
    return None

def test_model_with_onnxruntime(model_path):
    """使用 ONNX Runtime 测试模型"""
    try:
        import onnxruntime as ort
        import numpy as np
    except ImportError:
        print("❌ 缺少依赖: pip install onnxruntime numpy")
        return False
    
    print("\n" + "=" * 60)
    print("使用 ONNX Runtime 测试模型")
    print("=" * 60)
    
    # 加载模型
    try:
        session = ort.InferenceSession(str(model_path), providers=['CPUExecutionProvider'])
        print("✅ 模型加载成功")
    except Exception as e:
        print(f"❌ 模型加载失败: {e}")
        return False
    
    # 获取输入输出信息
    print("\n模型输入:")
    for inp in session.get_inputs():
        print(f"  - {inp.name}: shape={inp.shape}, type={inp.type}")
    
    print("\n模型输出:")
    for out in session.get_outputs():
        print(f"  - {out.name}: shape={out.shape}, type={out.type}")
    
    # 准备测试数据
    sample_rate = 16000
    frame_size = 512  # 32ms @ 16kHz
    
    # 创建测试音频
    test_cases = [
        ("静音", np.zeros(frame_size, dtype=np.float32)),
        ("语音（正弦波 440Hz）", np.sin(2 * np.pi * 440.0 * np.arange(frame_size) / sample_rate).astype(np.float32) * 0.5),
        ("语音（正弦波 880Hz）", np.sin(2 * np.pi * 880.0 * np.arange(frame_size) / sample_rate).astype(np.float32) * 0.5),
    ]
    
    # 获取输入名称
    input_names = [inp.name for inp in session.get_inputs()]
    output_names = [out.name for out in session.get_outputs()]
    
    print(f"\n输入名称: {input_names}")
    print(f"输出名称: {output_names}")
    
    # 初始化状态
    # 根据模型输入，状态可能是 [2, 1, 128] 或 [2, 1, 64]
    # 先尝试 128
    hidden_state = np.zeros((2, 1, 128), dtype=np.float32)
    sample_rate_array = np.array([sample_rate], dtype=np.int64)
    
    print("\n" + "=" * 60)
    print("测试推理")
    print("=" * 60)
    
    for name, audio in test_cases:
        print(f"\n测试: {name}")
        print(f"  音频: shape={audio.shape}, min={audio.min():.4f}, max={audio.max():.4f}, rms={np.sqrt(np.mean(audio**2)):.4f}")
        
        # 归一化到 [-1, 1]
        audio = np.clip(audio, -1.0, 1.0)
        
        # 准备输入（形状：[1, 512]）
        audio_input = audio.reshape(1, -1).astype(np.float32)
        
        # 准备输入字典
        inputs = {}
        if len(input_names) >= 1:
            inputs[input_names[0]] = audio_input
        if len(input_names) >= 2:
            inputs[input_names[1]] = hidden_state
        if len(input_names) >= 3:
            inputs[input_names[2]] = sample_rate_array
        
        try:
            # 运行推理
            outputs = session.run(output_names, inputs)
            
            # 解析输出
            output = outputs[0]
            new_state = outputs[1] if len(outputs) > 1 else None
            
            print(f"  输出形状: {output.shape}")
            print(f"  输出值: {output.flatten()}")
            
            # 更新状态
            if new_state is not None:
                hidden_state = new_state
                print(f"  状态形状: {hidden_state.shape}")
            
            # 根据输出形状解析
            if output.shape == (1, 2):
                silence_prob = output[0, 0]
                speech_prob = output[0, 1]
                print(f"  解析: silence_prob={silence_prob:.6f}, speech_prob={speech_prob:.6f}")
                print(f"  判断: {'语音' if speech_prob > 0.5 else '静音'}")
            elif output.shape == (1, 1):
                raw = output[0, 0]
                print(f"  原始输出: {raw:.6f}")
                # 尝试不同的解析方式
                print(f"    作为概率: {raw:.6f} ({'语音' if raw > 0.5 else '静音'})")
                print(f"    取反: {1.0 - raw:.6f} ({'语音' if (1.0 - raw) > 0.5 else '静音'})")
                print(f"    Sigmoid: {1.0 / (1.0 + np.exp(-raw)):.6f}")
            else:
                print(f"  警告: 未知的输出形状 {output.shape}")
                
        except Exception as e:
            print(f"  ❌ 推理失败: {e}")
            import traceback
            traceback.print_exc()
            # 如果状态形状不对，尝试 64
            if "shape" in str(e).lower() or "dimension" in str(e).lower():
                print("  尝试使用状态形状 [2, 1, 64]...")
                hidden_state = np.zeros((2, 1, 64), dtype=np.float32)
                if len(input_names) >= 2:
                    inputs[input_names[1]] = hidden_state
                try:
                    outputs = session.run(output_names, inputs)
                    output = outputs[0]
                    print(f"  成功！输出形状: {output.shape}, 输出值: {output.flatten()}")
                except Exception as e2:
                    print(f"  仍然失败: {e2}")
    
    return True

def main():
    print("=" * 60)
    print("Silero VAD 模型下载和测试")
    print("=" * 60)
    
    # 下载模型
    model_path = download_model()
    if not model_path:
        sys.exit(1)
    
    # 测试模型
    if test_model_with_onnxruntime(model_path):
        print("\n✅ 测试完成")
    else:
        print("\n❌ 测试失败")
        sys.exit(1)

if __name__ == "__main__":
    main()

