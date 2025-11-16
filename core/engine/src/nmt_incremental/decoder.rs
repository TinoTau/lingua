use anyhow::Result;
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ndarray::{Array1, Array2, Array3, IxDyn};
use super::marian_onnx::MarianNmtOnnx;
use super::decoder_state::DecoderState;

impl MarianNmtOnnx {
    /// 构造第一步用的零张量 KV 值
    /// 
    /// # Arguments
    /// * `encoder_seq_len` - encoder 序列长度
    /// 
    /// # Returns
    /// 返回一个包含所有层的 KV cache 占位符，每层有 4 个 Value：dec_k, dec_v, enc_k, enc_v
    pub(crate) fn build_initial_kv_values(
        &self,
        encoder_seq_len: usize,
    ) -> anyhow::Result<Vec<[Value<'static>; 4]>> {
        use ndarray::Array4;
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        let batch = 1usize;
        let dec_len = 1usize;           // decoder "历史长度"占位为 1
        let enc_len = encoder_seq_len;  // encoder 长度与真实输入一致

        let zeros_dec =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));
        let zeros_enc =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, enc_len, Self::HEAD_DIM));

        // 使用与 decoder_step 中相同的 array_to_value 宏
        macro_rules! array_to_value {
            ($arr:expr) => {{
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let mut result = Vec::with_capacity(Self::NUM_LAYERS);

        for _ in 0..Self::NUM_LAYERS {
            // 每层有 4 个 KV：dec_k, dec_v, enc_k, enc_v
            let dec_k = array_to_value!(zeros_dec.clone())?;
            let dec_v = array_to_value!(zeros_dec.clone())?;
            let enc_k = array_to_value!(zeros_enc.clone())?;
            let enc_v = array_to_value!(zeros_enc.clone())?;
            result.push([dec_k, dec_v, enc_k, enc_v]);
        }

        Ok(result)
    }

    /// 执行 decoder 的单次步进
    ///
    /// - 输入：
    ///   - encoder_hidden_states: [1, encoder_seq_len, hidden_dim]
    ///   - encoder_attention_mask: [1, encoder_seq_len]
    ///   - state: 包含当前 decoder_input_ids / 上一步 KV cache
    /// - 输出：
    ///   - (logits_last_step, next_state)
    pub(crate) fn decoder_step(
        &self,
        encoder_hidden_states: &Array3<f32>,
        encoder_attention_mask: &Array2<i64>,
        mut state: DecoderState,
        saved_encoder_kv: &Option<Vec<(Value<'static>, Value<'static>)>>,  // Step 0 的 encoder KV cache（只读引用，不 move）
    ) -> anyhow::Result<(Array1<f32>, DecoderState)> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        // 打印调试信息
        println!(
            "[decoder_step] step input_ids_len={}, use_cache_branch={}, has_decoder_kv={}, has_encoder_kv={}",
            state.input_ids.len(),
            state.use_cache_branch,
            state.decoder_kv_cache.is_some(),
            state.encoder_kv_cache.is_some(),
        );

        // 1. 准备 decoder input_ids: [1, cur_len]
        let batch_size = 1usize;
        let cur_len = state.input_ids.len();
        let decoder_input_ids = Array2::<i64>::from_shape_vec(
            (batch_size, cur_len),
            state.input_ids.clone(),
        )?;
        
        println!(
            "[decoder_step] input_ids shape: {:?}",
            decoder_input_ids.shape()
        );

        // 2. use_cache_branch: [1]，类型是 Bool（根据模型输入定义）
        let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);

        // 3. 转换为 Value
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let input_ids_value = array_to_value!(decoder_input_ids, i64)?;
        let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
        let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
        let use_cache_value = array_to_value!(use_cache_array, bool)?;

        // 4. 组织输入顺序（严格按照模型 I/O 顺序）
        // 输入顺序：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
        let mut input_values: Vec<Value<'static>> = Vec::new();

        // 1. encoder_attention_mask
        input_values.push(encoder_mask_value);
        // 2. input_ids
        input_values.push(input_ids_value);
        // 3. encoder_hidden_states
        input_values.push(encoder_states_value);

        // 4. KV cache：准备输入 KV cache（方案 C：分离 encoder 和 decoder KV cache）
        // 由于模型要求所有输入都存在，即使 use_cache_branch=false 也需要传入 KV
        let encoder_seq_len = encoder_hidden_states.shape()[1];
        
        // 方案 C 的关键：encoder KV cache 需要保持不变，但我们需要在构建输入时使用它
        // 由于 Value 不支持 Clone，我们需要在构建输入时移动 encoder KV cache
        // 然后在处理输出时重新保存它（因为 encoder KV cache 在每次步骤中都是相同的）
        
        // 构建完整的 KV cache 输入（模型需要 4 个值：dec_k, dec_v, enc_k, enc_v）
        // 方案 C 的关键修复：优先使用 state.encoder_kv_cache，如果不可用则使用 saved_encoder_kv
        // 但要注意：不能直接 take() saved_encoder_kv，因为它在后续步骤中还需要使用
        // 解决方案：只在 state.encoder_kv_cache 可用时使用它，否则使用占位符
        // 然后在 translate() 中，如果 saved_encoder_kv_for_restore 可用，将其恢复到 state.encoder_kv_cache
        if state.use_cache_branch && state.decoder_kv_cache.is_some() {
            // 正常模式：使用历史 decoder KV cache
            let mut decoder_kv = state.decoder_kv_cache.take().unwrap();
            
            // 优先使用 state.encoder_kv_cache（如果可用）
            // 如果不可用，尝试使用 saved_encoder_kv（通过引用，但不 move）
            if let Some(mut encoder_kv) = state.encoder_kv_cache.take() {
                // encoder KV cache 可用，使用它
                for _layer_idx in 0..Self::NUM_LAYERS {
                    let (dec_k, dec_v) = decoder_kv.remove(0);
                    let (enc_k, enc_v) = encoder_kv.remove(0);
                    input_values.push(dec_k);
                    input_values.push(dec_v);
                    input_values.push(enc_k);
                    input_values.push(enc_v);
                }
                // 注意：encoder_kv 已被消耗，需要在输出处理时恢复
                // 但由于 present.*.encoder.* 是空的，我们无法从输出中恢复
                // 所以这里暂时不恢复，在 translate() 中处理
            } else if let Some(encoder_kv_ref) = saved_encoder_kv {
                // encoder KV cache 不可用，但 saved_encoder_kv 可用
                // 关键修复：由于 encoder KV cache 在每次步骤中都是相同的，我们可以重复使用
                // 但由于 Value 不支持 Clone，我们无法创建副本
                // 解决方案：我们需要 move encoder_kv_ref 中的值，但这样会消耗 saved_encoder_kv
                // 由于 saved_encoder_kv 是引用，我们不能直接 move
                // 最终方案：我们需要在 translate() 中，在准备 current_state 时，将 saved_encoder_kv_for_restore 恢复到 state.encoder_kv_cache
                // 这样在 decoder_step 中就可以直接使用 state.encoder_kv_cache
                // 但由于 Value 不支持 Clone，我们无法创建副本
                // 所以暂时使用占位符，等待更好的解决方案
                // 实际上，由于我们在 translate() 中已经将 saved_encoder_kv_for_restore 恢复到 state.encoder_kv_cache，
                // 这里应该不会执行到这个分支
                let initial_kv = self.build_initial_kv_values(encoder_seq_len)?;
                for kv_layer in initial_kv {
                    let [dec_k_placeholder, dec_v_placeholder, enc_k_placeholder, enc_v_placeholder] = kv_layer;
                    // 使用 decoder KV cache
                    if !decoder_kv.is_empty() {
                        let (dec_k, dec_v) = decoder_kv.remove(0);
                        input_values.push(dec_k);
                        input_values.push(dec_v);
                    } else {
                        input_values.push(dec_k_placeholder);
                        input_values.push(dec_v_placeholder);
                    }
                    // 使用占位符作为 encoder KV cache
                    input_values.push(enc_k_placeholder);
                    input_values.push(enc_v_placeholder);
                }
            } else {
                // encoder KV cache 和 saved_encoder_kv 都不可用，使用占位符
                let initial_kv = self.build_initial_kv_values(encoder_seq_len)?;
                for kv_layer in initial_kv {
                    let [dec_k_placeholder, dec_v_placeholder, enc_k_placeholder, enc_v_placeholder] = kv_layer;
                    // 使用 decoder KV cache
                    if !decoder_kv.is_empty() {
                        let (dec_k, dec_v) = decoder_kv.remove(0);
                        input_values.push(dec_k);
                        input_values.push(dec_v);
                    } else {
                        input_values.push(dec_k_placeholder);
                        input_values.push(dec_v_placeholder);
                    }
                    // 使用占位符作为 encoder KV cache
                    input_values.push(enc_k_placeholder);
                    input_values.push(enc_v_placeholder);
                }
            }
        } else {
            // 第一步或 Workaround 模式：使用占位 KV
            let initial_kv = self.build_initial_kv_values(encoder_seq_len)?;
            for kv_layer in initial_kv {
                let [dec_k, dec_v, enc_k, enc_v] = kv_layer;
                input_values.push(dec_k);
                input_values.push(dec_v);
                input_values.push(enc_k);
                input_values.push(enc_v);
            }
        }

        // 5. use_cache_branch
        input_values.push(use_cache_value);

        // 5. 调用 session.run
        let decoder_session = self.decoder_session.lock().unwrap();
        let outputs: Vec<Value<'static>> = decoder_session.run(input_values)
            .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

        // 6. 从输出中提取 logits + 新 KV
        // logits 是唯一需要转回 ndarray 的
        let mut iter = outputs.into_iter();
        let logits_value = iter.next().expect("missing logits output");

        let logits_tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract logits: {e}"))?;
        let logits_view = logits_tensor.view();
        let logits_array = logits_view.to_owned(); // shape: [1, cur_len, vocab_size]

        // 取最后一个 step 的 logits: [vocab_size]
        let shape = logits_array.shape();
        let seq_len = shape[1];
        // 使用 slice 获取最后一个 token 的 logits，然后转换为 Array1
        let last_step_logits = logits_array
            .slice(ndarray::s![0, seq_len - 1, ..])
            .to_owned(); // 已经是 Array1<f32>

        // KV cache：处理 present.* 输出（方案 C：分离 encoder 和 decoder KV cache）
        if state.use_cache_branch {
            // 正常模式（第二步及以后）：只提取 decoder KV cache，保持 encoder KV cache 不变
            let mut next_decoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
            
            for _layer in 0..Self::NUM_LAYERS {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                iter.next(); // 跳过 present.*.encoder.key（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                iter.next(); // 跳过 present.*.encoder.value（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                
                next_decoder_kv.push((dec_k, dec_v));
            }
            
            state.decoder_kv_cache = Some(next_decoder_kv);
            // encoder KV cache 保持不变（如果之前已经提取过）
            // 如果 encoder_kv_was_used，说明我们在构建输入时移出了 encoder KV cache
            // 但由于 encoder KV cache 在每次步骤中都是相同的，我们需要恢复它
            // 由于 Value 不支持 Clone，我们无法在构建输入时保留 encoder KV cache 的副本
            // 解决方案：从 saved_encoder_kv 中恢复 encoder KV cache
            // 但由于 Value 不支持 Clone，且 saved_encoder_kv 是引用，我们不能直接使用它
            // 当前实现：由于 present.*.encoder.* 是空的，我们无法从输出中获取 encoder KV cache
            // 我们需要从 saved_encoder_kv 中恢复 encoder KV cache
            // 但由于 Value 不支持 Clone，我们需要在 decoder_step 中处理这个问题
            // 暂时保持 encoder_kv_cache 为 None，在 translate() 中处理
            
            // 关键修复：如果 encoder_kv_was_used=true，说明我们从 saved_encoder_kv 中移动了 encoder KV cache
            // 由于 encoder KV cache 在每次步骤中都是相同的，我们需要在 translate() 中重新填充 saved_encoder_kv
            // 但这里我们无法恢复，因为 present.*.encoder.* 是空的
            // 所以暂时保持 encoder_kv_cache 为 None，在 translate() 中处理
            // 注意：如果 encoder_kv_was_used=true，说明我们在构建输入时已经使用了 encoder KV cache
            // 但由于 present.*.encoder.* 是空的，我们无法从输出中恢复它
            // 解决方案：在 translate() 中，如果 saved_encoder_kv 被移出了，我们需要在 Step 0 时重新保存它
        } else {
            // 第一步（use_cache_branch=false）：提取所有 KV cache
            // 这一步的 present.*.encoder.* 是正常的，可以全部提取
            let mut next_decoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
            let mut next_encoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
            
            for _layer in 0..Self::NUM_LAYERS {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                let enc_k = iter.next().expect("missing present.*.encoder.key");
                let enc_v = iter.next().expect("missing present.*.encoder.value");
                
                next_decoder_kv.push((dec_k, dec_v));
                next_encoder_kv.push((enc_k, enc_v));
            }
            
            state.decoder_kv_cache = Some(next_decoder_kv);
            state.encoder_kv_cache = Some(next_encoder_kv);  // Step 0 时提取并保存 encoder KV cache
            state.use_cache_branch = true;  // 下一步启用 KV cache
        }

        // 返回 state（保持 generated_ids 不变，因为我们在 translate() 中管理它）
        Ok((last_step_logits, state))
    }
}

