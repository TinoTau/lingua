"""
M2M100 HF vs ONNX 非增量逐步对齐测试

目标：确认 ONNX decoder 行为是否和 HF 深度一致

执行方式：
1. 在 Python 中强制非增量解码（每次把完整目标序列传入 HF，不使用 past）
2. 在 ONNX 中执行同样的非增量步骤
3. 比对每步的 logits top-5

判定指标：
- 如果 HF 与 ONNX 前 3 步 logits 即开始偏离 → 表示 ONNX decoder 输入/分支选择仍然不正确
- 如果前 3~5 步一致，而后面开始偏离 → 表示循环、EOS、decoder_start_token 或 KV 分支逻辑未对齐
"""

import torch
import numpy as np
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
import onnxruntime as ort
from pathlib import Path

def test_hf_non_incremental():
    """测试 HF 模型的非增量解码"""
    print("=" * 80)
    print("测试 HF 模型的非增量解码")
    print("=" * 80)
    
    # 加载模型和 tokenizer
    model_name = "facebook/m2m100_418M"
    tokenizer = M2M100Tokenizer.from_pretrained(model_name)
    model = M2M100ForConditionalGeneration.from_pretrained(model_name)
    model.eval()
    
    # 测试句子
    source_text = "你好，欢迎参加测试。"
    print(f"\n源文本: {source_text}")
    
    # 编码源文本
    tokenizer.src_lang = "zh"
    encoded = tokenizer(source_text, return_tensors="pt")
    input_ids = encoded["input_ids"]
    attention_mask = encoded["attention_mask"]
    
    print(f"源文本 token IDs: {input_ids[0].tolist()}")
    print(f"源文本长度: {input_ids.shape[1]}")
    
    # 运行 encoder
    with torch.no_grad():
        encoder_outputs = model.model.encoder(
            input_ids=input_ids,
            attention_mask=attention_mask
        )
    
    encoder_hidden_states = encoder_outputs.last_hidden_state
    print(f"Encoder 输出形状: {encoder_hidden_states.shape}")
    
    # 获取目标语言 token
    tokenizer.tgt_lang = "en"
    tgt_lang_id = tokenizer.get_lang_id("en")
    eos_token_id = tokenizer.eos_token_id
    print(f"目标语言 token ID: {tgt_lang_id}")
    print(f"EOS token ID: {eos_token_id}")
    
    # 非增量解码循环
    generated_ids = [tgt_lang_id]
    max_steps = 32
    hf_logits_history = []
    
    print("\n" + "=" * 80)
    print("HF 非增量解码步骤（每步传入完整序列）")
    print("=" * 80)
    
    for step in range(max_steps):
        # 构造当前完整序列
        current_ids = torch.tensor([generated_ids], dtype=torch.long)
        
        # 运行 decoder（非增量模式：不使用 past_key_values）
        with torch.no_grad():
            decoder_outputs = model.model.decoder(
                input_ids=current_ids,
                encoder_hidden_states=encoder_hidden_states,
                encoder_attention_mask=attention_mask,
                use_cache=False,  # 关键：不使用 KV cache
            )
        
        # 获取 logits
        logits = model.lm_head(decoder_outputs.last_hidden_state)
        # 取最后一个位置的 logits
        last_logits = logits[0, -1, :].detach().cpu().numpy()
        
        # 记录 top-5 logits
        top5_indices = np.argsort(last_logits)[-5:][::-1]
        top5_values = last_logits[top5_indices]
        top5_pairs = list(zip(top5_indices.tolist(), top5_values.tolist()))
        
        hf_logits_history.append({
            'step': step,
            'generated_ids': generated_ids.copy(),
            'logits': last_logits,
            'top5': top5_pairs,
        })
        
        # 选择下一个 token（贪婪解码）
        next_token_id = int(np.argmax(last_logits))
        
        print(f"\n[Step {step}]")
        print(f"  Generated IDs: {generated_ids}")
        print(f"  Top 5 logits: {top5_pairs}")
        print(f"  Next token ID: {next_token_id}")
        
        # 检查 EOS
        if next_token_id == eos_token_id:
            print(f"  ✅ Generated EOS token, stopping")
            break
        
        generated_ids.append(next_token_id)
        
        # 检查重复模式
        if len(generated_ids) >= 4:
            last_four = generated_ids[-4:]
            if last_four[0] == last_four[2] and last_four[1] == last_four[3]:
                print(f"  [WARNING] Detected 2-token repetition pattern: {last_four}")
                break
    
    return hf_logits_history, generated_ids


