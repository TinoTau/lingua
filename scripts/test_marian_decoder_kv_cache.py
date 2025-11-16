#!/usr/bin/env python3
"""
测试 Marian NMT Decoder 模型的 KV Cache 功能

这个脚本用于验证：
1. 模型是否支持 KV cache
2. KV cache 在 Python 中是否正常工作
3. 是否会出现 Reshape 错误

使用方法:
    python scripts/test_marian_decoder_kv_cache.py \
        --model_dir core/engine/models/nmt/marian-en-zh
"""

import argparse
import sys
from pathlib import Path
import numpy as np

try:
    import onnxruntime as ort
except ImportError:
    print("Error: Missing onnxruntime package")
    print("Please install: pip install onnxruntime")
    sys.exit(1)


def load_model(model_dir: Path):
    """加载 ONNX 模型"""
    # 尝试不同的文件名
    decoder_path = model_dir / "decoder_model.onnx"
    if not decoder_path.exists():
        decoder_path = model_dir / "model.onnx"  # 备用名称
    encoder_path = model_dir / "encoder_model.onnx"
    
    if not decoder_path.exists():
        raise FileNotFoundError(f"Decoder model not found: {decoder_path}")
    if not encoder_path.exists():
        raise FileNotFoundError(f"Encoder model not found: {encoder_path}")
    
    print(f"Loading decoder model: {decoder_path}")
    decoder_session = ort.InferenceSession(str(decoder_path))
    
    print(f"Loading encoder model: {encoder_path}")
    encoder_session = ort.InferenceSession(str(encoder_path))
    
    return encoder_session, decoder_session


def get_model_info(session: ort.InferenceSession, name: str):
    """获取模型的输入/输出信息"""
    print(f"\n=== {name} Model Info ===")
    print("Inputs:")
    for inp in session.get_inputs():
        print(f"  {inp.name}: shape={inp.shape}, type={inp.type}")
    print("Outputs:")
    for out in session.get_outputs():
        print(f"  {out.name}: shape={out.shape}, type={out.type}")


def build_initial_kv_values(num_layers: int, batch_size: int, num_heads: int, 
                            encoder_seq_len: int, head_dim: int):
    """构建初始 KV cache 值（用于第一步）"""
    kv_values = {}
    
    for layer_idx in range(num_layers):
        # Decoder KV: [batch, heads, 0, head_dim] (初始长度为 0，但模型可能要求至少为 1)
        # 注意：根据模型要求，可能需要设置为 1 而不是 0
        dec_len = 1  # 第一步有 BOS token
        
        dec_k = np.zeros((batch_size, num_heads, dec_len, head_dim), dtype=np.float32)
        dec_v = np.zeros((batch_size, num_heads, dec_len, head_dim), dtype=np.float32)
        
        # Encoder KV: [batch, heads, encoder_seq_len, head_dim]
        enc_k = np.zeros((batch_size, num_heads, encoder_seq_len, head_dim), dtype=np.float32)
        enc_v = np.zeros((batch_size, num_heads, encoder_seq_len, head_dim), dtype=np.float32)
        
        kv_values[f"past_key_values.{layer_idx}.decoder.key"] = dec_k
        kv_values[f"past_key_values.{layer_idx}.decoder.value"] = dec_v
        kv_values[f"past_key_values.{layer_idx}.encoder.key"] = enc_k
        kv_values[f"past_key_values.{layer_idx}.encoder.value"] = enc_v
    
    return kv_values


