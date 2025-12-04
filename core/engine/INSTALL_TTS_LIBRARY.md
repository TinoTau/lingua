# 安装 TTS 库指南

## 问题

运行 `pip show TTS` 时提示 `WARNING: Package(s) not found: TTS`，说明 TTS 库未安装。

## 解决方案

### 方法 1：直接安装（推荐）

```bash
pip install TTS
```

**注意**：TTS 库较大（~500MB），安装可能需要一些时间。

### 方法 2：使用国内镜像（如果下载慢）

```bash
pip install TTS -i https://pypi.tuna.tsinghua.edu.cn/simple
```

### 方法 3：分步安装依赖（如果遇到问题）

```bash
# 1. 更新 pip
pip install --upgrade pip setuptools wheel

# 2. 安装基础依赖
pip install torch torchaudio

# 3. 安装 TTS
pip install TTS
```

## 验证安装

安装完成后，验证：

```bash
# 检查包信息
pip show TTS

# 测试导入
python -c "from TTS.api import TTS; print('TTS 库安装成功')"
```

## 常见问题

### 问题 1：安装失败，提示缺少依赖

**解决**：
```bash
# Windows: 确保已安装 Visual C++ Build Tools
# 或安装完整的 Visual Studio

# 然后重新安装
pip install --upgrade pip setuptools wheel
pip install TTS
```

### 问题 2：安装很慢或超时

**解决**：
```bash
# 使用国内镜像
pip install TTS -i https://pypi.tuna.tsinghua.edu.cn/simple

# 或增加超时时间
pip install TTS --timeout 300
```

### 问题 3：在虚拟环境中安装

**确保已激活虚拟环境**：

```powershell
# Windows PowerShell
.\venv\Scripts\Activate.ps1

# 然后安装
pip install TTS
```

### 问题 4：安装后仍然找不到

**检查 Python 环境**：

```bash
# 检查当前使用的 Python
python -c "import sys; print(sys.executable)"

# 确保在正确的环境中安装
python -m pip install TTS
```

## 安装后的使用

安装成功后，可以在 Python 中使用：

```python
from TTS.api import TTS

# 加载 YourTTS 模型
tts = TTS(model_name="tts_models/multilingual/multi-dataset/your_tts")
```

## 相关文档

- TTS 库官方文档：https://github.com/coqui-ai/TTS
- 安装问题：https://github.com/coqui-ai/TTS/issues

