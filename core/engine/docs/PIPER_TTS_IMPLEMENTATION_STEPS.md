# Piper-TTS 实施步骤详细说明

**创建日期**: 2024-12-19  
**基于文档**: `PIPER_TTS_PC_LOCAL_AND_MOBILE_ONLINE_PLAN_FULL.md`

---

## 📋 总体步骤概览

本次改造共涉及 **8 个步骤**，每个步骤都可以**独立测试和验证**。

---

## 步骤 1：在本地命令行直接验证 Piper + 模型

**目标**: 在 PC 上用 Piper 命令行生成中文 WAV

**可测试性**: ✅ **完全独立**，不依赖任何现有代码

**任务清单**:
- [ ] 下载 Piper 二进制（Windows/Linux/macOS）
- [ ] 下载中文模型（`zh_Hans` 系列）
- [ ] 命令行测试生成音频

**测试命令**:
```powershell
# Windows
echo 你好 | piper.exe --model "path/to/zh_female_1.onnx" --output_file test_zh.wav

# Linux/macOS
echo "你好" | piper --model "path/to/zh_female_1.onnx" --output_file test_zh.wav
```

**成功标准**:
- ✅ 生成 `test_zh.wav` 文件
- ✅ 文件大小 > 0
- ✅ 能够正常播放
- ✅ 音频内容清晰可理解

**预计时间**: 1-2 小时

**阻塞风险**: 低（如果 Piper 官方模型不可用，需要寻找替代方案）

---

## 步骤 2：启动本地 Piper HTTP 服务，使用 HTTP 调用验证

**目标**: 启动 `http://127.0.0.1:5005/tts` HTTP 服务

**可测试性**: ✅ **完全独立**，不依赖现有代码

**前置条件**: 步骤 1 完成

**任务清单**:
- [ ] 安装 Python 和 FastAPI
- [ ] 实现 `tts_service.py`（参考方案文档）
- [ ] 启动 HTTP 服务
- [ ] 使用 curl/Postman 测试

**测试命令**:
```powershell
# 启动服务
uvicorn tts_service:app --host 127.0.0.1 --port 5005

# 测试 HTTP 接口（另一个终端）
curl -X POST "http://127.0.0.1:5005/tts" `
     -H "Content-Type: application/json" `
     -d "{\"text\":\"你好\",\"lang\":\"zh-CN\"}" `
     --output test_api.wav
```

**成功标准**:
- ✅ HTTP 服务启动成功
- ✅ HTTP 200 响应
- ✅ 返回 WAV 文件
- ✅ 音频正常播放

**预计时间**: 2-4 小时

**阻塞风险**: 低（FastAPI 是成熟框架）

---

## 步骤 3：用独立 Rust 小程序调用 Piper HTTP（不接 CoreEngine）

**目标**: 验证 Rust HTTP 客户端调用逻辑

**可测试性**: ✅ **完全独立**，创建独立的 Rust 测试项目

**前置条件**: 步骤 2 完成

**任务清单**:
- [ ] 创建独立的 Rust 测试项目
- [ ] 添加 `reqwest` 依赖
- [ ] 实现 HTTP 客户端代码
- [ ] 测试调用 Piper 服务

**测试代码结构**:
```
test_piper_client/
├── Cargo.toml
└── src/
    └── main.rs
