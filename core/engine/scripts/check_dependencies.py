#!/usr/bin/env python3
"""
检查服务依赖

检查 Speaker Embedding 和 YourTTS 服务所需的所有依赖
"""

import sys
import subprocess

def check_package(package_name, import_name=None):
    """检查包是否安装"""
    if import_name is None:
        import_name = package_name
    
    try:
        __import__(import_name)
        print(f"[OK] {package_name} is installed")
        return True
    except ImportError:
        print(f"[FAIL] {package_name} is NOT installed")
        return False

def install_package(package_name):
    """安装包"""
    try:
        print(f"[*] Installing {package_name}...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", package_name])
        print(f"[OK] {package_name} installed successfully")
        return True
    except Exception as e:
        print(f"[FAIL] Failed to install {package_name}: {e}")
        return False

def check_torchaudio_compatibility():
    """检查 torchaudio 兼容性"""
    try:
        import torchaudio
        version = torchaudio.__version__
        print(f"[*] torchaudio version: {version}")
        
        # 检查是否有 list_audio_backends 方法
        if hasattr(torchaudio, 'list_audio_backends'):
            print("[OK] torchaudio has list_audio_backends (compatible)")
            return True
        else:
            print("[WARN] torchaudio does not have list_audio_backends (2.9+ compatibility issue)")
            print("       The service script will apply a compatibility fix")
            return True  # 可以修复，所以返回 True
    except ImportError:
        print("[FAIL] torchaudio is NOT installed")
        return False

def main():
    print("=" * 60)
    print("  Dependency Check for Speaker Services")
    print("=" * 60)
    print()
    
    all_ok = True
    
    # 基础依赖
    print("[*] Checking basic dependencies...")
    all_ok &= check_package("flask", "flask")
    all_ok &= check_package("numpy", "numpy")
    all_ok &= check_package("torch", "torch")
    all_ok &= check_torchaudio_compatibility()
    all_ok &= check_package("soundfile", "soundfile")
    print()
    
    # Speaker Embedding 依赖
    print("[*] Checking Speaker Embedding dependencies...")
    all_ok &= check_package("speechbrain", "speechbrain")
    print()
    
    # YourTTS 依赖
    print("[*] Checking YourTTS dependencies...")
    all_ok &= check_package("TTS", "TTS")
    print()
    
    # CUDA 检查
    print("[*] Checking CUDA availability...")
    try:
        import torch
        if torch.cuda.is_available():
            print(f"[OK] CUDA is available: {torch.cuda.get_device_name(0)}")
        else:
            print("[WARN] CUDA is not available (will use CPU)")
    except Exception as e:
        print(f"[WARN] Could not check CUDA: {e}")
    print()
    
    # 总结
    print("=" * 60)
    if all_ok:
        print("[OK] All dependencies are satisfied!")
    else:
        print("[FAIL] Some dependencies are missing")
        print("\nTo install missing dependencies:")
        print("  pip install flask numpy torch torchaudio soundfile speechbrain TTS")
    print("=" * 60)
    
    return 0 if all_ok else 1

if __name__ == '__main__':
    sys.exit(main())

