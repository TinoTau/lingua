#!/usr/bin/env python3
"""
设置 YourTTS HTTP 服务（用于 Zero-shot TTS）

使用方法：
    python download_yourtts_service.py

这将：
1. 安装 Coqui TTS
2. 下载 YourTTS 模型
3. 启动 HTTP 服务
"""

import subprocess
import sys
import os

def install_tts():
    """安装 Coqui TTS"""
    print("正在安装 Coqui TTS...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "TTS"])
        print("✅ Coqui TTS 安装成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ 安装失败: {e}")
        return False

def download_model():
    """下载 YourTTS 模型"""
    print("\n正在下载 YourTTS 模型...")
    print("这可能需要几分钟，模型大小约 500MB...")
    
    try:
        from TTS.api import TTS
        tts = TTS("tts_models/multilingual/multi-dataset/your_tts")
        print("✅ YourTTS 模型下载成功")
        return True
    except Exception as e:
        print(f"❌ 下载失败: {e}")
        return False

def start_server():
    """启动 TTS 服务器"""
    print("\n启动 YourTTS HTTP 服务...")
    print("服务将在 http://127.0.0.1:5002 启动")
    print("\n运行命令：")
    print("  python -m TTS.server.server --model_name tts_models/multilingual/multi-dataset/your_tts --port 5002")
    print("\n或使用 uvicorn：")
    print("  uvicorn TTS.server.server:app --host 127.0.0.1 --port 5002")

if __name__ == "__main__":
    print("=== YourTTS 服务设置 ===")
    
    if install_tts():
        if download_model():
            start_server()
            print("\n✅ 设置完成！")
        else:
            print("\n⚠️  模型下载失败，但可以稍后手动下载")
    else:
        print("\n❌ 安装失败，请手动安装：pip install TTS")

