# Python 3.10 依赖库版本修复说明

## 问题描述

YourTTS 服务在 Python 3.10 环境中运行时出现以下错误：
```
numpy.core._exceptions._UFuncNoLoopError: ufunc '_phasor_angles' did not contain a loop with signature matching types <class 'numpy.dtype[float64]'> -> None
```

## 根本原因

这是 `numpy` 和 `numba` 版本不兼容导致的：
- `numpy 2.x` 与 `numba 0.62.1` 不兼容
- `librosa.effects.time_stretch` 依赖 `numba` 编译的函数，版本不匹配会导致运行时错误

## 修复方案

### 1. 降级 numpy 到 1.x 版本

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
python3.10 -m pip install 'numpy<2.0' --upgrade
```

### 2. 降级 numba 到 0.59.1 版本

```bash
python3.10 -m pip install 'numba==0.59.1' --force-reinstall
```

### 3. 安装兼容的 librosa 版本

```bash
# 方案 1：使用 librosa 0.10.1（推荐）
python3.10 -m pip install 'librosa==0.10.1' --force-reinstall

# 如果方案 1 仍有问题，可以尝试降级 numpy 到 1.24.3
python3.10 -m pip install 'numpy==1.24.3' --force-reinstall
python3.10 -m pip install 'librosa==0.10.1' --force-reinstall
```

## 修复后的版本

推荐使用以下兼容版本组合：

**方案 1（推荐）**：使用较新的 numpy
- **numpy**: 1.26.4 (1.x，与 numba 兼容)
- **numba**: 0.59.1 (已从 0.62.1 降级)
- **librosa**: 0.10.1 (与 numpy 1.26.x 兼容良好)

**方案 2**：如果方案 1 仍有问题，使用更稳定的 numpy
- **numpy**: 1.24.3 (更稳定的版本)
- **numba**: 0.59.1
- **librosa**: 0.10.1

## 验证

运行测试脚本验证修复：

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
python3.10 core/engine/scripts/test_python310_librosa.py
```

应该看到：
```
✅ numpy: 1.26.4 (或 1.24.3)
✅ numba: 0.59.1
✅ librosa: 0.10.1
✅ librosa.effects.time_stretch 测试通过
```

## 注意事项

1. **Python 版本**: 确保 YourTTS 服务使用 Python 3.10 环境
2. **虚拟环境**: 使用 `venv-wsl` 虚拟环境
3. **启动脚本**: 确保启动脚本使用正确的 Python 版本

## 相关文件

- `core/engine/scripts/fix_python310_dependencies.py` - 自动修复脚本
- `core/engine/scripts/test_python310_librosa.py` - 测试脚本
- `core/engine/scripts/yourtts_service.py` - YourTTS 服务主文件

## 后续步骤

1. 重启 YourTTS 服务
2. 测试语速调整功能是否正常工作
3. 如果仍有问题，检查服务日志确认使用的 Python 版本

