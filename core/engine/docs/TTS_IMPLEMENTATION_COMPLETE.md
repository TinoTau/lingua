# TTS 实现完整总结

**创建日期**: 2024-12-19  
**最后更新**: 2025-11-21  
**状态**: ✅ 英文 TTS 完成，✅ 中文 TTS 完成（WSL2 Piper 方案）

---

## 执行摘要

TTS（文本转语音）模块已实现英文和中文支持：
- **英文 TTS**: 使用 MMS TTS (VITS) 模型，已完成并测试通过
- **中文 TTS**: 使用 Piper TTS（WSL2 HTTP 服务），已完成并测试通过

**注意**: 之前尝试的 3 个中文 TTS 模型（vits-zh-aishell3、breeze2-vits-zhTW、sherpa-onnx-vits-zh-ll）均无法生成可识别的音频，已放弃。最终采用 WSL2 + Piper TTS 方案成功解决。

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

## 中文 TTS ✅

### 最终方案: Piper TTS (WSL2 HTTP 服务)

**状态**: ✅ 已完成并测试通过  
**实施日期**: 2025-11-21  
**模型**: zh_CN-huayan-medium  
**部署方式**: WSL2 + FastAPI HTTP 服务

**实现细节**:
- HTTP 服务: `piper_http_server.py` (FastAPI)
- Rust 客户端: `PiperHttpTts` (`core/engine/src/tts_streaming/piper_http.rs`)
- 服务地址: `http://127.0.0.1:5005/tts`
- 音频格式: WAV, 16 bit, mono, 22050 Hz

**测试结果**:
- ✅ 音频生成成功
- ✅ 音频质量清晰可识别
- ✅ 端到端测试通过

**详细文档**: `../../docs/architecture/PIPER_TTS_IMPLEMENTATION_COMPLETE.md`

---

### 之前尝试的模型（已放弃）

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

### 之前尝试模型的共同问题
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

