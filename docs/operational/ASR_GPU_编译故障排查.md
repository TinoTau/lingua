# ASR GPU 编译故障排查指南

**最后更新**: 2025-11-28

本文档提供 ASR GPU 编译过程中常见问题的详细解决方案。

---

## ❌ 错误：No CUDA toolset found

### 错误信息

```
CMake Error at .../CMakeDetermineCompilerId.cmake:676 (message):
    No CUDA toolset found.
```

### 原因分析

这个错误表示 CMake 在使用 Visual Studio 生成器时，无法找到 CUDA 工具集。虽然 CUDA Toolkit 已安装，但 Visual Studio 需要额外的 CUDA 工具集组件才能编译 CUDA 代码。

### 解决方案

#### 方案 1：安装 Visual Studio CUDA 工具集（推荐）

1. **打开 Visual Studio Installer**
   ```powershell
   Start-Process "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe"
   ```

2. **修改 Visual Studio 安装**
   - 找到 "Visual Studio Build Tools 2022" 或 "Visual Studio 2022"
   - 点击"修改"按钮

3. **安装 CUDA 工具集**
   - 切换到"单个组件"选项卡
   - 在搜索框中输入 "CUDA"
   - 勾选：
     - ✅ **MSVC v143 - VS 2022 C++ x64/x86 CUDA 工具集 (最新)**
   - 点击"修改"开始安装

4. **安装完成后重新编译**
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

#### 方案 2：使用短路径名（如果方案 1 不可用）

有时路径中的空格会导致问题，可以使用短路径名：

**方法 A：使用提供的脚本**

```powershell
cd D:\Programs\github\lingua\core\engine
.\build_with_cuda_shortpath.ps1
```

**方法 B：手动设置**

```powershell
cd D:\Programs\github\lingua\core\engine

# 获取短路径名
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$fso = New-Object -ComObject Scripting.FileSystemObject
$shortPath = $fso.GetFolder($cudaPath).ShortPath

Write-Host "CUDA 短路径: $shortPath" -ForegroundColor Cyan

# 使用短路径设置环境变量
$env:CUDA_PATH = $shortPath
$env:CUDAToolkit_ROOT = $shortPath
$env:CUDA_ROOT = $shortPath
$env:CUDA_HOME = $shortPath
$env:CMAKE_CUDA_COMPILER = "$shortPath\bin\nvcc.exe"
$env:PATH = "$shortPath\bin;$shortPath\libnvvp;$env:PATH"

# 清理并重新编译
cargo clean
cargo build --release --bin core_engine
```

**注意**：即使使用短路径名，如果 Visual Studio Build Tools 缺少 CUDA 工具集支持，仍然会出现 "No CUDA toolset found" 错误。

#### 方案 3：验证 Visual Studio 工作负载

确保已安装必要的 Visual Studio 组件：

1. 打开 Visual Studio Installer
2. 点击"修改"
3. 确保已勾选：
   - ✅ **使用 C++ 的桌面开发**（工作负载）
   - ✅ **Windows 10/11 SDK**（单个组件）
   - ✅ **MSVC v143 - VS 2022 C++ x64/x86 生成工具**（单个组件）

#### 方案 4：如果无法安装 CUDA 工具集（最终方案）

如果您无法在 Visual Studio Installer 中找到或安装 CUDA 工具集，可能需要考虑以下替代方案：

**选项 A：安装完整的 Visual Studio 2022 Community**

完整的 Visual Studio 2022 Community 版本通常包含 CUDA 工具集支持：

1. 下载 Visual Studio 2022 Community：https://visualstudio.microsoft.com/downloads/
2. 安装时确保选择"使用 C++ 的桌面开发"工作负载
3. 安装完成后，CUDA 工具集应该会自动可用

**选项 B：使用 WSL2 + Linux 编译**

在 Windows 上使用 WSL2（Windows Subsystem for Linux）编译：

1. 安装 WSL2 和 Ubuntu
2. 在 WSL2 中安装 CUDA Toolkit（NVIDIA 提供 WSL2 版本的 CUDA）
3. 在 WSL2 中编译项目

**选项 C：暂时使用 CPU 版本**

如果 GPU 支持不是必需的，可以暂时使用 CPU 版本：

修改 `core/engine/Cargo.toml`：
```toml
# 注释掉 CUDA 支持
# whisper-rs = { version = "0.15.1", features = ["cuda"] }
whisper-rs = "0.15.1"  # 使用 CPU 版本
```

然后重新编译：
```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
cargo build --release --bin core_engine
```

---

## ❌ 错误：CMake 找不到 CUDA

### 错误信息

```
CMake Error: Could not find CUDA
```

### 解决方案

1. **验证 CUDA 安装**
   ```powershell
   nvcc --version
   $env:CUDA_PATH
   ```

2. **设置所有必要的环境变量**
   ```powershell
   $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
   $env:CUDA_PATH = $cudaPath
   $env:CUDAToolkit_ROOT = $cudaPath  # 重要！
   $env:CUDA_ROOT = $cudaPath
   $env:CUDA_HOME = $cudaPath
   $env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
   $env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"
   ```

3. **验证 CMake 能否找到 CUDA**
   ```powershell
   # 创建测试目录
   $testDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
   Set-Content -Path "$testDir\CMakeLists.txt" -Value @"
   cmake_minimum_required(VERSION 3.18)
   project(TestCUDA)
   find_package(CUDA REQUIRED)
   message(STATUS "CUDA found: `${CUDA_FOUND}")
   message(STATUS "CUDA version: `${CUDA_VERSION}")
   "@
   
   cd $testDir
   cmake . 2>&1 | Select-String -Pattern "CUDA|Found|version"
   
   # 清理
   cd ..
   Remove-Item -Recurse -Force $testDir
   ```

---

## ❌ 错误：编译时间过长或内存不足

### 解决方案

1. **确保使用 Release 模式**
   ```powershell
   cargo build --release --bin core_engine
   ```

2. **关闭其他占用内存的程序**

3. **增加虚拟内存**（如果系统提示内存不足）

4. **耐心等待**：首次编译 CUDA 支持可能需要 10-30 分钟

---

## ✅ 验证编译是否成功

编译成功后，检查可执行文件：

```powershell
Test-Path "D:\Programs\github\lingua\core\engine\target\release\core_engine.exe"
```

如果返回 `True`，说明编译成功。

---

## 📚 相关文档

- [ASR GPU 配置完成](./ASR_GPU_配置完成.md)
- [ASR GPU 编译完整步骤](./ASR_GPU_编译完整步骤.md)
- [CUDA Toolkit 安装指南](./CUDA_Toolkit_安装指南.md)

---

**最后更新**: 2025-11-28

