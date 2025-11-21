#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Export Marian zh-en encoder to ONNX with IR<=9 and opset 12.

Environment requirements:
  - Python 3.10.x
  - torch==1.13.1+cpu  (or matching 1.13.1 build)
  - transformers==4.40.0
  - onnx==1.14.0
"""

import argparse
from pathlib import Path

import torch
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import onnx


def export_marian_encoder_ir9(output_dir: Path, model_id: str) -> None:
    print(f"[INFO] Using model_id: {model_id}")
    print(f"[INFO] Output dir: {output_dir}")

    output_dir.mkdir(parents=True, exist_ok=True)
    onnx_path = output_dir / "encoder_model.onnx"

    # 1) Load tokenizer & full model, then take encoder
    print("[1/4] Loading tokenizer and model ...")
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = AutoModelForSeq2SeqLM.from_pretrained(model_id)
    encoder = model.get_encoder()
    encoder.eval()

    # 2) Prepare dummy inputs
    print("[2/4] Preparing dummy inputs ...")
    dummy_text = "你好，世界。"  # Chinese sentence as dummy input
    enc = tokenizer(dummy_text, return_tensors="pt")

    input_ids = enc["input_ids"]
    attention_mask = enc["attention_mask"]

    print(f"    input_ids.shape      = {tuple(input_ids.shape)}")
    print(f"    attention_mask.shape = {tuple(attention_mask.shape)}")


    # 3) Export encoder to ONNX (opset 12, IR<=9)
    print("[3/4] Exporting encoder ONNX (opset_version=12) ...")
    with torch.no_grad():
        torch.onnx.export(
            encoder,
            (input_ids, attention_mask),
            onnx_path.as_posix(),
            input_names=["input_ids", "attention_mask"],
            output_names=["last_hidden_state"],
            dynamic_axes={
                "input_ids": {0: "batch", 1: "src_seq"},
                "attention_mask": {0: "batch", 1: "src_seq"},
                "last_hidden_state": {0: "batch", 1: "src_seq"},
            },
            opset_version=12,
            do_constant_folding=True,
        )

    print(f"[INFO] Encoder ONNX model saved to: {onnx_path}")

    # 4) Inspect IR/opset
    print("[4/4] Inspecting ONNX encoder model IR/opset ...")
    model_onnx = onnx.load(onnx_path.as_posix())
    print(f"    IR version: {model_onnx.ir_version}")
    print("    Opset imports:")
    for op in model_onnx.opset_import:
        print(f"      domain='{op.domain}' version={op.version}")

    onnx.checker.check_model(model_onnx)
    print("[DONE] Encoder export finished and model is valid.")


def main():
    parser = argparse.ArgumentParser(description="Export Marian zh-en encoder to ONNX IR<=9, opset 12.")
    parser.add_argument(
        "--output_dir",
        type=str,
        default="core/engine/models/nmt/marian-zh-en",
        help="Directory to save encoder_model.onnx (default: core/engine/models/nmt/marian-zh-en)",
    )
    parser.add_argument(
        "--model_id",
        type=str,
        default="Helsinki-NLP/opus-mt-zh-en",
        help="Hugging Face model id or local path (default: Helsinki-NLP/opus-mt-zh-en)",
    )
    args = parser.parse_args()

    export_marian_encoder_ir9(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
