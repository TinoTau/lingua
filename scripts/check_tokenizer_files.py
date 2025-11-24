#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""检查 tokenizer 文件结构"""

import json
from pathlib import Path

model_dir = Path("core/engine/models/nmt/m2m100-en-zh")

print("检查文件:")
for f in ["tokenizer.json", "vocab.json", "added_tokens.json", "sentencepiece.bpe.model", "tokenizer_config.json"]:
    path = model_dir / f
    if path.exists():
        print(f"  ✅ {f} ({path.stat().st_size / 1024 / 1024:.2f} MB)")
    else:
        print(f"  ❌ {f}")

print("\n检查 vocab.json 格式:")
with open(model_dir / "vocab.json", 'r', encoding='utf-8') as f:
    vocab = json.load(f)
print(f"  类型: {type(vocab)}")
print(f"  大小: {len(vocab) if isinstance(vocab, dict) else 'N/A'}")
print(f"  示例键: {list(vocab.keys())[:10] if isinstance(vocab, dict) else 'N/A'}")

if (model_dir / "added_tokens.json").exists():
    print("\n检查 added_tokens.json:")
    with open(model_dir / "added_tokens.json", 'r', encoding='utf-8') as f:
        added = json.load(f)
    print(f"  类型: {type(added)}")
    if isinstance(added, list):
        print(f"  数量: {len(added)}")
        lang_tokens = [t for t in added if isinstance(t, dict) and t.get('content', '').startswith('__') and t.get('content', '').endswith('__')]
        print(f"  语言 token 数量: {len(lang_tokens)}")
        print(f"  示例: {lang_tokens[:5] if lang_tokens else 'None'}")

