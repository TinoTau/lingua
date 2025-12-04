# ONNX Runtime 版本问题修复

## 问题描述

测试 SileroVad 时遇到错误：
```
Unsupported model IR version: 10, max supported IR version: 9
```

这表示 ONNX Runtime 版本太旧，不支持 IR version 10 的模型。

## 解决方案

### 方案 1：升级 ONNX Runtime（推荐）

已更新 `Cargo.toml` 中的 `ort` 版本：
```toml
ort = { version = "1.19", default-features = false, features = ["download-binaries"] }
```

**操作步骤：**
1. 更新依赖：
   ```bash
   cd core/engine
   cargo update ort
   ```

2. 重新编译：
   ```bash
   cargo build --release
   ```

3. 重新运行测试：
   ```bash
   cargo run --example test_silero_vad_startup
   ```

### 方案 2：使用 IR version 9 的模型（如果方案 1 不行）

如果升级后仍然不支持，可以下载 IR version 9 的 Silero VAD 模型：

1. 访问 [Silero VAD GitHub](https://github.com/snakers4/silero-vad)
2. 下载 IR version 9 的模型文件
3. 替换 `core/engine/models/vad/silero/silero_vad.onnx`

### 方案 3：检查 ONNX Runtime 版本

如果问题仍然存在，可以检查当前使用的 ONNX Runtime 版本：

```bash
cd core/engine
cargo tree | grep ort
```

确保使用的是最新版本。

## 验证

测试成功后，应该看到：
```
✅ SileroVad 初始化成功
✅ 语音检测正常
✅ 静音检测正常
✅ 自然停顿检测成功
```

## 相关链接

- [ort crate (ONNX Runtime for Rust)](https://crates.io/crates/ort)
- [Silero VAD GitHub](https://github.com/snakers4/silero-vad)
- [ONNX IR Version](https://github.com/onnx/onnx/blob/main/docs/IR.md)

