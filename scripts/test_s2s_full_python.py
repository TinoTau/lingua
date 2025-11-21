#!/usr/bin/env python3
"""
完整 S2S 流集成测试（Python 版本）
使用真实的 ASR 和 NMT 进行完整的语音转语音翻译测试

使用方法：
    python scripts/test_s2s_full_python.py <input_wav_file>

示例：
    python scripts/test_s2s_full_python.py test_output/s2s_flow_test.wav

前提条件：
    1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
    2. Whisper ASR 模型已下载到 core/engine/models/asr/whisper-base/
    3. Marian NMT 模型已下载到 core/engine/models/nmt/marian-zh-en/
    4. 输入音频文件（WAV 格式）
"""

import sys
import os
import json
import requests
import subprocess
from pathlib import Path
from typing import List, Optional

def check_piper_service():
    """检查 Piper HTTP 服务是否运行"""
    try:
        response = requests.get("http://127.0.0.1:5005/health", timeout=2)
        if response.status_code == 200:
            print("[OK] Piper HTTP 服务正在运行")
            return True
        else:
            print(f"[ERROR] 服务返回错误状态: {response.status_code}")
            return False
    except Exception as e:
        print(f"[ERROR] 无法连接到服务: {e}")
        print("[INFO] 请确保 WSL2 中的 Piper HTTP 服务正在运行")
        return False

def run_rust_asr(wav_file: Path) -> Optional[str]:
    """使用 Rust 程序运行 ASR（通过编译好的二进制）"""
    print("\n[5/7] 执行 ASR 识别...")
    
    # 使用 cargo run 运行 ASR 测试程序
    # 注意：这里需要创建一个只做 ASR 的简单 Rust 程序
    # 或者使用现有的 whisper CLI 工具
    
    # 临时方案：使用 whisper-rs 的 CLI 工具（如果可用）
    # 或者提示用户手动运行 Rust ASR 测试
    
    print("[INFO] ASR 识别需要 Rust 程序，请手动运行：")
    print(f"  cd core/engine")
    print(f"  cargo run --example test_asr_only -- {wav_file}")
    print("\n[INFO] 或者使用其他 ASR 工具识别音频文件")
    
    # 返回模拟结果（实际应该从 ASR 结果中获取）
    return None

def run_rust_nmt(text: str) -> Optional[str]:
    """使用 Rust 程序运行 NMT"""
    print("\n[6/7] 执行 NMT 翻译...")
    
    print("[INFO] NMT 翻译需要 Rust 程序，请手动运行：")
    print(f"  cd core/engine")
    print(f"  cargo run --example test_nmt_only -- \"{text}\"")
    
    return None

def run_piper_tts(text: str, output_file: Path) -> bool:
    """使用 Piper HTTP 服务进行 TTS"""
    print("\n[7/7] 执行 TTS 合成（Piper HTTP）...")
    print(f"  说明: 合成中文语音用于回放源语言")
    
    try:
        request_data = {
            "text": text,
            "voice": "zh_CN-huayan-medium"
        }
        
        response = requests.post(
            "http://127.0.0.1:5005/tts",
            json=request_data,
            timeout=10
        )
        
        if response.status_code == 200:
            # 保存音频文件
            output_file.parent.mkdir(parents=True, exist_ok=True)
            with open(output_file, 'wb') as f:
                f.write(response.content)
            
            file_size = output_file.stat().st_size
            print(f"[OK] TTS 合成成功")
            print(f"  音频大小: {file_size} 字节")
            print(f"  文件路径: {output_file}")
            return True
        else:
            print(f"[ERROR] TTS 请求失败: {response.status_code}")
            print(f"  响应: {response.text}")
            return False
    except Exception as e:
        print(f"[ERROR] TTS 合成失败: {e}")
        return False

def main():
    print("=== 完整 S2S 流集成测试（Python 版本） ===\n")
    
    # 解析命令行参数
    if len(sys.argv) < 2:
        print("用法: python scripts/test_s2s_full_python.py <input_wav_file>")
        print("示例: python scripts/test_s2s_full_python.py test_output/s2s_flow_test.wav")
        sys.exit(1)
    
    input_wav = Path(sys.argv[1])
    if not input_wav.exists():
        print(f"[ERROR] 输入文件不存在: {input_wav}")
        sys.exit(1)
    
    print(f"[INFO] 输入文件: {input_wav}")
    
    # 检查服务
    print("\n[1/7] 检查 Piper HTTP 服务状态...")
    if not check_piper_service():
        sys.exit(1)
    
    # 检查模型目录
    print("\n[2/7] 检查模型目录...")
    project_root = Path(__file__).parent.parent
    asr_model_dir = project_root / "core" / "engine" / "models" / "asr" / "whisper-base"
    nmt_model_dir = project_root / "core" / "engine" / "models" / "nmt" / "marian-zh-en"
    
    if not asr_model_dir.exists():
        print(f"[ERROR] Whisper ASR 模型目录不存在: {asr_model_dir}")
        sys.exit(1)
    print(f"[OK] Whisper ASR 模型找到")
    
    if not nmt_model_dir.exists():
        print(f"[ERROR] Marian NMT 模型目录不存在: {nmt_model_dir}")
        sys.exit(1)
    print(f"[OK] Marian NMT 模型找到")
    
    # 检查输入文件
    print("\n[3/7] 检查输入音频文件...")
    if not input_wav.exists():
        print(f"[ERROR] 输入文件不存在: {input_wav}")
        sys.exit(1)
    print(f"[OK] 输入文件存在")
    
    # 由于 Rust 链接器问题，我们提供一个工作流程说明
    print("\n[4/7] 完整 S2S 流程说明...")
    print("\n由于 Rust 链接器兼容性问题，完整测试需要分步进行：")
    print("\n步骤 1: ASR 识别（需要 Rust 程序）")
    print("  由于 whisper-rs 的链接器问题，建议：")
    print("  - 使用其他 ASR 工具（如 OpenAI Whisper CLI）")
    print("  - 或等待链接器问题解决后运行 Rust ASR")
    print("\n步骤 2: NMT 翻译（需要 Rust 程序）")
    print("  运行: cd core/engine && cargo run --example test_nmt_only -- \"<中文文本>\"")
    print("\n步骤 3: TTS 合成（Python 可以完成）")
    print("  将使用 Piper HTTP 服务进行 TTS")
    
    # 如果用户提供了文本，直接进行 TTS 测试
    if len(sys.argv) >= 3:
        source_text = sys.argv[2]
        print(f"\n[使用提供的文本] 源文本: {source_text}")
        
        output_file = project_root / "test_output" / "s2s_full_python_test.wav"
        if run_piper_tts(source_text, output_file):
            print("\n=== 测试完成 ===")
            print(f"\n输出文件: {output_file}")
            print("\n注意: 这是部分测试（仅 TTS），完整流程需要 ASR 和 NMT")
        else:
            sys.exit(1)
    else:
        print("\n=== 测试说明 ===")
        print("\n由于 Rust 链接器问题，完整测试需要：")
        print("1. 手动运行 ASR 识别（或使用其他工具）")
        print("2. 手动运行 NMT 翻译")
        print("3. 使用此脚本进行 TTS 合成")
        print("\n或者，提供文本直接测试 TTS：")
        print(f"  python {sys.argv[0]} {input_wav} \"你好，欢迎使用语音翻译系统。\"")

if __name__ == "__main__":
    main()

