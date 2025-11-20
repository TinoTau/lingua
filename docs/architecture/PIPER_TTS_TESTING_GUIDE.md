# Piper TTS 测试指南

**最后更新**: 2025-11-21

## 测试步骤概览

根据 `PIPER_TTS_PC_LOCAL_AND_MOBILE_ONLINE_PLAN_FULL.md` 中的实施计划，以下是各步骤的测试方法。

---

## 步骤 3：独立 Rust 测试程序

### 前提条件
- WSL2 中已启动 Piper HTTP 服务
- 服务运行在 `http://127.0.0.1:5005/tts`

### 运行测试

```bash
cd core/engine
cargo run --example test_piper_http
```

### 成功标准
- ✅ Rust 代码成功保存 `test_output/test_piper_rust.wav`
- ✅ 音频文件大小 > 1024 字节
- ✅ 音频格式为 WAV (RIFF)
- ✅ 播放正常

### 预期输出

```
=== Piper HTTP TTS 独立测试程序 ===

[1/4] 检查 Piper HTTP 服务状态...
[OK] 服务正在运行

[2/4] 创建 Piper HTTP TTS 客户端...
  端点: http://127.0.0.1:5005/tts
  默认语音: zh_CN-huayan-medium
  超时: 8000ms
[OK] 客户端创建成功

[3/4] 准备 TTS 请求...
  文本: 你好，欢迎使用 Lingua 语音翻译系统。
  语音: zh_CN-huayan-medium
  语言: zh-CN

[4/4] 发送 TTS 请求并生成音频...
[OK] 音频生成成功
  耗时: XXXms
  音频大小: XXX 字节
  格式: WAV (RIFF)

[OK] 音频文件已保存
  文件路径: test_output/test_piper_rust.wav
  文件大小: XXX 字节

=== 测试完成 ===
```

---

## 步骤 4：单元测试

### 运行单元测试

```bash
cd core/engine
cargo test --lib tts_streaming::piper_http
```

### 运行集成测试（需要服务运行）

```bash
cd core/engine
cargo test --lib tts_streaming::piper_http -- --ignored
```

### 测试用例

1. **配置测试**
   - `test_piper_http_config_default` - 测试默认配置
   - `test_piper_http_config_custom` - 测试自定义配置

2. **客户端创建测试**
   - `test_piper_http_new` - 测试创建客户端
   - `test_piper_http_with_default_config` - 测试使用默认配置创建

3. **TTS 合成测试**（需要服务运行）
   - `test_piper_http_synthesize` - 基本合成测试
   - `test_piper_http_synthesize_with_default_voice` - 使用默认语音测试
   - `test_piper_http_synthesize_empty_text` - 空文本测试

4. **清理测试**
   - `test_piper_http_close` - 测试关闭方法

### 成功标准
- ✅ 所有单元测试通过
- ✅ 集成测试中 WAV 字节长度 > 1024
- ✅ WAV 格式验证通过（RIFF 头）

---

## 步骤 5：CoreEngine 集成测试

### 前提条件
- WSL2 中已启动 Piper HTTP 服务
- 服务运行在 `http://127.0.0.1:5005/tts`

### 运行测试

```bash
cd core/engine
cargo run --example test_coreengine_piper_tts
```

### 成功标准
- ✅ `test_output/coreengine_zh_test.wav` 成功生成
- ✅ 音频文件大小 > 1024 字节
- ✅ 音频格式为 WAV (RIFF)
- ✅ 播放正常

### 预期输出

```
=== CoreEngine Piper TTS 集成测试 ===

[1/5] 检查 Piper HTTP 服务状态...
[OK] 服务正在运行

[2/5] 构建 CoreEngine（使用 Piper HTTP TTS）...
[OK] CoreEngine 构建成功

[3/5] 初始化 CoreEngine...
[OK] CoreEngine 初始化成功

[4/5] 准备 TTS 请求...
  文本: 你好，欢迎使用 Lingua 语音翻译系统。
  语音: zh_CN-huayan-medium
  语言: zh-CN

[5/5] 执行 TTS 合成...
[OK] TTS 合成成功
  耗时: XXXms
  音频大小: XXX 字节
  格式: WAV (RIFF)

[OK] 音频文件已保存
  文件路径: test_output/coreengine_zh_test.wav

=== 测试完成 ===
```

---

## 故障排查

### 问题 1：无法连接到服务

**错误信息**:
```
[ERROR] 无法连接到服务: ...
```

**解决方法**:
1. 检查 WSL2 中的 Piper HTTP 服务是否正在运行
2. 在 WSL2 中执行：`bash scripts/wsl2_piper/start_piper_service.sh`
3. 验证服务健康检查：`curl http://127.0.0.1:5005/health`

### 问题 2：音频文件为空或很小

**错误信息**:
```
[WARN] 音频文件大小 <= 1024 字节，可能有问题
```

**解决方法**:
1. 检查服务端日志，查看是否有错误
2. 验证文本编码是否正确（UTF-8）
3. 检查模型文件是否存在

### 问题 3：编译错误

**错误信息**:
```
error[E0425]: cannot find function `...` in this scope
```

**解决方法**:
1. 确保所有依赖已正确添加到 `Cargo.toml`
2. 运行 `cargo clean && cargo build` 清理并重新编译

---

## 下一步

完成步骤 3-5 后，可以继续：

1. **步骤 6**: 完整 S2S 流集成测试（Whisper → NMT → Piper TTS）
2. **步骤 7**: 移动端路径验证（云端 Piper）
3. **步骤 8**: PC 端安装流程验证（工程化）

---

## 参考文档

- [WSL2_PIPER_IMPLEMENTATION_SUMMARY.md](./WSL2_PIPER_IMPLEMENTATION_SUMMARY.md) - 实现总结
- [PIPER_TTS_PLAN_PROGRESS.md](./PIPER_TTS_PLAN_PROGRESS.md) - 计划进度
- [scripts/wsl2_piper/README.md](../../scripts/wsl2_piper/README.md) - WSL2 Piper 使用说明

