# esaxx-rs 运行时库修复

**问题**: `esaxx-rs v0.1.10` 使用 `/MT` (静态运行时)，与 `whisper-rs` 的 `/MD` (动态运行时) 冲突。

**来源**: `tokenizers v0.15.2` → `esaxx-rs v0.1.10`

## 解决方案

由于 `esaxx-rs` 是间接依赖，无法直接控制其 feature。但可以通过以下方式修复：

### 方案 1: 检查 esaxx-rs 的 build.rs

如果 `esaxx-rs` 的 `build.rs` 使用了 `.static_crt(true)`，需要修改它。但由于它是外部 crate，可能需要：

1. Fork `esaxx-rs` 仓库
2. 修改 `build.rs` 移除 `.static_crt(true)`
3. 使用 `[patch.crates-io]` 指向 fork 版本

### 方案 2: 使用环境变量（推荐）

在编译时设置环境变量，强制所有 C/C++ 代码使用 `/MD`:

```powershell
$env:CC = "cl.exe"
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cargo clean
cargo check --lib
```

### 方案 3: 通过 .cargo/config.toml 配置

在 `.cargo/config.toml` 中添加：

```toml
[env]
CC = "cl.exe"
CFLAGS = "/MD"
CXXFLAGS = "/MD"
```

### 方案 4: 检查 tokenizers 是否有 feature 可以控制

检查 `tokenizers` 是否有 feature 可以禁用 `esaxx-rs` 或控制其编译方式。

