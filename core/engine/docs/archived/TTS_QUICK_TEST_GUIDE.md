# TTS 快速测试指南

## 问题：编译和脚本都卡住

如果 `cargo build` 和 Python 脚本都卡住，可能是系统环境问题。

## 快速诊断

### 1. 使用 PowerShell 脚本检查文件（最快）
```powershell
cd D:\Programs\github\lingua
.\scripts\check_tts_files_simple.ps1
```

这个脚本只检查文件是否存在，不读取内容，应该很快。

### 2. 如果 PowerShell 脚本也卡住

可能是：
- **防病毒软件**正在扫描大文件
- **文件系统**问题（网络驱动器、虚拟磁盘）
- **磁盘 I/O**阻塞

**解决方案**：
1. 将项目目录添加到防病毒软件白名单
2. 检查任务管理器，看是否有进程占用 CPU/磁盘
3. 尝试在 WSL 或 Linux 环境测试

### 3. 临时跳过 TTS 模块编译

如果编译持续卡住，可以临时注释掉 TTS 模块：

**在 `core/engine/src/lib.rs` 中**：
```rust
// 临时注释掉
// pub mod tts_streaming;
```

**在 `core/engine/src/lib.rs` 的 pub use 中**：
```rust
// 临时注释掉
// pub use tts_streaming::{TtsRequest, TtsStreamChunk, TtsStreaming, FastSpeech2TtsEngine, TtsStub};
```

然后测试其他模块是否能正常编译。

### 4. 在 Linux/macOS 环境测试

如果 Windows 环境持续有问题，建议：
- 使用 WSL (Windows Subsystem for Linux)
- 使用 Docker
- 使用 CI 环境（GitHub Actions）

## 当前 TTS 实现状态

✅ **已完成**：
- 基础结构（FastSpeech2TtsEngine）
- 文本预处理器框架
- Stub 实现
- FastSpeech2 和 HiFiGAN 推理框架

⚠️ **待完善**：
- 文本预处理（音素转换）
- FastSpeech2 输入形状调整
- 完整测试

## 建议

1. **先解决编译环境问题**
   - 检查防病毒软件
   - 检查系统资源
   - 考虑使用 Linux 环境

2. **分步测试**
   - 先测试模型文件是否存在
   - 再测试代码编译
   - 最后测试功能

3. **如果环境问题无法解决**
   - 可以先完成代码实现
   - 在 Linux/macOS 环境统一测试

