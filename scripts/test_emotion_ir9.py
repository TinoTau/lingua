#!/usr/bin/env python3
"""
测试 IR 9 模型是否能满足功能需求

使用方法:
    python scripts/test_emotion_ir9.py
"""

import onnx
import onnxruntime as ort
import numpy as np
from pathlib import Path


def test_ir9_model():
    """测试 IR 9 模型是否能正常加载和运行"""
    model_path = Path("core/engine/models/emotion/xlm-r/model_ir9.onnx")
    
    if not model_path.exists():
        print(f"❌ Model not found: {model_path}")
        return False
    
    print(f"=== Testing IR 9 Model ===")
    print(f"Model path: {model_path}")
    
    # 1. 检查模型 IR 版本
    print("\n=== Checking Model IR Version ===")
    try:
        model = onnx.load(str(model_path))
        print(f"IR Version: {model.ir_version}")
        print(f"Opset Version: {model.opset_import[0].version}")
        
        if model.ir_version > 9:
            print(f"⚠️  Warning: IR version {model.ir_version} > 9")
            return False
        else:
            print("✅ IR version is compatible with ort 1.16.3")
    except Exception as e:
        print(f"❌ Failed to load model: {e}")
        return False
    
    # 2. 检查模型输入/输出
    print("\n=== Checking Model Inputs/Outputs ===")
    try:
        session = ort.InferenceSession(str(model_path))
        inputs = session.get_inputs()
        outputs = session.get_outputs()
        
        print("Inputs:")
        for inp in inputs:
            print(f"  - {inp.name}: shape={inp.shape}, type={inp.type}")
        
        print("Outputs:")
        for out in outputs:
            print(f"  - {out.name}: shape={out.shape}, type={out.type}")
    except Exception as e:
        print(f"❌ Failed to create inference session: {e}")
        return False
    
    # 3. 测试推理
    print("\n=== Testing Inference ===")
    try:
        # 准备测试输入（batch_size=1, seq_len=128）
        input_ids = np.random.randint(0, 1000, (1, 128), dtype=np.int64)
        attention_mask = np.ones((1, 128), dtype=np.int64)
        
        # 运行推理
        outputs = session.run(
            None,
            {
                "input_ids": input_ids,
                "attention_mask": attention_mask,
            }
        )
        
        print(f"✅ Inference successful")
        print(f"Output shape: {outputs[0].shape}")
        print(f"Output type: {outputs[0].dtype}")
        
        # 检查输出是否合理
        logits = outputs[0]
        if logits.shape[0] == 1 and logits.shape[1] > 0:
            print(f"✅ Output shape is correct: {logits.shape}")
            print(f"Sample logits: {logits[0][:3]}")
            return True
        else:
            print(f"❌ Unexpected output shape: {logits.shape}")
            return False
            
    except Exception as e:
        print(f"❌ Inference failed: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_ir9_model()
    
    print("\n=== Test Result ===")
    if success:
        print("✅ IR 9 model can satisfy functional requirements")
    else:
        print("❌ IR 9 model may have issues")
    
    exit(0 if success else 1)

