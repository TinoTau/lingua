#!/usr/bin/env python3
# -*- coding: utf-8 -*-

\"\"\"
Export M2M100 encoder to ONNX (opset 13).
Compatible with existing Lingua NMT pipeline.
\"\"\"

import argparse
from pathlib import Path
import os

os.environ["TORCH_ONNX_DISABLE_DYNAMO"] = "1"

import torch
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
import onnx


def export_m2m100_encoder(output_dir: Path, model_id: str):
    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"[INFO] Output directory: {output_dir}")

    token = os.getenv("HF_TOKEN")
    extra = dict(token=token) if token else {}

    tokenizer = M2M100Tokenizer.from_pretrained(model_id, **extra)
    model = M2M100ForConditionalGeneration.from_pretrained(model_id, **extra)
    model.eval()
    encoder = model.get_encoder()

    # Dummy input
    text = "Hello, this is an encoder test."
    enc = tokenizer(text, return_tensors="pt")
    input_ids = enc["input_ids"]
    attn_mask = enc["attention_mask"]

    export_path = output_dir / "encoder.onnx"
    print(f"[INFO] Exporting encoder to {export_path}")

    torch.onnx.export(
        encoder,
        (input_ids, attn_mask),
        export_path.as_posix(),
        input_names=["input_ids", "attention_mask"],
        output_names=["last_hidden_state"],
        dynamic_axes={
            "input_ids": {0: "batch", 1: "seq"},
            "attention_mask": {0: "batch", 1: "seq"},
            "last_hidden_state": {0: "batch", 1: "seq"},
        },
        opset_version=13,
        do_constant_folding=True,
    )

    onnx_model = onnx.load(export_path.as_posix())
    onnx.checker.check_model(onnx_model)

    print("[DONE] M2M100 encoder exported successfully.")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output_dir", type=str, default="m2m100-en-zh")
    parser.add_argument("--model_id", type=str, default="facebook/m2m100_418M")
    args = parser.parse_args()

    export_m2m100_encoder(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
