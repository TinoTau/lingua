# WSL Ubuntu 22.04 安装指南

## 第一部分：安装 WSL 和 Ubuntu 22.04

### 步骤 1：检查 WSL 是否已安装

在 **Windows PowerShell（管理员权限）** 中运行：

```powershell
wsl --status
```

**如果输出显示 WSL 版本**，说明 WSL 已安装，跳到步骤 3。

**如果提示命令不存在**，需要先安装 WSL。

### 步骤 2：安装 WSL（如果还没有）

```powershell
# 以管理员身份运行 PowerShell，然后执行：
wsl --install
```

**注意**：
- 需要管理员权限
- 可能需要重启电脑
- 会自动安装 WSL 2 和默认的 Linux 发行版

### 步骤 3：查看可用的 Ubuntu 版本

```powershell
wsl --list --online
```

**预期输出**：
```
以下是可安装的有效分发的列表。
使用 'wsl --install -d <Distro>' 安装。

NAME            FRIENDLY NAME
Ubuntu          Ubuntu
Ubuntu-22.04    Ubuntu 22.04 LTS
Ubuntu-20.04    Ubuntu 20.04 LTS
...
```

### 步骤 4：安装 Ubuntu 22.04

```powershell
wsl --install -d Ubuntu-22.04
```

**安装过程**：
- 会下载 Ubuntu 22.04（可能需要几分钟）
- 首次启动时会要求设置用户名和密码

### 步骤 5：首次启动和设置

安装完成后，Ubuntu 会自动启动，或者手动启动：

```powershell
wsl -d Ubuntu-22.04
```

**首次启动需要**：
1. 创建用户名（建议使用小写字母，不要使用空格）
2. 设置密码（输入时不会显示，这是正常的）
3. 确认密码

**示例**：
```
Enter new UNIX username: tinot
New password: [输入密码，不显示]
Retype new password: [再次输入密码]
```

### 步骤 6：验证安装

在 Ubuntu 终端中运行：

```bash
# 查看系统信息
lsb_release -a

# 查看 Python 版本
python3 --version

# 查看当前用户
whoami
```

**预期输出**：
```
No LSB modules are available.
Distributor ID: Ubuntu
Description:    Ubuntu 22.04.x LTS
Release:        22.04
Codename:       jammy

Python 3.10.x
tinot
```

---

## 第二部分：配置 Ubuntu 环境

### 步骤 1：更新系统包

```bash
# 更新包列表
sudo apt update

# 升级系统包（可选，但推荐）
sudo apt upgrade -y
```

### 步骤 2：安装基础工具

```bash
# 安装常用工具
sudo apt install -y curl wget git build-essential

# 安装 Python 开发工具
sudo apt install -y python3-pip python3-venv python3-dev
```

### 步骤 3：验证 Python 版本

```bash
python3 --version
```

**预期输出**：`Python 3.10.x`（Ubuntu 22.04 默认是 3.10）

---

## 第三部分：进入项目目录

### 步骤 1：进入项目目录

```bash
# WSL 中的 Windows 路径映射
cd /mnt/d/Programs/github/lingua

# 验证目录
pwd
ls -la
```

**预期输出**：
```
/mnt/d/Programs/github/lingua
[显示项目文件列表]
```

### 步骤 2：验证可以访问项目文件

```bash
# 查看项目结构
ls -la core/engine/scripts/

# 应该能看到 yourtts_service.py 等文件
```

---

## 第四部分：创建虚拟环境

### 步骤 1：创建虚拟环境

```bash
# 确保在项目根目录
cd /mnt/d/Programs/github/lingua

# 创建虚拟环境（使用 Python 3.10）
python3.10 -m venv venv-wsl

# 如果系统默认是 3.10，也可以直接使用
# python3 -m venv venv-wsl
```

### 步骤 2：激活虚拟环境

```bash
source venv-wsl/bin/activate
```

