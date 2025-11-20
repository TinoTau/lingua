# VITS 中文 AISHELL3 实现总结

**日期**: 2024-12-19  
**状态**: ✅ 代码实现完成，待测试

---

## 实现概述

成功实现了对 `csukuangfj/vits-zh-aishell3` 模型的支持，这是一个中文多说话人 VITS TTS 模型。

---

## 已完成的工作

### 1. Tokenizer 实现
- ✅ **VitsZhAishell3Tokenizer** (`core/engine/src/tts_streaming/vits_zh_aishell3_tokenizer.rs`)
  - 从 `tokens.txt` 加载音素到 ID 的映射
  - 从 `lexicon.txt` 加载汉字到拼音的映射
  - 实现中文文本到音素序列的转换
  - 处理特殊 token：`sil` (0), `eos` (1), `sp` (2)

### 2. 模型加载
- ✅ 自动检测模型类型（通过检查 `tokens.txt` 和 `lexicon.txt`）
- ✅ 支持 `vits-aishell3.onnx` 和 `vits-aishell3.int8.onnx`
- ✅ 优先使用 `vits-zh-aishell3`，如果没有则回退到 `mms-tts-zh-Hans`

### 3. 推理实现
- ✅ **run_inference_aishell3** 方法
  - 处理 6 个输入：`x`, `x_length`, `noise_scale`, `length_scale`, `noise_scale_w`, `sid`
  - 提取输出音频波形 `[N, 1, L]` 并转换为 `[L]`
  - 支持多说话人（通过 `sid` 参数）

### 4. 代码集成
- ✅ 更新 `VitsTtsEngine` 结构体支持两种模型格式
- ✅ 根据 `locale` 自动选择模型类型和推理方法
- ✅ 保持向后兼容（英文模型仍使用 MMS TTS 格式）

---

## 模型输入输出

### 输入：
1. **x**: `tensor(int64)`, shape `[N, L]` - 音素 token IDs
2. **x_length**: `tensor(int64)`, shape `[N]` - 序列长度
3. **noise_scale**: `tensor(float)`, shape `[1]` - 噪声缩放（默认 0.667）
4. **length_scale**: `tensor(float)`, shape `[1]` - 长度缩放（默认 1.0）
5. **noise_scale_w**: `tensor(float)`, shape `[1]` - 噪声缩放 w（默认 0.8）
6. **sid**: `tensor(int64)`, shape `[1]` - 说话人 ID（默认 0）

### 输出：
- **y**: `tensor(float)`, shape `[N, 1, L]` - 音频波形

---

## 文本处理流程

1. **中文文本** → 汉字序列
2. **汉字** → 查找 `lexicon.txt` → 得到 (声母, 韵母, 音调)
3. **(声母, 韵母, 音调)** → 转换为音素 tokens
   - 例如：`^ i1` → `^` (7) + `i1` (79)
4. **添加特殊 token**：
   - 开头：`sil` (0)
   - 字之间：`sp` (2)
   - 结尾：`eos` (1)

---

## 使用方法

### 在 CoreEngine 中使用

```rust
use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::tts_streaming::VitsTtsEngine;

let engine = CoreEngineBuilder::new()
    // ... 其他配置
    .tts_with_default_vits()?  // 自动加载 vits-zh-aishell3（如果存在）
    .build()?;
```

### 直接使用

```rust
use core_engine::tts_streaming::{VitsTtsEngine, TtsRequest};

let tts_engine = VitsTtsEngine::new_from_models_root(&models_root)?;

// 中文合成
let request = TtsRequest {
    text: "你好，世界。".to_string(),
    voice: "default".to_string(),
    locale: "zh".to_string(),  // 或 "zh-CN", "zh-TW", "cmn"
};
let chunk = tts_engine.synthesize(request).await?;
```

---

## 文件结构

```
core/engine/
├── src/tts_streaming/
│   ├── mod.rs                          # 导出模块
│   ├── vits_tts.rs                     # VITS TTS 引擎（已更新）
│   └── vits_zh_aishell3_tokenizer.rs   # 中文 AISHELL3 Tokenizer（新建）
├── models/tts/
│   ├── mms-tts-eng/                    # 英文模型
│   └── vits-zh-aishell3/               # 中文模型（已下载）
│       ├── vits-aishell3.onnx          # ONNX 模型
│       ├── tokens.txt                  # 音素到 ID 映射
│       └── lexicon.txt                 # 汉字到拼音映射
└── tests/
    └── vits_tts_test.rs                # 测试（待更新）
```

---

## 下一步

### 1. 测试
- [ ] 创建中文 TTS 测试用例
- [ ] 验证 tokenizer 编码结果
- [ ] 验证音频输出质量

### 2. 功能扩展
- [ ] 支持多说话人选择（通过 `sid` 参数）
- [ ] 支持语速控制（通过 `length_scale` 参数）
- [ ] 支持音调控制（通过 `noise_scale` 参数）

### 3. 优化
- [ ] 优化 tokenizer 性能（缓存 lexicon 查找）
- [ ] 支持批量推理
- [ ] 支持流式输出

---

## 已知限制

1. **当前仅支持说话人 0**：需要从模型或配置中获取说话人数量
2. **固定参数**：`noise_scale`, `length_scale`, `noise_scale_w` 使用默认值
3. **无拼音转换工具**：依赖 `lexicon.txt`，如果汉字不在词典中会被跳过

---

## 相关文档

- `VITS_ZH_AISHELL3_IMPLEMENTATION_PLAN.md` - 实现计划
- `scripts/check_vits_zh_aishell3_manual.md` - 手动检查指南
- `scripts/download_vits_zh_aishell3_manual.md` - 下载指南

---

## 总结

✅ VITS 中文 AISHELL3 模型支持已实现  
✅ Tokenizer 和推理逻辑已完成  
✅ 代码已集成到 `VitsTtsEngine`  
⏳ 待测试验证

**下一步**：创建测试用例并验证功能。

