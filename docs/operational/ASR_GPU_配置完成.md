# ASR GPU 配置完成指南

**最后更新**: 2025-11-28

本文档记录了为 Whisper ASR 启用 GPU 支持的配置步骤。

---

## ✅ 已完成的配置

### 1. 修改 Cargo.toml

已修改 `core/engine/Cargo.toml`，为 `whisper-rs` 添加 CUDA 支持：

```toml
# Whisper ASR 支持（启用 CUDA GPU 加速）
whisper-rs = { version = "0.15.1", features = ["cuda"] }
```

---

## 🔨 编译步骤

### 方法 1: 使用自动编译脚本（推荐）

项目提供了一个自动编译脚本，会自动检测 CUDA 路径并设置环境变量：

```powershell
cd D:\Programs\github\lingua\core\engine
.\build_with_cuda.ps1
```

脚本会自动：
- 检测 CUDA 安装路径
- 设置必要的环境变量
- 验证 CUDA 和 CMake
- 执行编译

### 方法 2: 手动编译

#### 步骤 1: 清理旧的编译产物（推荐）

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

#### 步骤 2: 设置环境变量（重要！）

在编译前，确保设置 CUDA 环境变量，以便 CMake 能够找到 CUDA：

```powershell
# 设置 CUDA 路径（根据实际安装路径调整）
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$env:CUDA_PATH = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath  # CMake 查找这个变量
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"
```

**注意**：如果 CUDA_PATH 已设置为系统环境变量，可以跳过此步骤。但为了确保编译时能检测到，建议在编译前显式设置。

#### 步骤 3: 编译 CoreEngine（Release 模式）

```powershell
cargo build --release --bin core_engine
```

**注意**：
- 首次编译可能需要较长时间（10-30 分钟），因为需要编译 CUDA 支持
- 确保有足够的磁盘空间（至少 2-3 GB）
- 编译过程中会下载和编译 CUDA 相关的依赖
- 如果编译失败并提示找不到 CUDA，请确保已执行步骤 2 设置环境变量

### 4. 验证编译结果

编译成功后，可执行文件位于：
```
core\engine\target\release\core_engine.exe
```

---

## ✅ 验证 GPU 是否启用

### 方法 1: 查看启动日志

启动 CoreEngine 后，查看日志输出：

```powershell
cd D:\Programs\github\lingua
.\target\release\core_engine.exe --config lingua_core_config.toml
```

**GPU 已启用**的标志：
```
whisper_backend_init_gpu: using GPU
whisper_init_with_params_no_state: use gpu    = 1
```

**GPU 未启用**的标志：
```
whisper_backend_init_gpu: no GPU found
whisper_init_with_params_no_state: use gpu    = 0
```

### 方法 2: 使用 nvidia-smi 监控

在另一个终端运行：

```powershell
# 实时监控 GPU 使用情况
nvidia-smi -l 1
```

然后发送一个 S2S 请求，如果看到 GPU 使用率上升，说明正在使用 GPU。

### 方法 3: 性能对比

启用 GPU 后，ASR 处理时间应该显著减少：

- **CPU（之前）**: 6-7 秒
- **GPU（预期）**: 1-2 秒
- **提升**: 约 3-4 倍

查看日志中的 `[ASR] Transcription completed in Xms` 来确认。

---

## 🐛 故障排查

### 问题 1: 编译失败，提示找不到 CUDA

**错误信息**：
```
error: failed to run custom build command for `whisper-rs-sys`
CMake Error: No CUDA toolset found
```

**原因**: 
- CUDA Toolkit 未安装或环境变量未正确设置
- **重要**：安装 CUDA Toolkit 后**必须重启电脑**，否则系统无法识别 CUDA 环境

**解决方法**：

#### ⚠️ 第一步：重启电脑（重要！）

**安装 CUDA Toolkit 后，必须重启电脑**才能让：
- 环境变量（如 `CUDA_PATH`）被系统进程识别
- Visual Studio 检测到 CUDA 集成
- CUDA 驱动和工具集注册生效

#### 第二步：重启后验证

重启后，打开新的 PowerShell 窗口，验证 CUDA 是否可用：

