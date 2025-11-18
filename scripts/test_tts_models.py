#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试 TTS 模型文件是否存在和可访问

使用方法:
    python scripts/test_tts_models.py
"""

import sys
import os
from pathlib import Path

def test_tts_models():
    """测试 TTS 模型文件"""
    print("=== TTS 模型文件检查 ===\n")
    
    model_dir = Path("core/engine/models/tts")
    
    if not model_dir.exists():
        print(f"❌ TTS 模型目录不存在: {model_dir}")
        return False
    
    print(f"✅ TTS 模型目录存在: {model_dir}\n")
    
    # 检查 FastSpeech2 模型
    print("=== FastSpeech2 模型 ===")
    fastspeech2_dir = model_dir / "fastspeech2-lite"
    
    models_to_check = [
        ("fastspeech2_csmsc_streaming.onnx", "中文 FastSpeech2"),
        ("fastspeech2_ljspeech.onnx", "英文 FastSpeech2"),
        ("phone_id_map.txt", "音素 ID 映射"),
        ("speech_stats.npy", "语音统计信息"),
    ]
    
    all_ok = True
    for filename, desc in models_to_check:
        filepath = fastspeech2_dir / filename
        if filepath.exists():
            size_mb = filepath.stat().st_size / (1024 * 1024)
            print(f"  ✅ {desc}: {filepath.name} ({size_mb:.1f} MB)")
        else:
            print(f"  ❌ {desc}: {filepath.name} (不存在)")
            all_ok = False
    
    # 检查 HiFiGAN 模型
    print("\n=== HiFiGAN 模型 ===")
    hifigan_dir = model_dir / "hifigan-lite"
    
    vocoder_models = [
        ("hifigan_csmsc.onnx", "中文 HiFiGAN"),
        ("hifigan_ljspeech.onnx", "英文 HiFiGAN"),
    ]
    
    for filename, desc in vocoder_models:
        filepath = hifigan_dir / filename
        if filepath.exists():
            size_mb = filepath.stat().st_size / (1024 * 1024)
            print(f"  ✅ {desc}: {filepath.name} ({size_mb:.1f} MB)")
        else:
            print(f"  ❌ {desc}: {filepath.name} (不存在)")
            all_ok = False
    
    print("\n=== 测试结果 ===")
    if all_ok:
        print("✅ 所有必需的模型文件都存在")
        return True
    else:
        print("❌ 部分模型文件缺失")
        return False


if __name__ == "__main__":
    success = test_tts_models()
    sys.exit(0 if success else 1)

