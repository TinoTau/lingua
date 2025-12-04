# WSL 和 Windows Python 环境说明

## 重要概念

**WSL 和 Windows 的 Python 环境是完全独立的！**

- ✅ **WSL 中的 Python**：Linux 环境，独立的包管理
- ✅ **Windows 中的 Python**：Windows 环境，独立的包管理
- ❌ **它们不共享**：在 WSL 中安装的包不会出现在 Windows 中

## 当前情况

### WSL 环境（YourTTS 服务运行的地方）

```bash
# 在 WSL 中检查
wsl python3 -m pip show TTS
wsl python3 -c "from TTS.api import TTS; print('TTS 可用')"
```

**状态**：✅ TTS 已安装（因为服务能运行）

### Windows 环境（您运行 pip show 的地方）

```powershell
# 在 Windows PowerShell 中
pip show TTS
```

**状态**：❌ TTS 未安装（这是正常的！）

## 为什么这样设计？

1. **环境隔离**：WSL 是 Linux 环境，Windows 是 Windows 环境
2. **包兼容性**：Linux 包和 Windows 包可能不兼容
3. **独立性**：两个环境可以有不同的 Python 版本和包版本

## 实际使用场景

### 场景 1：运行 YourTTS 服务（在 WSL 中）

```bash
# 在 WSL 中运行（TTS 已安装）
wsl python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

✅ **不需要在 Windows 中安装 TTS**

### 场景 2：导出 ONNX 模型（可选，在 WSL 中）

```bash
# 在 WSL 中运行（TTS 已安装）
wsl python3 core/engine/scripts/export_yourtts_to_onnx.py
```

✅ **在 WSL 中运行即可**

### 场景 3：在 Windows 中开发 Python 脚本（如果需要）

```powershell
# 在 Windows 中安装（如果需要）
pip install TTS
```

⚠️ **通常不需要**，因为服务在 WSL 中运行

## 验证 WSL 中的 TTS 安装

### 方法 1：检查包信息

```powershell
# 从 Windows 检查 WSL 中的 TTS
wsl python3 -m pip show TTS
```

### 方法 2：测试导入

```powershell
# 从 Windows 测试 WSL 中的 TTS
wsl python3 -c "from TTS.api import TTS; print('✅ TTS 在 WSL 中可用')"
```

### 方法 3：直接在 WSL 中检查

```bash
# 进入 WSL
wsl

# 在 WSL 中检查
python3 -m pip show TTS
python3 -c "from TTS.api import TTS; print('✅ TTS 可用')"
```

## 总结

| 环境 | TTS 安装状态 | 用途 |
|------|------------|------|
| **WSL** | ✅ 已安装 | 运行 YourTTS 服务 |
| **Windows** | ❌ 未安装 | 通常不需要（服务在 WSL 中运行） |

## 结论

**您的 TTS 库已经正确安装在 WSL 中！**

- ✅ YourTTS 服务能在 WSL 中运行，说明 TTS 已安装
- ✅ 在 Windows 中找不到 TTS 是正常的（因为服务在 WSL 中运行）
- ✅ 不需要在 Windows 中安装 TTS（除非您想在 Windows 中开发 Python 脚本）

## 如果需要导出 ONNX 模型

在 WSL 中运行导出脚本：

```bash
# 在 WSL 中
wsl python3 core/engine/scripts/export_yourtts_to_onnx.py
```

或者：

```bash
# 进入 WSL
wsl

# 在 WSL 中运行
cd /mnt/d/Programs/github/lingua
python3 core/engine/scripts/export_yourtts_to_onnx.py
```