```powershell
# 检查 nvcc 是否在系统 PATH 中
nvcc --version

# 检查 CUDA 环境变量（如果已设置为系统变量）
$env:CUDA_PATH
$env:CUDA_PATH_V12_4

# 检查 GPU 是否可用
nvidia-smi
```

#### ⚠️ 重要说明：Visual Studio 中不需要手动配置 CUDA

**常见误解**：很多用户认为需要在 Visual Studio Installer 中安装 CUDA 组件，或者在 Visual Studio 项目中手动启用 CUDA。

**实际情况**：
- CUDA Toolkit 是独立安装的（您已经安装完成）
- Visual Studio 本身**不需要**"启用 CUDA Toolkit"选项
- Rust 项目（whisper-rs）使用 CMake 自动检测和使用 CUDA
- CMake 会自动查找 CUDA_PATH 环境变量来定位 CUDA

**验证 CMake 是否能找到 CUDA**：

```powershell
# 设置环境变量（如果尚未设置为系统变量）
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$env:CUDA_PATH = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath

# 创建临时测试目录
$testDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
Set-Content -Path "$testDir\CMakeLists.txt" -Value @"
cmake_minimum_required(VERSION 3.18)
project(TestCUDA)
find_package(CUDA REQUIRED)
message(STATUS "CUDA found: `${CUDA_FOUND}")
message(STATUS "CUDA version: `${CUDA_VERSION}")
"@

# 测试 CMake 是否能找到 CUDA
cd $testDir
cmake . 2>&1 | Select-String -Pattern "CUDA|Found|version"

# 清理测试目录
cd ..
Remove-Item -Recurse -Force $testDir
```

**预期输出**：
```
-- Found CUDA: C:/Program Files/NVIDIA GPU Computing Toolkit/CUDA/v12.4 (found version "12.4")
-- CUDA found: TRUE
-- CUDA version: 12.4
```

如果看到上述输出，说明 CMake 能够正确检测到 CUDA，可以直接进行编译。

#### 第三步：设置环境变量并编译

如果验证通过，设置环境变量并重新编译：

```powershell
# 设置环境变量（在同一会话中）
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
$env:CUDA_PATH = $cudaPath
$env:CUDA_ROOT = $cudaPath
$env:CUDA_HOME = $cudaPath
$env:CUDAToolkit_ROOT = $cudaPath  # CMake 可能查找这个变量
$env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
$env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"

