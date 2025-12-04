# 安装 Ubuntu 22.04 LTS（WSL）

## 当前状态

- ✅ WSL 2 已安装
- ✅ 已有 Ubuntu 24.04 运行中
- ⚠️ 需要安装 Ubuntu 22.04（用于 TTS 服务，Python 3.10）

## 安装步骤

### 步骤 1：安装 Ubuntu 22.04

在 **PowerShell** 中运行：

```powershell
wsl --install -d Ubuntu-22.04
```

**安装过程**：
- 会下载 Ubuntu 22.04（可能需要几分钟）
- 首次启动时会要求设置用户名和密码

### 步骤 2：首次启动和设置

安装完成后，启动 Ubuntu 22.04：

```powershell
wsl -d Ubuntu-22.04
```

**首次启动需要**：
1. 创建用户名（建议使用小写字母，不要使用空格）
   - 可以与 Ubuntu 24.04 使用相同的用户名，也可以不同
2. 设置密码（输入时不会显示，这是正常的）
3. 确认密码

**示例**：
```
Enter new UNIX username: tinot
New password: [输入密码，不显示]
Retype new password: [再次输入密码]
```

### 步骤 3：验证安装

在 Ubuntu 22.04 终端中运行：

```bash
# 查看系统信息
lsb_release -a

# 查看 Python 版本
python3 --version
```

**预期输出**：
```
Distributor ID: Ubuntu
Description:    Ubuntu 22.04.x LTS
Release:        22.04
Codename:       jammy

Python 3.10.x
```

### 步骤 4：更新系统包

```bash
# 更新包列表
sudo apt update

# 升级系统包（可选，但推荐）
sudo apt upgrade -y
```

### 步骤 5：安装基础工具

```bash
# 安装常用工具
sudo apt install -y curl wget git build-essential

# 安装 Python 开发工具
sudo apt install -y python3-pip python3-venv python3-dev
```

---

## 管理多个 WSL 发行版

### 查看所有已安装的发行版

```powershell
wsl --list --verbose
```

**预期输出**：
```
  NAME            STATE           VERSION
* Ubuntu          Running         2
  Ubuntu-22.04    Stopped         2
```

### 启动特定的发行版

```powershell
# 启动 Ubuntu 22.04
wsl -d Ubuntu-22.04

# 启动 Ubuntu 24.04（默认）
wsl -d Ubuntu
# 或直接
wsl
```

### 设置默认发行版

```powershell
# 设置 Ubuntu-22.04 为默认
wsl --set-default Ubuntu-22.04

# 设置回 Ubuntu 24.04
wsl --set-default Ubuntu
```

### 停止发行版

```bash
# 在 WSL 中
exit

# 或在 Windows PowerShell 中
wsl --terminate Ubuntu-22.04
```

---

## 下一步

安装完 Ubuntu 22.04 后，继续按照 `VIRTUAL_ENVIRONMENT_SETUP.md` 的**第二部分：WSL 环境配置**进行：

1. 进入项目目录
2. 创建虚拟环境
3. 安装依赖
4. 测试服务

---

## 故障排除

### 问题 1：安装失败

**解决**：
```powershell
# 检查 WSL 状态
wsl --status

# 确保是 WSL 2
wsl --set-default-version 2

# 重新安装
wsl --unregister Ubuntu-22.04
wsl --install -d Ubuntu-22.04
```

### 问题 2：无法访问 Windows 文件

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

### 问题 3：Python 版本不对

**解决**：
```bash
# Ubuntu 22.04 默认是 3.10，如果不可用：
sudo apt update
sudo apt install -y python3.10 python3.10-venv python3.10-dev
```

