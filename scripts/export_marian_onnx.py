#!/usr/bin/env python3
"""
导出 Marian NMT 模型为 ONNX 格式
将 encoder 和 decoder 分别导出，支持增量解码

使用方法:
    python scripts/export_marian_onnx.py \
        --model_name Helsinki-NLP/opus-mt-en-zh \
        --output_dir core/engine/models/nmt/marian-en-zh

或者从本地目录加载:
    python scripts/export_marian_onnx.py \
        --model_path /path/to/local/model \
        --output_dir core/engine/models/nmt/marian-en-zh
"""

import argparse
import os
import sys
from pathlib import Path

try:
    import torch
    from transformers import MarianMTModel, MarianTokenizer
    from transformers.onnx import export, FeaturesManager
    import onnx
    from onnxruntime.quantization import quantize_dynamic, QuantType
except ImportError as e:
    print(f"Error: Missing required package: {e}")
    print("\nPlease install required packages:")
    print("  pip install torch transformers onnx onnxruntime")
    sys.exit(1)


def export_encoder(model, tokenizer, output_dir: Path, model_name: str):
    """导出 encoder 模型"""
    print("\n=== Exporting Encoder ===")
    
    encoder = model.get_encoder()
    encoder.eval()
    
    # Encoder 的输入：input_ids 和 attention_mask
    encoder_output_path = output_dir / "encoder_model.onnx"
    
    # 创建示例输入
    dummy_input_ids = torch.randint(0, tokenizer.vocab_size, (1, 10), dtype=torch.long)
    dummy_attention_mask = torch.ones((1, 10), dtype=torch.long)
    
    print(f"Encoder input shape: {dummy_input_ids.shape}")
    
    # 导出 encoder
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
        opset_version=14,
        do_constant_folding=True,
    )
    
    print(f"✓ Encoder exported to: {encoder_output_path}")
    return encoder_output_path


