#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Get decoder model input/output signature"""

import onnxruntime as ort
import sys
from pathlib import Path

def main():
    model_path = Path("core/engine/models/nmt/marian-zh-en/model.onnx")
    
    if not model_path.exists():
        print(f"Error: Model not found at {model_path}")
        sys.exit(1)
    
    sess = ort.InferenceSession(str(model_path), providers=['CPUExecutionProvider'])
    
    print("=== Decoder Model Input Signature ===")
    print("")
    print(f"Total inputs: {len(sess.get_inputs())}")
    print("")
    for i, inp in enumerate(sess.get_inputs()):
        print(f'Input[{i}] name="{inp.name}" type={inp.type} shape={inp.shape}')
    
    print("")
    print("=== Decoder Model Output Signature ===")
    print("")
    print(f"Total outputs: {len(sess.get_outputs())}")
    print("")
    for i, out in enumerate(sess.get_outputs()):
        print(f'Output[{i}] name="{out.name}" type={out.type} shape={out.shape}')

if __name__ == "__main__":
    main()

