# S2S 完整测试链接器问题说明

## 问题描述

在 Windows 上运行完整的 S2S 测试时遇到链接器错误：

```
error LNK2038: 检测到"RuntimeLibrary"的不匹配项: 值"MD_DynamicRelease"不匹配值"MT_StaticRelease"
```

## 原因

`whisper-rs-sys` 和 `esaxx-rs` 使用了不同的 C++ 运行时库：
- `whisper-rs-sys`: 使用 `MD_DynamicRelease`（动态链接）
- `esaxx-rs`: 使用 `MT_StaticRelease`（静态链接）

这是底层依赖的编译设置问题，无法在应用层直接解决。

## 解决方案

### 方案 1: 分步测试（推荐）

由于完整测试涉及多个组件，可以分步进行：

1. **ASR 测试**: 使用独立的 ASR 测试程序
2. **NMT 测试**: 使用独立的 NMT 测试程序  
3. **TTS 测试**: 使用 Python 脚本或已成功的 Rust 测试程序

### 方案 2: 使用 Python 脚本

使用 `scripts/test_s2s_full_python.py` 进行部分测试（TTS 部分）。

### 方案 3: 修复链接器问题（长期）

需要：
1. 重新编译 `whisper-rs-sys` 使用静态运行时库
2. 或重新编译 `esaxx-rs` 使用动态运行时库
3. 或使用不同的依赖版本

## 当前状态

- ✅ **Piper TTS**: 已成功测试（`test_piper_http_simple.rs`）
- ✅ **简化 S2S 流**: 已成功测试（`test_s2s_flow_simple.rs`，使用模拟 ASR/NMT）
- ❌ **完整 S2S 流**: 链接器错误（需要真实的 Whisper ASR）

## 建议

1. **短期**: 使用分步测试验证各组件功能
2. **中期**: 考虑使用其他 ASR 库或工具（如 OpenAI Whisper CLI）
3. **长期**: 解决链接器兼容性问题或切换到 Linux 环境

## 相关文件

- `core/engine/examples/test_s2s_full_simple.rs` - 完整测试程序（链接器错误）
- `core/engine/examples/test_s2s_flow_simple.rs` - 简化测试程序（成功）
- `scripts/test_s2s_full_python.py` - Python 版本测试脚本

