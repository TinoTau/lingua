use async_trait::async_trait;
use std::path::PathBuf;
use crate::error::{EngineError, EngineResult};
use super::nmt_trait::NmtIncremental;
use super::types::{TranslationRequest, TranslationResponse};
use super::utils::{load_marian_onnx_for_smoke_test, translate_full_sentence_stub};

/// Marian NMT 的增量翻译 stub 实现：
/// - initialize(): 只做一次模型加载 smoke test，确认模型 OK；
/// - translate(): 调用上面的 translate_full_sentence_stub，返回占位翻译；
/// - finalize(): 目前什么都不做。
pub struct MarianNmtStub {
    model_path: PathBuf,
}

impl MarianNmtStub {
    /// 创建一个新的 NMT stub，传入 Marian ONNX 模型路径
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }
}

#[async_trait]
impl NmtIncremental for MarianNmtStub {
    async fn initialize(&self) -> EngineResult<()> {
        if let Err(_e) = load_marian_onnx_for_smoke_test(&self.model_path) {
            // 这里先不携带 e 的详细信息，避免 &'static str 生命周期问题
            return Err(EngineError::new("failed to initialize MarianNmtStub"));
        }
        Ok(())
    }

    async fn translate(
        &self,
        _request: TranslationRequest,
    ) -> EngineResult<TranslationResponse> {
        // 这里暂时**不去访问 PartialTranscript 的内部字段**，
        // 避免和你现有 struct 定义不匹配导致无法编译。
        //
        // 后续你可以根据 PartialTranscript 的实际结构，
        // 把下面这个 input 替换成从 _request.transcript 里取出来的文本。
        let input = "NMT stub input";

        let translated = translate_full_sentence_stub(input, &self.model_path)
            .map_err(|_e| EngineError::new("MarianNmtStub translate failed"))?;

        Ok(TranslationResponse {
            translated_text: translated,
            is_stable: true,
            speaker_id: None,
            source_text: None,
            source_audio_duration_ms: None,
            quality_metrics: None,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}

