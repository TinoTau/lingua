# Windows 链接器错误修复步骤（修正版）

## 问题

设置 `CC = "cl.exe"` 导致找不到编译器工具。应该让 Rust 自动检测编译器，只设置编译标志。

## 正确的修复步骤

### 步骤 1: 清理编译缓存

```powershell
cd D:\Programs\github\lingua\core\engine
cargo clean
```

### 步骤 2: 只设置编译标志（不设置 CC）

在 PowerShell 中执行：

```powershell
$env:CFLAGS = "/MD"
$env:CXXFLAGS = "/MD"
cargo check --lib
```

**注意**: 
- 不要设置 `CC = "cl.exe"`，让 Rust 自动检测
- 只设置 `CFLAGS` 和 `CXXFLAGS` 来指定运行时库

### 步骤 3: 如果步骤 2 不起作用

`cc-rs` crate 可能不会自动使用 `CFLAGS` 和 `CXXFLAGS`。可以尝试：

#### 方案 A: 使用 `cc` crate 的环境变量

```powershell
$env:CC_x86_64_pc_windows_msvc = "cl"
$env:CFLAGS_x86_64_pc_windows_msvc = "/MD"
$env:CXXFLAGS_x86_64_pc_windows_msvc = "/MD"
cargo check --lib
```

#### 方案 B: 检查是否有 Visual Studio 环境

如果系统安装了 Visual Studio，可能需要先设置 Visual Studio 环境：

```powershell
# 找到 Visual Studio 的 vcvarsall.bat（通常在以下路径之一）
# "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
# 或
# "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvarsall.bat"

# 然后运行：
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
cargo check --lib
```

#### 方案 C: 如果环境变量不起作用

可能需要通过 `build.rs` 或修改依赖项来修复。但更简单的方法是使用 WSL/Linux 环境。

## 验证

编译成功后，应该不再出现链接器错误。

