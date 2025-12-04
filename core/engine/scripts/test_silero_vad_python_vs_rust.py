#!/usr/bin/env python3
"""
Silero VAD Python vs Rust 对比测试
使用官方模型对比 Python 和 Rust 实现的输出差异
"""

import numpy as np
import onnxruntime as ort
import os
import sys
from pathlib import Path

def create_speech_frame(sample_rate=16000, duration_ms=32, frequency=440.0):
    """创建模拟语音帧（正弦波）"""
    num_samples = int(sample_rate * duration_ms / 1000)
    t = np.arange(num_samples) / sample_rate
    audio = np.sin(2 * np.pi * frequency * t) * 0.5
    return audio.astype(np.float32)

def create_silence_frame(sample_rate=16000, duration_ms=32):
    """创建静音帧"""
    num_samples = int(sample_rate * duration_ms / 1000)
    return np.zeros(num_samples, dtype=np.float32)

def test_silero_vad_python(model_path, test_cases):
    """使用 Python ONNX Runtime 测试 Silero VAD"""
    print("=" * 60)
    print("Python 实现测试")
    print("=" * 60)
    
    # 加载模型
    session = ort.InferenceSession(model_path, providers=['CPUExecutionProvider'])
    
    # 获取输入输出信息
    input_names = [inp.name for inp in session.get_inputs()]
    output_names = [out.name for out in session.get_outputs()]
    
    print(f"\n模型输入: {input_names}")
    print(f"模型输出: {output_names}")
    
    # 初始化隐藏状态
    hidden_state = np.zeros((2, 1, 128), dtype=np.float32)
    sample_rate = np.array([16000], dtype=np.int64)
    
    results = []
    
    for i, (name, audio) in enumerate(test_cases):
        # 确保音频长度为 512 样本（32ms @ 16kHz）
        if len(audio) != 512:
            # 如果长度不对，进行填充或截断
            if len(audio) < 512:
                audio = np.pad(audio, (0, 512 - len(audio)), mode='constant')
            else:
                audio = audio[:512]
        
        # 归一化到 [-1, 1]
        audio = np.clip(audio, -1.0, 1.0)
        
        # 准备输入（形状：[1, 512]）
        audio_input = audio.reshape(1, -1).astype(np.float32)
        
        # 运行推理
        inputs = {
            input_names[0]: audio_input,  # input
            input_names[1]: hidden_state,  # state
            input_names[2]: sample_rate,   # sr
        }
        
        outputs = session.run(output_names, inputs)
        
        # 提取输出
        # outputs[0] 是 [output, new_state]
        # output 形状通常是 [1, 2] 或 [1, 1]
        output = outputs[0]
        new_state = outputs[1] if len(outputs) > 1 else None
        
        # 更新隐藏状态
        if new_state is not None:
            hidden_state = new_state
        
        # 解析输出
        output_shape = output.shape
        print(f"\n[{i+1}] {name}")
        print(f"  输入音频: shape={audio_input.shape}, min={audio.min():.4f}, max={audio.max():.4f}, mean={audio.mean():.4f}, rms={np.sqrt(np.mean(audio**2)):.4f}")
        print(f"  模型输出: shape={output_shape}, values={output.flatten()}")
        
        # 根据输出形状解析
        if output_shape == (1, 2):
            # [1, 2] 格式：第一列是静音概率，第二列是语音概率
            silence_prob = output[0, 0]
            speech_prob = output[0, 1]
            print(f"  解析结果: silence_prob={silence_prob:.6f}, speech_prob={speech_prob:.6f}")
            results.append((name, speech_prob, silence_prob))
        elif output_shape == (1, 1) or len(output_shape) == 1:
            # [1, 1] 或 [1] 格式：可能是单一概率值
            prob = float(output.flatten()[0])
            print(f"  解析结果: raw_output={prob:.6f}")
            # 尝试判断是语音概率还是静音概率
            # 如果值很小（< 0.5），可能是静音概率，需要反转
            if prob < 0.5:
                speech_prob = 1.0 - prob
                silence_prob = prob
                print(f"  推断: 可能是静音概率，反转后 speech_prob={speech_prob:.6f}, silence_prob={silence_prob:.6f}")
            else:
                speech_prob = prob
                silence_prob = 1.0 - prob
                print(f"  推断: 可能是语音概率，speech_prob={speech_prob:.6f}, silence_prob={silence_prob:.6f}")
            results.append((name, speech_prob, silence_prob))
        else:
            print(f"  警告: 未知的输出形状 {output_shape}")
            results.append((name, None, None))
    
    return results

def main():
    # 确定模型路径
    script_dir = Path(__file__).parent
    core_engine_dir = script_dir.parent
    model_path = core_engine_dir / "models" / "vad" / "silero" / "silero_vad_official.onnx"
    
    if not model_path.exists():
        print(f"❌ 模型文件不存在: {model_path}")
        print("请先运行下载脚本下载模型")
        sys.exit(1)
    
    print(f"使用模型: {model_path}")
    print()
    
    # 创建测试用例
    test_cases = [
        ("语音帧（440Hz正弦波）", create_speech_frame()),
        ("静音帧（全零）", create_silence_frame()),
        ("语音帧2", create_speech_frame(frequency=880.0)),
        ("静音帧2", create_silence_frame()),
    ]
    
    # 运行 Python 测试
    python_results = test_silero_vad_python(str(model_path), test_cases)
    
    print("\n" + "=" * 60)
    print("测试结果总结")
    print("=" * 60)
    print("\nPython 实现结果:")
    for name, speech_prob, silence_prob in python_results:
        if speech_prob is not None:
            print(f"  {name}:")
            print(f"    speech_prob = {speech_prob:.6f}")
            print(f"    silence_prob = {silence_prob:.6f}")
            print(f"    判断: {'语音' if speech_prob > 0.5 else '静音'}")
        else:
            print(f"  {name}: 解析失败")
    
    print("\n" + "=" * 60)
    print("对比说明")
    print("=" * 60)
    print("""
请将上述 Python 输出与 Rust 实现的输出进行对比：

1. 检查模型输出的原始形状和值
2. 检查 speech_prob 和 silence_prob 的计算方式
3. 检查阈值判断（> 0.5 为语音）是否正确

如果 Python 和 Rust 的输出不一致，可能需要调整 Rust 实现中的输出解析逻辑。
    """)

if __name__ == "__main__":
    main()

