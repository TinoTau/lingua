# 手动编译诊断指南

**鏈€鍚庢洿鏂?*: 2024-12-19

---


**������**: 2024-12-19

---

# 手动编译诊断指南

**目的**: 通过逐步执行命令，定位编译卡住的具体步骤

---

## 📋 准备工作

### 1. 打开新的 PowerShell 窗口

**重要**: 使用**管理员权�?*打开 PowerShell，避免权限问题�?

### 2. 切换到项目目�?

```powershell
cd D:\Programs\github\lingua\core\engine
```

---

## 🔍 步骤 1: 检�?Rust 环境

### 命令 1.1: 检�?Rust 版本

```powershell
rustc --version
```

**预期结果**: 显示 Rust 版本（如 `rustc 1.70.0`�?

**如果卡住**: Rust 安装可能有问题，需要重新安�?Rust

**如果成功**: 继续下一�?

---

### 命令 1.2: 检�?Cargo 版本

```powershell
cargo --version
```

**预期结果**: 显示 Cargo 版本（如 `cargo 1.70.0`�?

**如果卡住**: Cargo 安装可能有问�?

**如果成功**: 继续下一�?

---

## 🔍 步骤 2: 清理编译缓存

### 命令 2.1: 清理编译缓存

```powershell
cargo clean
```

**预期结果**: 命令快速完成（几秒内）

**如果卡住超过 30 �?*: 
- 可能是文件系统问�?
- 尝试手动删除 `target` 目录�?
  ```powershell
  Remove-Item -Recurse -Force target -ErrorAction SilentlyContinue
  ```

**如果成功**: 继续下一�?

---

## 🔍 步骤 3: 检查依�?

### 命令 3.1: 只检查依赖（不编译代码）

```powershell
cargo check --lib --message-format=short 2>&1 | Select-Object -First 20
```

**预期结果**: 
- 开始下载依赖（如果有）
- 或显示编译错误信�?

**如果卡住超过 5 分钟**: 
- 可能是网络问题（下载依赖�?
- 或依赖解析问�?
- **记录卡住的时间点**

**如果成功**: 继续下一�?

---

## 🔍 步骤 4: 检查语法（最小范围）

### 命令 4.1: 只检�?lib.rs

```powershell
rustc --crate-type lib --edition 2021 src/lib.rs --extern async_trait=target\debug\deps\libasync_trait-*.rlib 2>&1 | Select-Object -First 30
```

**注意**: 这个命令可能因为缺少依赖而失败，但可以检查语法错误�?

**如果卡住**: 可能�?`lib.rs` 有语法问�?

**如果显示错误**: 记录错误信息

---

## 🔍 步骤 5: 逐个模块检�?

### 命令 5.1: 临时禁用 TTS 模块

**编辑 `src/lib.rs`**，注释掉�?
```rust
// pub mod tts_streaming;
```

**编辑 `src/lib.rs`**，注释掉�?
```rust
// pub use tts_streaming::{...};
```

**编辑 `src/bootstrap.rs`**，注释掉所�?TTS 相关代码（参�?`QUICK_FIX_DISABLE_TTS.md`�?

然后运行�?
```powershell
cargo check --lib --message-format=short 2>&1 | Select-Object -First 30
```

**如果成功**: 说明问题�?TTS 模块

**如果仍然卡住**: 问题不在 TTS 模块，继续下一�?

---

### 命令 5.2: 只编译特定模�?

如果禁用 TTS 后能编译，逐个启用 TTS 子模块：

**编辑 `src/tts_streaming/mod.rs`**，注释掉�?
```rust
// mod fastspeech2_tts;
// mod text_processor;
// mod audio_utils;
// mod stub;
```

然后逐个取消注释，找到导致问题的模块�?

---

## 🔍 步骤 6: 检查特定文�?

### 命令 6.1: 检�?TTS 模块的语�?

```powershell
# 检�?fastspeech2_tts.rs
rustc --crate-type lib --edition 2021 src/tts_streaming/fastspeech2_tts.rs --allow unused 2>&1 | Select-Object -First 30

# 检�?text_processor.rs
rustc --crate-type lib --edition 2021 src/tts_streaming/text_processor.rs --allow unused 2>&1 | Select-Object -First 30

# 检�?audio_utils.rs
rustc --crate-type lib --edition 2021 src/tts_streaming/audio_utils.rs --allow unused 2>&1 | Select-Object -First 30
```

**注意**: 这些命令会显示很多错误（因为缺少依赖），但可以检查语法问题�?

---

## 🔍 步骤 7: 使用详细输出

### 命令 7.1: 详细编译输出

```powershell
$env:RUST_BACKTRACE=1
cargo check --lib -v 2>&1 | Tee-Object -FilePath cargo_check_verbose.log
```

**预期结果**: 
- 显示详细的编译步�?
- 可以看到卡在哪一�?

