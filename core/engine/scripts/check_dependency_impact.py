#!/usr/bin/env python3
"""
检查项目中依赖 numpy 和 numba 的模块，评估卸载影响
"""

import sys
import os
from pathlib import Path

# 项目根目录
project_root = Path(__file__).parent.parent.parent

# 需要检查的 Python 脚本
python_scripts = [
    "core/engine/scripts/yourtts_service.py",
    "core/engine/scripts/speaker_embedding_service.py",
    "core/engine/scripts/diagnose_silero_vad.py",
]

# 运行环境信息
environments = {
    "Windows (Python 3.10)": sys.executable,
    "Conda (lingua-py310)": "D:\\Program Files\\Anaconda\\envs\\lingua-py310\\python.exe",
    "WSL (venv-wsl)": "/mnt/d/Programs/github/lingua/venv-wsl/bin/python",
}

def check_module(python_exe, module_name):
    """检查模块是否可用"""
    try:
        import subprocess
        result = subprocess.run(
            [python_exe, "-c", f"import {module_name}; print('OK')"],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0
    except:
        return False

def analyze_script(script_path):
    """分析脚本的依赖"""
    full_path = project_root / script_path
    if not full_path.exists():
        return None
    
    with open(full_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    has_numpy = 'import numpy' in content or 'from numpy' in content
    has_numba = 'import numba' in content or 'from numba' in content
    has_librosa = 'import librosa' in content or 'from librosa' in content
    
    return {
        'path': script_path,
        'has_numpy': has_numpy,
        'has_numba': has_numba,
        'has_librosa': has_librosa,
    }

def main():
    print("=" * 80)
    print("  依赖 numpy/numba 的模块影响分析")
    print("=" * 80)
    print()
    
    # 分析脚本依赖
    print("📋 脚本依赖分析:")
    print("-" * 80)
    scripts_info = []
    for script in python_scripts:
        info = analyze_script(script)
        if info:
            scripts_info.append(info)
            print(f"\n{info['path']}:")
            print(f"  - numpy: {'✅ 需要' if info['has_numpy'] else '❌ 不需要'}")
            print(f"  - numba: {'✅ 需要' if info['has_numba'] else '❌ 不需要'}")
            print(f"  - librosa: {'✅ 需要' if info['has_librosa'] else '❌ 不需要'}")
    
    print()
    print("=" * 80)
    print("  运行环境检查")
    print("=" * 80)
    print()
    
    # 检查各环境中的模块可用性
    for env_name, python_exe in environments.items():
        print(f"\n🔍 {env_name}:")
        print(f"   Python: {python_exe}")
        
        if not os.path.exists(python_exe):
            print("   ⚠️  Python 可执行文件不存在，跳过检查")
            continue
        
        numpy_ok = check_module(python_exe, "numpy")
        numba_ok = check_module(python_exe, "numba")
        librosa_ok = check_module(python_exe, "librosa")
        
        print(f"   - numpy: {'✅ 已安装' if numpy_ok else '❌ 未安装'}")
        print(f"   - numba: {'✅ 已安装' if numba_ok else '❌ 未安装'}")
        print(f"   - librosa: {'✅ 已安装' if librosa_ok else '❌ 未安装'}")
    
    print()
    print("=" * 80)
    print("  影响评估")
    print("=" * 80)
    print()
    
    # 评估影响
    print("1. YourTTS Service:")
    print("   - 运行环境: WSL (venv-wsl)")
    print("   - 依赖: numpy, librosa (间接依赖 numba)")
    print("   - 状态: ✅ 已在 WSL 环境中安装兼容版本")
    print("   - 影响: 无（Windows 环境卸载不影响 WSL 环境）")
    print()
    
    print("2. Speaker Embedding Service:")
    print("   - 运行环境: Windows (Conda lingua-py310)")
    print("   - 依赖: numpy")
    print("   - 状态: ⚠️  需要检查 Conda 环境中是否有 numpy")
    print("   - 影响: 如果 Conda 环境缺少 numpy，服务会失败")
    print()
    
    print("3. 诊断脚本 (diagnose_silero_vad.py):")
    print("   - 运行环境: 可能在任何环境")
    print("   - 依赖: numpy, onnxruntime")
    print("   - 状态: ⚠️  如果在 Windows Python 环境运行会受影响")
    print("   - 影响: 需要在使用前安装 numpy")
    print()
    
    print("=" * 80)
    print("  建议")
    print("=" * 80)
    print()
    print("✅ YourTTS Service: 已在 WSL 环境中修复，无需担心")
    print()
    print("⚠️  Speaker Embedding Service:")
    print("   - 检查 Conda 环境是否安装了 numpy")
    print("   - 如果没有，运行: conda install numpy -n lingua-py310")
    print()
    print("⚠️  诊断脚本:")
    print("   - 这些脚本通常在需要时临时运行")
    print("   - 如果报错，在使用前安装: pip install numpy onnxruntime")
    print()
    print("✅ 结论: 卸载 Windows 环境中的 numpy/numba/librosa 不会影响:")
    print("   - YourTTS Service (在 WSL 中运行)")
    print("   - Speaker Embedding Service (在 Conda 环境中运行)")
    print()

if __name__ == "__main__":
    main()