```

**成功标准**:
- ✅ Rust 代码编译通过
- ✅ 成功调用 Piper HTTP 服务
- ✅ 保存 `test_rust.wav` 文件
- ✅ 音频正常播放

**预计时间**: 2-3 小时

**阻塞风险**: 低（`reqwest` 是成熟库）

---

## 步骤 4：CoreEngine 内实现 ChinesePiperTtsBackend（单元测试级验证）

**目标**: 在 CoreEngine 中实现 `ChinesePiperTtsBackend`，通过单元测试验证

**可测试性**: ✅ **可独立测试**，使用单元测试框架

**前置条件**: 步骤 3 完成

**任务清单**:
- [ ] 定义 `TtsBackend` trait
- [ ] 实现 `ChinesePiperTtsBackend` 结构体
- [ ] 实现 `synthesize` 方法
- [ ] 编写单元测试

**测试代码示例**:
```rust
#[tokio::test]
async fn test_chinese_piper_backend() {
    let backend = ChinesePiperTtsBackend::new(
        "http://127.0.0.1:5005/tts".to_string(),
        "zh_female_1".to_string(),
    );
    
    let req = TtsRequest {
        text: "你好".to_string(),
        lang: LanguageId::Zh,
    };
    
    let result = backend.synthesize(req).await.unwrap();
    assert!(result.wav_bytes.len() > 1024);
}
```

**成功标准**:
- ✅ 单元测试通过
- ✅ WAV 字节长度 > 1024
- ✅ 音频格式正确

**预计时间**: 4-6 小时

**阻塞风险**: 中（需要设计 trait 接口）

---

## 步骤 5：接入 TtsRouter，在 CoreEngine 中测试文本→语音（不走完整 S2S）

**目标**: 实现 `TtsRouter`，测试文本到语音的转换

**可测试性**: ✅ **可独立测试**，创建集成测试

**前置条件**: 步骤 4 完成

**任务清单**:
- [ ] 实现 `TtsRouter` 结构体
- [ ] 实现路由逻辑（根据语言选择后端）
- [ ] 创建集成测试
- [ ] 测试中文 TTS 流程

**测试代码示例**:
```rust
#[tokio::test]
async fn test_tts_router_chinese() {
    let router = TtsRouter::new(
        english_backend,
        chinese_backend,
    );
    
    let req = TtsRequest {
        text: "你好，欢迎使用语音翻译系统。".to_string(),
        lang: LanguageId::Zh,
    };
    
    let result = router.synthesize(req).await.unwrap();
    // 保存到文件验证
    std::fs::write("coreengine_zh_test.wav", result.wav_bytes).unwrap();
}
```

**成功标准**:
- ✅ 集成测试通过
- ✅ 生成 `coreengine_zh_test.wav` 文件
- ✅ 音频正常播放
- ✅ 内容正确

**预计时间**: 3-4 小时

**阻塞风险**: 低（逻辑相对简单）

---

## 步骤 6：完整 S2S 流集成测试（Whisper → NMT → Piper TTS）

**目标**: 测试完整的语音转语音翻译流程

**可测试性**: ✅ **可独立测试**，创建端到端测试

**前置条件**: 步骤 5 完成

**任务清单**:
- [ ] 更新 `CoreEngineBuilder` 使用新的 `TtsRouter`
- [ ] 创建端到端测试
- [ ] 测试完整 S2S 流程

**测试流程**:
1. 输入中文音频
2. Whisper ASR → 中文文本
3. NMT → 英文文本
4. Piper TTS → 英文音频

**成功标准**:
- ✅ API 返回 200
- ✅ 返回目标语音（英文）
- ✅ 音频可理解
- ✅ 翻译正确

**预计时间**: 4-6 小时

**阻塞风险**: 中（需要确保各模块正确集成）

---

## 步骤 7：移动端路径验证（云端 Piper）

**目标**: 验证云端部署的 Piper 服务

**可测试性**: ✅ **可独立测试**，在云端环境测试

**前置条件**: 步骤 6 完成

**任务清单**:
- [ ] 在云端服务器部署 Piper 服务
- [ ] 配置 Docker 镜像（可选）
- [ ] 更新 CoreEngine 配置使用云端 endpoint
- [ ] 测试移动端 API 调用

**测试方式**:
```bash
# 云端部署
docker run -d -p 5005:5005 piper-tts-service

# 测试 API
curl -X POST "https://api.xxx.com/v1/s2s_translate" \
     -H "Content-Type: application/json" \
     -d '{"audio": "...", "source_lang": "zh-CN", "target_lang": "en-US"}'
