# WSL systemd 警告修复

## 问题描述

启动服务时看到警告：
```
wsl: Failed to start the systemd user session for 'tinot'. See journalctl for more details.
NVIDIA GeForce RTX 4060 Laptop GPU
```

## 问题分析

### 这是什么？

这是 **WSL systemd 的警告**，不是错误。原因：
- WSL 2 默认不启用 systemd
- 某些命令尝试启动 systemd 会话但失败
- **不影响 GPU 的使用**

### GPU 实际可用吗？

从输出看：
- ✅ GPU 名称被检测到了：`NVIDIA GeForce RTX 4060 Laptop GPU`
- ✅ nvidia-smi 命令执行成功
- ⚠️ 只是有 systemd 相关的警告

## 影响评估

### ✅ 不影响 GPU 使用

- PyTorch 仍然可以使用 CUDA
- GPU 计算仍然可以工作
- 只是检测命令有警告输出

### 为什么会有警告？

WSL 中的某些命令（如通过 `wsl -d "Ubuntu-22.04"` 执行）可能尝试启动 systemd，但默认配置不支持。

## 解决方案

### 方案 1: 忽略警告（推荐）

如果 GPU 可以正常使用，可以直接忽略这个警告：
- ✅ 不影响功能
- ✅ 不需要额外配置
- ⚠️ 只是输出有些乱

### 方案 2: 修复启动脚本（已实现）

已更新 `start_yourtts_wsl.ps1` 脚本：
- 抑制 systemd 警告输出
- 改进 GPU 检测逻辑
- 即使有警告也能正确检测 GPU

### 方案 3: 启用 WSL systemd（可选）

如果想完全消除警告，可以启用 WSL systemd：

```bash
# 在 Windows 中编辑 WSL 配置
# C:\Users\<你的用户名>\.wslconfig

[wsl2]
systemd=true
```

然后重启 WSL：
```powershell
wsl --shutdown
# 然后重新打开 WSL
```

**注意**：启用 systemd 可能影响其他配置，需要测试。

## 验证 GPU 是否可用

### 方法 1: 在 WSL 中直接检查

```bash
# 在 WSL 终端中
nvidia-smi
```

如果没有错误，说明 GPU 可用。

### 方法 2: 在 Python 中检查

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl-py310/bin/activate
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU name:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

应该显示：
```
CUDA available: True
GPU name: NVIDIA GeForce RTX 4060 Laptop GPU
```

### 方法 3: 检查服务日志

启动服务后，查看服务日志：
- 应该看到 `✅ Using GPU: NVIDIA GeForce RTX 4060 Laptop GPU`
- 如果没有，会看到 `⚠️  GPU requested but not available, using CPU`

## 总结

### 当前状态

- ✅ **GPU 可用**：已检测到 GPU 名称
- ⚠️ **有警告**：systemd 相关警告（不影响使用）
- ✅ **功能正常**：可以忽略警告

### 建议

1. **无需担心**：警告不影响 GPU 使用
2. **已验证**：GPU 已被检测到
3. **继续使用**：服务应该可以正常使用 GPU

如果实际使用中发现 GPU 无法使用，可以：
- 检查服务日志
- 在 WSL 中直接运行 `nvidia-smi` 验证
- 在 Python 中检查 `torch.cuda.is_available()`

## 相关文件

- `core/engine/scripts/start_yourtts_wsl.ps1` - 已更新，改进 GPU 检测

