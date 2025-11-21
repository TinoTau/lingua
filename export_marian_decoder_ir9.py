#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Export Marian zh-en decoder+LM head to ONNX with IR<=9 and opset 12.

Environment requirements:
  - Python 3.10.x
  - torch==1.13.1+cpu  (or matching 1.13.1 build)
  - transformers==4.40.0
  - onnx==1.14.0
"""

import argparse
from pathlib import Path

import torch
from torch import nn
from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
import onnx


class MarianDecoderWrapper(nn.Module):
    """Wraps Marian decoder + LM head into a single module.

    Forward signature:
      (decoder_input_ids, encoder_hidden_states, encoder_attention_mask) -> logits
    """

    def __init__(self, model: AutoModelForSeq2SeqLM):
        super().__init__()
        # For MarianMTModel, the underlying encoder/decoder live in model.model
        self.decoder = model.model.decoder
        self.lm_head = model.lm_head
        self.config = model.config

    def forward(
        self,
        decoder_input_ids: torch.Tensor,
        encoder_hidden_states: torch.Tensor,
        encoder_attention_mask: torch.Tensor = None,
    ) -> torch.Tensor:
        decoder_outputs = self.decoder(
            input_ids=decoder_input_ids,
            encoder_hidden_states=encoder_hidden_states,
            encoder_attention_mask=encoder_attention_mask,
        )
        hidden_states = decoder_outputs.last_hidden_state
        logits = self.lm_head(hidden_states)
        return logits


def export_marian_decoder_ir9(output_dir: Path, model_id: str) -> None:
    print(f"[INFO] Using model_id: {model_id}")
    print(f"[INFO] Output dir: {output_dir}")

    output_dir.mkdir(parents=True, exist_ok=True)
    onnx_path = output_dir / "model.onnx"

    # 1) Load tokenizer & full model, then wrap decoder+lm_head
    print("[1/4] Loading tokenizer and model ...")
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = AutoModelForSeq2SeqLM.from_pretrained(model_id)
    wrapper = MarianDecoderWrapper(model)
    wrapper.eval()

    # 2) Prepare dummy inputs
    print("[2/4] Preparing dummy inputs ...")
    # Use same dummy sentence to get encoder sequence length
    dummy_text = "你好，世界。"
    enc = tokenizer(dummy_text, return_tensors="pt")
    input_ids = enc["input_ids"]
    attention_mask = enc["attention_mask"]

    # Suppose encoder_hidden_states has shape [batch, src_seq, hidden]
    with torch.no_grad():
        encoder_hidden_states = model.get_encoder()(
            input_ids=input_ids,
            attention_mask=attention_mask,
        )[0]

    # For decoder_input_ids, start with BOS token
    if hasattr(model.config, "decoder_start_token_id") and model.config.decoder_start_token_id is not None:
        bos_id = model.config.decoder_start_token_id
    else:
        bos_id = tokenizer.bos_token_id if tokenizer.bos_token_id is not None else int(input_ids[0, 0])

    decoder_input_ids = torch.tensor([[bos_id]], dtype=torch.long)

    print(f"    encoder_hidden_states.shape = {tuple(encoder_hidden_states.shape)}")
    print(f"    decoder_input_ids.shape     = {tuple(decoder_input_ids.shape)}")
    print(f"    encoder_attention_mask.shape= {tuple(attention_mask.shape)}")


    # 3) Export decoder+lm_head to ONNX (opset 12, IR<=9)
    print("[3/4] Exporting decoder ONNX (opset_version=12) ...")
    with torch.no_grad():
        torch.onnx.export(
            wrapper,
            (decoder_input_ids, encoder_hidden_states, attention_mask),
            onnx_path.as_posix(),
            input_names=["decoder_input_ids", "encoder_hidden_states", "encoder_attention_mask"],
            output_names=["logits"],
            dynamic_axes={
                "decoder_input_ids": {0: "batch", 1: "tgt_seq"},
                "encoder_hidden_states": {0: "batch", 1: "src_seq"},
                "encoder_attention_mask": {0: "batch", 1: "src_seq"},
                "logits": {0: "batch", 1: "tgt_seq"},
            },
            opset_version=12,
            do_constant_folding=True,
        )

    print(f"[INFO] Decoder ONNX model saved to: {onnx_path}")

    # 4) Inspect IR/opset
    print("[4/4] Inspecting ONNX decoder model IR/opset ...")
    model_onnx = onnx.load(onnx_path.as_posix())
    print(f"    IR version: {model_onnx.ir_version}")
    print("    Opset imports:")
    for op in model_onnx.opset_import:
        print(f"      domain='{op.domain}' version={op.version}")

    onnx.checker.check_model(model_onnx)
    print("[DONE] Decoder export finished and model is valid.")


def main():
    parser = argparse.ArgumentParser(description="Export Marian zh-en decoder to ONNX IR<=9, opset 12.")
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

    export_marian_decoder_ir9(Path(args.output_dir), args.model_id)


if __name__ == "__main__":
    main()