# 清理并重新编译
cd D:\Programs\github\lingua\core\engine
cargo clean
cargo build --release --bin core_engine
```

#### 如果重启后仍然失败：安装 Visual Studio CUDA 工具集

如果重启后仍然出现 `No CUDA toolset found` 错误，这是因为 Visual Studio 生成器需要 CUDA 工具集支持。

**解决方案：在 Visual Studio Installer 中安装 CUDA 工具集**

1. **打开 Visual Studio Installer**
   ```powershell
   Start-Process "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe"
   ```

2. **修改已安装的 Visual Studio**
   - 找到 "Visual Studio Build Tools 2022" 或 "Visual Studio 2022"
   - 点击"修改"按钮

3. **安装 CUDA 工具集组件**
   - 切换到"单个组件"选项卡
   - 在搜索框中输入 "CUDA"
   - 勾选以下组件：
     - ✅ **MSVC v143 - VS 2022 C++ x64/x86 CUDA 工具集 (最新)**
     - ✅ **CUDA 12.4 SDK**（如果可用，可选）
   - 点击"修改"开始安装

4. **安装完成后重新编译**
   ```powershell
   # 重新设置环境变量（在同一会话中）
   $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
   $env:CUDA_PATH = $cudaPath
   $env:CUDAToolkit_ROOT = $cudaPath
   $env:CUDA_ROOT = $cudaPath
   $env:CUDA_HOME = $cudaPath
   $env:CMAKE_CUDA_COMPILER = "$cudaPath\bin\nvcc.exe"
   $env:PATH = "$cudaPath\bin;$cudaPath\libnvvp;$env:PATH"
   
   # 清理并重新编译
   cd D:\Programs\github\lingua\core\engine
   cargo clean
   cargo build --release --bin core_engine
   ```

**替代方案：如果无法安装 CUDA 工具集**

如果 Visual Studio Installer 中没有 CUDA 工具集选项，或者安装后仍然失败，可以尝试：

1. **使用短路径名**（避免空格问题）：
   ```powershell
   # 获取短路径名
   $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4"
   $fso = New-Object -ComObject Scripting.FileSystemObject
   $shortPath = $fso.GetFolder($cudaPath).ShortPath
   
   # 使用短路径设置环境变量
   $env:CUDA_PATH = $shortPath
   $env:CUDAToolkit_ROOT = $shortPath
   $env:CUDA_ROOT = $shortPath
   $env:CUDA_HOME = $shortPath
   $env:CMAKE_CUDA_COMPILER = "$shortPath\bin\nvcc.exe"
   $env:PATH = "$shortPath\bin;$shortPath\libnvvp;$env:PATH"
   
   # 重新编译
   cargo build --release --bin core_engine
   ```

2. **确保 Visual Studio 工作负载已安装**：
   - 在 Visual Studio Installer 中，确保已安装"使用 C++ 的桌面开发"工作负载
   - 确保已安装 Windows 10/11 SDK

---

**原始安装步骤**（如果尚未安装）：

1. **安装 CUDA Toolkit**（不仅仅是驱动）
   - 下载地址：https://developer.nvidia.com/cuda-downloads
   - 推荐版本：CUDA 12.1 或 12.4（与 PyTorch 兼容）
   - 详细步骤：参考 [CUDA Toolkit 安装指南](./CUDA_Toolkit_安装指南.md)

2. **安装完成后重启电脑**（必须！）

3. **设置环境变量**（重启后，在编译前设置）

   参考上面的"第三步：设置环境变量并编译"

### 问题 2: 编译成功但运行时显示 "no GPU found"

**可能原因**：
1. CUDA 驱动版本不匹配
2. GPU 驱动未正确安装

**解决方法**：
1. 检查 GPU 驱动：
   ```powershell
   nvidia-smi
   ```

2. 确保驱动版本与 CUDA Toolkit 兼容

3. 重启计算机（有时需要）

### 问题 3: 编译时间过长

**说明**：
- 首次编译 CUDA 支持可能需要 10-30 分钟
- 这是正常的，因为需要编译 CUDA 相关的 C++ 代码

**建议**：
- 使用 `--release` 模式编译（性能更好）
- 确保有足够的磁盘空间
- 耐心等待

### 问题 4: 内存不足

**错误信息**：
```
error: failed to allocate memory
```

**解决方法**：
1. 关闭其他占用内存的程序
2. 增加虚拟内存
3. 使用更小的模型（如果可用）

---

## 📊 预期性能提升

启用 GPU 后，完整的 S2S 流程性能预期：

| 组件 | CPU 耗时 | GPU 耗时 | 提升倍数 |
|------|---------|---------|---------|
| ASR | 6-7 秒 | 1-2 秒 | 3-4x |
| NMT | 3-4 秒 | 0.5-1 秒 | 3-4x |
| TTS | 3-4 秒 | 3-4 秒 | 1x（未启用 GPU） |
| **总计** | **13-15 秒** | **4.5-7 秒** | **2-3x** |

**注意**：TTS 目前未启用 GPU，如果后续启用，总耗时可能降至 2-4 秒。

---

## 🔄 回退到 CPU 版本

如果需要回退到 CPU 版本：

1. 修改 `core/engine/Cargo.toml`：
   ```toml
   # Whisper ASR 支持
   whisper-rs = "0.15.1"
   ```

2. 重新编译：
   ```powershell
   cd core\engine
   cargo clean
   cargo build --release --bin core_engine
   ```

---

## 📚 相关文档

- [GPU 启用指南](../GPU_启用指南.md)
- [PyTorch CUDA 安装指南](./PyTorch_CUDA_安装指南.md)
- [编译和启动命令参考](./编译和启动命令参考.md)

---

## ✅ 配置检查清单

- [x] 修改 `Cargo.toml` 添加 `cuda` feature
- [ ] 清理旧的编译产物
- [ ] 重新编译 CoreEngine
- [ ] 验证 GPU 是否启用（查看日志）
- [ ] 测试性能提升
- [ ] 使用 `nvidia-smi` 监控 GPU 使用

---

**最后更新**: 2025-11-28

