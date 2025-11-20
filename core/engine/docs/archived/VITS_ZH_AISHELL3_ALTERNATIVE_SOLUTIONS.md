# VITS 中文 AISHELL3 替代方案

**日期**: 2024-12-19  
**问题**: vits-zh-aishell3 模型无法生成可识别的音频

---

## 问题总结

- ✅ Tokenizer 编码正确
- ✅ 模型可以正常加载和推理
- ✅ 生成的音频文件存在且长度合理
- ❌ **所有格式的音频都无法识别任何一个字**

经过多次测试（不同参数、不同格式、不同说话人），问题仍然存在，说明可能是模型本身的问题。

---

## 替代方案

### 方案 1: 使用 MMS TTS 中文模型

**优点**:
- MMS TTS 是 Meta 的官方模型，质量有保障
- 英文版本已经验证可用
- 支持多语言

**缺点**:
- 中文模型可能不存在或质量不佳
- 需要重新实现 tokenizer

**实施步骤**:
1. 查找 MMS TTS 中文模型（`facebook/mms-tts-zh-Hans` 或类似）
2. 如果存在，下载并测试
3. 如果可用，集成到现有系统

---

### 方案 2: 使用其他 VITS 中文模型

**可能的模型**:
- 其他 Hugging Face 上的 VITS 中文模型
- 其他开源的中文 TTS 模型

**实施步骤**:
1. 搜索 Hugging Face 上的其他 VITS 中文模型
2. 下载并测试
3. 如果可用，集成到现有系统

---

### 方案 3: 使用 FastSpeech2 + HiFiGAN（之前尝试过）

**状态**: 之前尝试过，但 HiFiGAN 输出的是 mel 特征而不是音频波形

**如果重新尝试**:
1. 检查是否有其他 HiFiGAN 模型
2. 或者使用其他声码器（如 WaveGlow、MelGAN 等）
3. 或者使用完整的 FastSpeech2 + HiFiGAN 管道

---

### 方案 4: 暂时只支持英文 TTS

**优点**:
- 英文 TTS 已经验证可用（MMS TTS）
- 可以快速上线

**缺点**:
- 不支持中文 TTS
- 功能不完整

**实施步骤**:
1. 暂时禁用中文 TTS
2. 只支持英文 TTS
3. 后续再添加中文 TTS 支持

---

## 推荐方案

**推荐**: **方案 1（MMS TTS 中文模型）** 或 **方案 4（暂时只支持英文）**

**理由**:
- MMS TTS 英文版本已经验证可用，中文版本可能也有
- 如果中文版本不可用，可以先支持英文，后续再添加中文支持

---

## 下一步

1. **搜索 MMS TTS 中文模型**，如果存在，下载并测试
2. **如果 MMS TTS 中文模型不可用**，考虑暂时只支持英文 TTS
3. **后续再寻找其他中文 TTS 解决方案**

---

## 参考信息

- MMS TTS: https://huggingface.co/docs/transformers/model_doc/mms
- VITS 中文模型: https://huggingface.co/models?search=vits+chinese
- FastSpeech2: https://github.com/ming024/FastSpeech2

