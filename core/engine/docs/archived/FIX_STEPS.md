# Windows 链接器错误修复步骤

根据 `WINDOWS_RUNTIME_LIBRARY_MISMATCH_FIX.md` 的指导，按以下步骤修复：

## 步骤 1: 修改 .cargo/config.toml ✅

已移除 `NODEFAULTLIB` 配置，让各 crate 使用统一的 /MD 默认行为。

## 步骤 2: 检查 esaxx-rs 的来源

执行以下命令（逐个执行，不要用管道）：

```powershell
cd D:\Programs\github\lingua\core\engine
cargo tree
```

在输出中搜索 `esaxx`，找到是哪个依赖引入了它。

## 步骤 3: 检查 esaxx-rs 的配置

如果找到了 esaxx-rs，检查它是否有 `static` 或 `msvc-static` feature。

## 步骤 4: 清理并重新编译

```powershell
cargo clean
cargo check --lib
```

## 步骤 5: 如果还有错误

根据错误信息，检查对应的 crate 是否使用了 static CRT。

