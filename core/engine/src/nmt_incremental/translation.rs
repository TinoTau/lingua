use anyhow::{Result, anyhow};
use ort::value::Value;
use async_trait::async_trait;

use crate::error::{EngineError, EngineResult};
use super::nmt_trait::NmtIncremental;
use super::types::{TranslationRequest, TranslationResponse};
use super::marian_onnx::MarianNmtOnnx;
use super::decoder_state::DecoderState;

impl MarianNmtOnnx {
    /// 执行完整的翻译流程
    /// 
    /// # Arguments
    /// * `source_text` - 源文本（需要翻译的文本）
    /// 
    /// # Returns
    /// 翻译后的文本
    /// 
    /// # Note
    /// 这是一个简化版本，假设 encoder_hidden_states 已经准备好。
    /// 完整的实现需要先运行 encoder 模型。
    pub fn translate(&self, source_text: &str) -> Result<String> {
        // 1. 使用 tokenizer 编码源文本
        let source_ids = self.tokenizer.encode(source_text, true);
        println!("Source text: '{}'", source_text);
        println!("Encoded source IDs: {:?} (length: {})", source_ids, source_ids.len());

        // 2. 运行 encoder 获取真实的 encoder_hidden_states
        let (encoder_hidden_states, encoder_attention_mask) = self.run_encoder(&source_ids)?;
        println!("Encoder output shape: {:?}", encoder_hidden_states.shape());

        // 3. 初始化 DecoderState（根据 marian_nmt_interface_spec.md：只维护 decoder KV cache）
        // 第一步：不使用 KV cache，input_ids 只包含 BOS token
        let mut state = DecoderState {
            input_ids: vec![self.decoder_start_token_id],
            generated_ids: vec![self.decoder_start_token_id],  // 一开始就包含 BOS
            decoder_kv_cache: None,
            encoder_kv_cache: None,  // 第一步使用全零占位符
            use_cache_branch: false,  // 第一步：禁用 KV 分支
        };

        // 4. 进入解码循环
        let max_steps = self.max_length.min(128); // 限制最大步数
        let encoder_seq_len = encoder_hidden_states.shape()[1];  // 用于创建静态 encoder KV

        for step in 0..max_steps {
            // 准备当前步骤的 state（根据 marian_nmt_interface_spec.md：只维护 decoder KV cache）
            // 关键：如果使用 KV cache，input_ids 应该只包含新 token（单个 token）
            let current_state = if state.use_cache_branch && state.decoder_kv_cache.is_some() {
                // 正常模式（使用 KV cache）：只输入新 token
                // 注意：这里应该使用上一步生成的最后一个 token
                let last_token = state.generated_ids.last().copied().unwrap_or(self.decoder_start_token_id);
                DecoderState {
                    input_ids: vec![last_token],  // 关键：只包含新 token
                    generated_ids: state.generated_ids.clone(),
                    decoder_kv_cache: state.decoder_kv_cache.take(),  // 使用历史 decoder KV cache
                    encoder_kv_cache: state.encoder_kv_cache.take(),  // 使用历史 encoder KV cache
                    use_cache_branch: true,  // 启用 KV 分支
                }
            } else {
                // 第一步：使用完整历史序列
                let current_generated_ids = state.generated_ids.clone();
                DecoderState {
                    input_ids: current_generated_ids.clone(),  // 使用完整历史序列
                    generated_ids: current_generated_ids.clone(),
                    decoder_kv_cache: None,
                    encoder_kv_cache: None,  // 使用全零占位符
                    use_cache_branch: false,  // 禁用 KV 分支
                }
            };
            
            println!("[DEBUG] Step {}: decoder_input_ids={:?} (length: {}), use_cache_branch={}, has_decoder_kv={}", 
                step, current_state.input_ids, current_state.input_ids.len(), 
                current_state.use_cache_branch, 
                current_state.decoder_kv_cache.is_some());
            
            // 创建静态 encoder KV 占位符（根据 marian_nmt_interface_spec.md）
            // 注意：由于 Value 不支持 Clone，我们需要在每次步骤中重新创建
            // 但由于 encoder KV 是静态的（全零），每次创建相同的值
            let static_encoder_kv = self.build_static_encoder_kv(encoder_seq_len)?;
            let (logits, next_state) = self.decoder_step(
                &encoder_hidden_states,
                &encoder_attention_mask,
                current_state,
                &static_encoder_kv,  // 传递静态 encoder KV 占位符
            )?;

            // decoder_step 返回的 logits 已经是最后一个位置的 Array1<f32>
            // 所以直接使用即可
            
            // 选择概率最高的 token（贪婪解码）
            let next_token_id = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as i64)
                .ok_or_else(|| anyhow!("failed to find next token"))?;

            // 检查是否生成 EOS
            if next_token_id == self.eos_token_id {
                println!("Generated EOS token at step {}", step);
                break;
            }

            // 更新 state：添加新 token，并保存 KV cache（根据 marian_nmt_interface_spec.md：只维护 decoder KV cache）
            state.generated_ids.push(next_token_id);
            state.decoder_kv_cache = next_state.decoder_kv_cache;  // 保存 decoder KV cache 供下一步使用
            state.use_cache_branch = next_state.use_cache_branch;  // 更新 use_cache_branch 状态
            
            println!("[DEBUG] After Step {}: use_cache_branch={}, has_decoder_kv={}", 
                step, state.use_cache_branch, 
                state.decoder_kv_cache.is_some());
        }

        println!("[NMT][translate] Generated IDs: {:?} (length: {})", state.generated_ids, state.generated_ids.len());

        // 5. 使用 tokenizer 解码（跳过 BOS token）
        let translated_ids: Vec<i64> = state.generated_ids.iter()
            .skip(1)  // 跳过 BOS token
            .copied()
            .collect();
        let translated_text = self.tokenizer.decode(&translated_ids);
        println!("[NMT][translate] Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}

/// 为 MarianNmtOnnx 实现 NmtIncremental trait
#[async_trait]
impl NmtIncremental for MarianNmtOnnx {
    async fn initialize(&self) -> EngineResult<()> {
        // ONNX 模型在 new_from_dir 时已经加载，这里只需要验证
        // 可以尝试运行一个简单的翻译来验证模型是否正常工作
        Ok(())
    }

    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        // 从 TranslationRequest 中提取源文本
        let source_text = request.transcript.text.clone();
        
        // 由于 self.translate() 是同步方法，但 trait 要求是 async，
        // 我们直接调用同步方法（虽然会阻塞当前任务，但对于翻译这种 CPU 密集型操作是合理的）
        let translated = self.translate(&source_text)
            .map_err(|e| {
                // 将 anyhow::Error 转换为 EngineError
                // String 可以转换为 Cow<'static, str>
                let error_msg = format!("Translation failed: {}", e);
                EngineError::new(error_msg)
            })?;

        Ok(TranslationResponse {
            translated_text: translated,
            is_stable: request.wait_k.is_none() || request.wait_k == Some(0),
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        // ONNX 会话会在对象销毁时自动清理
        Ok(())
    }
}

