# Piper TTS 环境状态说明

## 环境配置

### 当前状态

**Piper TTS 使用独立的虚拟环境**，**没有**切换到项目的 Python 3.10 环境。

### 环境详情

1. **独立虚拟环境**：
   - 位置：`$HOME/piper_env/.venv`
   - 由 `scripts/wsl2_piper/install_piper_in_wsl.sh` 创建
   - 与项目的 `venv-wsl` 和 `venv-wsl-py310` 完全独立

2. **启动方式**：
   - 启动脚本：`scripts/wsl2_piper/start_piper_service.sh`
   - 自动激活：`$HOME/piper_env/.venv`
   - 使用：`piper-tts[http]` Python 包

3. **Python 版本**：
   - 使用系统默认的 `python3`
   - 安装时创建虚拟环境时使用的是系统 Python 版本
   - 未强制要求 Python 3.10

## 是否需要切换到 Python 3.10？

### 分析

**通常不需要**，原因：

1. **独立运行**：
   - Piper TTS 不依赖 `librosa`、`numba` 等有版本冲突的库
   - 主要依赖：`piper-tts[http]`、`fastapi`、`uvicorn`、`onnxruntime`
   - 这些依赖与 Python 3.12 兼容

2. **无版本冲突**：
   - YourTTS 的问题是因为 `numba`、`librosa` 和 `numpy` 的版本兼容性
   - Piper TTS 不使用这些库

3. **性能影响**：
   - Piper TTS 主要使用 ONNX Runtime（C++），Python 只是 HTTP 接口
   - Python 版本对性能影响很小

### 如果需要统一环境

如果希望所有服务都使用 Python 3.10，可以：

1. **修改启动脚本**：
   编辑 `scripts/wsl2_piper/start_piper_service.sh`，改为使用 `venv-wsl-py310`

2. **重新安装**：
   ```bash
   # 在 Python 3.10 环境中重新安装 piper-tts
   source venv-wsl-py310/bin/activate
   pip install "piper-tts[http]"
   ```

3. **更新启动脚本**：
   将 `source $HOME/piper_env/.venv/bin/activate` 改为 `source venv-wsl-py310/bin/activate`

## 推荐做法

### 当前配置（推荐）

**保持 Piper TTS 使用独立环境**：
- ✅ 环境隔离，避免依赖冲突
- ✅ 不影响其他服务
- ✅ 独立管理和更新

### 统一环境（可选）

如果需要统一所有服务到 Python 3.10：
- 可以修改启动脚本
- 需要重新安装 `piper-tts[http]` 到 Python 3.10 环境
- 测试确保功能正常

## 检查当前环境

在 WSL 中检查：

```bash
# 检查 Piper TTS 使用的 Python 版本
cd ~/piper_env
source .venv/bin/activate
python --version

# 检查依赖
pip list | grep -E "piper|onnxruntime|fastapi"
```

## 总结

- **Piper TTS 使用独立环境**：`$HOME/piper_env/.venv`
- **不需要切换到 Python 3.10**：除非有特定需求
- **YourTTS 已切换到 Python 3.10**：`venv-wsl-py310`
- **两者可以共存**：使用不同的虚拟环境

