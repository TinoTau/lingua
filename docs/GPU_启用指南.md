# Whisper ASR GPU 启用指南

## 当前状态

- ✅ **线程数优化**：已自动使用所有可用 CPU 核心（留一个给系统）
- ⚠️ **GPU 支持**：需要额外配置

## 启用 GPU 支持的步骤

### 1. 检查 GPU 类型

根据您的 GPU 类型选择相应的配置：

- **NVIDIA GPU**：需要 CUDA 支持
- **AMD GPU**：需要 ROCm 支持  
- **Apple Silicon (M1/M2/M3)**：需要 Metal 支持

### 2. 安装必要的驱动和工具

#### NVIDIA GPU 用户

1. **安装 NVIDIA 驱动**
   - 从 [NVIDIA 官网](https://www.nvidia.com/Download/index.aspx) 下载并安装最新驱动

2. **安装 CUDA Toolkit**
   - 从 [NVIDIA CUDA 官网](https://developer.nvidia.com/cuda-downloads) 下载并安装
   - 建议版本：CUDA 11.8 或 12.x

3. **安装 cuDNN**
   - 从 [NVIDIA cuDNN 官网](https://developer.nvidia.com/cudnn) 下载并安装

4. **配置环境变量（Windows）**
   ```
   CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8
   Path=%CUDA_PATH%\bin;%Path%
   ```

#### AMD GPU 用户

1. **安装 ROCm**
   - 从 [ROCm 官网](https://rocm.docs.amd.com/) 下载并安装
   - 注意：ROCm 主要支持 Linux，Windows 支持有限

#### Apple Silicon 用户

- Metal 支持通常已内置，无需额外安装

### 3. 重新编译 whisper-rs（如果需要）

`whisper-rs` 是基于 `whisper.cpp` 的 Rust 绑定。要启用 GPU 支持，可能需要：

1. **检查 whisper-rs 的 feature flags**
   - 查看 [whisper-rs GitHub](https://github.com/tazz4843/whisper-rs) 了解可用的 feature flags
   - 可能的 feature flags：`cuda`, `metal`, `opencl`

2. **修改 Cargo.toml**
   ```toml
   [dependencies]
   whisper-rs = { version = "0.15.1", features = ["cuda"] }  # 对于 NVIDIA GPU
   # 或
   whisper-rs = { version = "0.15.1", features = ["metal"] }  # 对于 Apple Silicon
   ```

3. **重新编译项目**
   ```bash
   cd core/engine
   cargo build --release --bin core_engine
   ```

### 4. 验证 GPU 是否启用

运行程序后，查看日志输出：

- ✅ **GPU 已启用**：会看到类似 `whisper_backend_init_gpu: using GPU` 的日志
- ❌ **GPU 未启用**：会看到 `whisper_backend_init_gpu: no GPU found` 的日志

## 性能对比

启用 GPU 后，预期性能提升：

- **CPU（当前）**：~18.5 秒（5.28 秒音频，实时倍率 3.5x）
- **GPU（预期）**：~1-3 秒（实时倍率 0.2-0.6x）

## 故障排除

### 问题：GPU 未被检测到

1. 检查驱动是否正确安装：
   ```bash
   # Windows
   nvidia-smi  # NVIDIA GPU
   
   # Linux
   nvidia-smi  # NVIDIA GPU
   rocm-smi    # AMD GPU
   ```

2. 检查环境变量是否正确设置

3. 检查 whisper-rs 是否编译了 GPU 支持

### 问题：编译错误

1. 确保安装了对应 GPU 的 SDK（CUDA/ROCm/Metal）
2. 检查 Rust 工具链是否支持 GPU 编译
3. 查看 whisper-rs 的文档了解编译要求

## 当前优化（已完成）

- ✅ 自动检测 CPU 核心数并设置线程数
- ✅ 日志输出显示使用的线程数
- ✅ 预留一个核心给系统，避免系统卡顿

## 下一步

1. 根据您的 GPU 类型，按照上述步骤安装驱动和 SDK
2. 修改 `Cargo.toml` 添加 GPU feature flags
3. 重新编译项目
4. 运行测试，查看 GPU 是否被正确使用

