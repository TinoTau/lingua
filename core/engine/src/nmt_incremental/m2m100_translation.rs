// M2M100 Translation 实现

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use crate::error::{EngineError, EngineResult};
use super::nmt_trait::NmtIncremental;
use super::types::{TranslationRequest, TranslationResponse};
use super::m2m100_onnx::M2M100NmtOnnx;
// ✅ 使用非增量解码，每次传入完整序列，使用全零 KV cache，不维护状态

impl M2M100NmtOnnx {
    /// 执行完整的翻译流程
    /// 
    /// # Arguments
    /// * `source_text` - 源文本（需要翻译的文本）
    /// 
    /// # Returns
    /// 翻译后的文本
    pub fn translate(&self, source_text: &str) -> Result<String> {
        // 1. 使用 tokenizer 编码源文本（包含源语言 token）
        let source_ids = self.tokenizer.encode(source_text, &self.src_lang, true)?;
        println!("Source text: '{}'", source_text);
        println!("Encoded source IDs: {:?} (length: {})", source_ids, source_ids.len());

        // 2. 运行 encoder 获取真实的 encoder_hidden_states
        let (encoder_hidden_states, encoder_attention_mask) = self.run_encoder(&source_ids)?;
        println!("Encoder output shape: {:?}", encoder_hidden_states.shape());

        // 3. 获取配置信息
        // ✅ 工程版实时翻译改造：使用简单的 Vec<i64> 管理生成序列，不再需要 DecoderState
        let tgt_lang_id = self.tokenizer.get_lang_id(&self.tgt_lang);
        let eos_token_id = self.tokenizer.eos_token_id();

        // 打印配置信息
        println!("[Config] src_lang: {}, tgt_lang: {}", self.src_lang, self.tgt_lang);
        println!("[Config] tgt_lang_id: {}, eos_token_id: {} (from tokenizer), pad_token_id: {}", 
            tgt_lang_id, eos_token_id, self.pad_token_id);

        // 4. 进入解码循环（非增量解码版本，不维护 KV cache）
        let max_steps = self.max_length.min(128);
        let encoder_seq_len = encoder_hidden_states.shape()[1];

        // 初始化生成序列（包含目标语言 token）
        let mut generated_ids = vec![tgt_lang_id];

        for step in 0..max_steps {
            println!("[Step {}] 非增量解码: generated_ids = {:?} (长度: {})", 
                step, &generated_ids[..generated_ids.len().min(10)], generated_ids.len());

            // ✅ 使用非增量解码：每次传入完整的 generated_ids，每次都使用全零 KV cache
            let logits = self.decode_next_token_non_incremental(
                &generated_ids,
                &encoder_hidden_states,
                &encoder_attention_mask,
                encoder_seq_len,
                self.use_new_format,
            )?;

            // 打印 logits 信息（前几步）
            if step < 3 {
                println!("[Step {}] logits shape: {:?}", step, logits.shape());
                // 获取 top 5 logits
                let mut logits_with_idx: Vec<(usize, f32)> = logits.iter()
                    .enumerate()
                    .map(|(i, &v)| (i, v))
                    .collect();
                logits_with_idx.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                println!("[Step {}] top 5 logits: {:?}", step, &logits_with_idx[..5.min(logits_with_idx.len())]);
                if (self.eos_token_id as usize) < logits.len() {
                    println!("[Step {}] logits[eos_token_id={}]: {}", step, self.eos_token_id, logits[self.eos_token_id as usize]);
                }
            }

            // ✅ 使用 argmax 选择概率最高的 token（贪婪解码）
            let next_token_id = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as i64)
                .ok_or_else(|| anyhow!("failed to find next token"))?;

            // 打印调试日志
            if step < 10 {
                let mut logits_with_idx: Vec<(usize, f32)> = logits.iter()
                    .enumerate()
                    .map(|(i, &v)| (i, v))
                    .collect();
                logits_with_idx.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let top5: Vec<(usize, f32)> = logits_with_idx.iter().take(5).copied().collect();
                
                println!("  logits(top5)={:?}", top5);
                println!("  next_id={}", next_token_id);
            }

            // ✅ EOS 检查
            if next_token_id == eos_token_id {
                println!("[Step {}] Generated EOS token ({}), stopping", step, eos_token_id);
                break;
            }
            
            // ✅ 加入生成序列
            generated_ids.push(next_token_id);
            
            // ⚠️ 重复检测：检查是否陷入重复模式
            if generated_ids.len() >= 4 {
                let last_four: Vec<i64> = generated_ids.iter().rev().take(4).copied().collect();
                if last_four.len() >= 4 && last_four[0] == last_four[2] && last_four[1] == last_four[3] {
                    println!("[Step {}] ⚠️  检测到 2-token 重复模式: {:?}, 停止解码", step, last_four);
                    println!("[WARNING] 解码陷入重复循环，可能是模型或 KV cache 处理有问题");
                    break;
                }
            }
            
            // ⚠️ 长度限制：防止无限循环
            if generated_ids.len() >= 128 {
                println!("[Step {}] ⚠️  达到最大长度限制 (128), 强制停止", step);
                break;
            }
        }

        println!("[NMT][translate] Generated IDs: {:?} (length: {})", generated_ids, generated_ids.len());

        // ✅ 生成的序列格式: [tgt_lang_id, ...tokens..., eos_token_id]
        // 需要跳过第一个目标语言 token，只保留实际的翻译 token
        let translated_ids: Vec<i64> = generated_ids.iter()
            .skip(1)  // ✅ 跳过第一个目标语言 token
            .filter(|&&id| id != eos_token_id)  // ✅ 过滤掉 EOS token
            .copied()
            .collect();
        
        // 调试：打印每个 token 对应的文本
        println!("[NMT][translate] Translated IDs (after filtering): {:?}", translated_ids);
        for &id in &translated_ids {
            if let Some(piece) = self.tokenizer.id_to_piece(id) {
                println!("  Token {} -> '{}'", id, piece);
            }
        }
        
        let translated_text = self.tokenizer.decode(&translated_ids, true)?;
        println!("[NMT][translate] Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}

/// 为 M2M100NmtOnnx 实现 NmtIncremental trait
#[async_trait]
impl NmtIncremental for M2M100NmtOnnx {
    async fn initialize(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        let source_text = request.transcript.text.clone();
        
        let translated = self.translate(&source_text)
            .map_err(|e| {
                let error_msg = format!("Translation failed: {}", e);
                EngineError::new(error_msg)
            })?;

        Ok(TranslationResponse {
            translated_text: translated,
            is_stable: request.wait_k.is_none() || request.wait_k == Some(0),
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}

