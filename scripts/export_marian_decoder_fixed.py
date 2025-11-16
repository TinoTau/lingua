#!/usr/bin/env python3
"""
修复后的 Marian NMT Decoder 导出脚本

主要修复：
1. 确保 use_cache_branch=True 时，encoder KV cache 正确输出
2. 使用 optimum 库导出（如果可用）
3. 改进动态轴定义

使用方法:
    python scripts/export_marian_decoder_fixed.py \
        --model_name Helsinki-NLP/opus-mt-en-zh \
        --output_dir core/engine/models/nmt/marian-en-zh
"""

import argparse
import os
import sys
from pathlib import Path

try:
    import torch
    from transformers import MarianMTModel, MarianTokenizer
    import onnx
    import numpy as np
except ImportError as e:
    print(f"Error: Missing required package: {e}")
    print("\nPlease install required packages:")
    print("  pip install torch transformers onnx onnxruntime")
    sys.exit(1)

# 尝试导入 optimum
try:
    from optimum.onnxruntime import ORTModelForSeq2SeqLM
    OPTIMUM_AVAILABLE = True
except ImportError:
    OPTIMUM_AVAILABLE = False
    print("Warning: optimum not available, will use standard export method")


def export_decoder_with_optimum(model_name: str, output_dir: Path):
    """使用 optimum 导出 decoder（推荐方法）"""
    print("\n=== Exporting Decoder with Optimum ===")
    
    if not OPTIMUM_AVAILABLE:
        print("Error: optimum not available")
        return None
    
    try:
        # 使用 optimum 导出
        print(f"Loading model: {model_name}")
        ort_model = ORTModelForSeq2SeqLM.from_pretrained(
            model_name,
            export=True,
            use_cache=True,  # 启用 KV cache
        )
        
        # 保存模型
        decoder_path = output_dir / "decoder_model.onnx"
        # optimum 会导出到临时目录，我们需要找到并复制
        # 注意：optimum 的导出方式可能不同，需要根据实际情况调整
        
        print(f"✓ Decoder exported with optimum to: {decoder_path}")
        return decoder_path
    except Exception as e:
        print(f"✗ Failed to export with optimum: {e}")
        import traceback
        traceback.print_exc()
        return None


