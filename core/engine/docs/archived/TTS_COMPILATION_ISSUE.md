# TTS 编译卡住问题诊断

**问题**: `cargo build` 和 Python 脚本都会卡住数小时

## 可能的原因

### 1. 文件系统问题
- 模型文件很大（可能 > 1GB），Windows 文件系统访问慢
- 磁盘 I/O 阻塞
- 网络驱动器或虚拟磁盘问题

### 2. 防病毒软件
- 实时扫描大文件（.onnx 文件）
- 扫描 Rust 编译输出
- 建议：将项目目录添加到防病毒软件白名单

### 3. 系统资源
- 内存不足
- CPU 占用过高
- 磁盘空间不足

### 4. 代码问题
- 编译时无限递归
- 宏展开问题
- 依赖编译问题

## 建议的解决方案

### 方案 1: 检查系统资源
```powershell
# 检查磁盘空间
Get-PSDrive C

# 检查内存使用
Get-Process | Sort-Object WorkingSet -Descending | Select-Object -First 10
```

### 方案 2: 临时禁用防病毒软件
- 将项目目录添加到防病毒软件白名单
- 或临时禁用实时扫描

### 方案 3: 使用 WSL 或 Linux 环境
```bash
# 在 WSL 中编译
wsl
cd /mnt/d/Programs/github/lingua/core/engine
cargo build --lib
```

### 方案 4: 简化代码测试
- 先注释掉 TTS 相关代码
- 确认其他模块能正常编译
- 再逐步添加 TTS 代码

### 方案 5: 检查是否有编译错误
创建一个最小测试文件：
```rust
// core/engine/src/tts_streaming/test_minimal.rs
pub fn test() {
    println!("test");
}
```

## 快速诊断步骤

1. **检查模型文件大小**
   ```powershell
   Get-ChildItem -Path "core\engine\models\tts" -Recurse -File | Select-Object Name, @{Name="Size(MB)";Expression={[math]::Round($_.Length/1MB,2)}}
   ```

2. **检查是否有进程占用**
   ```powershell
   Get-Process | Where-Object {$_.Path -like "*lingua*"}
   ```

3. **尝试清理编译缓存**
   ```powershell
   cd core\engine
   cargo clean
   ```

4. **尝试只编译库（不编译测试）**
   ```powershell
   cargo build --lib --release
   ```

## 临时解决方案

如果编译持续卡住，建议：

1. **先跳过 TTS 模块编译**
   - 在 `core/engine/src/lib.rs` 中注释掉 `pub mod tts_streaming;`
   - 确认其他模块能正常编译

2. **在 Linux/macOS 环境测试**
   - 使用 WSL、Docker 或 CI 环境

3. **分步实现**
   - 先实现基础结构（不加载模型）
   - 再逐步添加模型加载逻辑

---

**建议**: 如果问题持续，优先在 Linux/macOS 环境进行开发和测试。