def test_onnx_non_incremental():
    """测试 ONNX 模型的非增量解码"""
    print("\n" + "=" * 80)
    print("测试 ONNX 模型的非增量解码")
    print("=" * 80)
    
    # 加载 tokenizer（用于编码）
    model_name = "facebook/m2m100_418M"
    tokenizer = M2M100Tokenizer.from_pretrained(model_name)
    
    # 加载 ONNX 模型
    model_dir = Path("core/engine/models/nmt/m2m100-en-zh")
    if not model_dir.exists():
        # 尝试从项目根目录查找
        model_dir = Path(__file__).parent.parent.parent / "core" / "engine" / "models" / "nmt" / "m2m100-en-zh"
    
    if not model_dir.exists():
        print(f"[ERROR] 模型目录不存在: {model_dir}")
        return None, None
    
    encoder_path = model_dir / "encoder.onnx"
    decoder_path = model_dir / "decoder.onnx"
    
    if not encoder_path.exists() or not decoder_path.exists():
        print(f"[ERROR] ONNX 模型文件不存在")
        print(f"  Encoder: {encoder_path.exists()}")
        print(f"  Decoder: {decoder_path.exists()}")
        return None, None
    
    # 创建 ONNX Runtime session
    encoder_session = ort.InferenceSession(str(encoder_path))
    decoder_session = ort.InferenceSession(str(decoder_path))
    
    print(f"[OK] Encoder session created")
    print(f"[OK] Decoder session created")
    
    # 打印 decoder 输入信息
    print("\nDecoder 输入信息:")
    decoder_inputs = decoder_session.get_inputs()
    input_names = [inp.name for inp in decoder_inputs]  # 保存输入名称列表，用于后续构造输入字典
    for i, input_info in enumerate(decoder_inputs):
        print(f"  Input[{i}]: {input_info.name}, shape: {input_info.shape}, type: {input_info.type}")
    
    # 检查是否有 use_cache_branch 输入
    has_use_cache = any("use_cache" in inp.name.lower() or "flag" in inp.name.lower() 
                        for inp in decoder_inputs)
    use_new_format = not has_use_cache
    print(f"\n是否有 use_cache_branch 输入: {has_use_cache}")
    print(f"使用新格式（无 use_cache_branch）: {use_new_format}")
    
    # 测试句子
    source_text = "你好，欢迎参加测试。"
    print(f"\n源文本: {source_text}")
    
    # 编码源文本
    tokenizer.src_lang = "zh"
    encoded = tokenizer(source_text, return_tensors="pt")
    input_ids = encoded["input_ids"].numpy()
    attention_mask = encoded["attention_mask"].numpy()
    
    print(f"源文本 token IDs: {input_ids[0].tolist()}")
    print(f"源文本长度: {input_ids.shape[1]}")
    
    # 运行 encoder
    encoder_inputs = {
        "input_ids": input_ids.astype(np.int64),
        "attention_mask": attention_mask.astype(np.int64),
    }
    encoder_outputs = encoder_session.run(None, encoder_inputs)
    encoder_hidden_states = encoder_outputs[0]  # [batch, seq_len, hidden]
    print(f"Encoder 输出形状: {encoder_hidden_states.shape}")
    
    # 获取目标语言 token
    tokenizer.tgt_lang = "en"
    tgt_lang_id = tokenizer.get_lang_id("en")
    eos_token_id = tokenizer.eos_token_id
    print(f"目标语言 token ID: {tgt_lang_id}")
    print(f"EOS token ID: {eos_token_id}")
    
    # 非增量解码循环
    generated_ids = [tgt_lang_id]
    max_steps = 32
    onnx_logits_history = []
    
    # 模型常量
    NUM_LAYERS = 12
    NUM_HEADS = 16
    HEAD_DIM = 64
    encoder_seq_len = encoder_hidden_states.shape[1]
    
    print("\n" + "=" * 80)
    print("ONNX 非增量解码步骤（每步传入完整序列 + 全零 KV）")
    print("=" * 80)
    
    for step in range(max_steps):
        # 构造当前完整序列
        current_ids = np.array([generated_ids], dtype=np.int64)
        current_seq_len = len(generated_ids)
        
        # 构造全零 KV cache
        # Decoder KV: [1, 16, tgt_seq_len, 64]
        decoder_kv = []
        for _ in range(NUM_LAYERS):
            dec_k = np.zeros((1, NUM_HEADS, current_seq_len, HEAD_DIM), dtype=np.float32)
            dec_v = np.zeros((1, NUM_HEADS, current_seq_len, HEAD_DIM), dtype=np.float32)
            decoder_kv.extend([dec_k, dec_v])
        
        # Encoder KV: [1, 16, encoder_seq_len, 64]
        encoder_kv = []
        for _ in range(NUM_LAYERS):
            enc_k = np.zeros((1, NUM_HEADS, encoder_seq_len, HEAD_DIM), dtype=np.float32)
            enc_v = np.zeros((1, NUM_HEADS, encoder_seq_len, HEAD_DIM), dtype=np.float32)
            encoder_kv.extend([enc_k, enc_v])
        
        # 构造 decoder 输入字典（按照输入名称）
        decoder_inputs_dict = {}
        
        # 按照输入名称顺序添加
        idx = 0
        decoder_inputs_dict[input_names[idx]] = attention_mask.astype(np.int64)  # encoder_attention_mask
        idx += 1
        decoder_inputs_dict[input_names[idx]] = current_ids  # input_ids
        idx += 1
        decoder_inputs_dict[input_names[idx]] = encoder_hidden_states.astype(np.float32)  # encoder_hidden_states
        idx += 1
        
        # 添加 decoder KV (12 层 × 2 = 24 个)
        for layer_idx in range(NUM_LAYERS):
            decoder_inputs_dict[input_names[idx]] = decoder_kv[layer_idx * 2]  # decoder.key
            idx += 1
            decoder_inputs_dict[input_names[idx]] = decoder_kv[layer_idx * 2 + 1]  # decoder.value
            idx += 1
        
        # 添加 encoder KV (12 层 × 2 = 24 个)
        for layer_idx in range(NUM_LAYERS):
            decoder_inputs_dict[input_names[idx]] = encoder_kv[layer_idx * 2]  # encoder.key
            idx += 1
            decoder_inputs_dict[input_names[idx]] = encoder_kv[layer_idx * 2 + 1]  # encoder.value
            idx += 1
        
        # 如果是旧格式，添加 use_cache_branch（设为 False）
        if not use_new_format:
            use_cache_branch = np.array([False], dtype=bool)
            decoder_inputs_dict[input_names[idx]] = use_cache_branch
        
        # 运行 decoder
        decoder_outputs = decoder_session.run(None, decoder_inputs_dict)
        
        # 第一个输出是 logits: [batch, seq_len, vocab_size]
        logits = decoder_outputs[0]
        # 取最后一个位置的 logits
        last_logits = logits[0, -1, :]
        
        # 记录 top-5 logits
        top5_indices = np.argsort(last_logits)[-5:][::-1]
        top5_values = last_logits[top5_indices]
        top5_pairs = list(zip(top5_indices.tolist(), top5_values.tolist()))
        
        onnx_logits_history.append({
            'step': step,
            'generated_ids': generated_ids.copy(),
            'logits': last_logits,
            'top5': top5_pairs,
        })
        
        # 选择下一个 token（贪婪解码）
        next_token_id = int(np.argmax(last_logits))
        
        print(f"\n[Step {step}]")
        print(f"  Generated IDs: {generated_ids}")
        print(f"  Top 5 logits: {top5_pairs}")
        print(f"  Next token ID: {next_token_id}")
        
        # 检查 EOS
        if next_token_id == eos_token_id:
            print(f"  ✅ Generated EOS token, stopping")
            break
        
        generated_ids.append(next_token_id)
        
        # 检查重复模式
        if len(generated_ids) >= 4:
            last_four = generated_ids[-4:]
            if last_four[0] == last_four[2] and last_four[1] == last_four[3]:
                print(f"  [WARNING] Detected 2-token repetition pattern: {last_four}")
                break
    
    return onnx_logits_history, generated_ids