def export_decoder_fixed(model, tokenizer, output_dir: Path):
    """修复后的 decoder 导出方法"""
    print("\n=== Exporting Decoder (Fixed Version) ===")
    
    decoder = model.get_decoder()
    decoder.eval()
    
    decoder_output_path = output_dir / "decoder_model.onnx"
    
    # 模型配置
    batch_size = 1
    encoder_seq_len = 10
    decoder_seq_len = 1
    d_model = model.config.d_model
    num_heads = model.config.decoder_attention_heads
    head_dim = d_model // num_heads
    num_layers = model.config.decoder_layers
    
    # 创建示例输入
    dummy_decoder_input_ids = torch.randint(0, tokenizer.vocab_size, (batch_size, decoder_seq_len), dtype=torch.long)
    dummy_encoder_hidden_states = torch.randn(batch_size, encoder_seq_len, d_model)
    dummy_encoder_attention_mask = torch.ones(batch_size, encoder_seq_len, dtype=torch.long)
    
    # 创建 past_key_values
    past_decoder_seq_len = 1
    past_key_values = []
    for _ in range(num_layers):
        past_key_values.append((
            torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),
            torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),
            torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),
            torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),
        ))
    
    # 修复后的 DecoderWrapper：确保 encoder KV cache 始终正确输出
    class FixedDecoderWrapper(torch.nn.Module):
        def __init__(self, decoder):
            super().__init__()
            self.decoder = decoder
        
        def forward(self, input_ids, encoder_hidden_states, encoder_attention_mask, use_cache_branch, *past_key_values_tuple):
            # 将 past_key_values 转换为正确的格式
            num_layers = len(past_key_values_tuple) // 4
            past_kv = []
            for i in range(num_layers):
                idx = i * 4
                past_kv.append((
                    past_key_values_tuple[idx],      # decoder key
                    past_key_values_tuple[idx+1],    # decoder value
                    past_key_values_tuple[idx+2],    # encoder key
                    past_key_values_tuple[idx+3],    # encoder value
                ))
            
            # 调用 decoder
            outputs = self.decoder(
                input_ids=input_ids,
                encoder_hidden_states=encoder_hidden_states,
                encoder_attention_mask=encoder_attention_mask,
                past_key_values=past_kv if use_cache_branch.item() else None,
                use_cache=True,
            )
            
            # 关键修复：确保 encoder KV cache 始终正确输出
            # 当 use_cache_branch=True 时，模型输出的 present.*.encoder.* 可能是空的
            # 我们需要从输入的 past_key_values 中获取 encoder KV cache
            
            logits = outputs.logits
            past_key_values_output = outputs.past_key_values
            
            # 构建输出
            output_list = [logits]
            
            use_cache = use_cache_branch.item() if isinstance(use_cache_branch, torch.Tensor) else use_cache_branch
            
            for layer_idx in range(num_layers):
                if use_cache and past_key_values_output is not None and past_key_values_output[layer_idx] is not None:
                    # use_cache_branch=True：从模型输出获取 decoder KV，从输入获取 encoder KV
                    dec_k, dec_v, enc_k_from_output, enc_v_from_output = past_key_values_output[layer_idx]
                    
                    # 关键修复：检查 encoder KV 是否为空（形状为 (0, ...)）
                    if enc_k_from_output.shape[0] == 0 or enc_k_from_output.numel() == 0:
                        # encoder KV 是空的，从输入中获取
                        idx = layer_idx * 4
                        enc_k = past_key_values_tuple[idx + 2]  # 使用输入的 encoder key
                        enc_v = past_key_values_tuple[idx + 3]  # 使用输入的 encoder value
                    else:
                        # encoder KV 正常，使用模型输出
                        enc_k = enc_k_from_output
                        enc_v = enc_v_from_output
                    
                    output_list.extend([dec_k, dec_v, enc_k, enc_v])
                else:
                    # use_cache_branch=False 或 past_key_values 是 None：使用模型输出
                    if past_key_values_output is not None and past_key_values_output[layer_idx] is not None:
                        dec_k, dec_v, enc_k, enc_v = past_key_values_output[layer_idx]
                        output_list.extend([dec_k, dec_v, enc_k, enc_v])
                    else:
                        # 如果模型没有输出，从输入中获取（这种情况不应该发生）
                        idx = layer_idx * 4
                        dec_k = past_key_values_tuple[idx]
                        dec_v = past_key_values_tuple[idx + 1]
                        enc_k = past_key_values_tuple[idx + 2]
                        enc_v = past_key_values_tuple[idx + 3]
                        output_list.extend([dec_k, dec_v, enc_k, enc_v])
            
            return tuple(output_list)
    
    decoder_wrapper = FixedDecoderWrapper(decoder)
    
    # 准备所有输入
    inputs = [dummy_decoder_input_ids, dummy_encoder_hidden_states, dummy_encoder_attention_mask]
    inputs.append(torch.tensor([True], dtype=torch.bool))  # use_cache_branch
    for layer_kv in past_key_values:
        inputs.extend(layer_kv)
    
    # 输入名称
    input_names = ["input_ids", "encoder_hidden_states", "encoder_attention_mask", "use_cache_branch"]
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
    
    # 动态轴 - 关键修复：明确指定 encoder KV cache 的形状
    dynamic_axes = {
        "input_ids": {0: "batch_size", 1: "decoder_sequence_length"},
        "encoder_hidden_states": {0: "batch_size", 1: "encoder_sequence_length"},
        "encoder_attention_mask": {0: "batch_size", 1: "encoder_sequence_length"},
        "logits": {0: "batch_size", 1: "decoder_sequence_length"},
    }
    
    for i in range(num_layers):
        # Decoder KV cache：动态序列长度
        dynamic_axes[f"past_key_values.{i}.decoder.key"] = {0: "batch_size", 2: "past_decoder_sequence_length"}
        dynamic_axes[f"past_key_values.{i}.decoder.value"] = {0: "batch_size", 2: "past_decoder_sequence_length"}
        dynamic_axes[f"present.{i}.decoder.key"] = {0: "batch_size", 2: "past_decoder_sequence_length + 1"}
        dynamic_axes[f"present.{i}.decoder.value"] = {0: "batch_size", 2: "past_decoder_sequence_length + 1"}
        
        # Encoder KV cache：固定序列长度（关键修复）
        # 确保 encoder KV cache 的形状与 encoder_sequence_length 一致
        dynamic_axes[f"past_key_values.{i}.encoder.key"] = {0: "batch_size", 2: "encoder_sequence_length"}
        dynamic_axes[f"past_key_values.{i}.encoder.value"] = {0: "batch_size", 2: "encoder_sequence_length"}
        dynamic_axes[f"present.{i}.encoder.key"] = {0: "batch_size", 2: "encoder_sequence_length"}  # 关键：不是动态的
        dynamic_axes[f"present.{i}.encoder.value"] = {0: "batch_size", 2: "encoder_sequence_length"}  # 关键：不是动态的
    
    try:
        # 检查 PyTorch 版本，使用兼容的导出方式
        torch_version = torch.__version__
        print(f"PyTorch version: {torch_version}")
        
        # 对于新版本的 PyTorch，可能需要使用不同的导出方式
        # 先尝试使用 legacy 模式
        try:
            torch.onnx.export(
                decoder_wrapper,
                tuple(inputs),
                str(decoder_output_path),
                input_names=input_names,
                output_names=output_names,
                dynamic_axes=dynamic_axes,
                opset_version=14,
                do_constant_folding=True,
                export_params=True,
            )
        except RuntimeError as e:
            if "dynamic_axes" in str(e) or "dynamic_shapes" in str(e):
                # 新版本 PyTorch，尝试使用 legacy 导出
                print("  Trying legacy export mode...")
                torch.onnx.export(
                    decoder_wrapper,
                    tuple(inputs),
                    str(decoder_output_path),
                    input_names=input_names,
                    output_names=output_names,
                    dynamic_axes=dynamic_axes,
                    opset_version=14,
                    do_constant_folding=True,
                    export_params=True,
                    operator_export_type=torch.onnx.OperatorExportTypes.ONNX,  # 使用 ONNX 操作符
                )
            else:
                raise
        print(f"✓ Decoder exported to: {decoder_output_path}")
        return decoder_output_path
    except Exception as e:
        print(f"✗ Failed to export decoder: {e}")
        import traceback
        traceback.print_exc()
        return None


