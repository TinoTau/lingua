# 下一步行动建议

## 📊 当前项目状态

### ✅ 已完成模块

1. **NMT 翻译模块** - ✅ **100% 完成**
   - ✅ 真实 ONNX 推理实现
   - ✅ KV Cache 优化（已根据 `marian_nmt_interface_spec.md` 修复）
   - ✅ 完整测试套件
   - ✅ 已集成到 CoreEngine

2. **ASR Whisper 模块** - ✅ **96% 完成**
   - ✅ 完整实现（推理、流式、VAD 集成）
   - ✅ 完整测试覆盖
   - ✅ 已集成到 CoreEngine
   - ⚠️ 少量优化待完成（性能、错误处理）

---

## 🎯 推荐下一步任务（按优先级）

### 🔴 P0 - 立即开始（核心功能完善）

#### 1. Emotion 适配器实现 ⭐ **推荐优先**
- **任务 ID**: `emotion-1`, `emotion-2`, `emotion-3`
- **状态**: ❌ 未开始（仅有 trait 定义）
- **文件**: `core/engine/src/emotion_adapter/mod.rs`
- **任务**:
  - [ ] 步骤 1: 集成 XLM-R ONNX 模型（加载和推理）
  - [ ] 步骤 2: 实现 `EmotionAdapter` trait（情感分类逻辑）
  - [ ] 步骤 3: 添加 Emotion 测试用例
- **预计时间**: 3-5 天
- **优先级原因**: 
  - 完整业务流程的关键组件
  - 相对独立，可以快速实现
  - 为后续 Persona 和 TTS 提供基础

**具体步骤**:
1. 准备 XLM-R ONNX 模型（情感分类）
2. 实现模型加载和推理逻辑
3. 实现 `EmotionAdapter` trait
4. 编写单元测试和集成测试

---

### 🟡 P1 - 高优先级（业务流程完善）

#### 2. Persona 适配器实现
- **任务 ID**: `persona-1`, `persona-2`
- **状态**: ❌ 未开始（仅有 trait 定义）
- **文件**: `core/engine/src/persona_adapter/mod.rs`
- **任务**:
  - [ ] 实现 `PersonaAdapter` trait（文本个性化规则/模板）
  - [ ] 添加 Persona 测试用例
- **预计时间**: 2-3 天
- **优先级原因**: 
  - 完整业务流程的一部分
  - 相对简单（主要是文本处理规则）

**具体步骤**:
1. 设计个性化规则引擎
2. 实现文本转换逻辑
3. 编写测试用例

#### 3. ASR Whisper 优化（可选）
- **任务 ID**: `asr-optimization`
- **状态**: ⚠️ 部分完成
- **任务**:
  - [ ] 性能优化（内存使用、推理速度）
  - [ ] 日志和监控增强
  - [ ] 错误处理完善
- **预计时间**: 4-6 小时
- **优先级原因**: 提升用户体验，但不影响核心功能

---

### 🟢 P2 - 中优先级（完整流程）

#### 4. TTS 合成实现
- **任务 ID**: `tts-1`, `tts-2`, `tts-3`, `tts-4`
- **状态**: ❌ 未开始（仅有 trait 定义）
- **文件**: `core/engine/src/tts_streaming/`
- **任务**:
  - [ ] 集成 FastSpeech2 + HiFiGAN 模型
  - [ ] 实现 `TtsStreaming` trait（PCM 音频生成）
  - [ ] 实现流式音频输出（chunk 拼接）
  - [ ] 添加 TTS 测试用例（输出 WAV 文件验证）
- **预计时间**: 7-11 天
- **优先级原因**: 
  - 完整业务流程的最后一步
  - 实现复杂度较高
  - 建议在 Emotion/Persona 完成后开始

---

## 📋 推荐执行顺序

### 第一阶段：核心功能完善（1-2 周）

**本周（推荐）**:
1. **Emotion 适配器实现** (P0) - 3-5 天
   - 这是最优先的任务
   - 相对独立，可以快速完成
   - 为后续模块提供基础

**下周**:
2. **Persona 适配器实现** (P1) - 2-3 天
   - 与 Emotion 可以并行开发
   - 相对简单，主要是文本处理

3. **ASR Whisper 优化** (P1) - 1 天（可选）
   - 性能优化和错误处理完善

**小计**: 6-9 天

---

### 第二阶段：TTS 模块（1-2 周）

4. **TTS 合成实现** (P1) - 7-11 天
   - 完成音频合成功能
   - 实现流式输出

**小计**: 7-11 天

---

### 第三阶段：WASM 和浏览器集成（4-6 周）

5. **WASM 构建环境准备** (P2) - 7-12 天
6. **Chrome 插件集成** (P2) - 15-25 天

**小计**: 22-37 天

---

## 🎯 立即行动项

### 今天/明天可以开始

**Emotion 适配器实现 - 步骤 1: 模型准备和加载**

1. **研究 XLM-R 情感分类模型**
   - 查找可用的 ONNX 模型
   - 了解模型输入输出格式
   - 准备模型文件

2. **实现模型加载逻辑**
   - 参考 `MarianNmtOnnx` 的实现
   - 创建 `XlmREmotionEngine` 结构
   - 实现模型加载方法

3. **实现基础推理**
   - 实现文本编码
   - 实现模型推理
   - 实现结果解码

**文件结构建议**:
```
core/engine/src/emotion_adapter/
├── mod.rs              # trait 定义（已有）
├── xlmr_emotion.rs     # XLM-R 模型实现
└── stub.rs             # stub 实现（可选）
```

---

## 📝 注意事项

1. **NMT KV Cache 已修复** ✅
   - 根据 `marian_nmt_interface_spec.md` 的方案已实现
   - 不再需要 workaround 模式
   - 可以标记为完成

2. **模块依赖关系**
   - Emotion 和 Persona 可以并行开发
   - TTS 依赖 Emotion/Persona 的输出
   - WASM 需要所有模块完成

3. **测试策略**
   - 每个模块都应该有完整的单元测试
   - 集成测试验证端到端流程
   - 性能测试确保满足要求

---

## 🚀 快速开始指南

### 开始 Emotion 适配器实现

1. **创建实现文件**:
   ```bash
   # 在 core/engine/src/emotion_adapter/ 目录下
   touch xlmr_emotion.rs
   ```

2. **参考 NMT 实现**:
   - 查看 `core/engine/src/nmt_incremental/marian_onnx.rs`
   - 参考模型加载和推理模式
   - 复用 ONNX Runtime 工具函数

3. **实现步骤**:
   - 步骤 1: 模型加载（1-2 天）
   - 步骤 2: 推理逻辑（1-2 天）
   - 步骤 3: 测试用例（1 天）

---

**最后更新**: 2024-12-19
**当前状态**: NMT KV Cache 已修复，建议开始 Emotion 适配器实现

