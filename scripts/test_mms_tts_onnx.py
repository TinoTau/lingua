#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试 MMS TTS ONNX 模型

使用方法:
    python scripts/test_mms_tts_onnx.py
"""

import sys
import io
from pathlib import Path

# 设置 UTF-8 编码输出（Windows 兼容）
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

def main():
    # 自动检测脚本所在目录和项目根目录
    script_dir = Path(__file__).parent
    repo_root = script_dir.parent
    current_dir = Path.cwd()
    
    # 尝试多个可能的路径
    possible_paths = [
        current_dir / "core" / "engine" / "models" / "tts" / "mms-tts-eng" / "onnx" / "model.onnx",
        repo_root / "core" / "engine" / "models" / "tts" / "mms-tts-eng" / "onnx" / "model.onnx",
        Path("core/engine/models/tts/mms-tts-eng/onnx/model.onnx"),
    ]
    
    onnx_model_path = None
    for path in possible_paths:
        abs_path = path.resolve()
        if abs_path.exists():
            onnx_model_path = abs_path
            break
    
    if onnx_model_path is None:
        print("❌ 无法找到 ONNX 模型文件")
        print(f"当前工作目录: {current_dir.absolute()}")
        print("尝试过的路径:")
        for path in possible_paths:
            abs_path = path.resolve()
            exists = "✅" if abs_path.exists() else "❌"
            print(f"  {exists} {abs_path}")
        return
    
    print(f"✅ 找到 ONNX 模型: {onnx_model_path}\n")
    
    # 检查依赖
    print("=== 检查依赖 ===")
    try:
        import onnxruntime as ort
        print(f"✅ onnxruntime: {ort.__version__}")
    except ImportError:
        print("❌ onnxruntime 未安装")
        print("   请执行: pip install onnxruntime")
        return
    
    try:
        from transformers import VitsTokenizer
        print("✅ transformers: 已安装")
    except ImportError:
        print("❌ transformers 未安装")
        print("   请执行: pip install transformers")
        return
    
    try:
        import numpy as np
        print(f"✅ numpy: {np.__version__}")
    except ImportError:
        print("❌ numpy 未安装")
        print("   请执行: pip install numpy")
        return
    
    try:
        import scipy.io.wavfile
        print("✅ scipy: 已安装")
    except ImportError:
        print("❌ scipy 未安装")
        print("   请执行: pip install scipy")
        return
    
    print()
    
    # 1. 加载 tokenizer
    print("=== 加载 Tokenizer ===")
    try:
        tokenizer = VitsTokenizer.from_pretrained("facebook/mms-tts-eng")
        print("✅ Tokenizer 加载成功")
    except Exception as e:
        print(f"❌ Tokenizer 加载失败: {e}")
        return
    
    print()
    
    # 2. 准备输入文本
    test_text = "Hello from Lingua. This is a test of the MMS TTS ONNX model."
    print(f"=== 测试文本 ===")
    print(f"文本: '{test_text}'")
    print()
    
    # 3. 编码文本
    print("=== 编码文本 ===")
    try:
        inputs = tokenizer(test_text, return_tensors="np")
        input_ids = inputs["input_ids"].astype("int64")  # ONNX 需要 int64
        
        # 生成 attention_mask（1 表示有效 token，0 表示 padding）
        # 对于 TTS，通常所有 token 都是有效的，所以全部设为 1
        attention_mask = np.ones_like(input_ids, dtype="int64")
        
        print(f"✅ 编码成功")
        print(f"   input_ids shape: {input_ids.shape}")
        print(f"   input_ids dtype: {input_ids.dtype}")
        print(f"   input_ids 前10个值: {input_ids[0][:10].tolist()}")
        print(f"   attention_mask shape: {attention_mask.shape}")
        print(f"   attention_mask dtype: {attention_mask.dtype}")
    except Exception as e:
        print(f"❌ 编码失败: {e}")
        import traceback
        traceback.print_exc()
        return
    
    print()
    
    # 4. 加载 ONNX 模型
    print("=== 加载 ONNX 模型 ===")
    try:
        sess = ort.InferenceSession(
            str(onnx_model_path),
            providers=["CPUExecutionProvider"]
        )
        print("✅ ONNX 模型加载成功")
        
        # 检查输入/输出信息
        print("\n模型输入信息:")
        for inp in sess.get_inputs():
            print(f"  名称: {inp.name}")
            print(f"  形状: {inp.shape}")
            print(f"  类型: {inp.type}")
        
        print("\n模型输出信息:")
        for out in sess.get_outputs():
            print(f"  名称: {out.name}")
            print(f"  形状: {out.shape}")
            print(f"  类型: {out.type}")
    except Exception as e:
        print(f"❌ ONNX 模型加载失败: {e}")
        import traceback
        traceback.print_exc()
        return
    
    print()
    
    # 5. 运行推理
    print("=== 运行推理 ===")
    try:
        # 准备所有必需的输入
        input_names = [inp.name for inp in sess.get_inputs()]
        print(f"模型需要的输入: {input_names}")
        
        # 构建输入字典
        input_feed = {}
        for inp in sess.get_inputs():
            if inp.name == "input_ids":
                input_feed[inp.name] = input_ids
            elif inp.name == "attention_mask":
                input_feed[inp.name] = attention_mask
            else:
                print(f"⚠️  警告: 未知输入 '{inp.name}'，跳过")
        
        print(f"准备输入: {list(input_feed.keys())}")
        
        outputs = sess.run(None, input_feed)
        audio = outputs[0]  # waveform 是第一个输出
        
        print(f"✅ 推理成功")
        print(f"   输出数量: {len(outputs)}")
        for i, out in enumerate(sess.get_outputs()):
            print(f"   输出 {i} ({out.name}): shape={outputs[i].shape}, dtype={outputs[i].dtype}")
        
        # waveform 是第一个输出
        audio = outputs[0]
        print(f"\n使用输出: waveform")
        print(f"   输出形状: {audio.shape}")
        print(f"   输出数据类型: {audio.dtype}")
        
        # 如果是 2D，取第一行
        if len(audio.shape) == 2:
            audio = audio.squeeze(0)
        elif len(audio.shape) == 1:
            pass
        else:
            print(f"⚠️  警告: 意外的输出形状 {audio.shape}，尝试 squeeze")
            audio = audio.squeeze()
        
        print(f"   处理后形状: {audio.shape}")
        
        # 检查音频数据范围
        min_val = float(audio.min())
        max_val = float(audio.max())
        mean_val = float(audio.mean())
        print(f"   音频范围: min={min_val:.6f}, max={max_val:.6f}, mean={mean_val:.6f}")
        
        # 检查是否在合理范围内（通常应该在 [-1, 1] 或需要归一化）
        if abs(max_val) > 1.0 or abs(min_val) > 1.0:
            print(f"⚠️  警告: 音频值超出 [-1, 1] 范围，可能需要归一化")
            # 尝试归一化
            audio_max = max(abs(min_val), abs(max_val))
            if audio_max > 1e-6:
                audio = audio / audio_max
                print(f"   已归一化到 [-1, 1]")
        
    except Exception as e:
        print(f"❌ 推理失败: {e}")
        import traceback
        traceback.print_exc()
        return
    
    print()
    
    # 6. 保存音频文件
    print("=== 保存音频文件 ===")
    output_dir = repo_root / "test_output"
    output_dir.mkdir(exist_ok=True)
    output_wav = output_dir / "mms_tts_onnx_test.wav"
    
    try:
        # MMS TTS 的采样率通常是 16000 Hz
        sample_rate = 16000
        scipy.io.wavfile.write(
            str(output_wav),
            sample_rate,
            audio.astype("float32")
        )
        print(f"✅ 音频已保存: {output_wav}")
        print(f"   采样率: {sample_rate} Hz")
        print(f"   时长: {len(audio) / sample_rate:.2f} 秒")
        print(f"   样本数: {len(audio)}")
        print()
        print("💡 请播放此文件检查音频质量")
    except Exception as e:
        print(f"❌ 保存音频失败: {e}")
        import traceback
        traceback.print_exc()
        return
    
    print()
    print("=== 验证完成 ===")
    print("✅ 所有步骤成功完成！")
    print(f"📁 音频文件: {output_wav}")
    print()
    print("下一步:")
    print("1. 播放音频文件确认质量")
    print("2. 如果音频正常，可以开始实现 Rust 端的 VitsTtsEngine")

if __name__ == "__main__":
    main()

