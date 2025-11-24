#!/usr/bin/env python3
"""
为 M2M100 创建基本的 merges.txt 文件
注意：这是一个临时方案，可能需要根据实际模型调整
"""

import argparse
import sys
from pathlib import Path

def create_basic_merges_txt(output_path: Path):
    """创建一个基本的 merges.txt 文件"""
    
    print(f"创建基本的 merges.txt: {output_path}")
    
    # 创建一个基本的 merges.txt
    # 对于 M2M100，merges 可能不是必需的，或者可以从其他地方获取
    # 这里我们创建一个空的 merges.txt（只有版本头）
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write("#version: 0.2\n")
        # 空的 merges.txt 可能无法工作，但我们可以尝试
        # 实际上，对于某些 BPE 模型，merges.txt 可能不是必需的
    
    print(f"✅ merges.txt 已创建: {output_path}")
    print(f"\n⚠️  注意: 这是一个基本的 merges.txt，可能不完整")
    print(f"   如果 tokenizer 仍然无法加载，可能需要:")
    print(f"   1. 使用 HuggingFace 的转换工具")
    print(f"   2. 或者使用其他 tokenizer 实现（如 rust-tokenizers）")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="为 M2M100 创建基本的 merges.txt 文件"
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="输出的 merges.txt 文件路径"
    )
    
    args = parser.parse_args()
    
    create_basic_merges_txt(args.output)

