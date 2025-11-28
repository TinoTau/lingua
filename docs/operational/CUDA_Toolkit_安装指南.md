# CUDA Toolkit 安装指南

**最后更新**: 2025-11-28

本文档提供在 Windows 上安装 CUDA Toolkit 的详细步骤。

---

## 📋 前置条件

根据您的系统信息：
- **GPU**: NVIDIA GeForce RTX 4060 Laptop GPU
- **驱动版本**: 566.26
- **CUDA 版本**: 12.7（驱动支持）

---

## 🔧 安装步骤

### 步骤 1: 下载 CUDA Toolkit

1. 访问 NVIDIA CUDA 下载页面：
   - https://developer.nvidia.com/cuda-downloads

2. 选择以下选项：
   - **操作系统**: Windows
   - **架构**: x86_64
   - **版本**: Windows 10/11
   - **安装程序类型**: exe (local) 或 exe (network)

3. **推荐版本**: CUDA 12.1 或 12.4
   - 您的驱动支持 CUDA 12.7，但 PyTorch 和 whisper-rs 通常使用 CUDA 12.1 或 12.4
   - CUDA 12.1 向后兼容，推荐使用

### 步骤 2: 安装 CUDA Toolkit

1. **运行安装程序**
   - 双击下载的 `.exe` 文件
   - 如果提示需要管理员权限，选择"是"

2. **安装选项**
   - 选择"快速安装"（Express Installation）或"自定义安装"（Custom Installation）
   - **推荐**: 使用"快速安装"（会自动配置环境变量）

3. **安装路径**
   - 默认路径：`C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1`
   - 如果选择自定义安装，记住安装路径

4. **等待安装完成**
   - 安装过程可能需要 10-20 分钟
   - 安装完成后，可能需要重启计算机

### 步骤 3: 验证安装

#### 方法 1: 检查 CUDA 编译器

打开 PowerShell（以管理员身份运行）：

```powershell
nvcc --version
```

**预期输出**：
```
nvcc: NVIDIA (R) Cuda compiler driver
Copyright (c) 2005-2024 NVIDIA Corporation
Built on ...
Cuda compilation tools, release 12.1, V12.1.xx
Build cuda_12.1.r12.1/...
```

#### 方法 2: 检查安装目录

```powershell
# 检查默认安装路径
Test-Path "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1"
```

如果返回 `True`，说明已安装。

#### 方法 3: 检查环境变量

```powershell
# 检查 CUDA_PATH 环境变量
$env:CUDA_PATH

# 检查 PATH 中是否包含 CUDA
$env:PATH -split ';' | Select-String -Pattern "CUDA"
```

---

## 🔧 设置环境变量

### 如果安装程序没有自动设置环境变量

#### 方法 1: 使用 PowerShell（临时设置，当前会话有效）

```powershell
# 设置 CUDA_PATH（根据实际安装路径调整）
$env:CUDA_PATH = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1"

# 添加到 PATH
$env:PATH = "$env:CDA_PATH\bin;$env:CUDA_PATH\libnvvp;$env:PATH"

# 验证
nvcc --version
```

#### 方法 2: 使用系统环境变量（永久设置，推荐）

1. **打开系统环境变量设置**
   - 按 `Win + R`，输入 `sysdm.cpl`，回车
   - 点击"高级"选项卡
   - 点击"环境变量"按钮

2. **添加 CUDA_PATH 变量**
   - 在"系统变量"部分，点击"新建"
   - 变量名：`CUDA_PATH`
   - 变量值：`C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1`
   - 点击"确定"

3. **添加到 PATH**
   - 在"系统变量"部分，找到 `Path` 变量
   - 点击"编辑"
   - 点击"新建"，添加以下路径（根据实际安装路径调整）：
     ```
     C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1\bin
     C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1\libnvvp
     ```
   - 点击"确定"保存所有更改

4. **重启 PowerShell 或命令提示符**
   - 关闭所有 PowerShell 窗口
   - 重新打开 PowerShell
   - 验证环境变量：
     ```powershell
     $env:CUDA_PATH
     nvcc --version
     ```

