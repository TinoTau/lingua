# ONNX Runtime (ort) 升级影响分析

**日期**: 2025-11-21  
**状态**: 📋 分析完成

## 当前情况

### ort 版本
- **当前版本**: `ort = "1.16.3"`
- **需要支持**: IR 10（`marian-zh-en` encoder 模型）

### 依赖 ort 的功能模块

1. **NMT (Marian ONNX)** - 核心功能 ✅
   - `nmt_incremental/marian_onnx.rs`
   - `nmt_incremental/encoder.rs`
   - `nmt_incremental/decoder.rs`
   - `nmt_incremental/translation.rs`
   - **使用情况**: 所有 NMT 翻译功能

2. **Emotion (XLM-R)** - 核心功能 ⚠️
   - `emotion_adapter/xlmr_emotion.rs`
   - **使用情况**: 情感分析功能
   - **已知问题**: Emotion 模型也是 IR 10，当前无法加载

3. **TTS (FastSpeech2, VITS)** - 核心功能 ✅
   - `tts_streaming/fastspeech2_tts.rs`
   - `tts_streaming/vits_tts.rs`
   - **使用情况**: 文本转语音功能（英文）
   - **注意**: 中文 TTS 已改用 Piper HTTP，不依赖 ort

## 升级风险分析

### 风险点

1. **API 兼容性**
   - `ort` 不同版本之间可能有 API 变化
   - 需要检查所有使用 `ort` 的代码

2. **模型兼容性**
   - 升级后可能影响现有 IR 9 模型的加载
   - 需要测试所有现有模型

3. **依赖冲突**
   - 可能与其他依赖产生冲突
   - 需要全面测试

### 影响范围

**高风险模块**:
- ✅ NMT: 所有测试都使用 `marian-en-zh`（IR 9），应该兼容
- ⚠️ Emotion: 当前无法加载（IR 10），升级后可能可以加载
- ✅ TTS: 英文 TTS 使用 IR 9 模型，应该兼容

## 替代方案

### 方案 1: 不升级 ort，使用 `marian-en-zh` 进行测试 ⭐

**优点**:
- 无需修改任何代码
- 不影响现有功能
- 可以验证 S2S 流程（英文→中文）

**缺点**:
- 测试的是英文→中文流程，不是中文→英文流程
- 不符合原始测试目标

**实施步骤**:
1. 修改 `test_s2s_full_simple.rs` 使用 `marian-en-zh`
2. 调整测试流程为：英文语音 → 英文文本 → 中文文本 → 中文语音

### 方案 2: 寻找 IR 9 版本的 `marian-zh-en` 模型

**优点**:
- 无需升级 ort
- 可以测试中文→英文流程

**缺点**:
- 可能需要重新导出模型
- 需要旧版本 PyTorch

**实施步骤**:
1. 使用旧版本 PyTorch（支持 opset 12）重新导出 `marian-zh-en`
2. 验证模型兼容性

### 方案 3: 升级 ort（需要全面测试）⚠️

**优点**:
- 支持 IR 10 模型
- 可以加载 Emotion 模型
- 使用最新版本

**缺点**:
- 需要全面测试所有功能
- 可能有 API 变化
- 风险较高

**实施步骤**:
1. 检查 `ort` 最新版本和支持的 IR 版本
2. 升级 `ort` 版本
3. 运行所有测试（NMT、Emotion、TTS）
4. 修复可能的 API 变化

## 推荐方案

**推荐使用方案 1（使用 `marian-en-zh` 进行测试）**，原因：

1. **风险最低**: 无需修改任何依赖或模型
2. **快速验证**: 可以立即验证 S2S 流程
3. **不影响现有功能**: 所有现有功能继续正常工作
4. **后续可扩展**: 等 ort 升级方案验证后再测试中文→英文流程

## 下一步行动

1. **短期方案**: 使用 `marian-en-zh` 进行 S2S 测试 ✅
2. **长期方案**: 评估升级 ort 的可行性 🔴
   - 检查 ort 最新版本
   - 评估 API 兼容性
   - 制定测试计划

## 相关文件

- `core/engine/Cargo.toml` - ort 版本定义
- `core/engine/src/nmt_incremental/` - NMT 模块
- `core/engine/src/emotion_adapter/` - Emotion 模块
- `core/engine/src/tts_streaming/` - TTS 模块