**预期输出**：提示符变为 `(venv-wsl) tinot@Tino-Lenovo:/mnt/d/Programs/github/lingua$`

### 步骤 3：升级 pip

```bash
# 确保在虚拟环境中
pip install --upgrade pip
```

---

## 第五部分：安装依赖

### 步骤 1：安装基础依赖

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 安装基础依赖
pip install numpy soundfile flask
```

### 步骤 2：安装 PyTorch（GPU 版）

```bash
# 安装 PyTorch + CUDA 12.1
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**安装时间**：可能需要 10-30 分钟

### 步骤 3：安装其他依赖

```bash
# 安装 ONNX（可选）
pip install onnx onnxruntime

# 安装 TTS 库（YourTTS）
pip install TTS

# 安装 Piper TTS 依赖（如果使用）
pip install fastapi uvicorn pydantic
```

### 步骤 4：验证安装

```bash
# 验证 PyTorch 和 CUDA
python3 -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"

# 验证 TTS
python3 -c "from TTS.api import TTS; print('✅ TTS 库安装成功')"
```

---

## 第六部分：测试服务启动

### 步骤 1：测试 YourTTS 服务

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 启动服务
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ YourTTS model loaded successfully
🚀 Starting server on http://0.0.0.0:5004
```

### 步骤 2：健康检查（从 Windows）

在 Windows PowerShell 中：

```powershell
curl http://127.0.0.1:5004/health
```

**预期输出**：
```json
{"status":"ok","model_loaded":true}
```

---

## 故障排除

### 问题 1：WSL 安装失败

**解决**：
1. 确保以管理员身份运行 PowerShell
2. 启用虚拟化功能（在 BIOS 中）
3. 启用 Windows 功能：`启用或关闭 Windows 功能` → 勾选 `适用于 Linux 的 Windows 子系统` 和 `虚拟机平台`

### 问题 2：Ubuntu 22.04 安装失败

**解决**：
```powershell
# 检查 WSL 版本
wsl --status

# 如果版本是 1，升级到 WSL 2
wsl --set-default-version 2

# 重新安装
wsl --unregister Ubuntu-22.04
wsl --install -d Ubuntu-22.04
```

### 问题 3：无法访问 Windows 文件

**解决**：
```bash
# 检查挂载点
ls /mnt/

# 应该能看到 c, d 等驱动器
# 如果看不到，重启 WSL
exit
# 在 Windows 中
wsl --shutdown
wsl -d Ubuntu-22.04
```

### 问题 4：Python 3.10 不可用

**解决**：
```bash
# Ubuntu 22.04 默认是 3.10，如果不可用：
sudo apt update
sudo apt install -y python3.10 python3.10-venv python3.10-dev
```

### 问题 5：GPU 不可用

**解决**：
```bash
# 检查 WSL GPU 支持
nvidia-smi

# 如果不可用，需要：
# 1. 安装最新的 NVIDIA 驱动（支持 WSL）
# 2. 安装 NVIDIA Container Toolkit（如果使用 Docker）
```

---

## 快速参考

### 进入 WSL

```powershell
# 从 Windows
wsl -d Ubuntu-22.04

# 或直接
wsl
```

### 退出 WSL

```bash
exit
```

### 关闭 WSL

```powershell
# 在 Windows PowerShell 中
wsl --shutdown
```

### 激活虚拟环境

```bash
# 在 WSL 中
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
```

---

## 完成检查清单

- [ ] WSL 2 已安装
- [ ] Ubuntu 22.04 已安装
- [ ] 用户名和密码已设置
- [ ] 系统包已更新
- [ ] Python 3.10 可用
- [ ] 项目目录可访问
- [ ] 虚拟环境 `venv-wsl` 已创建
- [ ] 所有依赖已安装
- [ ] PyTorch GPU 可用
- [ ] TTS 库安装成功
- [ ] YourTTS 服务能启动

完成以上所有步骤后，WSL 环境就配置完成了！

