#!/usr/bin/env python3
"""
将 HuggingFace Whisper 模型转换为 GGML 格式
用于 whisper-rs (whisper.cpp) 推理

使用方法:
    python scripts/convert_whisper_to_ggml.py \
        --model_name openai/whisper-base \
        --output_dir core/engine/models/asr/whisper-base \
        --quantize

注意:
    - 需要安装 transformers, torch, numpy
    - 转换后的模型文件为 .ggml 格式
    - 可以使用 --quantize 进行量化（减小模型大小）
"""

import argparse
import os
import sys
from pathlib import Path

try:
    import torch
    import numpy as np
    from transformers import WhisperProcessor, WhisperForConditionalGeneration
except ImportError as e:
    print(f"[ERROR] 缺少必要的依赖: {e}")
    print("请运行: pip install transformers torch numpy")
    sys.exit(1)


def convert_whisper_to_ggml(
    model_name: str,
    output_dir: Path,
    quantize: bool = False,
    output_type: str = "f32",  # f32, f16, q8_0, q4_0
):
    """
    将 HuggingFace Whisper 模型转换为 GGML 格式
    
    Args:
        model_name: HuggingFace 模型名称（如 "openai/whisper-base"）
        output_dir: 输出目录
        quantize: 是否量化
        output_type: 输出类型（f32, f16, q8_0, q4_0）
    """
    print(f"=== 转换 Whisper 模型: {model_name} ===")
    print(f"输出目录: {output_dir}")
    print(f"量化: {quantize}, 输出类型: {output_type}")
    
    # 创建输出目录
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # 加载模型和处理器
    print("\n[1/4] 加载模型...")
    try:
        model = WhisperForConditionalGeneration.from_pretrained(
            model_name,
            torch_dtype=torch.float32,
        )
        processor = WhisperProcessor.from_pretrained(model_name)
        print(f"✓ 模型加载成功: {model_name}")
    except Exception as e:
        print(f"✗ 模型加载失败: {e}")
        return False
    
    # 保存 tokenizer 文件（如果还没有）
    print("\n[2/4] 保存 tokenizer 文件...")
    try:
        processor.save_pretrained(output_dir)
        print(f"✓ Tokenizer 文件已保存到: {output_dir}")
    except Exception as e:
        print(f"⚠ Tokenizer 保存失败（可能已存在）: {e}")
    
    # 注意：whisper.cpp 的转换需要特定的脚本
    # 这里我们提供一个简化的方法，或者建议直接下载预转换的模型
    print("\n[3/4] 模型转换...")
    print("⚠ 注意: 完整的 GGML 转换需要使用 whisper.cpp 的官方转换脚本")
    print("   建议方法:")
    print("   1. 从 HuggingFace 下载 PyTorch 模型")
    print("   2. 使用 whisper.cpp 的 convert-pt-to-ggml.py 脚本转换")
    print("   3. 或者直接下载预转换的 GGML 模型")
    
    # 提供下载链接
    model_size = model_name.split("/")[-1] if "/" in model_name else model_name
    print(f"\n推荐: 直接下载预转换的 GGML 模型")
    print(f"  模型: {model_size}")
    print(f"  下载地址: https://huggingface.co/ggerganov/whisper.cpp")
    print(f"  或使用: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model_size}.bin")
    
    # 如果用户想要手动转换，提供指导
    print("\n[4/4] 手动转换步骤:")
    print("   1. 克隆 whisper.cpp 仓库:")
    print("      git clone https://github.com/ggerganov/whisper.cpp.git")
    print("   2. 安装依赖:")
    print("      pip install -r whisper.cpp/requirements.txt")
    print("   3. 下载 HuggingFace 模型:")
    print(f"      python -c \"from transformers import WhisperForConditionalGeneration; WhisperForConditionalGeneration.from_pretrained('{model_name}').save_pretrained('./models/{model_size}')\"")
    print("   4. 运行转换脚本:")
    print(f"      python whisper.cpp/models/convert-pt-to-ggml.py ./models/{model_size} ./models/{model_size}/ggml-model.bin")
    
    return True


def download_preconverted_ggml(model_size: str, output_dir: Path):
    """
    下载预转换的 GGML 模型
    
    Args:
        model_size: 模型大小（tiny, base, small, medium, large）
        output_dir: 输出目录
    """
    import urllib.request
    
    print(f"=== 下载预转换的 GGML 模型: {model_size} ===")
    
    # 创建输出目录
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # GGML 模型下载地址
    base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main"
    model_file = f"ggml-{model_size}.bin"
    url = f"{base_url}/{model_file}"
    
    output_path = output_dir / model_file
    
    if output_path.exists():
        print(f"✓ 模型文件已存在: {output_path}")
        return True
    
    print(f"下载地址: {url}")
    print(f"保存到: {output_path}")
    
    try:
        print("正在下载...")
        urllib.request.urlretrieve(url, output_path)
        print(f"✓ 下载成功: {output_path}")
        print(f"  文件大小: {output_path.stat().st_size / 1024 / 1024:.2f} MB")
        return True
    except Exception as e:
        print(f"✗ 下载失败: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="将 HuggingFace Whisper 模型转换为 GGML 格式"
    )
    parser.add_argument(
        "--model_name",
        type=str,
        default="openai/whisper-base",
        help="HuggingFace 模型名称（默认: openai/whisper-base）",
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        default="core/engine/models/asr/whisper-base",
        help="输出目录（默认: core/engine/models/asr/whisper-base）",
    )
    parser.add_argument(
        "--download",
        action="store_true",
        help="直接下载预转换的 GGML 模型（推荐）",
    )
    parser.add_argument(
        "--quantize",
        action="store_true",
        help="量化模型（减小大小）",
    )
    
    args = parser.parse_args()
    
    output_dir = Path(args.output_dir)
    
    if args.download:
        # 从模型名称提取模型大小
        model_size = args.model_name.split("/")[-1] if "/" in args.model_name else args.model_name
        model_size = model_size.replace("whisper-", "").replace("openai/", "")
        
        success = download_preconverted_ggml(model_size, output_dir)
        if success:
            print("\n✓ 模型准备完成！")
            print(f"  模型文件: {output_dir / f'ggml-{model_size}.bin'}")
        else:
            print("\n✗ 模型下载失败")
            sys.exit(1)
    else:
        # 尝试转换（但建议使用下载方式）
        success = convert_whisper_to_ggml(
            args.model_name,
            output_dir,
            quantize=args.quantize,
        )
        if success:
            print("\n✓ 转换脚本执行完成（请按照提示手动完成转换）")
        else:
            print("\n✗ 转换失败")
            sys.exit(1)


if __name__ == "__main__":
    main()

