# TTS 实现完整总结

**创建日期**: 2024-12-19  
**最后更新**: 2024-12-19  
**状态**: ✅ 英文 TTS 完成，❌ 中文 TTS 阻塞

---

## 执行摘要

TTS（文本转语音）模块已实现英文支持，使用 MMS TTS (VITS) 模型。中文 TTS 经过多次尝试（3个模型），均无法生成可识别的音频，目前处于阻塞状态。

---

## 英文 TTS ✅

### 实现状态
- ✅ **模型**: MMS TTS (VITS) - `Xenova/mms-tts-eng`
- ✅ **实现**: `VitsTtsEngine` (`core/engine/src/tts_streaming/vits_tts.rs`)
- ✅ **功能**: 字符级 tokenizer、ONNX 推理、PCM 音频生成
- ✅ **测试**: 已通过单元测试和集成测试
- ✅ **集成**: 已集成到 `CoreEngineBuilder`

### 技术细节
- Tokenizer: 字符级，支持 `add_blank`
- 模型输入: `input_ids` (int64), `attention_mask` (int64)
- 模型输出: `waveform` (float32)
- 采样率: 16000 Hz
- 音频格式: PCM 16-bit

---

## 中文 TTS ❌

### 尝试的模型

#### 1. vits-zh-aishell3
- **状态**: ❌ 无法识别任何一个字
- **问题**: 所有编码格式和参数组合都无法生成可识别的音频
- **详细报告**: `VITS_ZH_AISHELL3_ISSUE_SUMMARY.md`

#### 2. Breeze2-VITS-zhTW
- **状态**: ❌ 无法识别任何一个字
- **问题**: 所有编码格式和参数组合都无法生成可识别的音频
- **详细报告**: `BREEZE2_VITS_ISSUE_SUMMARY.md`

#### 3. sherpa-onnx-vits-zh-ll
- **状态**: ❌ 无法识别任何一个字
- **问题**: 所有编码格式和参数组合都无法生成可识别的音频
- **详细报告**: `SHERPA_ONNX_VITS_ZH_ISSUE_SUMMARY.md`

### 共同问题
- ✅ 模型可以正常加载和推理
- ✅ 生成的音频文件存在且长度合理
- ✅ 语速可以通过参数调整
- ❌ **所有生成的音频都无法识别任何一个字**

---

## 影响

### 系统功能
- ✅ **中文 → 英文翻译**: 完整可用（ASR 中文 → NMT 中译英 → TTS 英文）
- ❌ **英文 → 中文翻译**: 阻塞（ASR 英文 → NMT 英译中 → TTS 中文 ❌）

### 用户影响
- 中文用户可以将中文语音翻译成英文语音 ✅
- 英文用户无法将英文语音翻译成中文语音 ❌

---

## 下一步

1. **继续寻找可用的中文 TTS 模型**（推荐）
2. **暂时只支持中文 → 英文翻译**
3. **使用在线 TTS API 作为临时方案**

详细建议请参考：`SPEECH_TO_SPEECH_TRANSLATION_STATUS.md`

---

## 相关文档

- `VITS_TTS_IMPLEMENTATION_SUMMARY.md` - 英文 TTS 实现详情
- `VITS_ZH_AISHELL3_ISSUE_SUMMARY.md` - vits-zh-aishell3 问题总结
- `BREEZE2_VITS_ISSUE_SUMMARY.md` - Breeze2-VITS 问题总结
- `SHERPA_ONNX_VITS_ZH_ISSUE_SUMMARY.md` - Sherpa-ONNX-VITS-ZH-LL 问题总结
- `SPEECH_TO_SPEECH_TRANSLATION_STATUS.md` - 语音转语音翻译系统状态

