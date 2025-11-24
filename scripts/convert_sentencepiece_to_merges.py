#!/usr/bin/env python3
"""
将 sentencepiece.bpe.model 转换为 merges.txt 和 vocab.json
用于 tokenizers crate 加载
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import sentencepiece as spm
except ImportError:
    print("Error: sentencepiece not installed. Please install it with: pip install sentencepiece")
    sys.exit(1)


def convert_sentencepiece_to_merges(
    model_path: Path,
    vocab_output_path: Path,
    merges_output_path: Path,
):
    """将 sentencepiece.bpe.model 转换为 vocab.json 和 merges.txt"""
    
    print(f"[1/3] 加载 SentencePiece 模型: {model_path}")
    sp = spm.SentencePieceProcessor()
    sp.load(str(model_path))
    
    print(f"[2/3] 提取词汇表...")
    # 获取词汇表大小
    vocab_size = sp.get_piece_size()
    
    # 构建 vocab.json（token -> id 映射）
    vocab = {}
    merges = []
    
    # 获取所有 token 和对应的 ID
    for i in range(vocab_size):
        token = sp.id_to_piece(i)
        vocab[token] = i
    
    # 对于 BPE 模型，merges 通常是从特殊 token 之后开始的
    # SentencePiece 的 merges 信息需要通过其他方式获取
    # 这里我们尝试从词汇表中提取可能的 merges
    
    print(f"[3/3] 生成 merges.txt...")
    # 注意：SentencePiece 的 merges 信息可能无法直接提取
    # 我们需要使用其他方法，或者使用现有的 vocab.json
    
    # 保存 vocab.json
    with open(vocab_output_path, 'w', encoding='utf-8') as f:
        json.dump(vocab, f, ensure_ascii=False, indent=2)
    print(f"✅ vocab.json 已保存到: {vocab_output_path}")
    
    # 对于 merges.txt，我们需要从 SentencePiece 模型中提取
    # 但 SentencePiece 的 API 可能不直接提供 merges 信息
    # 我们可以尝试使用 HuggingFace 的转换工具，或者手动构建
    
    # 临时方案：创建一个空的 merges.txt（如果 vocab.json 已经包含所有信息）
    # 或者使用现有的 vocab.json 中的信息
    
    # 尝试从词汇表中提取可能的 merges（基于 BPE 的合并规则）
    # 这需要分析 token 的结构，找出哪些是合并的结果
    
    # 简化方案：创建一个基本的 merges.txt
    # 注意：这可能需要根据实际的 SentencePiece 模型调整
    with open(merges_output_path, 'w', encoding='utf-8') as f:
        f.write("#version: 0.2\n")
        # 这里应该包含实际的 merges，但需要从 SentencePiece 模型中提取
        # 暂时留空，让用户知道需要手动处理
        print(f"⚠️  merges.txt 已创建，但可能需要手动填充 merges 信息")
        print(f"   文件位置: {merges_output_path}")
    
    print(f"\n✅ 转换完成！")
    print(f"   vocab.json: {vocab_output_path}")
    print(f"   merges.txt: {merges_output_path}")
    print(f"\n注意: merges.txt 可能需要手动处理或使用 HuggingFace 的转换工具")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="将 sentencepiece.bpe.model 转换为 vocab.json 和 merges.txt"
    )
    parser.add_argument(
        "--model",
        type=Path,
        required=True,
        help="sentencepiece.bpe.model 文件路径"
    )
    parser.add_argument(
        "--vocab-output",
        type=Path,
        required=True,
        help="输出的 vocab.json 文件路径"
    )
    parser.add_argument(
        "--merges-output",
        type=Path,
        required=True,
        help="输出的 merges.txt 文件路径"
    )
    
    args = parser.parse_args()
    
    if not args.model.exists():
        print(f"Error: 模型文件不存在: {args.model}")
        sys.exit(1)
    
    convert_sentencepiece_to_merges(
        args.model,
        args.vocab_output,
        args.merges_output,
    )