def export_decoder_with_past(model, tokenizer, output_dir: Path, model_name: str):
    """导出支持增量解码的 decoder 模型"""
    print("\n=== Exporting Decoder with Past Key Values ===")
    
    decoder = model.get_decoder()
    decoder.eval()
    
    decoder_output_path = output_dir / "decoder_model.onnx"
    
    # Decoder 的输入（支持增量解码）：
    # - input_ids: decoder 输入 token IDs
    # - encoder_hidden_states: encoder 的输出
    # - encoder_attention_mask: encoder 的 attention mask
    # - past_key_values: 之前的 KV cache（可选）
    
    # 创建示例输入
    batch_size = 1
    encoder_seq_len = 10
    decoder_seq_len = 1
    d_model = model.config.d_model
    num_heads = model.config.decoder_attention_heads
    head_dim = d_model // num_heads
    num_layers = model.config.decoder_layers
    
    dummy_decoder_input_ids = torch.randint(0, tokenizer.vocab_size, (batch_size, decoder_seq_len), dtype=torch.long)
    dummy_encoder_hidden_states = torch.randn(batch_size, encoder_seq_len, d_model)
    dummy_encoder_attention_mask = torch.ones(batch_size, encoder_seq_len, dtype=torch.long)
    
    # 创建空的 past_key_values（注意：ORT 不支持维度为 0，所以使用 1）
    past_decoder_seq_len = 1  # 使用 1 而不是 0
    past_key_values = []
    for _ in range(num_layers):
        past_key_values.append((
            torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder key
            torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder value
            torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),  # encoder key
            torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),  # encoder value
        ))
    
    # 包装 decoder 以便导出
    class DecoderWrapper(torch.nn.Module):
        def __init__(self, decoder):
            super().__init__()
            self.decoder = decoder
        
        def forward(self, input_ids, encoder_hidden_states, encoder_attention_mask, *past_key_values_tuple):
            # 将 past_key_values 转换为正确的格式
            num_layers = len(past_key_values_tuple) // 4
            past_kv = []
            for i in range(num_layers):
                idx = i * 4
                # 如果 past_decoder_seq_len 为 1 且都是 0，表示没有 past，传入 None
                dec_key = past_key_values_tuple[idx]
                if dec_key.shape[2] == 1 and dec_key.sum() == 0:
                    # 第一次调用，没有 past
                    past_kv.append(None)
                else:
                    past_kv.append((
                        past_key_values_tuple[idx],      # decoder key
                        past_key_values_tuple[idx+1],    # decoder value
                        past_key_values_tuple[idx+2],    # encoder key
                        past_key_values_tuple[idx+3],    # encoder value
                    ))
            
            # 如果所有都是 None，传入 None
            if all(kv is None for kv in past_kv):
                past_kv = None
            else:
                # 需要处理混合情况，这里简化处理
                past_kv = [kv if kv is not None else (torch.zeros_like(past_key_values_tuple[0]), 
                                                      torch.zeros_like(past_key_values_tuple[1]),
                                                      past_key_values_tuple[2],
                                                      past_key_values_tuple[3]) for kv in past_kv]
            
            outputs = self.decoder(
                input_ids=input_ids,
                encoder_hidden_states=encoder_hidden_states,
                encoder_attention_mask=encoder_attention_mask,
                past_key_values=past_kv,
                use_cache=True,
            )
            return outputs
    
    decoder_wrapper = DecoderWrapper(decoder)
    
    # 准备所有输入
    inputs = [dummy_decoder_input_ids, dummy_encoder_hidden_states, dummy_encoder_attention_mask]
    for layer_kv in past_key_values:
        inputs.extend(layer_kv)
    
    # 输入名称
    input_names = ["input_ids", "encoder_hidden_states", "encoder_attention_mask"]
    for i in range(num_layers):
        input_names.extend([
            f"past_key_values.{i}.decoder.key",
            f"past_key_values.{i}.decoder.value",
            f"past_key_values.{i}.encoder.key",
            f"past_key_values.{i}.encoder.value",
        ])
    
    # 输出名称
    output_names = ["logits"]
    for i in range(num_layers):
        output_names.extend([
            f"present.{i}.decoder.key",
            f"present.{i}.decoder.value",
            f"present.{i}.encoder.key",
            f"present.{i}.encoder.value",
        ])
    
    # 动态轴
    dynamic_axes = {
        "input_ids": {0: "batch_size", 1: "decoder_sequence_length"},
        "encoder_hidden_states": {0: "batch_size", 1: "encoder_sequence_length"},
        "encoder_attention_mask": {0: "batch_size", 1: "encoder_sequence_length"},
        "logits": {0: "batch_size", 1: "decoder_sequence_length"},
    }
    
    for i in range(num_layers):
        dynamic_axes[f"past_key_values.{i}.decoder.key"] = {0: "batch_size", 2: "past_decoder_sequence_length"}
        dynamic_axes[f"past_key_values.{i}.decoder.value"] = {0: "batch_size", 2: "past_decoder_sequence_length"}
        dynamic_axes[f"present.{i}.decoder.key"] = {0: "batch_size", 2: "past_decoder_sequence_length + 1"}
        dynamic_axes[f"present.{i}.decoder.value"] = {0: "batch_size", 2: "past_decoder_sequence_length + 1"}
    
    # 添加 use_cache_branch 输入（如果需要）
    # 注意：根据你的模型，可能需要这个输入
    try:
        # 尝试添加 use_cache_branch
        dummy_use_cache = torch.tensor([True], dtype=torch.bool)
        inputs_with_cache = inputs + [dummy_use_cache]
        input_names_with_cache = input_names + ["use_cache_branch"]
        
        torch.onnx.export(
            decoder_wrapper,
            tuple(inputs),
            str(decoder_output_path),
            input_names=input_names,
            output_names=output_names,
            dynamic_axes=dynamic_axes,
            opset_version=14,
            do_constant_folding=True,
        )
        print(f"✓ Decoder exported to: {decoder_output_path}")
    except Exception as e:
        print(f"✗ Failed to export decoder: {e}")
        print(f"Error details: {str(e)}")
        import traceback
        traceback.print_exc()
        print("\nTrying alternative export method...")
        # 如果上面的方法失败，尝试使用 transformers 的导出工具
        export_decoder_alternative(model, tokenizer, output_dir)
    
    return decoder_output_path


