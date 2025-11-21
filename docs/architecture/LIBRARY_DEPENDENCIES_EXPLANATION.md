# 库依赖关系说明

## 问题澄清

### 1. 这些库都不要求 Linux

**whisper-rs-sys**:
- **用途**: Whisper ASR（语音识别）的 Rust 绑定
- **平台**: 支持 Windows、Linux、macOS
- **问题**: 在 Windows 上遇到链接器问题（C++ 运行时库不匹配）

**esaxx-rs**:
- **用途**: 文本处理库（用于 tokenizer，特别是 SentencePiece）
- **平台**: 支持 Windows、Linux、macOS
- **问题**: 在 Windows 上遇到链接器问题（C++ 运行时库不匹配）
- **来源**: 这是 `tokenizers` crate 的间接依赖（用于 Emotion 模块的 XLM-R tokenizer）

### 2. 链接器问题的原因

这是 **Windows 特有的编译设置问题**，不是 Linux 要求：

- `whisper-rs-sys`: 使用 `MD_DynamicRelease`（动态链接 C++ 运行时库）
- `esaxx-rs`: 使用 `MT_StaticRelease`（静态链接 C++ 运行时库）

这两个设置不兼容，导致链接器错误。

### 3. 与 Piper 的关系

**Piper 和这些库没有直接关系**：

- **Piper TTS**: 通过 WSL2 运行是因为 Windows 版本的 `piper.exe` 崩溃了（`0xC0000409` 错误），不是因为技术限制
- **Piper HTTP 服务**: 在 WSL2 中运行，但 Rust 客户端（`PiperHttpTts`）在 Windows 上运行正常
- **链接器问题**: 只影响使用 `whisper-rs` 的 Rust 程序，不影响 Piper TTS

## 依赖关系图

```
CoreEngine
├── whisper-rs (ASR)
│   └── whisper-rs-sys (C++ 绑定)
│       └── 使用 MD_DynamicRelease ❌
│
├── tokenizers (Emotion 模块)
│   └── esaxx-rs (间接依赖)
│       └── 使用 MT_StaticRelease ❌
│
└── PiperHttpTts (TTS)
    └── reqwest (HTTP 客户端)
        └── 无链接器问题 ✅
```

## 解决方案

### 方案 1: 修复链接器问题（推荐）

需要统一 C++ 运行时库设置：

1. **重新编译 whisper-rs-sys** 使用静态运行时库
2. **或重新编译 esaxx-rs** 使用动态运行时库
3. **或使用不同的依赖版本**

### 方案 2: 分离依赖

- **ASR**: 使用其他库或工具（如 OpenAI Whisper CLI）
- **Emotion**: 如果不需要，可以移除 tokenizers 依赖
- **TTS**: 已正常工作（Piper HTTP）

### 方案 3: 在 Linux/WSL2 中运行

在 Linux 环境中，这些链接器问题通常不会出现，因为：
- Linux 使用不同的链接器（ld）
- 不依赖 Windows 的 C++ 运行时库设置

## 当前状态

| 组件 | 状态 | 平台 | 问题 |
|------|------|------|------|
| Piper TTS | ✅ 正常 | Windows (通过 WSL2) | 无 |
| PiperHttpTts (Rust) | ✅ 正常 | Windows | 无 |
| Whisper ASR | ❌ 链接器错误 | Windows | C++ 运行时库不匹配 |
| Marian NMT | ✅ 正常 | Windows | 无 |
| Emotion (XLM-R) | ❌ 链接器错误 | Windows | C++ 运行时库不匹配 |

## 总结

1. **这些库都不要求 Linux**，都可以在 Windows 上运行
2. **链接器问题是 Windows 特有的编译设置问题**
3. **Piper 在 WSL2 中运行是因为 Windows 版本崩溃，不是技术限制**
4. **链接器问题和 Piper 没有关系**，它们是独立的组件

