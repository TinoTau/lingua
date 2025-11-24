#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
验证导出的 M2M100 模型文件
"""

import sys
from pathlib import Path
import onnxruntime as ort
import onnx

def verify_model(model_dir: Path, direction: str) -> bool:
    """验证单个方向的模型"""
    print(f"\n=== 验证 M2M100 {direction} 模型 ===")
    print(f"模型目录: {model_dir}")
    
    if not model_dir.exists():
        print(f"❌ 模型目录不存在: {model_dir}")
        return False
    
    # 1. 验证 Encoder
    encoder_path = model_dir / "encoder.onnx"
    if not encoder_path.exists():
        print(f"❌ Encoder 文件不存在: {encoder_path}")
        return False
    
    try:
        session = ort.InferenceSession(str(encoder_path), providers=['CPUExecutionProvider'])
        model = onnx.load(str(encoder_path))
        
        inputs = session.get_inputs()
        outputs = session.get_outputs()
        
        print(f"✅ Encoder:")
        print(f"   输入数量: {len(inputs)}")
        for i, inp in enumerate(inputs):
            print(f"     Input[{i}]: {inp.name}, shape={inp.shape}, type={inp.type}")
        print(f"   输出数量: {len(outputs)}")
        for i, out in enumerate(outputs):
            print(f"     Output[{i}]: {out.name}, shape={out.shape}, type={out.type}")
        print(f"   IR version: {model.ir_version}")
        print(f"   Opset version: {model.opset_import[0].version}")
        
        if model.ir_version > 9:
            print(f"   ⚠️  WARNING: IR version {model.ir_version} > 9，可能不兼容 ort 1.16.3")
        
        if len(inputs) != 2 or len(outputs) != 1:
            print(f"   ⚠️  WARNING: 输入/输出数量不符合预期")
    except Exception as e:
        print(f"❌ Encoder 验证失败: {e}")
        return False
    
    # 2. 验证 Decoder
    decoder_path = model_dir / "decoder.onnx"
    if not decoder_path.exists():
        print(f"❌ Decoder 文件不存在: {decoder_path}")
        return False
    
    try:
        session = ort.InferenceSession(str(decoder_path), providers=['CPUExecutionProvider'])
        model = onnx.load(str(decoder_path))
        
        inputs = session.get_inputs()
        outputs = session.get_outputs()
        
        print(f"\n✅ Decoder:")
        print(f"   输入数量: {len(inputs)} (期望: 52)")
        print(f"   输出数量: {len(outputs)} (期望: 49)")
        
        # 验证输入名称
        expected_inputs = [
            "encoder_attention_mask",
            "input_ids",
            "encoder_hidden_states"
        ]
        for i in range(12):  # 12 layers
            expected_inputs.extend([
                f"past_key_values.{i}.decoder.key",
                f"past_key_values.{i}.decoder.value",
                f"past_key_values.{i}.encoder.key",
                f"past_key_values.{i}.encoder.value",
            ])
        expected_inputs.append("use_cache_branch")
        
        if len(inputs) != 52:
            print(f"   ❌ 输入数量不匹配: 期望 52，实际 {len(inputs)}")
            return False
        
        if len(outputs) != 49:
            print(f"   ❌ 输出数量不匹配: 期望 49，实际 {len(outputs)}")
            return False
        
        print(f"   IR version: {model.ir_version}")
        print(f"   Opset version: {model.opset_import[0].version}")
        
        if model.ir_version > 9:
            print(f"   ⚠️  WARNING: IR version {model.ir_version} > 9，可能不兼容 ort 1.16.3")
        
        print(f"   ✅ 输入/输出数量正确")
    except Exception as e:
        print(f"❌ Decoder 验证失败: {e}")
        return False
    
    # 3. 验证 Tokenizer 文件
    required_files = [
        "tokenizer.json",
        "sentencepiece.model",
        "config.json"
    ]
    
    print(f"\n✅ Tokenizer 文件:")
    all_exists = True
    for file in required_files:
        file_path = model_dir / file
        if file_path.exists():
            size_mb = file_path.stat().st_size / (1024 * 1024)
            print(f"   ✅ {file} ({size_mb:.2f} MB)")
        else:
            print(f"   ❌ {file} 不存在")
            all_exists = False
    
    if not all_exists:
        print(f"   ⚠️  部分 Tokenizer 文件缺失")
        return False
    
    # 4. 验证 config.json 中的 lang_to_id
    try:
        import json
        config_path = model_dir / "config.json"
        with open(config_path, 'r', encoding='utf-8') as f:
            config = json.load(f)
        
        if 'lang_to_id' in config:
            lang_to_id = config['lang_to_id']
            print(f"\n✅ Config 文件:")
            print(f"   支持的语言: {list(lang_to_id.keys())[:10]}...")  # 只显示前10个
            if 'en' in lang_to_id and 'zh' in lang_to_id:
                print(f"   ✅ 包含 en 和 zh 语言 ID")
            else:
                print(f"   ⚠️  缺少 en 或 zh 语言 ID")
        else:
            print(f"   ⚠️  config.json 中缺少 lang_to_id 字段")
    except Exception as e:
        print(f"   ⚠️  无法读取 config.json: {e}")
    
    print(f"\n✅ M2M100 {direction} 模型验证通过！")
    return True

def main():
    if len(sys.argv) < 2:
        print("用法: python verify_m2m100_models.py <模型目录1> [模型目录2]")
        print("示例: python verify_m2m100_models.py core/engine/models/nmt/m2m100-en-zh")
        sys.exit(1)
    
    model_dirs = [Path(arg) for arg in sys.argv[1:]]
    
    all_success = True
    for model_dir in model_dirs:
        # 从路径推断方向
        direction = model_dir.name.replace("m2m100-", "")
        success = verify_model(model_dir, direction)
        if not success:
            all_success = False
    
    if all_success:
        print("\n✅ 所有模型验证通过！")
        sys.exit(0)
    else:
        print("\n❌ 部分模型验证失败，请检查上述输出")
        sys.exit(1)

if __name__ == "__main__":
    main()