```

**成功标准**:
- ✅ 移动端通过后端 API 获得音频
- ✅ 日志显示 CoreEngine 调用云端 Piper
- ✅ 音频质量与本地一致

**预计时间**: 4-8 小时（取决于部署环境）

**阻塞风险**: 中（需要云端环境）

---

## 步骤 8：PC 端安装流程验证（工程化）

**目标**: 完成 PC 端安装器和自动化部署

**可测试性**: ✅ **可独立测试**，在干净环境测试安装

**前置条件**: 步骤 7 完成

**任务清单**:
- [ ] 创建安装器（NSIS/Inno Setup）
- [ ] 打包 Python + Piper + 模型
- [ ] 创建服务管理脚本
- [ ] 测试自动启动
- [ ] 测试 Chrome 插件调用

**测试环境**:
- 干净的 Windows 虚拟机
- 无 Python 环境
- 无 Piper 安装

**成功标准**:
- ✅ 在干净环境中安装后可直接使用
- ✅ Piper 服务自动随程序启动
- ✅ Chrome 插件/桌面端调用完整链路成功
- ✅ 无需手动配置

**预计时间**: 8-16 小时（取决于安装器复杂度）

**阻塞风险**: 中（安装器开发可能遇到问题）

---

## 📊 步骤依赖关系图

```
步骤 1 (验证 Piper)
    ↓
步骤 2 (HTTP 服务)
    ↓
步骤 3 (Rust 客户端)
    ↓
步骤 4 (ChinesePiperTtsBackend)
    ↓
步骤 5 (TtsRouter)
    ↓
步骤 6 (完整 S2S 流程)
    ↓
步骤 7 (云端部署) ──┐
    ↓                │
步骤 8 (PC 安装器) ──┘
```

**注意**: 步骤 7 和 8 可以并行开发（云端部署和 PC 安装器互不依赖）

---

## ⏱️ 时间估算

| 步骤 | 预计时间 | 累计时间 |
|------|---------|---------|
| 步骤 1 | 1-2 小时 | 1-2 小时 |
| 步骤 2 | 2-4 小时 | 3-6 小时 |
| 步骤 3 | 2-3 小时 | 5-9 小时 |
| 步骤 4 | 4-6 小时 | 9-15 小时 |
| 步骤 5 | 3-4 小时 | 12-19 小时 |
| 步骤 6 | 4-6 小时 | 16-25 小时 |
| 步骤 7 | 4-8 小时 | 20-33 小时 |
| 步骤 8 | 8-16 小时 | 28-49 小时 |

**总计**: 约 **3.5 - 6 个工作日**（按每天 8 小时计算）

---

## ✅ 每个步骤的独立测试能力

所有 8 个步骤都**可以独立测试**：

1. **步骤 1-3**: 完全独立，不依赖现有代码
2. **步骤 4-5**: 使用单元测试和集成测试，可独立运行
3. **步骤 6**: 端到端测试，可独立运行
4. **步骤 7**: 云端环境测试，可独立验证
5. **步骤 8**: 安装器测试，可在干净环境独立验证

**优势**:
- ✅ 每个步骤都有明确的成功标准
- ✅ 可以逐步验证，降低风险
- ✅ 发现问题可以及时回退
- ✅ 可以并行开发（步骤 7 和 8）

---

## 🎯 建议的实施策略

1. **按顺序执行步骤 1-6**: 确保核心功能正确
2. **并行执行步骤 7 和 8**: 提高效率
3. **每步验证通过后再继续**: 降低风险
4. **保留回退方案**: 如果某步失败，可以回退到上一步

---

## 📝 测试检查清单

每个步骤完成后，检查以下项目：

- [ ] 代码编译通过
- [ ] 单元测试/集成测试通过
- [ ] 生成的音频文件可播放
- [ ] 音频内容正确
- [ ] 性能可接受（延迟 < 500ms）
- [ ] 错误处理完善
- [ ] 日志记录完整

