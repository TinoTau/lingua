#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
检查并修复 tokenizer.json 文件
如果 vocab.json 实际上是 tokenizer.json 格式，则重命名
"""

import json
import os
import shutil
from pathlib import Path
import sys

def check_and_fix_tokenizer_json(model_dir: Path):
    """检查并修复 tokenizer.json 文件"""
    print(f"检查模型目录: {model_dir}")
    
    vocab_path = model_dir / "vocab.json"
    tokenizer_json_path = model_dir / "tokenizer.json"
    
    # 如果 tokenizer.json 已存在，检查它是否是有效的 tokenizer.json
    if tokenizer_json_path.exists():
        try:
            with open(tokenizer_json_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            if isinstance(data, dict) and 'added_tokens' in data:
                print(f"✅ tokenizer.json 已存在且格式正确")
                return True
            else:
                print(f"⚠️  tokenizer.json 存在但格式不正确（不是 tokenizer.json 格式）")
        except Exception as e:
            print(f"⚠️  无法读取 tokenizer.json: {e}")
    
    # 检查 vocab.json 是否是 tokenizer.json 格式
    if vocab_path.exists():
        try:
            with open(vocab_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            # 检查是否是 tokenizer.json 格式（包含 added_tokens, model, normalizer 等字段）
            is_tokenizer_json_format = (
                isinstance(data, dict) and 
                any(key in data for key in ['added_tokens', 'model', 'normalizer', 'pre_tokenizer'])
            )
            
            if is_tokenizer_json_format:
                print(f"✅ 发现 vocab.json 实际上是 tokenizer.json 格式")
                print(f"   重命名 vocab.json -> tokenizer.json")
                
                # 备份原 vocab.json（如果需要）
                vocab_backup = model_dir / "vocab.json.backup"
                if not vocab_backup.exists():
                    shutil.copy2(vocab_path, vocab_backup)
                    print(f"   已备份原文件到: {vocab_backup.name}")
                
                # 复制为 tokenizer.json
                shutil.copy2(vocab_path, tokenizer_json_path)
                print(f"✅ 已创建 tokenizer.json")
                return True
            else:
                print(f"ℹ️  vocab.json 是标准 vocab 格式（token -> id 映射），不是 tokenizer.json")
        except Exception as e:
            print(f"❌ 无法读取 vocab.json: {e}")
    
    # 如果都没有，提示下载
    print(f"❌ 未找到 tokenizer.json 文件")
    print(f"   请下载: huggingface-cli download facebook/m2m100_418M tokenizer.json --local-dir {model_dir}")
    return False

def main():
    if len(sys.argv) < 2:
        print("用法: python check_and_fix_tokenizer_json.py <model_dir1> [model_dir2]")
        print("示例: python check_and_fix_tokenizer_json.py core/engine/models/nmt/m2m100-en-zh")
        sys.exit(1)
    
    model_dirs = [Path(arg) for arg in sys.argv[1:]]
    
    all_success = True
    for model_dir in model_dirs:
        print(f"\n{'='*60}")
        success = check_and_fix_tokenizer_json(model_dir)
        if not success:
            all_success = False
    
    if all_success:
        print(f"\n{'='*60}")
        print("✅ 所有模型目录的 tokenizer.json 已就绪")
        sys.exit(0)
    else:
        print(f"\n{'='*60}")
        print("❌ 部分模型目录缺少 tokenizer.json，请下载")
        sys.exit(1)

if __name__ == "__main__":
    main()