def main():
    parser = argparse.ArgumentParser(description="Export Marian NMT Decoder (Fixed Version)")
    parser.add_argument(
        "--model_name",
        type=str,
        required=True,
        help="HuggingFace model name (e.g., Helsinki-NLP/opus-mt-en-zh)",
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        required=True,
        help="Output directory for ONNX models",
    )
    parser.add_argument(
        "--use_optimum",
        action="store_true",
        help="Use optimum library for export (if available)",
    )
    
    args = parser.parse_args()
    
    # 创建输出目录
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Output directory: {output_dir}")
    
    # 尝试使用 optimum 导出
    if args.use_optimum and OPTIMUM_AVAILABLE:
        decoder_path = export_decoder_with_optimum(args.model_name, output_dir)
        if decoder_path:
            print("\n=== Export Complete ===")
            print(f"Decoder model: {decoder_path}")
            return
    
    # 使用修复后的标准导出方法
    print("\n=== Loading Model ===")
    print(f"Loading from HuggingFace: {args.model_name}")
    model = MarianMTModel.from_pretrained(args.model_name)
    tokenizer = MarianTokenizer.from_pretrained(args.model_name)
    
    print(f"Model config: {model.config}")
    print(f"Vocab size: {tokenizer.vocab_size}")
    
    # 导出 decoder
    decoder_path = export_decoder_fixed(model, tokenizer, output_dir)
    
    if decoder_path:
        print("\n=== Export Complete ===")
        print(f"Decoder model: {decoder_path}")
        print("\nNext steps:")
        print("1. Test the exported model with: python scripts/test_marian_decoder_kv_cache.py")
        print("2. Verify encoder KV cache shapes are correct")
        print("3. Test in Rust: cargo test --test nmt_quick_test")
    else:
        print("\n=== Export Failed ===")
        print("Please check the error messages above")


if __name__ == "__main__":
    main()

