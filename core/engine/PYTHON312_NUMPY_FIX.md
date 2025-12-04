# Python 3.12 环境下 numpy 安装问题修复

## 问题描述

在 Python 3.12 环境下安装 numpy 1.24.3 时失败：
```
ModuleNotFoundError: No module named 'distutils'
```

## 原因

1. **Python 3.12 移除了 distutils**：numpy 1.24.3 需要从源码编译，依赖 distutils
2. **numpy 1.24.3 不支持 Python 3.12**：需要 Python 3.8-3.11

## 解决方案

### 方案 1：使用兼容 Python 3.12 的版本（推荐）

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate

# 清除缓存
rm -rf ~/.cache/numba ~/.cache/pip

# 卸载现有版本
pip uninstall -y numpy numba librosa llvmlite

# 安装支持 Python 3.12 的版本
pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall
```

### 方案 2：使用修复脚本（自动检测 Python 版本）

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
bash core/engine/scripts/fix_python312_numpy_install.sh
```

### 方案 3：创建 Python 3.10 虚拟环境（最稳定）

```bash
cd /mnt/d/Programs/github/lingua

# 创建 Python 3.10 虚拟环境
python3.10 -m venv venv-wsl-py310

# 激活新环境
source venv-wsl-py310/bin/activate

# 安装依赖
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir
```

然后修改启动脚本使用新的虚拟环境。

## 版本兼容性

| Python 版本 | numpy 1.24.3 | numpy 1.26.4 | 说明 |
|------------|--------------|--------------|------|
| Python 3.10 | ✅ 支持 | ✅ 支持 | 推荐 |
| Python 3.11 | ✅ 支持 | ✅ 支持 | 可用 |
| Python 3.12 | ❌ 不支持 | ✅ 支持 | 使用 1.26.4 |

## 注意事项

1. **如果使用 numpy 1.26.4**：
   - 与 numba 0.59.1 可能存在兼容性问题
   - 代码已配置自动 fallback 到 scipy，服务仍可使用

2. **如果使用 Python 3.10**：
   - 可以使用 numpy 1.24.3（最稳定）
   - 推荐使用此方案

3. **当前代码状态**：
   - 已有自动 fallback 机制
   - librosa 失败时自动使用 scipy
   - 即使有兼容性问题，服务仍可正常运行

## 验证安装

```bash
python -c "
import numpy as np
import librosa
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa 测试通过')
except Exception as e:
    print(f'⚠️  librosa 测试失败（会使用 scipy fallback）: {e}')
"
```

