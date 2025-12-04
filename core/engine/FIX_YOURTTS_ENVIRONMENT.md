# 修复 YourTTS 服务环境问题

## 问题描述

从日志路径 `/mnt/d/Programs/github/lingua/venv-wsl/lib/python3.10/` 可以看出，服务仍然在使用**旧的 `venv-wsl` 环境**，而不是新的 `venv-wsl-py310` 环境。

这导致：
- ❌ 仍然出现 `_phasor_angles` 错误
- ❌ librosa 无法正常工作

## 原因

服务可能在以下情况下启动：
1. 使用旧的启动脚本（更新前启动）
2. 手动启动时没有激活正确的环境
3. 启动脚本没有正确执行

## 解决步骤

### 1. 停止当前运行的服务

```bash
# 在 WSL 中查找并停止 YourTTS 服务
ps aux | grep yourtts_service.py | grep -v grep

# 如果发现有进程，停止它
kill <PID>

# 或使用
pkill -f yourtts_service.py
```

### 2. 检查环境配置

运行检查脚本：
```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/check_and_fix_yourtts_env.sh
```

### 3. 确保使用正确的启动脚本

**方法 1：使用更新后的启动脚本（推荐）**

```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/start_yourtts_wsl.sh
```

这个脚本已经更新为使用 `venv-wsl-py310`。

**方法 2：手动启动（确保使用正确环境）**

```bash
cd /mnt/d/Programs/github/lingua

# 激活 Python 3.10 环境
source venv-wsl-py310/bin/activate

# 验证 Python 版本
python --version  # 应该显示 Python 3.10.19

# 验证依赖版本
python -c "import numpy, numba, librosa; print(f'numpy: {numpy.__version__}, numba: {numba.__version__}, librosa: {librosa.__version__}')"
# 应该显示: numpy: 1.24.3, numba: 0.59.1, librosa: 0.10.1

# 启动服务
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 0.0.0.0
```

### 4. 验证修复

启动服务后，检查日志中的路径：
- ✅ 正确：`/mnt/d/Programs/github/lingua/venv-wsl-py310/lib/python3.10/`
- ❌ 错误：`/mnt/d/Programs/github/lingua/venv-wsl/lib/python3.10/`

发送测试请求后，应该看到：
- ✅ `✅ Speech rate adjusted using librosa` (而不是错误)
- ❌ 不再出现 `_phasor_angles` 错误

## 快速修复命令

```bash
# 1. 停止旧服务
pkill -f yourtts_service.py

# 2. 等待几秒
sleep 2

# 3. 使用正确环境启动
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/start_yourtts_wsl.sh
```

## 检查清单

- [ ] 已停止旧服务进程
- [ ] 确认使用 `venv-wsl-py310` 环境启动
- [ ] 日志路径显示 `venv-wsl-py310`
- [ ] librosa 测试通过
- [ ] 服务日志中不再有 `_phasor_angles` 错误

