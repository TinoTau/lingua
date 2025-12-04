#!/usr/bin/env python3
# 下载官方 Silero VAD 模型（Python 版本）
# 使用方法：python download_silero_vad_official.py

import os
import sys
import urllib.request
from pathlib import Path

# 获取脚本所在目录
script_dir = Path(__file__).parent
core_engine_dir = script_dir.parent
model_dir = core_engine_dir / "models" / "vad" / "silero"

# 创建模型目录（如果不存在）
model_dir.mkdir(parents=True, exist_ok=True)

# 模型文件路径
model_path = model_dir / "silero_vad_official.onnx"
backup_path = model_dir / "silero_vad.onnx.backup"

# 备份现有模型（如果存在）
existing_model = model_dir / "silero_vad.onnx"
if existing_model.exists():
    print("备份现有模型...")
    import shutil
    shutil.copy2(existing_model, backup_path)
    print(f"已备份到: {backup_path}")

# 官方模型下载地址（使用 Hugging Face）
# 主地址（可能需要认证）：
# - https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx
# 备用地址（已验证可用）：
# - https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx
model_url = "https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx"

print("=" * 60)
print("  下载官方 Silero VAD 模型")
print("=" * 60)
print()
print(f"下载地址: {model_url}")
print(f"保存路径: {model_path}")
print()

try:
    print("开始下载...")
    
    def show_progress(block_num, block_size, total_size):
        downloaded = block_num * block_size
        percent = min(downloaded * 100 / total_size, 100)
        size_mb = total_size / (1024 * 1024)
        downloaded_mb = downloaded / (1024 * 1024)
        print(f"\r进度: {percent:.1f}% ({downloaded_mb:.2f} MB / {size_mb:.2f} MB)", end="", flush=True)
    
    # 创建请求并添加 User-Agent 头
    request = urllib.request.Request(model_url)
    request.add_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
    urllib.request.urlretrieve(request, model_path, reporthook=show_progress)
    print()  # 换行
    
    # 验证文件
    if model_path.exists():
        file_size_mb = model_path.stat().st_size / (1024 * 1024)
        print()
        print("✓ 下载成功！")
        print(f"  文件大小: {file_size_mb:.2f} MB")
        print(f"  文件路径: {model_path}")
        print()
        print("提示: 请更新配置文件中的模型路径为: models/vad/silero/silero_vad_official.onnx")
    else:
        print("✗ 下载失败：文件不存在")
        sys.exit(1)
        
except Exception as e:
    print()
    print(f"✗ 下载失败: {e}")
    print()
    print("如果下载失败，可以尝试以下方法：")
    print(f"1. 使用浏览器直接下载: {model_url}")
    print("2. 在 WSL 中使用 wget: wget <url> -O silero_vad_official.onnx")
    sys.exit(1)

print("=" * 60)

