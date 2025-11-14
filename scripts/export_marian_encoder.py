#!/usr/bin/env python3
"""
导出 Marian NMT 的 Encoder 模型为 ONNX 格式
这个脚本专门用于导出 encoder，配合现有的 decoder 模型使用

使用方法:
    python scripts/export_marian_encoder.py \
        --model_name Helsinki-NLP/opus-mt-en-zh \
        --output_dir core/engine/models/nmt/marian-en-zh

或者从本地目录加载:
    python scripts/export_marian_encoder.py \
        --model_path /path/to/local/model \
        --output_dir core/engine/models/nmt/marian-en-zh
"""

import argparse
import sys
from pathlib import Path

try:
    import torch
    from transformers import MarianMTModel, MarianTokenizer
    import onnx
except ImportError as e:
    print(f"Error: Missing required package: {e}")
    print("\nPlease install required packages:")
    print("  pip install torch transformers onnx")
    sys.exit(1)


def export_encoder(model, tokenizer, output_dir: Path):
    """导出 encoder 模型"""
    print("\n=== Exporting Encoder ===")
    
    encoder = model.get_encoder()
    encoder.eval()
    
    encoder_output_path = output_dir / "encoder_model.onnx"
    
    # Encoder 的输入：input_ids 和 attention_mask
    # 创建示例输入
    batch_size = 1
    seq_len = 10
    dummy_input_ids = torch.randint(0, tokenizer.vocab_size, (batch_size, seq_len), dtype=torch.long)
    dummy_attention_mask = torch.ones((batch_size, seq_len), dtype=torch.long)
    
    print(f"Encoder input shape: input_ids={dummy_input_ids.shape}, attention_mask={dummy_attention_mask.shape}")
    print(f"Model config: d_model={model.config.d_model}, encoder_layers={model.config.encoder_layers}")
    
    # 导出 encoder
    try:
        torch.onnx.export(
            encoder,
            (dummy_input_ids, dummy_attention_mask),
            str(encoder_output_path),
            input_names=["input_ids", "attention_mask"],
            output_names=["last_hidden_state"],
            dynamic_axes={
                "input_ids": {0: "batch_size", 1: "sequence_length"},
                "attention_mask": {0: "batch_size", 1: "sequence_length"},
                "last_hidden_state": {0: "batch_size", 1: "sequence_length"},
            },
            opset_version=13,  # 降低到 13 以兼容 ort 1.16.3 (支持 IR 9)
            do_constant_folding=True,
            verbose=False,
        )
        
        # 降级 IR 版本以兼容 ort 1.16.3 (支持 IR 9)
        print("  Converting IR version from 10 to 9 for ort 1.16.3 compatibility...")
        model_proto = onnx.load(str(encoder_output_path))
        model_proto.ir_version = 9
        onnx.save(model_proto, str(encoder_output_path))
        
        file_size_mb = encoder_output_path.stat().st_size / (1024 * 1024)
        print(f"[OK] Encoder exported to: {encoder_output_path}")
        print(f"  File size: {file_size_mb:.2f} MB")
        print(f"  IR version: {model_proto.ir_version}")
        
        return encoder_output_path
        
    except Exception as e:
        print(f"[ERROR] Failed to export encoder: {e}")
        import traceback
        traceback.print_exc()
        return None


def verify_encoder_model(encoder_path: Path, model, tokenizer):
    """验证导出的 encoder 模型"""
    print("\n=== Verifying Encoder Model ===")
    
    try:
        import onnxruntime as ort
        
        # 创建 ONNX Runtime session
        session = ort.InferenceSession(str(encoder_path))
        
        # 准备测试输入
        test_text = "Hello world"
        encoded = tokenizer(test_text, return_tensors="pt", padding=True)
        input_ids = encoded["input_ids"].numpy()
        attention_mask = encoded["attention_mask"].numpy()
        
        # 运行 ONNX 模型
        outputs = session.run(
            None,
            {
                "input_ids": input_ids,
                "attention_mask": attention_mask,
            }
        )
        
        encoder_hidden_states = outputs[0]
        print(f"[OK] Encoder model works correctly")
        print(f"  Input shape: {input_ids.shape}")
        print(f"  Output shape: {encoder_hidden_states.shape}")
        print(f"  Expected shape: (batch_size, seq_len, {model.config.d_model})")
        
        # 验证输出形状
        expected_shape = (input_ids.shape[0], input_ids.shape[1], model.config.d_model)
        if encoder_hidden_states.shape == expected_shape:
            print(f"[OK] Output shape matches expected: {expected_shape}")
        else:
            print(f"[WARN] Output shape mismatch: got {encoder_hidden_states.shape}, expected {expected_shape}")
        
        return True
        
    except ImportError:
        print("[WARN] onnxruntime not installed, skipping verification")
        print("  Install with: pip install onnxruntime")
        return False
    except Exception as e:
        print(f"[ERROR] Verification failed: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    parser = argparse.ArgumentParser(description="Export Marian NMT Encoder to ONNX format")
    parser.add_argument(
        "--model_name",
        type=str,
        help="HuggingFace model name (e.g., Helsinki-NLP/opus-mt-en-zh)",
    )
    parser.add_argument(
        "--model_path",
        type=str,
        help="Local path to model directory",
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        required=True,
        help="Output directory for ONNX encoder model",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify the exported model works correctly",
    )
    
    args = parser.parse_args()
    
    if not args.model_name and not args.model_path:
        parser.error("Either --model_name or --model_path must be provided")
    
    # 创建输出目录
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Output directory: {output_dir}")
    
    # 加载模型
    print("\n=== Loading Model ===")
    try:
        if args.model_path:
            print(f"Loading from local path: {args.model_path}")
            model = MarianMTModel.from_pretrained(args.model_path)
            tokenizer = MarianTokenizer.from_pretrained(args.model_path)
        else:
            print(f"Loading from HuggingFace: {args.model_name}")
            model = MarianMTModel.from_pretrained(args.model_name)
            tokenizer = MarianTokenizer.from_pretrained(args.model_name)
        
        print(f"[OK] Model loaded successfully")
        print(f"  Model type: {model.config.model_type}")
        print(f"  Vocab size: {tokenizer.vocab_size}")
        print(f"  d_model: {model.config.d_model}")
        print(f"  Encoder layers: {model.config.encoder_layers}")
        
    except Exception as e:
        print(f"[ERROR] Failed to load model: {e}")
        sys.exit(1)
    
    # 导出 encoder
    encoder_path = export_encoder(model, tokenizer, output_dir)
    
    if encoder_path and encoder_path.exists():
        print(f"\n[OK] Encoder export completed successfully!")
        
        # 验证模型
        if args.verify:
            verify_encoder_model(encoder_path, model, tokenizer)
        
        print("\n=== Next Steps ===")
        print("1. Update your Rust code to load encoder_model.onnx")
        print("2. Run encoder to get encoder_hidden_states")
        print("3. Use encoder_hidden_states with your existing decoder_model.onnx")
        print("\nExample usage in Rust:")
        print("  let encoder = load_encoder_model(\"encoder_model.onnx\")?;")
        print("  let encoder_hidden_states = encoder.run(input_ids, attention_mask)?;")
        print("  let translated = decoder.run(encoder_hidden_states, ...)?;")
    else:
        print("\n[ERROR] Encoder export failed")
        sys.exit(1)


if __name__ == "__main__":
    main()

