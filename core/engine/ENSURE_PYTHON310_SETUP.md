# 确保 TTS 服务在 Python 3.10 环境中启动

## 概述

YourTTS 服务需要在 **Python 3.10** 的 WSL 环境中运行，以确保 librosa 正常工作。

## 环境设置

### 1. 确认 Python 3.10 环境已创建

```bash
cd /mnt/d/Programs/github/lingua
ls -la venv-wsl-py310/
```

如果不存在，运行：
```bash
bash core/engine/scripts/setup_python310_env.sh
```

### 2. 验证环境

```bash
source venv-wsl-py310/bin/activate
python --version  # 应该显示: Python 3.10.19
python -c "import librosa; print('librosa:', librosa.__version__)"
```

## 启动服务的方法

### 方法 1: 使用统一的启动脚本（推荐）

```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/start_all_tts_wsl.sh
```

这会启动：
- Speaker Embedding 服务 (端口 5003)
- YourTTS 服务 (端口 5004)

### 方法 2: 分别启动服务

#### 启动 YourTTS 服务

```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/start_yourtts_wsl.sh
```

或使用专门的 Python 3.10 脚本：
```bash
bash core/engine/scripts/start_yourtts_wsl_py310.sh
```

#### 启动 Speaker Embedding 服务

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl-py310/bin/activate
python core/engine/scripts/speaker_embedding_service.py --gpu --port 5003 --host 0.0.0.0
```

### 方法 3: 从 Windows PowerShell 启动

```powershell
# 启动 YourTTS 服务（会自动使用 Python 3.10 环境）
.\core\engine\scripts\start_yourtts_wsl.ps1
```

## 启动脚本说明

### 已更新的脚本

1. **`start_yourtts_wsl.sh`** ✅
   - 已更新为使用 `venv-wsl-py310`
   - 自动验证 Python 版本

2. **`start_yourtts_wsl_py310.sh`** ✅
   - 专门用于 Python 3.10 环境
   - 强制使用 `venv-wsl-py310`

3. **`start_services.sh`** ✅
   - 已更新为使用 `venv-wsl-py310`
   - 同时启动两个服务

4. **`start_yourtts_wsl.ps1`** ✅
   - 从 Windows 调用，已更新为使用 Python 3.10 环境

5. **`start_all_tts_wsl.sh`** ✅ (新建)
   - 统一的启动脚本
   - 自动激活 Python 3.10 环境
   - 后台运行并保存 PID

## 验证服务是否正确启动

### 检查日志

启动后，检查日志确认使用的是 Python 3.10：

```bash
# 查看 YourTTS 日志
tail -f /tmp/yourtts.log

# 或在启动终端中查看实时输出
```

### 检查 Python 版本

如果服务正在运行，可以通过进程查看：

```bash
ps aux | grep yourtts_service.py
```

应该看到类似：
```
tinot ... /mnt/d/Programs/github/lingua/venv-wsl-py310/bin/python ... yourtts_service.py
```

### 测试服务

```bash
# 测试 YourTTS 服务
curl -X POST http://127.0.0.1:5004/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "test", "language": "en"}'

# 测试 Speaker Embedding 服务
curl -X POST http://127.0.0.1:5003/extract \
  -H "Content-Type: application/json" \
  -d '{"audio": [0.1, 0.2, 0.3]}'
```

## 常见问题

### Q: 服务启动失败，提示找不到虚拟环境

**A:** 确保已创建 Python 3.10 环境：
```bash
bash core/engine/scripts/setup_python310_env.sh
```

### Q: 启动后仍然是 Python 3.12

**A:** 检查启动脚本是否正确：
```bash
# 查看启动脚本内容
cat core/engine/scripts/start_yourtts_wsl.sh | grep venv-wsl-py310
```

### Q: librosa 仍然报错

**A:** 
1. 确认使用的是 Python 3.10 环境
2. 验证依赖版本：
   ```bash
   source venv-wsl-py310/bin/activate
   python -c "import numpy, numba, librosa; print(numpy.__version__, numba.__version__, librosa.__version__)"
   ```
   应该显示：`1.24.3 0.59.1 0.10.1`

## 快速检查清单

- [ ] Python 3.10 环境已创建（`venv-wsl-py310/` 目录存在）
- [ ] 环境中的 Python 版本是 3.10.19
- [ ] 已安装所有依赖（numpy 1.24.3, numba 0.59.1, librosa 0.10.1）
- [ ] 使用正确的启动脚本（使用 `venv-wsl-py310`）
- [ ] 服务启动后日志显示使用 Python 3.10

## 相关文件

- `core/engine/scripts/setup_python310_env.sh` - 环境设置脚本
- `core/engine/scripts/start_yourtts_wsl.sh` - YourTTS 启动脚本（已更新）
- `core/engine/scripts/start_yourtts_wsl_py310.sh` - Python 3.10 专用启动脚本
- `core/engine/scripts/start_services.sh` - 启动所有服务（已更新）
- `core/engine/scripts/start_all_tts_wsl.sh` - 统一的 TTS 服务启动脚本（新建）
- `core/engine/scripts/start_yourtts_wsl.ps1` - Windows PowerShell 启动脚本（已更新）

