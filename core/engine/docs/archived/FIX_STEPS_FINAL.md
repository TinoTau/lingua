# Windows 链接器错误修复步骤（最终版）

## 问题

环境变量 `CC = "cl.exe"` 仍然存在，导致找不到编译器。需要先清除它。

## 正确的修复步骤

### 步骤 1: 清除所有相关环境变量

在 PowerShell 中执行：

```powershell
Remove-Item Env:\CC -ErrorAction SilentlyContinue
Remove-Item Env:\CFLAGS -ErrorAction SilentlyContinue
Remove-Item Env:\CXXFLAGS -ErrorAction SilentlyContinue
```

### 步骤 2: 清理编译缓存

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

### 步骤 3: 只设置编译标志（不设置 CC）

```powershell
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cargo check --lib
```

**重要**: 
- 不要设置 `CC`
- 只设置 `CFLAGS` 和 `CXXFLAGS`

### 步骤 4: 如果步骤 3 仍然有链接器错误

环境变量可能不起作用。根据 `WINDOWS_RUNTIME_LIBRARY_MISMATCH_FIX.md`，根本原因是 `esaxx-rs` 的 `build.rs` 使用了 `.static_crt(true)`。

由于 `esaxx-rs` 是间接依赖，我们无法直接修改它。可能需要：

#### 方案 A: 使用 WSL/Linux 环境（推荐）

```bash
wsl
cd /mnt/d/Programs/github/lingua/core/engine
cargo check --lib
```

#### 方案 B: 检查是否有全局配置

检查仓库根目录是否有 `.cargo/config.toml`，确保没有强制 `crt-static` 的配置。

#### 方案 C: 临时禁用 tokenizers（如果不需要）

如果暂时不需要 Emotion 功能，可以临时注释掉 `tokenizers` 依赖：

```toml
# tokenizers = "0.15"  # 临时注释掉
```

## 验证

编译成功后，应该不再出现链接器错误。

