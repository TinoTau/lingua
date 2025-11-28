# ASR GPU 编译完整步骤

**最后更新**: 2025-11-28

本文档提供完整的 ASR GPU 编译步骤，包括所有必需的环境变量设置。

---

## 🔧 完整编译步骤

### 步骤 1: 设置所有必需的环境变量

在 PowerShell 中执行（**必须在同一个会话中完成所有步骤**）：

```powershell
# 设置 CUDA 路径
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"

# 设置所有可能需要的环境变量
$env:CUDA_PATH = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath

# 设置 CMAKE CUDA 编译器路径（重要！）
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"

# 添加到 PATH
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 验证
Write-Host "=== CUDA Environment Variables ===" -ForegroundColor Cyan
Write-Host "CUDA_PATH: $env:CUDA_PATH"
Write-Host "CUDA_ROOT: $env:CUDA_ROOT"
Write-Host "CUDA_HOME: $env:CUDA_HOME"
Write-Host "CMAKE_CUDA_COMPILER: $env:CMAKE_CUDA_COMPILER"
Write-Host ""
nvcc --version
Write-Host ""
```

### 步骤 2: 清理旧的编译产物

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

### 步骤 3: 编译 CoreEngine

```powershell
cargo build --release --bin core_engine
```

**注意**：
- 首次编译可能需要 10-30 分钟
- 确保在**同一个 PowerShell 会话**中执行所有步骤
- 如果关闭 PowerShell，需要重新设置环境变量

---

## 🐛 如果仍然失败：安装 Visual Studio CUDA 工具集

错误信息 `No CUDA toolset found` 通常表示 Visual Studio 缺少 CUDA 工具集支持。

### 解决方案：安装 Visual Studio CUDA 工具集

1. **打开 Visual Studio Installer**
   - 在开始菜单搜索 "Visual Studio Installer"
   - 或运行：`C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe`

2. **修改已安装的 Visual Studio**
   - 找到 "Visual Studio Build Tools 2022" 或 "Visual Studio 2022"
   - 点击"修改"

3. **安装 CUDA 工具集**
   - 切换到"单个组件"选项卡
   - 搜索 "CUDA"
   - 勾选以下组件：
     - ✅ **MSVC v143 - VS 2022 C++ x64/x86 CUDA 工具集 (最新)**
     - ✅ **CUDA 12.4 SDK**（如果可用）
   - 点击"修改"开始安装

4. **安装完成后重新编译**
   ```powershell
   # 重新设置环境变量（在同一会话中）
   $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
   $env:CUDA_PATH = $cudaPath
   $env:CUDA_ROOT = $cudaPath
   $env:CUDA_HOME = $cudaPath
   $env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
   $env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"
   
   # 重新编译
   cd D:\Programs\github\lingua\core\engine
   cargo build --release --bin core_engine
   ```

### 替代方案：使用 Ninja 生成器（如果 Visual Studio 工具集不可用）

如果无法安装 Visual Studio CUDA 工具集，可以尝试使用 Ninja 生成器：

1. **安装 Ninja**
   ```powershell
   # 使用 Chocolatey（如果已安装）
   choco install ninja
   
   # 或从 GitHub 下载：https://github.com/ninja-build/ninja/releases
   ```

2. **设置 CMake 生成器**
   ```powershell
   $env:CMAKE_GENERATOR = "Ninja"
   ```

3. **重新编译**
   ```powershell
   cd D:\Programs\github\lingua\core\engine
   cargo build --release --bin core_engine
   ```

### 或者使用完整路径设置

```powershell
# 设置所有环境变量
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$env:CUDA_PATH = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 设置 CMake 查找路径
$env:CMAKE_PREFIX_PATH = $cudaPath

# 编译
cd D:\Programs\github\lingua\core\engine
cargo build --release --bin core_engine
```

---

## 📝 一键编译脚本

创建 `build_core_engine_gpu.ps1`：

```powershell
# build_core_engine_gpu.ps1
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"

Write-Host "=== Setting CUDA Environment Variables ===" -ForegroundColor Cyan

# 设置所有环境变量
$env:CUDA_PATH = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 验证
Write-Host "CUDA_PATH: $env:CUDA_PATH" -ForegroundColor Green
Write-Host "CMAKE_CUDA_COMPILER: $env:CMAKE_CUDA_COMPILER" -ForegroundColor Green
nvcc --version
Write-Host ""

Write-Host "=== Cleaning Build Artifacts ===" -ForegroundColor Cyan
cd D:\Programs\github\lingua\core\engine
cargo clean

Write-Host "=== Building CoreEngine with GPU Support ===" -ForegroundColor Cyan
cargo build --release --bin core_engine
```

使用方法：
```powershell
.\build_core_engine_gpu.ps1
```

---

**最后更新**: 2025-11-28