def compare_logits(hf_logits_history, onnx_logits_history):
    """比对 HF 和 ONNX 的 logits"""
    print("\n" + "=" * 80)
    print("比对 HF 和 ONNX 的 logits")
    print("=" * 80)
    
    if hf_logits_history is None or onnx_logits_history is None:
        print("[ERROR] 无法比对：缺少 logits 历史")
        return
    
    min_steps = min(len(hf_logits_history), len(onnx_logits_history))
    
    print(f"\n比对前 {min_steps} 步的 logits top-5")
    print("-" * 80)
    
    for step in range(min_steps):
        hf_data = hf_logits_history[step]
        onnx_data = onnx_logits_history[step]
        
        print(f"\n[Step {step}]")
        print(f"  HF   Top 5: {hf_data['top5']}")
        print(f"  ONNX Top 5: {onnx_data['top5']}")
        
        # 检查 top-1 是否一致
        hf_top1 = hf_data['top5'][0][0]
        onnx_top1 = onnx_data['top5'][0][0]
        
        if hf_top1 == onnx_top1:
            print(f"  [OK] Top-1 token 一致: {hf_top1}")
        else:
            print(f"  [ERROR] Top-1 token 不一致: HF={hf_top1}, ONNX={onnx_top1}")
        
        # 检查 top-5 的重叠度
        hf_top5_ids = {pair[0] for pair in hf_data['top5']}
        onnx_top5_ids = {pair[0] for pair in onnx_data['top5']}
        overlap = len(hf_top5_ids & onnx_top5_ids)
        print(f"  Top-5 重叠度: {overlap}/5")
        
        # 如果前 3 步就开始偏离，标记为严重问题
        if step < 3 and hf_top1 != onnx_top1:
            print(f"  [WARNING] 前 3 步即开始偏离，可能表示 ONNX decoder 输入/分支选择不正确")


if __name__ == "__main__":
    print("M2M100 HF vs ONNX 非增量逐步对齐测试")
    print("=" * 80)
    
    # 测试 HF
    hf_logits_history, hf_generated_ids = test_hf_non_incremental()
    
    # 测试 ONNX（需要补充完整实现）
    onnx_logits_history, onnx_generated_ids = test_onnx_non_incremental()
    
    # 比对
    if hf_logits_history and onnx_logits_history:
        compare_logits(hf_logits_history, onnx_logits_history)
    else:
        print("\n[WARNING] ONNX 测试未完成，需要补充完整实现")
        print("当前只完成了 HF 部分的测试")

