#!/usr/bin/env python3
"""检查 AudioChunk 的结构"""

from pathlib import Path
from piper.voice import PiperVoice

model_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx"
config_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx.json"

voice = PiperVoice.load(str(model_path), config_path=str(config_path), use_cuda=False)
chunk = next(voice.synthesize("test"))

print("Chunk type:", type(chunk))
print("Chunk attributes:", [attr for attr in dir(chunk) if not attr.startswith('_')])
print("Chunk:", chunk)

