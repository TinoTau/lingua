#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Export Marian zh-en model to ONNX with:
- IR <= 9
- opset_version = 12
Compatible with ONNX Runtime 1.16.x
"""

import argparse
from pathlib import Path
import torch
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import onnx


def export_marian_ir9(output_dir: Path, model_id: str):
    print(f"[INFO] Loading model: {model_id}")
    output_dir.mkdir(parents=True, exist_ok=True)

    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = AutoModelForSeq2SeqLM.from_pretrained(model_id)
    model.eval()

    # Dummy input for tracing
    text = "你好，世界。"
    enc = tokenizer(text, return_tensors="pt")
    input_ids = enc["input_ids"]
    attention_mask = enc["attention_mask"]

    # decoder start token
    bos = (
        model.config.decoder_start_token_id
        if model.config.decoder_start_token_id is not None
        else tokenizer.bos_token_id or int(input_ids[0, 0])
    )
    decoder_input_ids = torch.tensor([[bos]])

    out_path = output_dir / "model_ir9.onnx"

    print("[INFO] Exporting ONNX model …")
    with torch.no_grad():
        torch.onnx.export(
            model,
            (input_ids, attention_mask, decoder_input_ids),
            out_path.as_posix(),
            input_names=["input_ids", "attention_mask", "decoder_input_ids"],
            output_names=["logits"],
            dynamic_axes={
                "input_ids": {0: "batch", 1: "src_seq"},
                "attention_mask": {0: "batch", 1: "src_seq"},
                "decoder_input_ids": {0: "batch", 1: "tgt_seq"},
                "logits": {0: "batch", 1: "tgt_seq"},
            },
            opset_version=12,
            do_constant_folding=True,
        )

    print(f"[INFO] Saved to {out_path}")

    model_onnx = onnx.load(out_path)
    print("[CHECK] IR version:", model_onnx.ir_version)
    print("[CHECK] Opsets:", [(o.domain, o.version) for o in model_onnx.opset_import])
    onnx.checker.check_model(model_onnx)
    print("[DONE] Model is valid.")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output_dir", type=str, default="core/engine/models/nmt/marian-zh-en")
    parser.add_argument("--model_id", type=str, default="Helsinki-NLP/opus-mt-zh-en")
    args = parser.parse_args()

    export_marian_ir9(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
