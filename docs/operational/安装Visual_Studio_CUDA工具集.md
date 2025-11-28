# 在 Visual Studio 2022 Community 中安装 CUDA 工具集

**最后更新**: 2025-11-28

本文档提供在 Visual Studio 2022 Community 中安装 CUDA 工具集的详细步骤。

---

## 📋 前置条件

- ✅ Visual Studio 2022 Community 已安装
- ✅ CUDA Toolkit 12.4 已安装

---

## 🔧 安装步骤

### 步骤 1: 打开 Visual Studio Installer

1. **方法 A：通过开始菜单**
   - 在 Windows 开始菜单中搜索 "Visual Studio Installer"
   - 点击打开

2. **方法 B：通过命令行**
   ```powershell
   Start-Process "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe"
   ```

### 步骤 2: 修改 Visual Studio 2022 Community

1. 在 Visual Studio Installer 中找到 **"Visual Studio Community 2022"**
2. 点击 **"修改"** 按钮

### 步骤 3: 安装 CUDA 工具集组件

1. **切换到"单个组件"选项卡**
   - 在顶部标签页中，点击 **"单个组件"**

2. **搜索 CUDA 组件**
   - 在搜索框中输入：`CUDA`
   - 或者滚动查找 CUDA 相关组件

3. **勾选以下组件**：
   - ✅ **MSVC v143 - VS 2022 C++ x64/x86 CUDA 工具集 (最新)**
   - ✅ **CUDA 12.4 SDK**（如果可用，可选）

4. **点击"修改"按钮**
   - 等待安装完成（可能需要几分钟）

### 步骤 4: 验证安装

安装完成后，验证 CUDA 工具集是否已安装：

```powershell
# 检查 Visual Studio 是否包含 CUDA 工具集
Test-Path "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\*\bin\Hostx64\x64\nvcc.exe"
```

或者检查 Visual Studio 的扩展目录：

```powershell
Test-Path "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\IDE\Extensions\NVIDIA"
```

---

## ✅ 安装后重新编译

安装完成后，在新的 PowerShell 窗口中重新编译：

```powershell
cd D:\Programs\github\lingua\core\engine

# 设置环境变量
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$env:CUDA_PATH = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 清理并重新编译
cargo clean
cargo build --release --bin core_engine
```

---

## 🐛 故障排查

### 问题 1: 在"单个组件"中找不到 CUDA 工具集

**可能原因**：
- Visual Studio 版本不完整
- 需要更新 Visual Studio Installer

**解决方法**：
1. 在 Visual Studio Installer 中，点击"更新"按钮，确保 Visual Studio 是最新版本
2. 确保已安装"使用 C++ 的桌面开发"工作负载
3. 如果仍然找不到，尝试重新安装 Visual Studio 2022 Community

### 问题 2: 安装后仍然提示 "No CUDA toolset found"

**解决方法**：
1. **重启电脑**（重要！）
   - 安装 CUDA 工具集后，需要重启电脑才能生效

2. **验证环境变量**
   ```powershell
   $env:CUDA_PATH
   nvcc --version
   ```

3. **使用短路径名**（如果路径中有空格）
   ```powershell
   $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
   $fso = New-Object -ComObject Scripting.FileSystemObject
   $shortPath = $fso.GetFolder($cudaPath).ShortPath
   $env:CUDA_PATH = $shortPath
   $env:CUDAToolkit_ROOT = $shortPath
   ```

### 问题 3: 安装过程中出错

**解决方法**：
1. 以管理员身份运行 Visual Studio Installer
2. 关闭所有 Visual Studio 相关进程
3. 重新尝试安装

---

## 📚 相关文档

- [ASR GPU 配置完成](./ASR_GPU_配置完成.md)
- [ASR GPU 编译故障排查](./ASR_GPU_编译故障排查.md)
- [CUDA Toolkit 安装指南](./CUDA_Toolkit_安装指南.md)

---

**最后更新**: 2025-11-28

