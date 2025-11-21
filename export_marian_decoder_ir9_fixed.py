#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Export Marian zh→en decoder (+LM head + KV cache) to ONNX with IR<=9 and opset 12.

⚠ 说明：
- 这是一个“修复版”导出脚本，目标是接近现有 `export_marian_onnx.py` 中的
  `export_decoder_with_past` 行为：
  - 支持 KV cache（past_key_values 输入 + present_key_values 输出）
  - 支持 use_cache_branch 输入
  - 输出 logits + 24 个 KV（以 6 层、每层 4 个 KV 为例）
- 具体输入 / 输出名称和顺序，仍建议对照现有 `marian-en-zh` 的导出脚本和
  `core/engine/src/nmt_incremental/marian_onnx.rs` 进行微调。

环境要求：
  - Python 3.10.x
  - torch==1.13.1+cpu  (或兼容 1.13.1 的 CPU 版本)
  - transformers==4.40.0
  - onnx==1.14.0
"""

import argparse
from pathlib import Path
from typing import List, Tuple

import torch
from torch import nn
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import onnx


class MarianDecoderWithPastWrapper(nn.Module):
    """
    包装 Marian Decoder + LM head，显式暴露 KV cache 输入/输出。

    逻辑约定：
    - 输入顺序（建议与现有 Rust loader 对齐后再使用）：
        0: encoder_attention_mask      (int64[batch, src_seq])
        1: decoder_input_ids          (int64[batch, tgt_seq])
        2: encoder_hidden_states      (float32[batch, src_seq, hidden])
        3..(3+4*L-1): past_*          (float32[batch, num_heads, past_seq, head_dim])
        最后: use_cache_branch        (int64[1] 或 bool[1])

      其中 L = num_layers，且每层有 4 个 KV：
        - self_attn_key
        - self_attn_value
        - cross_attn_key
        - cross_attn_value

    - 输出：
        - logits                       (float32[batch, tgt_seq, vocab])
        - present_* (与 past_* 数量相同，名称可按需要调整)
    """

    def __init__(self, model: AutoModelForSeq2SeqLM, num_kv_layers: int = None):
        super().__init__()
        # 对于 MarianMTModel，encoder/decoder 在 model.model 下
        self.model = model
        self.decoder = model.model.decoder
        self.lm_head = model.lm_head
        self.config = model.config

        # 推断层数
        if num_kv_layers is None:
            # Marian 通常为 config.encoder_layers == config.decoder_layers
            num_kv_layers = getattr(self.config, "decoder_layers", None) or getattr(
                self.config, "num_layers", 6
            )
        self.num_kv_layers = num_kv_layers

        # 注意：num_heads 和 d_model 用于构造 dummy past 形状
        self.num_heads = getattr(self.config, "decoder_attention_heads", None) or getattr(
            self.config, "num_attention_heads", 8
        )
        self.hidden_size = getattr(self.config, "d_model", getattr(self.config, "hidden_size", 512))
        self.head_dim = self.hidden_size // self.num_heads

    def _rebuild_past(
        self, past_flat: List[torch.Tensor]
    ) -> Tuple[Tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor], ...]:
        """将扁平 past 列表还原为 HF 期望的 past_key_values 结构。

        这里假设每层 4 个张量：
          (self_k, self_v, cross_k, cross_v)
        """
        assert len(past_flat) == self.num_kv_layers * 4, (
            f"Expected {self.num_kv_layers * 4} past tensors, got {len(past_flat)}"
        )
        past_key_values = []
        idx = 0
        for _ in range(self.num_kv_layers):
            self_k = past_flat[idx + 0]
            self_v = past_flat[idx + 1]
            cross_k = past_flat[idx + 2]
            cross_v = past_flat[idx + 3]
            past_key_values.append((self_k, self_v, cross_k, cross_v))
            idx += 4
        return tuple(past_key_values)

    def forward(
        self,
        encoder_attention_mask: torch.Tensor,
        input_ids: torch.Tensor,  # 注意：导出时使用 "input_ids" 名称，但实际是 decoder_input_ids
        encoder_hidden_states: torch.Tensor,
        *past_and_flag: torch.Tensor,
    ) -> Tuple[torch.Tensor, ...]:
        """
        Forward 输入说明：

        - encoder_attention_mask: [batch, src_seq]
        - input_ids: [batch, tgt_seq] - 注意：虽然参数名是 input_ids，但实际是 decoder_input_ids
        - encoder_hidden_states: [batch, src_seq, hidden]
        - *past_and_flag: 4 * L 个 past KV + 1 个 use_cache_branch
        """
        if len(past_and_flag) < self.num_kv_layers * 4 + 1:
            raise RuntimeError(
                f"Expected at least {self.num_kv_layers * 4 + 1} extra inputs (past + use_cache_branch), "
                f"got {len(past_and_flag)}"
            )

        past_flat = list(past_and_flag[:-1])
        use_cache_branch = past_and_flag[-1]

        # 将 flat past 还原成 HF 需要的结构
        past_key_values = self._rebuild_past(past_flat)

        # use_cache_branch 在导出时一般只作为控制分支使用，
        # 这里可以简单作为 boolean 标记是否启用 cache。为了 ONNX 简化，通常直接启用 cache。
        use_cache = True

        decoder_outputs = self.decoder(
            input_ids=input_ids,  # 使用参数名 input_ids（虽然实际是 decoder_input_ids）
            encoder_hidden_states=encoder_hidden_states,
            encoder_attention_mask=encoder_attention_mask,
            past_key_values=past_key_values,
            use_cache=use_cache,
        )

        hidden_states = decoder_outputs.last_hidden_state
        logits = self.lm_head(hidden_states)

        # present_key_values 结构与 past_key_values 相同：tuple(num_layers)[(self_k, self_v, cross_k, cross_v)]
        present = decoder_outputs.past_key_values

        flat_present: List[torch.Tensor] = []
        for layer_present in present:
            self_k, self_v, cross_k, cross_v = layer_present
            flat_present.extend([self_k, self_v, cross_k, cross_v])

        # 输出顺序：logits 在最前，后面 flatten 所有 present_*，方便在 ONNX 侧直接对应 25 个输出
        return (logits, *flat_present)


def build_io_names(num_kv_layers: int) -> Tuple[List[str], List[str]]:
    """根据层数构造输入 / 输出名称列表。

    使用与现有 `marian-en-zh` 模型相同的命名格式，确保兼容性。
    """
    input_names = [
        "encoder_attention_mask",   # 0
        "input_ids",                # 1 - 注意：代码期望 "input_ids"，不是 "decoder_input_ids"
        "encoder_hidden_states",    # 2
    ]

    # past_key_values: 使用与现有模型相同的命名格式
    # 格式：past_key_values.{layer}.decoder.key, past_key_values.{layer}.decoder.value,
    #       past_key_values.{layer}.encoder.key, past_key_values.{layer}.encoder.value
    for layer in range(num_kv_layers):
        input_names.extend([
            f"past_key_values.{layer}.decoder.key",
            f"past_key_values.{layer}.decoder.value",
            f"past_key_values.{layer}.encoder.key",
            f"past_key_values.{layer}.encoder.value",
        ])

    # use_cache_branch / use_cache flag
    input_names.append("use_cache_branch")  # 最后一个输入

    output_names = ["logits"]
    # present: 使用与现有模型相同的命名格式
    # 格式：present.{layer}.decoder.key, present.{layer}.decoder.value,
    #       present.{layer}.encoder.key, present.{layer}.encoder.value
    for layer in range(num_kv_layers):
        output_names.extend([
            f"present.{layer}.decoder.key",
            f"present.{layer}.decoder.value",
            f"present.{layer}.encoder.key",
            f"present.{layer}.encoder.value",
        ])

    return input_names, output_names


def export_marian_decoder_ir9_fixed(output_dir: Path, model_id: str) -> None:
    print(f"[INFO] Using model_id: {model_id}")
    print(f"[INFO] Output dir: {output_dir}")

    output_dir.mkdir(parents=True, exist_ok=True)
    onnx_path = output_dir / "model.onnx"

    # 1) 加载 tokenizer & 模型
    print("[1/5] Loading tokenizer and model ...")
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = AutoModelForSeq2SeqLM.from_pretrained(model_id)
    model.eval()

    # Marian 通常 decoder_layers == encoder_layers（例如 6）
    num_kv_layers = getattr(model.config, "decoder_layers", None) or getattr(
        model.config, "num_layers", 6
    )
    print(f"[INFO] Using num_kv_layers = {num_kv_layers}")

    wrapper = MarianDecoderWithPastWrapper(model, num_kv_layers=num_kv_layers)
    wrapper.eval()

    # 2) 准备 dummy inputs
    print("[2/5] Preparing dummy inputs ...")
    dummy_text = "你好，世界。"
    enc = tokenizer(dummy_text, return_tensors="pt")
    encoder_input_ids = enc["input_ids"]
    encoder_attention_mask = enc["attention_mask"]

    with torch.no_grad():
        encoder_hidden_states = model.get_encoder()(
            input_ids=encoder_input_ids,
            attention_mask=encoder_attention_mask,
        )[0]

    # decoder_input_ids：使用 BOS token 或 input_ids 的第一个 token
    if getattr(model.config, "decoder_start_token_id", None) is not None:
        bos_id = model.config.decoder_start_token_id
    else:
        bos_id = tokenizer.bos_token_id if tokenizer.bos_token_id is not None else int(
            encoder_input_ids[0, 0]
        )

    decoder_input_ids = torch.tensor([[bos_id]], dtype=torch.long)

    batch_size = decoder_input_ids.shape[0]
    src_seq = encoder_hidden_states.shape[1]
    tgt_seq = decoder_input_ids.shape[1]

    num_heads = wrapper.num_heads
    head_dim = wrapper.head_dim
    past_seq = 1  # dummy 过去序列长度，可以为 1

    print(f"    encoder_hidden_states.shape = {tuple(encoder_hidden_states.shape)}")
    print(f"    decoder_input_ids.shape     = {tuple(decoder_input_ids.shape)}")
    print(f"    encoder_attention_mask.shape= {tuple(encoder_attention_mask.shape)}")
    print(f"    num_heads = {num_heads}, head_dim = {head_dim}, past_seq = {past_seq}")

    # 构造 dummy past_key_values
    # 注意：decoder KV 使用 past_seq，encoder KV 使用 encoder_seq_len
    past_list: List[torch.Tensor] = []
    for _layer in range(num_kv_layers):
        # self_k (decoder key): [batch, num_heads, past_seq, head_dim]
        past_list.append(
            torch.zeros(
                (batch_size, num_heads, past_seq, head_dim),
                dtype=encoder_hidden_states.dtype,
            )
        )
        # self_v (decoder value): [batch, num_heads, past_seq, head_dim]
        past_list.append(
            torch.zeros(
                (batch_size, num_heads, past_seq, head_dim),
                dtype=encoder_hidden_states.dtype,
            )
        )
        # cross_k (encoder key): [batch, num_heads, encoder_seq_len, head_dim]
        past_list.append(
            torch.zeros(
                (batch_size, num_heads, src_seq, head_dim),  # 使用 src_seq (encoder_seq_len)
                dtype=encoder_hidden_states.dtype,
            )
        )
        # cross_v (encoder value): [batch, num_heads, encoder_seq_len, head_dim]
        past_list.append(
            torch.zeros(
                (batch_size, num_heads, src_seq, head_dim),  # 使用 src_seq (encoder_seq_len)
                dtype=encoder_hidden_states.dtype,
            )
        )

    # use_cache_branch：使用 bool 类型（与代码期望一致）
    use_cache_branch = torch.tensor([True], dtype=torch.bool)

    # 3) 构建输入/输出名称
    print("[3/5] Building input/output names ...")
    input_names, output_names = build_io_names(num_kv_layers)
    print(f"    #inputs = {len(input_names)}, #outputs = {len(output_names)}")


    # 4) 导出 ONNX（opset 12，IR<=9）
    print("[4/5] Exporting decoder ONNX (opset_version=12) ...")
    # 注意：虽然变量名是 decoder_input_ids，但导出时使用 "input_ids" 作为输入名称
    dummy_inputs = (
        encoder_attention_mask,
        decoder_input_ids,  # 传递给 forward 的 input_ids 参数
        encoder_hidden_states,
        *past_list,
        use_cache_branch,
    )

    dynamic_axes = {
        "encoder_attention_mask": {0: "batch", 1: "src_seq"},
        "decoder_input_ids": {0: "batch", 1: "tgt_seq"},
        "encoder_hidden_states": {0: "batch", 1: "src_seq"},
        # 所有 KV 的 batch / past_seq 设为动态
    }

    for name in input_names:
        if name.startswith("past_"):
            if "decoder" in name:
                # Decoder KV: 第三个维度是 past_seq
                dynamic_axes[name] = {0: "batch", 2: "past_seq"}
            elif "encoder" in name:
                # Encoder KV: 第三个维度是 src_seq (encoder_seq_len)
                dynamic_axes[name] = {0: "batch", 2: "src_seq"}

    dynamic_axes["logits"] = {0: "batch", 1: "tgt_seq"}
    for name in output_names:
        if name.startswith("present_"):
            dynamic_axes[name] = {0: "batch", 2: "present_seq"}

    with torch.no_grad():
        torch.onnx.export(
            wrapper,
            dummy_inputs,
            onnx_path.as_posix(),
            input_names=input_names,
            output_names=output_names,
            dynamic_axes=dynamic_axes,
            opset_version=12,
            do_constant_folding=True,
        )

    print(f"[INFO] Decoder ONNX model saved to: {onnx_path}")

    # 5) 检查 IR / opset
    print("[5/5] Inspecting ONNX decoder model IR/opset ...")
    model_onnx = onnx.load(onnx_path.as_posix())
    print(f"    IR version: {model_onnx.ir_version}")
    print("    Opset imports:")
    for op in model_onnx.opset_import:
        print(f"      domain='{op.domain}' version={op.version}")

    onnx.checker.check_model(model_onnx)
    print("[DONE] Decoder export with KV cache finished and model is valid.")


def main():
    parser = argparse.ArgumentParser(
        description="Export Marian zh-en decoder+KV to ONNX IR<=9, opset 12 (fixed version)."
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        default="core/engine/models/nmt/marian-zh-en",
        help="Directory to save model.onnx (default: core/engine/models/nmt/marian-zh-en)",
    )
    parser.add_argument(
        "--model_id",
        type=str,
        default="Helsinki-NLP/opus-mt-zh-en",
        help="Hugging Face model id or local path (default: Helsinki-NLP/opus-mt-zh-en)",
    )
    args = parser.parse_args()

    export_marian_decoder_ir9_fixed(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
