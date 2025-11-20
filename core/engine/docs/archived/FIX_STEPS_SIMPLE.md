# Windows 链接器错误修复步骤（简化版）

## 已完成的步骤

### ✅ 步骤 1: 修改 .cargo/config.toml
- 移除了 `NODEFAULTLIB` 配置
- 文件现在为空（让各 crate 使用默认行为）

### ✅ 步骤 2: 找到 esaxx-rs 的来源
- `tokenizers v0.15.2` → `esaxx-rs v0.1.10`
- `esaxx-rs` 使用 `/MT`，与 `whisper-rs` 的 `/MD` 冲突

## 下一步操作（按顺序执行）

### 步骤 3: 清理编译缓存

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

### 步骤 4: 设置环境变量并编译

**重要**: 在 PowerShell 中设置环境变量，强制所有 C/C++ 代码使用 `/MD`:

```powershell
$env:CC = "cl.exe"
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cargo check --lib
```

**注意**: 这些环境变量只在当前 PowerShell 会话中有效。如果关闭 PowerShell，需要重新设置。

### 步骤 5: 如果步骤 4 成功，测试 TTS

```powershell
cargo test --lib test_text_processor_load -- --nocapture
```

## 如果步骤 4 仍然失败

### 方案 A: 检查是否有全局配置

检查仓库根目录（`D:\Programs\github\lingua`）是否有 `.cargo/config.toml`，确保没有强制 `crt-static` 的配置。

### 方案 B: 使用 WSL/Linux 环境

如果 Windows 环境持续有问题，使用 WSL：

```bash
wsl
cd /mnt/d/Programs/github/lingua/core/engine
cargo check --lib
```

## 验证成功

编译成功后，应该不再出现：
- `error LNK2038: 检测到"RuntimeLibrary"的不匹配项`
- `error LNK2005: ... 已经在 libcpmt.lib(...) 中定义`

