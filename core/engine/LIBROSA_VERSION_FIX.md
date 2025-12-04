# librosa 版本兼容性修复指南

## 问题描述

YourTTS 服务在使用 librosa 时出现 numba 与 numpy 不兼容的错误。当前环境：
- numba: 0.59.1
- numpy: 1.26.4
- librosa: 0.11.0（不兼容）

## 推荐的兼容版本组合

经过测试，以下版本组合具有良好的兼容性：

### 方案 1（推荐）
```bash
numpy==1.26.4
numba==0.59.1
librosa==0.10.1
```

### 方案 2（如果方案 1 仍有问题）
```bash
numpy==1.24.3
numba==0.59.1
librosa==0.10.1
```

## 快速修复步骤

### 方法 1：使用自动修复脚本（推荐）

```bash
cd /path/to/lingua
source venv-wsl/bin/activate  # 或激活你的虚拟环境
python3.10 core/engine/scripts/fix_python310_dependencies.py
```

脚本会自动尝试方案 1，如果失败则尝试方案 2。

### 方法 2：手动安装

```bash
# 激活虚拟环境
source venv-wsl/bin/activate  # 或激活你的虚拟环境

# 方案 1：使用 numpy 1.26.4
python3.10 -m pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall

# 如果方案 1 失败，尝试方案 2：使用 numpy 1.24.3
python3.10 -m pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall
```

## 验证安装

运行测试脚本验证：

```bash
python3.10 core/engine/scripts/test_python310_librosa.py
```

应该看到：
```
✅ numpy: 1.26.4 (或 1.24.3)
✅ numba: 0.59.1
✅ librosa: 0.10.1
✅ librosa.effects.time_stretch 测试通过
```

## 为什么选择 librosa 0.10.1？

1. **稳定性**：librosa 0.10.1 是一个稳定版本，与 numpy 1.24.x 和 1.26.x 都兼容良好
2. **兼容性**：librosa 0.11.0 可能引入了新的依赖或变化，导致与 numba 0.59.1 的兼容性问题
3. **功能完整**：librosa 0.10.1 包含 YourTTS 服务需要的所有功能（如 `time_stretch`）

## 相关文件

- `core/engine/PYTHON310_DEPENDENCIES_FIX.md` - 详细的依赖修复说明
- `core/engine/scripts/fix_python310_dependencies.py` - 自动修复脚本
- `core/engine/scripts/test_python310_librosa.py` - 测试脚本
- `core/engine/scripts/yourtts_service.py` - YourTTS 服务主文件

## 注意事项

1. **虚拟环境**：确保在正确的虚拟环境中安装（如 `venv-wsl`）
2. **Python 版本**：YourTTS 服务需要使用 Python 3.10
3. **重启服务**：安装完成后需要重启 YourTTS 服务才能生效