def test_decoder_kv_cache(encoder_session: ort.InferenceSession, 
                          decoder_session: ort.InferenceSession,
                          model_dir: Path):
    """测试 Decoder 的 KV cache 功能"""
    print("\n" + "="*60)
    print("Testing Decoder KV Cache")
    print("="*60)
    
    # 获取模型信息
    get_model_info(encoder_session, "Encoder")
    get_model_info(decoder_session, "Decoder")
    
    # 准备测试数据
    batch_size = 1
    encoder_seq_len = 4  # 简短的测试序列
    num_layers = 6  # Marian base 模型有 6 层
    num_heads = 8
    head_dim = 64  # d_model / num_heads = 512 / 8 = 64
    
    # 1. 先运行 Encoder
    print("\n=== Step 0: Running Encoder ===")
    encoder_input_ids = np.array([[0, 3828, 431, 0]], dtype=np.int64)  # "Hello world" 的 token IDs
    encoder_attention_mask = np.ones((batch_size, encoder_seq_len), dtype=np.int64)
    
    encoder_inputs = {
        "input_ids": encoder_input_ids,
        "attention_mask": encoder_attention_mask,
    }
    
    try:
        encoder_outputs = encoder_session.run(None, encoder_inputs)
        encoder_hidden_states = encoder_outputs[0]
        print(f"[OK] Encoder output shape: {encoder_hidden_states.shape}")
    except Exception as e:
        print(f"[ERROR] Encoder failed: {e}")
        return False
    
    # 2. Decoder Step 0: 第一步（use_cache_branch=False）
    print("\n=== Step 1: Decoder First Step (use_cache_branch=False) ===")
    decoder_start_token_id = 65000  # BOS token
    
    # 构建初始 KV cache
    initial_kv = build_initial_kv_values(
        num_layers, batch_size, num_heads, encoder_seq_len, head_dim
    )
    
    decoder_inputs_step0 = {
        "input_ids": np.array([[decoder_start_token_id]], dtype=np.int64),
        "encoder_hidden_states": encoder_hidden_states,
        "encoder_attention_mask": encoder_attention_mask,
        "use_cache_branch": np.array([False], dtype=bool),
        **initial_kv,
    }
    
    print(f"Input shapes:")
    for name, value in decoder_inputs_step0.items():
        if isinstance(value, np.ndarray):
            print(f"  {name}: {value.shape} {value.dtype}")
        else:
            print(f"  {name}: {value}")
    
    try:
        decoder_outputs_step0 = decoder_session.run(None, decoder_inputs_step0)
        print(f"[OK] Step 0 completed successfully")
        print(f"  Number of outputs: {len(decoder_outputs_step0)}")
        
        # 找到 logits 输出（通常是第一个）
        logits_step0 = decoder_outputs_step0[0]
        print(f"  Logits shape: {logits_step0.shape}")
        
        # 提取 present.* 输出（用于下一步）
        present_outputs = {}
        output_names = [out.name for out in decoder_session.get_outputs()]
        for i, name in enumerate(output_names):
            if name.startswith("present."):
                present_outputs[name] = decoder_outputs_step0[i]
                print(f"  {name}: shape={decoder_outputs_step0[i].shape}")
        
    except Exception as e:
        print(f"[ERROR] Step 0 failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    
    # 3. Decoder Step 1: 第二步（use_cache_branch=True）
    print("\n=== Step 2: Decoder Second Step (use_cache_branch=True) ===")
    
    # 从 logits 中选择下一个 token（简化：选择 argmax）
    next_token_id = int(np.argmax(logits_step0[0, -1, :]))
    print(f"Next token ID: {next_token_id}")
    
    # 构建 past_key_values（使用 step 0 的 present.*）
    past_kv = {}
    for name, value in present_outputs.items():
        # 将 present.* 转换为 past_key_values.*
        # present.0.decoder.key -> past_key_values.0.decoder.key
        past_name = name.replace("present.", "past_key_values.")
        past_kv[past_name] = value
        print(f"  {past_name}: shape={value.shape}")
    
    decoder_inputs_step1 = {
        "input_ids": np.array([[next_token_id]], dtype=np.int64),  # 只输入新 token
        "encoder_hidden_states": encoder_hidden_states,  # 保持不变
        "encoder_attention_mask": encoder_attention_mask,  # 保持不变
        "use_cache_branch": np.array([True], dtype=bool),  # 启用 KV cache
        **past_kv,
    }
    
    print(f"Input shapes:")
    for name, value in decoder_inputs_step1.items():
        if isinstance(value, np.ndarray):
            print(f"  {name}: {value.shape} {value.dtype}")
        else:
            print(f"  {name}: {value}")
    
    try:
        decoder_outputs_step1 = decoder_session.run(None, decoder_inputs_step1)
        print(f"[OK] Step 1 completed successfully")
        print(f"  Number of outputs: {len(decoder_outputs_step1)}")
        
        logits_step1 = decoder_outputs_step1[0]
        print(f"  Logits shape: {logits_step1.shape}")
        
        # 检查 present.* 输出
        for i, name in enumerate(output_names):
            if name.startswith("present."):
                print(f"  {name}: shape={decoder_outputs_step1[i].shape}")
        
        print("\n[SUCCESS] KV Cache test PASSED!")
        return True
        
    except Exception as e:
        print(f"[ERROR] Step 1 failed: {e}")
        import traceback
        traceback.print_exc()
        
        # 检查是否是 Reshape 错误
        error_str = str(e)
        if "Reshape" in error_str or "reshape" in error_str:
            print("\n⚠️  This is the Reshape error we're trying to fix!")
            print("   The error occurs in the model's Reshape node.")
            print("   This suggests the issue is in the model export, not the code.")
        return False


def main():
    parser = argparse.ArgumentParser(description="Test Marian NMT Decoder KV Cache")
    parser.add_argument(
        "--model_dir",
        type=str,
        default="core/engine/models/nmt/marian-en-zh",
        help="Path to the model directory",
    )
    args = parser.parse_args()
    
    model_dir = Path(args.model_dir)
    if not model_dir.exists():
        print(f"Error: Model directory not found: {model_dir}")
        sys.exit(1)
    
    try:
        # 加载模型
        encoder_session, decoder_session = load_model(model_dir)
        
        # 测试 KV cache
        success = test_decoder_kv_cache(encoder_session, decoder_session, model_dir)
        
        if success:
            print("\n" + "="*60)
            print("CONCLUSION: KV Cache works in Python!")
            print("  → This means the issue is likely in the Rust code (方案 1)")
            print("  → Try fixing the code implementation first")
            print("="*60)
            sys.exit(0)
        else:
            print("\n" + "="*60)
            print("CONCLUSION: KV Cache fails in Python too!")
            print("  → This means the issue is in the model export (方案 2)")
            print("  → Need to fix the model export script")
            print("="*60)
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()

