# 系统架构与技术方案总览

**鏈€鍚庢洿鏂?*: 2024-12-19

---


**������**: 2024-12-19

---

# 系统架构与技术方案总览

## 1. 核心流程
- VAD �?ASR（Whisper�?�?Emotion（XLM-R�?�?Persona（Rule-based�?�?NMT（Marian ONNX�?�?TTS（FastSpeech2 + HiFiGAN�?
- `CoreEngine` 作为统一编排中心，通过 `EventBus` 发布 Asr/Emotion/Translation/TTS 事件�?

## 2. 模块划分
- `asr_streaming`：Whisper-RS 引擎，支持流式缓冲与自然停顿触发�?
- `emotion_adapter`：XLM-R ONNX 推理，IR9 + ORT 1.16.3 兼容方案，含 tokenizer + 后处理规则�?
- `persona_adapter`：规则映射，根据语调/文化调整文本�?
- `nmt_incremental`：Marian ONNX，Plan C KV cache，仅维护 decoder KV�?
- `tts_streaming`：FastSpeech2 + HiFiGAN，当前模型输�?80 维特征，尚未生成真实音频�?
- `bootstrap/CoreEngineBuilder`：注�?EventBus/VAD/ASR/NMT/Emotion/Persona/TTS/Config/Cache/Telemetry�?

## 3. 技术栈
- Rust + Tokio async；ndarray/serde/tokenizers�?
- ONNX Runtime 1.16.3（Emotion/NMT/TTS），whisper-rs（ASR）�?
- `core/engine/models/` 目录管理多语言模型，配套脚本：`export_emotion_model_ir9_old_pytorch.py`、`check_tts_model_io.py` 等�?
- `ConfigManager`、`CacheManager`、`TelemetrySink` �?trait + Arc 注入�?

## 4. 当前关键问题
- **TTS 模块失效**：HiFiGAN 输出 `[1, time_steps, 80]` 特征而非波形；生�?WAV 仅约 20ms，听感为空�?
- **ORT 版本锁定**：Emotion/NMT 依赖 ORT 1.16.3，如需更高版本（TTS）需考虑运行时隔离�?
- **多语言扩展**：ASR/Emotion/Persona 支持多语言，TTS 尚不可用，影响端到端体验�?

## 5. 关键证据
- `python scripts/test_hifigan_model.py`：显�?HiFiGAN 输出 `[1, None, 80]`，值域�?`[-30, 30]`�?
- `cargo test test_tts_synthesize_english -- --nocapture`：日志显�?Flatten 后仅 320 样本（约 20ms）�?

## 6. 调整建议
1. **替换 TTS 技术栈**  
   - 方案 A：获取标�?FastSpeech2(80) + HiFiGAN(80→wave) 模型对�? 
   - 方案 B：评�?VITS 等端到端 TTS�? 
   - 方案 C：临时接入第三方 TTS 服务�?
2. **运行时隔�?*  
   - �?TTS 需更高版本 ORT，可拆为独立进程/服务，避免与 NMT/Emotion 冲突�?
3. **保持接口稳定**  
   - 继承 `AsrStreaming`、`TtsStreaming` trait，可在不影响 `CoreEngine` 的情况下替换实现�?

## 7. 相关文档/脚本
- `core/engine/docs/TESTING_GUIDE.md`：端到端与模块化测试流程�?
- `core/engine/docs/TTS_HIFIGAN_OUTPUT_ISSUE.md`：TTS 问题详述�?
- `scripts/test_hifigan_model.py`、`scripts/check_tts_model_io.py`：模型检验脚本�?


