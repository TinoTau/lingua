#!/usr/bin/env python3
"""
修复 Python 3.10 环境下的依赖库版本兼容性问题

问题：numpy 2.x 与 numba 0.62.1 不兼容
解决方案：降级 numpy 到 1.x 版本
"""

import subprocess
import sys
import os

def run_command(cmd):
    """运行命令并返回输出"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)

def check_versions(python_executable=None):
    """检查当前版本"""
    if python_executable is None:
        python_executable = sys.executable
    
    print("=" * 60)
    print(f"  检查依赖库版本 (Python: {python_executable})")
    print("=" * 60)
    print()
    
    cmd = f"{python_executable} -c \"import numpy; import numba; import librosa; print(f'numpy: {{numpy.__version__}}'); print(f'numba: {{numba.__version__}}'); print(f'librosa: {{librosa.__version__}}')\""
    success, stdout, stderr = run_command(cmd)
    
    if success:
        print(stdout)
        # 解析版本
        lines = stdout.strip().split('\n')
        versions = {}
        for line in lines:
            if ':' in line:
                key, value = line.split(':', 1)
                versions[key.strip()] = value.strip()
        return versions
    else:
        print(f"❌ 导入失败: {stderr}")
        return None

def fix_versions(python_executable=None):
    """安装兼容的版本组合"""
    if python_executable is None:
        python_executable = sys.executable
    
    print()
    print("=" * 60)
    print("  修复依赖库版本兼容性")
    print("=" * 60)
    print()
    
    # 先尝试方案 1：numpy 1.26.4 + librosa 0.10.1
    print("方案 1：安装 numpy 1.26.4 + librosa 0.10.1 + numba 0.59.1...")
    cmds = [
        f"{python_executable} -m pip install 'numpy==1.26.4' --upgrade",
        f"{python_executable} -m pip install 'numba==0.59.1' --force-reinstall",
        f"{python_executable} -m pip install 'librosa==0.10.1' --force-reinstall"
    ]
    
    all_success = True
    for cmd in cmds:
        success, stdout, stderr = run_command(cmd)
        if not success:
            print(f"❌ 安装失败: {cmd}")
            if stderr:
                print(stderr)
            all_success = False
            break
        if stdout:
            print(stdout.strip())
    
    if all_success:
        print("✅ 方案 1 安装成功")
        return True, 1
    
    # 如果方案 1 失败，尝试方案 2：numpy 1.24.3 + librosa 0.10.1
    print()
    print("方案 1 失败，尝试方案 2：numpy 1.24.3 + librosa 0.10.1 + numba 0.59.1...")
    cmds = [
        f"{python_executable} -m pip install 'numpy==1.24.3' --upgrade",
        f"{python_executable} -m pip install 'numba==0.59.1' --force-reinstall",
        f"{python_executable} -m pip install 'librosa==0.10.1' --force-reinstall"
    ]
    
    all_success = True
    for cmd in cmds:
        success, stdout, stderr = run_command(cmd)
        if not success:
            print(f"❌ 安装失败: {cmd}")
            if stderr:
                print(stderr)
            all_success = False
            break
        if stdout:
            print(stdout.strip())
    
    if all_success:
        print("✅ 方案 2 安装成功")
        return True, 2
    
    return False, 0

def fix_numpy_version(python_executable=None):
    """降级 numpy 到 1.x 版本（保留向后兼容）"""
    return fix_versions(python_executable)[0]

def verify_fix(python_executable=None):
    """验证修复结果"""
    if python_executable is None:
        python_executable = sys.executable
    
    print()
    print("=" * 60)
    print("  验证修复结果")
    print("=" * 60)
    print()
    
    versions = check_versions(python_executable)
    if versions is None:
        return False
    
    numpy_version = versions.get('numpy', '')
    if not numpy_version:
        print("❌ 无法获取 numpy 版本")
        return False
    
    # 检查 numpy 版本
    major_version = int(numpy_version.split('.')[0])
    if major_version >= 2:
        print()
        print("⚠️  警告: numpy 版本仍然是 2.x，可能需要手动降级")
        return False
    
    # 测试 librosa.effects.time_stretch 是否工作
    print()
    print("测试 librosa.effects.time_stretch...")
    test_cmd = f"""{python_executable} -c "
import numpy as np
import librosa
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
except Exception as e:
    print(f'❌ librosa.effects.time_stretch 测试失败: {{e}}')
    exit(1)
\""""
    
    success, stdout, stderr = run_command(test_cmd)
    if success:
        print(stdout)
        return True
    else:
        print(stderr)
        return False

def main():
    print()
    print("=" * 60)
    print("  Python 3.10 依赖库版本修复工具")
    print("=" * 60)
    print()
    
    # 检查是否有 Python 3.10 环境
    project_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    python310_path = os.path.join(project_root, "venv-wsl", "bin", "python3.10")
    
    # 检查 Python 3.10 是否存在
    if os.path.exists(python310_path):
        print(f"✅ 找到 Python 3.10: {python310_path}")
        python_executable = python310_path
    else:
        print(f"⚠️  未找到 Python 3.10，使用当前 Python: {sys.executable}")
        python_executable = sys.executable
    
    # 检查 Python 版本
    version_cmd = f"{python_executable} --version"
    success, stdout, stderr = run_command(version_cmd)
    if success:
        print(f"Python 版本: {stdout.strip()}")
    print()
    
    # 检查当前版本
    versions = check_versions(python_executable)
    
    if versions is None:
        print("❌ 无法检查版本，请先安装依赖库")
        return 1
    
    # 检查是否需要修复
    numpy_version = versions.get('numpy', '')
    if not numpy_version:
        print("❌ 无法获取 numpy 版本")
        return 1
    
    major_version = int(numpy_version.split('.')[0])
    
    # 检查是否需要修复
    numba_version = versions.get('numba', '')
    librosa_version = versions.get('librosa', '')
    
    needs_fix = major_version >= 2 or numba_version != '0.59.1' or librosa_version not in ['0.10.1', '0.10.2']
    
    if needs_fix:
        print()
        if major_version >= 2:
            print(f"⚠️  检测到 numpy {numpy_version} (2.x)，与 numba 不兼容")
        if numba_version != '0.59.1':
            print(f"⚠️  检测到 numba {numba_version}，需要 0.59.1")
        if librosa_version not in ['0.10.1', '0.10.2']:
            print(f"⚠️  检测到 librosa {librosa_version}，推荐使用 0.10.1")
        print("   将安装兼容版本组合...")
        print()
        
        success, scheme = fix_versions(python_executable)
        if success:
            if verify_fix(python_executable):
                print()
                print("=" * 60)
                print(f"  ✅ 修复完成（使用方案 {scheme}）！请重启 YourTTS 服务")
                print("=" * 60)
                return 0
            else:
                print()
                print("=" * 60)
                print("  ⚠️  修复可能未完全成功，请检查错误信息")
                print("=" * 60)
                return 1
        else:
            print()
            print("=" * 60)
            print("  ❌ 修复失败")
            print("=" * 60)
            return 1
    else:
        print()
        print("✅ numpy 版本已经是 1.x，无需修复")
        
        # 仍然验证一下功能
        if verify_fix(python_executable):
            print()
            print("=" * 60)
            print("  ✅ 所有依赖库版本正常")
            print("=" * 60)
            return 0
        else:
            print()
            print("=" * 60)
            print("  ⚠️  版本正常但功能测试失败，可能需要重新安装")
            print("=" * 60)
            return 1

if __name__ == "__main__":
    sys.exit(main())
