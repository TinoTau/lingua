# YourTTS 模型重新下载原因分析

## 问题

之前能正常运行，但现在切换到 Python 3.10 环境后需要重新下载模型。

## 原因分析

### 1. 代码逻辑流程

从 `yourtts_service.py` 的 `load_model` 函数可以看到：

```python
# 步骤 1: 尝试使用本地路径加载
tts_model = TTS(model_path=str(model_path), ...)  # 如果失败

# 步骤 2: 检查 model.pth 文件
model_file = model_path / "model.pth"
if model_file.exists():
    # 使用模型名称加载（会使用缓存）
    tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", ...)
else:
    # 如果 model.pth 不存在，也会使用模型名称下载
    raise FileNotFoundError(...)
```

### 2. 实际发生的情况

根据日志：
```
⚠️  TTS API loading failed, trying direct load...
> Downloading model to /home/tinot/.local/share/tts/...
```

说明：
1. 尝试从 `core/engine/models/tts/your_tts` 加载失败
2. 检查 `model.pth` 文件可能不存在或无效
3. Fallback 到使用模型名称 `"tts_models/multilingual/multi-dataset/your_tts"`
4. TTS 库检查缓存目录，但可能因为某种原因找不到或无效

### 3. 可能的原因

#### 原因 A：本地模型路径不存在或无效

- `core/engine/models/tts/your_tts` 目录可能不存在
- 或者目录存在但没有 `model.pth` 文件
- 导致代码 fallback 到模型名称下载

#### 原因 B：TTS 库缓存问题

- 旧环境中的模型在：`~/.local/share/tts/`
- 新环境应该能找到同样的缓存（用户目录相同）
- 但可能因为：
  - 缓存索引损坏
  - 模型版本不匹配
  - 权限问题

#### 原因 C：TTS 库版本更新

- 新环境中安装的 TTS 版本可能与旧环境不同
- 新版本可能使用不同的模型格式或路径

## 检查步骤

### 1. 检查本地模型路径

```bash
# 在 WSL 中
ls -la /mnt/d/Programs/github/lingua/core/engine/models/tts/your_tts/
```

### 2. 检查 TTS 缓存目录

```bash
# 检查是否有已下载的模型
ls -la ~/.local/share/tts/
```

### 3. 检查 TTS 版本

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl-py310/bin/activate
pip show TTS
```

## 解决方案

### 方案 1：使用已存在的缓存（推荐）

如果缓存目录中有模型，可以：

1. **设置环境变量指向缓存**：
   ```bash
   export TTS_HOME=~/.local/share/tts
   ```

2. **或者修改代码使用缓存路径**：
   检查代码中的模型路径逻辑，确保使用缓存

### 方案 2：创建模型路径的符号链接

如果模型在缓存中，可以创建符号链接：

```bash
# 在 WSL 中
mkdir -p /mnt/d/Programs/github/lingua/core/engine/models/tts/
cd /mnt/d/Programs/github/lingua/core/engine/models/tts/

# 如果缓存中有模型，创建符号链接
if [ -d ~/.local/share/tts/tts_models--multilingual--multi-dataset--your_tts ]; then
    ln -s ~/.local/share/tts/tts_models--multilingual--multi-dataset--your_tts your_tts
fi
```

### 方案 3：等待下载完成（最简单）

如果下载速度可以接受：
- 等待当前下载完成
- 模型会保存到 `~/.local/share/tts/`
- 下次启动时会直接使用

## 为什么之前能运行？

可能的原因：
1. **旧环境中模型已经下载**：在 `venv-wsl` 环境中首次使用时已下载
2. **缓存被共享**：TTS 库使用用户目录缓存，环境切换不影响缓存位置
3. **但新环境检测失败**：可能因为：
   - 代码逻辑先检查本地路径
   - 本地路径不存在
   - 虽然缓存中有，但代码没有正确使用缓存

## 建议

### 短期（当前）

**等待下载完成**：
- 模型正在下载（约 3-4 分钟）
- 下载完成后会保存到缓存
- 下次启动会直接使用

### 长期（优化）

1. **检查并修复模型路径逻辑**：
   - 确保代码能正确使用缓存目录
   - 或者在本地路径不存在时直接使用缓存

2. **预先下载模型**：
   - 在项目目录中预先下载模型
   - 确保 `core/engine/models/tts/your_tts/model.pth` 存在

3. **环境变量配置**：
   - 设置 `TTS_HOME` 环境变量
   - 统一模型路径管理

## 总结

**为什么会重新下载？**

1. 代码首先检查本地路径 `core/engine/models/tts/your_tts`
2. 该路径可能不存在或无效
3. Fallback 到使用模型名称，触发下载
4. 虽然缓存中可能有旧模型，但代码逻辑没有优先使用缓存

**解决方法**：
- 当前：等待下载完成（最简单）
- 长期：优化代码逻辑，优先使用缓存或本地模型