**如果卡住**: 
- 查看 `cargo_check_verbose.log` 的最后几�?
- 确定卡在哪一�?

---

## 🔍 步骤 8: 检查系统资�?

### 命令 8.1: 监控资源使用

�?*另一�?PowerShell 窗口**运行�?

```powershell
while ($true) {
    $proc = Get-Process cargo,rustc -ErrorAction SilentlyContinue
    if ($proc) {
        $proc | Select-Object Id, ProcessName, CPU, WorkingSet, @{N="Memory(MB)";E={[math]::Round($_.WorkingSet/1MB,1)}} | Format-Table
    }
    Start-Sleep -Seconds 5
}
```

**观察**:
- CPU 使用率是否持�?100%
- 内存使用是否持续增长
- 是否有多�?cargo/rustc 进程

---

## 📊 诊断结果记录�?

| 步骤 | 命令 | 结果 | 卡住时间 | 备注 |
|------|------|------|----------|------|
| 1.1 | `rustc --version` | �?�?| ___ �?| |
| 1.2 | `cargo --version` | �?�?| ___ �?| |
| 2.1 | `cargo clean` | �?�?| ___ �?| |
| 3.1 | `cargo check --lib` | �?�?| ___ �?| 卡在：___ |
| 5.1 | 禁用 TTS 后编�?| �?�?| ___ �?| |
| 7.1 | 详细输出 | �?�?| ___ �?| 最后输出：___ |

---

## 🚨 如果手动编译也无法解�?

### 方案 A: 使用 WSL（推荐）

如果 Windows 环境持续有问题，使用 WSL�?

```bash
# 1. 安装 WSL（如果未安装�?
wsl --install

# 2. 进入 WSL
wsl

# 3. 安装 Rust（在 WSL 中）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. 切换到项目目�?
cd /mnt/d/Programs/github/lingua/core/engine

# 5. 编译
cargo check --lib
```

**优点**: 
- 完全隔离的环�?
- 通常�?Windows 更稳�?
- 可以排除 Windows 特定问题

---

### 方案 B: 使用 Docker

如果 WSL 不可用，使用 Docker�?

```powershell
# 1. 确保 Docker Desktop 运行

# 2. 运行 Rust 容器
docker run -it --rm -v D:\Programs\github\lingua:/workspace rust:latest bash

# 3. 在容器中
cd /workspace/core/engine
cargo check --lib
```

**优点**: 
- 完全隔离的环�?
- 不依赖系统配�?
- 可以快速重�?

---

### 方案 C: 使用 GitHub Actions CI

如果本地环境持续有问题，使用 CI 环境�?

1. 创建 `.github/workflows/check.yml`:
```yaml
name: Check

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Check
        run: cd core/engine && cargo check --lib
```

2. 提交代码，查�?CI 结果

**优点**: 
- 完全干净�?Linux 环境
- 可以验证代码本身是否有问�?
- 不依赖本地环�?

---

### 方案 D: 使用虚拟�?

如果以上方案都不可用，使用虚拟机�?

1. 安装 VirtualBox �?VMware
2. 安装 Ubuntu Linux
3. 在虚拟机中编�?

**优点**: 
- 完全隔离的环�?
- 可以排除所�?Windows 问题

---

### 方案 E: 联系支持

如果所有方案都失败，可能需要：

1. **收集信息**:
   - Windows 版本
   - Rust 版本 (`rustc --version`)
   - Cargo 版本 (`cargo --version`)
   - 系统资源（内存、CPU�?
   - 防病毒软件信�?
   - 编译日志（如果有�?

2. **创建最小复�?*:
   - 创建一个最小的 Rust 项目
   - 逐步添加代码，找到导致问题的代码

3. **提交 Issue**:
   - Rust 官方 Issue: https://github.com/rust-lang/rust/issues
   - 或项�?Issue（如果有�?

---

## 📝 快速诊断流�?

```
1. rustc --version          �?如果卡住：Rust 安装问题
2. cargo --version          �?如果卡住：Cargo 安装问题
3. cargo clean              �?如果卡住：文件系统问�?
4. cargo check --lib        �?如果卡住：继续下一�?
5. 禁用 TTS 模块后编�?     �?如果成功：问题在 TTS 模块
6. 使用 WSL/Docker/CI       �?如果成功：Windows 环境问题
```

---

## 🎯 推荐流程

1. **先尝试步�?1-3**（基础检查）
2. **如果步骤 3 卡住**，尝试手动删�?`target` 目录
3. **如果步骤 4 卡住**，尝试禁�?TTS 模块（步�?5�?
4. **如果禁用 TTS 后仍卡住**，使�?WSL/Docker（方�?A/B�?
5. **如果 WSL/Docker 也卡�?*，使�?CI（方�?C�?

---

**最后更�?*: 2024-12-19

