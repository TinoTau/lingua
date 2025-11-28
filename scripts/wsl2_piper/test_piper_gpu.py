#!/usr/bin/env python3
"""
测试 Piper TTS GPU 支持
"""

import os
from pathlib import Path
from piper.voice import PiperVoice

# 模型路径（实际文件在 zh 目录下，不在子目录中）
model_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx"
config_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx.json"

print("=== Piper TTS GPU 测试 ===\n")

# 检查文件是否存在
print(f"模型路径: {model_path}")
print(f"模型存在: {model_path.exists()}")
print(f"\n配置路径: {config_path}")
print(f"配置存在: {config_path.exists()}\n")

if not model_path.exists():
    print("✗ 模型文件不存在！")
    exit(1)

if not config_path.exists():
    print("✗ 配置文件不存在！")
    exit(1)

# 尝试加载模型（使用 GPU）
print("尝试加载模型（使用 GPU）...")
try:
    voice = PiperVoice.load(
        str(model_path),
        config_path=str(config_path),
        use_cuda=True
    )
    print("✓ 模型加载成功（GPU 模式）")
    
    # 检查实际使用的执行提供程序
    try:
        session = voice.session
        providers = session.get_providers()
        print(f"  实际使用的执行提供程序: {providers}")
        if 'CUDAExecutionProvider' in providers:
            print("  ✓ 确认使用 GPU 加速")
        else:
            print("  ⚠ 警告: 虽然请求了 GPU，但实际使用的是 CPU")
    except Exception as e:
        print(f"  无法检查执行提供程序: {e}")
    
    # 测试合成
    print("\n测试语音合成...")
    test_text = "测试GPU加速"
    # synthesize 返回 generator，每个元素是 AudioChunk 对象
    audio_generator = voice.synthesize(test_text)
    audio_chunks = list(audio_generator)
    
    # AudioChunk 对象有 audio_int16_bytes 属性，是 bytes
    if audio_chunks:
        # 使用 audio_int16_bytes 属性获取音频数据
        audio_bytes = b''.join(chunk.audio_int16_bytes for chunk in audio_chunks if chunk.audio_int16_bytes)
        print(f"✓ 合成成功，音频块数: {len(audio_chunks)}, 总大小: {len(audio_bytes)} 字节")
    else:
        print("✗ 合成失败：没有生成音频块")
        audio_bytes = b''
    
    # 保存测试文件
    output_path = Path.home() / "piper_env" / "test_gpu_output.wav"
    with open(output_path, "wb") as f:
        f.write(audio_bytes)
    print(f"✓ 音频已保存到: {output_path}")
    
except Exception as e:
    print(f"✗ 加载失败: {e}")
    import traceback
    traceback.print_exc()
    
    # 尝试 CPU 模式
    print("\n尝试 CPU 模式...")
    try:
        voice = PiperVoice.load(
            str(model_path),
            config_path=str(config_path),
            use_cuda=False
        )
        print("✓ 模型加载成功（CPU 模式）")
    except Exception as e2:
        print(f"✗ CPU 模式也失败: {e2}")

print("\n=== 测试完成 ===")

