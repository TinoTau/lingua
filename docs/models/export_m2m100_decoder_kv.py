#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
from pathlib import Path
from typing import List
import os

os.environ["TORCH_ONNX_DISABLE_DYNAMO"] = "1"

import torch
from torch import nn
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
import onnx


class M2M100DecoderWrapper(nn.Module):
    def __init__(self, model: M2M100ForConditionalGeneration):
        super().__init__()
        self.model = model
        self.decoder = model.get_decoder()
        self.lm_head = model.lm_head

        self.layers = model.config.decoder_layers
        self.num_heads = model.config.decoder_attention_heads
        self.hidden_size = model.config.d_model
        self.head_dim = self.hidden_size // self.num_heads

    def reconstruct_past(self, flat: List[torch.Tensor]):
        past = []
        idx = 0
        for _ in range(self.layers):
            self_k = flat[idx + 0]
            self_v = flat[idx + 1]
            cross_k = flat[idx + 2]
            cross_v = flat[idx + 3]
            past.append((self_k, self_v, cross_k, cross_v))
            idx += 4
        return tuple(past)

    def forward(self, encoder_attention_mask, input_ids, encoder_hidden_states, *past_and_flag):
        past_flat = list(past_and_flag[:-1])
        flag = past_and_flag[-1]

        # Ensure all KV + flag participate in the graph to avoid pruning
        dummy = 0.0
        for t in past_flat:
            dummy = dummy + t.sum() * 0.0
        encoder_hidden_states = encoder_hidden_states + dummy + flag.to(encoder_hidden_states.dtype) * 0.0

        past = self.reconstruct_past(past_flat)

        out = self.decoder(
            input_ids=input_ids,
            encoder_hidden_states=encoder_hidden_states,
            encoder_attention_mask=encoder_attention_mask,
            past_key_values=past,
            use_cache=True,
        )

        logits = self.lm_head(out.last_hidden_state)

        flat_out: List[torch.Tensor] = []
        for p in out.past_key_values:
            flat_out.extend([p[0], p[1], p[2], p[3]])

        return (logits, *flat_out)


def export_m2m100_decoder(output_dir: Path, model_id: str):
    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"[INFO] Output directory: {output_dir}")

    token = os.getenv("HF_TOKEN")
    extra = dict(token=token) if token else {}

    tokenizer = M2M100Tokenizer.from_pretrained(model_id, **extra)
    model = M2M100ForConditionalGeneration.from_pretrained(model_id, **extra)
    model.eval()

    wrapper = M2M100DecoderWrapper(model)
    wrapper.eval()

    # Dummy inputs
    enc = tokenizer("Hello world", return_tensors="pt")
    encoder_hidden_states = model.get_encoder()(**enc)[0]
    encoder_attention_mask = enc["attention_mask"]
    bos = torch.tensor([[tokenizer.get_lang_id("zh")]], dtype=torch.long)

    layers = wrapper.layers
    past: List[torch.Tensor] = [
        torch.zeros(
            (1, wrapper.num_heads, 1, wrapper.head_dim),
            dtype=encoder_hidden_states.dtype,
        )
        for _ in range(layers * 4)
    ]
    flag = torch.tensor([False], dtype=torch.bool)

    inputs = (encoder_attention_mask, bos, encoder_hidden_states, *past, flag)

    input_names = ["encoder_attention_mask", "input_ids", "encoder_hidden_states"]
    for i in range(layers):
        input_names += [
            f"past_key_values.{i}.decoder.key",
            f"past_key_values.{i}.decoder.value",
            f"past_key_values.{i}.encoder.key",
            f"past_key_values.{i}.encoder.value",
        ]
    input_names.append("use_cache_branch")

    output_names = ["logits"]
    for i in range(layers):
        output_names += [
            f"present.{i}.decoder.key",
            f"present.{i}.decoder.value",
            f"present.{i}.encoder.key",
            f"present.{i}.encoder.value",
        ]

    export_path = output_dir / "decoder.onnx"
    print(f"[INFO] Exporting decoder to {export_path}")

    # dynamic_axes: keep batch dynamic; seq lengths are also dynamic
    dynamic_axes = {name: {0: "batch"} for name in input_names}
    # encoder_attention_mask 的序列长度应该是动态的
    dynamic_axes["encoder_attention_mask"] = {0: "batch", 1: "src_seq"}
    # encoder_hidden_states 的序列长度也应该是动态的
    dynamic_axes["encoder_hidden_states"] = {0: "batch", 1: "src_seq"}
    # input_ids 的序列长度是动态的
    dynamic_axes["input_ids"] = {0: "batch", 1: "tgt_seq"}
    dynamic_axes["logits"] = {0: "batch", 1: "tgt_seq"}
    for name in output_names:
        if name.startswith("present."):
            dynamic_axes[name] = {0: "batch", 2: "seq"}

    torch.onnx.export(
        wrapper,
        inputs,
        export_path.as_posix(),
        input_names=input_names,
        output_names=output_names,
        dynamic_axes=dynamic_axes,
        opset_version=12,  # 使用 opset 12 以确保 IR <= 9，兼容 ort 1.16.3
        do_constant_folding=False,
    )

    onnx_model = onnx.load(export_path.as_posix())
    onnx.checker.check_model(onnx_model)

    print("[DONE] M2M100 decoder exported successfully.")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output_dir", type=str, default="m2m100-en-zh")
    parser.add_argument("--model_id", type=str, default="facebook/m2m100_418M")
    args = parser.parse_args()

    export_m2m100_decoder(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
