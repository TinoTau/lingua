# TTS 库安装状态说明

## 当前情况分析

### 您的情况

1. ✅ **YourTTS 服务在 WSL 中运行成功**
2. ❌ **在 Windows 中运行 `pip show TTS` 找不到包**
3. ❓ **在 WSL 中也可能找不到（需要确认）**

### 可能的原因

#### 原因 1：服务脚本自动安装（最可能）

`yourtts_service.py` 脚本中有自动安装逻辑：

```python
def check_and_install_tts():
    """检查并安装 TTS 模块"""
    try:
        import TTS
        return True
    except ImportError:
        print("⚠️  TTS module not found. Attempting to install...")
        # 自动安装 TTS
        subprocess.check_call([sys.executable, "-m", "pip", "install", "TTS"])
        return True
```

**这意味着**：
- 服务启动时，如果 TTS 未安装，脚本会自动安装
- 安装可能是在服务启动时临时安装的
- 或者安装在了特定的 Python 环境中

#### 原因 2：安装在虚拟环境中

TTS 可能安装在：
- WSL 中的虚拟环境
- 特定的 Python 环境（不是默认的 python3）

#### 原因 3：使用系统 Python vs 用户 Python

- 系统 Python：`/usr/bin/python3`
- 用户 Python：`~/.local/bin/python3`
- 虚拟环境 Python：`venv/bin/python3`

## 如何确认 TTS 安装位置

### 方法 1：检查服务启动时的输出

查看服务启动时的日志，看是否有：
- `⚠️  TTS module not found. Attempting to install...`
- `✅ TTS module installed successfully`

### 方法 2：在 WSL 中检查

```bash
# 进入 WSL
wsl

# 检查所有 Python 环境中的 TTS
python3 -m pip show TTS
python3 -c "import sys; print(sys.executable)"

# 检查虚拟环境（如果有）
source venv/bin/activate  # 如果有虚拟环境
python -m pip show TTS
```

### 方法 3：检查服务使用的 Python

```bash
# 查看服务进程使用的 Python
wsl ps aux | grep yourtts_service
```

## 解决方案

### 如果 TTS 确实未安装（但服务能运行）

**可能的情况**：
- 服务在启动时自动安装了 TTS
- 但安装在了特定的环境中

**建议**：
1. **保持现状**：如果服务能正常运行，不需要额外操作
2. **如果需要导出 ONNX**：在 WSL 中运行导出脚本，脚本会自动安装 TTS（如果需要）

### 如果需要手动安装 TTS（在 WSL 中）

```bash
# 在 WSL 中安装
wsl python3 -m pip install TTS

# 或使用虚拟环境
wsl
source venv/bin/activate  # 如果有虚拟环境
pip install TTS
```

### 如果需要在 Windows 中安装（通常不需要）

```powershell
# 仅在需要在 Windows 中开发 Python 脚本时
pip install TTS
```

## 验证 TTS 是否可用

### 在 WSL 中验证

```bash
# 方法 1：直接测试
wsl python3 -c "from TTS.api import TTS; print('✅ TTS 可用')"

# 方法 2：运行服务脚本的检查
wsl python3 core/engine/scripts/yourtts_service.py --check-deps
```

### 在服务运行时验证

如果服务正在运行，说明 TTS 肯定可用（否则服务无法启动）。

## 总结

### 对于运行服务

✅ **不需要担心**：如果服务能运行，TTS 肯定已安装（可能在特定环境中）

### 对于导出 ONNX

✅ **在 WSL 中运行导出脚本**：
```bash
wsl python3 core/engine/scripts/export_yourtts_to_onnx.py
```

脚本会自动处理 TTS 的安装（如果需要）。

### 对于开发

⚠️ **如果需要**：在相应的环境中安装 TTS
- WSL 开发：在 WSL 中安装
- Windows 开发：在 Windows 中安装（通常不需要）

## 建议

**保持现状即可**：
- ✅ 服务能运行 = TTS 已安装（在某个环境中）
- ✅ 导出 ONNX 时，脚本会自动处理依赖
- ✅ 不需要在 Windows 中安装 TTS（服务在 WSL 中运行）