#### 方法 3: 使用 PowerShell 脚本（永久设置）

```powershell
# 以管理员身份运行 PowerShell

# 设置 CUDA_PATH（根据实际安装路径调整）
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1"

# 设置系统环境变量
[System.Environment]::SetEnvironmentVariable("CUDA_PATH", $cudaPath, [System.EnvironmentVariableTarget]::Machine)

# 添加到 PATH
$currentPath = [System.Environment]::GetEnvironmentVariable("Path", [System.EnvironmentVariableTarget]::Machine)
$newPaths = @(
    "$cudaPath\bin",
    "$cudaPath\libnvvp"
)

$pathsToAdd = $newPaths | Where-Object { $currentPath -notlike "*$_*" }
if ($pathsToAdd.Count -gt 0) {
    $updatedPath = $currentPath + ";" + ($pathsToAdd -join ";")
    [System.Environment]::SetEnvironmentVariable("Path", $updatedPath, [System.EnvironmentVariableTarget]::Machine)
    Write-Host "Added to PATH: $($pathsToAdd -join ', ')" -ForegroundColor Green
} else {
    Write-Host "Paths already in PATH" -ForegroundColor Yellow
}

Write-Host "CUDA_PATH set to: $cudaPath" -ForegroundColor Green
Write-Host "Please restart PowerShell for changes to take effect" -ForegroundColor Yellow
```

---

## ✅ 验证安装

### 完整验证步骤

```powershell
# 1. 检查 CUDA 编译器
nvcc --version

# 2. 检查环境变量
$env:CUDA_PATH
$env:PATH -split ';' | Select-String -Pattern "CUDA"

# 3. 检查 CUDA 库文件
Test-Path "$env:CUDA_PATH\bin\cublas64_12.dll"
Test-Path "$env:CUDA_PATH\bin\cudart64_12.dll"

# 4. 检查 nvidia-smi（应该已经可用）
nvidia-smi
```

**预期结果**：
- `nvcc --version` 显示 CUDA 版本信息
- `$env:CUDA_PATH` 显示 CUDA 安装路径
- PATH 中包含 CUDA 的 bin 目录
- CUDA 库文件存在

---

## 🐛 故障排查

### 问题 1: `nvcc: command not found`

**原因**: CUDA 未安装或环境变量未设置

**解决方法**:
1. 确认 CUDA Toolkit 已安装
2. 检查环境变量是否正确设置
3. 重启 PowerShell

### 问题 2: 环境变量设置后仍然无效

**解决方法**:
1. 完全关闭所有 PowerShell 窗口
2. 重新打开 PowerShell（以管理员身份）
3. 验证环境变量：
   ```powershell
   [System.Environment]::GetEnvironmentVariable("CUDA_PATH", [System.EnvironmentVariableTarget]::Machine)
   ```

### 问题 3: 找不到 CUDA 库文件

**解决方法**:
1. 确认 CUDA Toolkit 完整安装（不是只有驱动）
2. 检查安装路径是否正确
3. 重新安装 CUDA Toolkit

### 问题 4: 版本不匹配

**说明**:
- 驱动支持 CUDA 12.7
- PyTorch 使用 CUDA 12.1
- 这是正常的，CUDA 向后兼容

**解决方法**: 无需处理，CUDA 12.1 可以在支持 CUDA 12.7 的驱动上运行

---

## 📚 相关文档

- [ASR GPU 配置完成](./ASR_GPU_配置完成.md)
- [PyTorch CUDA 安装指南](./PyTorch_CUDA_安装指南.md)
- [编译和启动命令参考](./编译和启动命令参考.md)

---

## ✅ 安装检查清单

- [ ] 下载 CUDA Toolkit（推荐 12.1 或 12.4）
- [ ] 安装 CUDA Toolkit
- [ ] 设置 CUDA_PATH 环境变量
- [ ] 将 CUDA bin 目录添加到 PATH
- [ ] 验证 `nvcc --version` 可以运行
- [ ] 验证环境变量正确设置
- [ ] 重启 PowerShell
- [ ] 重新尝试编译 CoreEngine

---

**最后更新**: 2025-11-28