def export_decoder_alternative(model, tokenizer, output_dir: Path):
    """使用 transformers 的导出工具导出 decoder（备用方法）"""
    print("\n=== Trying alternative decoder export method ===")
    
    # 这个方法可能不够灵活，但作为备选
    try:
        from transformers.onnx import export, FeaturesManager
        
        # 获取模型的特征
        model_kind, model_onnx_config = FeaturesManager.check_supported_model_or_raise(model)
        onnx_config = model_onnx_config(model.config)
        
        # 导出完整模型（包含 encoder 和 decoder）
        # 注意：这个方法可能导出完整的模型，而不是分离的 decoder
        print("Note: This method may export the full model, not just decoder")
        
    except Exception as e:
        print(f"Alternative method also failed: {e}")


def export_full_model(model, tokenizer, output_dir: Path):
    """导出完整的 encoder-decoder 模型（用于验证）"""
    print("\n=== Exporting Full Model (for verification) ===")
    
    full_model_path = output_dir / "model_full.onnx"
    
    # 创建示例输入
    dummy_input_ids = torch.randint(0, tokenizer.vocab_size, (1, 10), dtype=torch.long)
    
    try:
        torch.onnx.export(
            model,
            dummy_input_ids,
            str(full_model_path),
            input_names=["input_ids"],
            output_names=["logits"],
            dynamic_axes={
                "input_ids": {0: "batch_size", 1: "sequence_length"},
                "logits": {0: "batch_size", 1: "sequence_length"},
            },
            opset_version=14,
            do_constant_folding=True,
        )
        print(f"✓ Full model exported to: {full_model_path}")
    except Exception as e:
        print(f"✗ Failed to export full model: {e}")


def main():
    parser = argparse.ArgumentParser(description="Export Marian NMT model to ONNX format")
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
        help="Output directory for ONNX models",
    )
    parser.add_argument(
        "--quantize",
        action="store_true",
        help="Quantize models to int8 (optional)",
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
    if args.model_path:
        print(f"Loading from local path: {args.model_path}")
        model = MarianMTModel.from_pretrained(args.model_path)
        tokenizer = MarianTokenizer.from_pretrained(args.model_path)
    else:
        print(f"Loading from HuggingFace: {args.model_name}")
        model = MarianMTModel.from_pretrained(args.model_name)
        tokenizer = MarianTokenizer.from_pretrained(args.model_name)
    
    print(f"Model config: {model.config}")
    print(f"Vocab size: {tokenizer.vocab_size}")
    
    # 导出 encoder
    encoder_path = export_encoder(model, tokenizer, output_dir, args.model_name or args.model_path)
    
    # 导出 decoder
    decoder_path = export_decoder_with_past(model, tokenizer, output_dir, args.model_name or args.model_path)
    
    # 可选：导出完整模型用于验证
    # export_full_model(model, tokenizer, output_dir)
    
    # 可选：量化模型
    if args.quantize:
        print("\n=== Quantizing Models ===")
        try:
            quantize_dynamic(
                str(encoder_path),
                str(encoder_path).replace(".onnx", "_int8.onnx"),
                weight_type=QuantType.QUInt8,
            )
            print(f"✓ Encoder quantized")
            
            quantize_dynamic(
                str(decoder_path),
                str(decoder_path).replace(".onnx", "_int8.onnx"),
                weight_type=QuantType.QUInt8,
            )
            print(f"✓ Decoder quantized")
        except Exception as e:
            print(f"✗ Quantization failed: {e}")
    
    print("\n=== Export Complete ===")
    print(f"Encoder model: {encoder_path}")
    print(f"Decoder model: {decoder_path}")
    print("\nNext steps:")
    print("1. Verify the exported models work correctly")
    print("2. Update your code to use encoder_model.onnx and decoder_model.onnx")
    print("3. Test the translation pipeline")


if __name__ == "__main__":
    main()

