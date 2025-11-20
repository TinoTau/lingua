# Windows 链接器错误修复步骤（完整版）

## 已完成的步骤

### ✅ 步骤 1: 修改 .cargo/config.toml
- 移除了 `NODEFAULTLIB` 配置（这是掩盖症状，不是根治）
- 添加了环境变量配置，强制所有 C/C++ 代码使用 `/MD` (动态运行时)

### ✅ 步骤 2: 找到 esaxx-rs 的来源
- `tokenizers v0.15.2` → `esaxx-rs v0.1.10`
- `esaxx-rs` 使用 `/MT` (静态运行时)，与 `whisper-rs` 的 `/MD` 冲突

## 下一步操作

### 步骤 3: 清理并重新编译

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

### 步骤 4: 测试编译

```powershell
cargo check --lib
```

如果还有链接器错误，尝试设置环境变量：

```powershell
$env:CC = "cl.exe"
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cargo check --lib
```

## 如果仍然失败

### 方案 A: 检查是否有全局 crt-static 配置

检查仓库根目录是否有 `.cargo/config.toml` 或 `.cargo/config`，确保没有：
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-Ctarget-feature=+crt-static"]
```

### 方案 B: 使用环境变量（临时）

在 PowerShell 中设置环境变量后编译：

```powershell
$env:CC = "cl.exe"
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cd D:\Programs\github\lingua\core\engine
cargo clean
cargo check --lib
```

### 方案 C: 如果环境变量不起作用

可能需要通过 `[patch.crates-io]` 来修复 `esaxx-rs`，但这需要 fork 仓库并修改其 `build.rs`。

## 验证

编译成功后，应该不再出现：
- `error LNK2038: 检测到"RuntimeLibrary"的不匹配项`
- `error LNK2005: ... 已经在 libcpmt.lib(...) 中定义`

