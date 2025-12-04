#!/usr/bin/env python3
"""
Silero VAD 官方实现测试
使用官方 silero-vad 库测试模型
"""

import sys
from pathlib import Path

try:
    import torch
    import numpy as np
    from silero_vad import load_model, get_speech_timestamps, read_audio
except ImportError as e:
    print(f"❌ 缺少依赖: {e}")
    print("请安装依赖: pip install torch torchaudio silero-vad")
    sys.exit(1)

def test_official_silero_vad():
    """使用官方 silero-vad 库测试"""
    print("=" * 60)
    print("Silero VAD 官方实现测试")
    print("=" * 60)
    
    # 加载模型
    print("\n1. 加载模型...")
    try:
        model, utils = load_model()
        get_speech_timestamps_fn = utils[0]
        print("✅ 模型加载成功")
    except Exception as e:
        print(f"❌ 模型加载失败: {e}")
        return
    
    # 创建测试音频
    print("\n2. 创建测试音频...")
    sample_rate = 16000
    duration_sec = 1.0
    
    # 测试1: 静音
    silence = np.zeros(int(sample_rate * duration_sec), dtype=np.float32)
    print(f"   静音音频: {len(silence)} 样本")
    
    # 测试2: 语音（正弦波）
    t = np.arange(int(sample_rate * duration_sec)) / sample_rate
    speech = np.sin(2 * np.pi * 440.0 * t).astype(np.float32) * 0.5
    print(f"   语音音频: {len(speech)} 样本")
    
    # 测试3: 混合（前半段语音，后半段静音）
    mixed = np.concatenate([speech[:len(speech)//2], silence[:len(silence)//2]])
    print(f"   混合音频: {len(mixed)} 样本")
    
    # 测试模型
    print("\n3. 测试模型输出...")
    
    test_cases = [
        ("静音", silence),
        ("语音", speech),
        ("混合", mixed),
    ]
    
    for name, audio in test_cases:
        print(f"\n   测试: {name}")
        try:
            # 使用官方函数获取语音时间戳
            timestamps = get_speech_timestamps_fn(
                torch.from_numpy(audio),
                model,
                threshold=0.5,
                sampling_rate=sample_rate
            )
            print(f"     检测到的语音段数: {len(timestamps)}")
            if timestamps:
                for i, ts in enumerate(timestamps):
                    print(f"       段 {i+1}: {ts['start']/sample_rate:.3f}s - {ts['end']/sample_rate:.3f}s")
            else:
                print(f"       无语音检测")
        except Exception as e:
            print(f"     ❌ 测试失败: {e}")
    
    # 测试单帧推理（512 样本）
    print("\n4. 测试单帧推理（512 样本，32ms @ 16kHz）...")
    try:
        # 获取模型的原始推理函数
        model.eval()
        
        # 创建单帧输入
        frame_size = 512
        test_frames = [
            ("静音帧", np.zeros(frame_size, dtype=np.float32)),
            ("语音帧", speech[:frame_size]),
        ]
        
        for name, frame in test_frames:
            print(f"\n   测试: {name}")
            # 准备输入
            audio_tensor = torch.from_numpy(frame).unsqueeze(0)  # [1, 512]
            
            # 初始化状态
            state = torch.zeros(2, 1, 64, dtype=torch.float32)  # 注意：可能是 64 而不是 128
            sample_rate_tensor = torch.tensor([sample_rate], dtype=torch.int64)
            
            # 尝试推理
            try:
                with torch.no_grad():
                    # 尝试不同的输入格式
                    # 根据 silero-vad 的实现，可能需要不同的输入格式
                    output, new_state = model(audio_tensor, state, sample_rate_tensor)
                    print(f"     输出形状: {output.shape}")
                    print(f"     输出值: {output.flatten().tolist()}")
                    print(f"     状态形状: {new_state.shape}")
                    
                    # 如果输出是 [1, 2]，第一列是静音概率，第二列是语音概率
                    if output.shape == (1, 2):
                        silence_prob = output[0, 0].item()
                        speech_prob = output[0, 1].item()
                        print(f"     静音概率: {silence_prob:.6f}")
                        print(f"     语音概率: {speech_prob:.6f}")
                    elif output.shape == (1, 1):
                        prob = output[0, 0].item()
                        print(f"     原始输出: {prob:.6f}")
                        # 尝试判断
                        if prob < 0.5:
                            print(f"     推断为静音概率，语音概率 = {1.0 - prob:.6f}")
                        else:
                            print(f"     推断为语音概率 = {prob:.6f}")
            except Exception as e:
                print(f"     ❌ 推理失败: {e}")
                import traceback
                traceback.print_exc()
                
    except Exception as e:
        print(f"❌ 单帧推理测试失败: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_official_silero_vad()

