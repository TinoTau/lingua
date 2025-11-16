use anyhow::Result;
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ndarray::{Array1, Array2, Array3, IxDyn};
use super::marian_onnx::MarianNmtOnnx;
use super::decoder_state::DecoderState;

impl MarianNmtOnnx {
    /// 构造静态 encoder KV 占位符（根据 marian_nmt_interface_spec.md）
    /// 
    /// # Arguments
    /// * `encoder_seq_len` - encoder 序列长度
    /// 
    /// # Returns
    /// 返回一个包含所有层的 encoder KV cache 占位符，每层有 2 个 Value：enc_k, enc_v
    /// 这些占位符在每次步骤中都使用相同的值（静态）
    pub(crate) fn build_static_encoder_kv(
        &self,
        encoder_seq_len: usize,
    ) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
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
            // 每层有 2 个 encoder KV：enc_k, enc_v
            let enc_k = array_to_value!(zeros_enc.clone())?;
            let enc_v = array_to_value!(zeros_enc.clone())?;
            result.push((enc_k, enc_v));
        }

        Ok(result)
    }

    /// 构造第一步用的 decoder KV 占位符
    /// 
    /// # Returns
    /// 返回一个包含所有层的 decoder KV cache 占位符，每层有 2 个 Value：dec_k, dec_v
    pub(crate) fn build_zero_decoder_kv(
        &self,
    ) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
        use ndarray::Array4;
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        let batch = 1usize;
        let dec_len = 1usize;  // decoder "历史长度"占位为 1

        let zeros_dec =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));

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
            // 每层有 2 个 decoder KV：dec_k, dec_v
            let dec_k = array_to_value!(zeros_dec.clone())?;
            let dec_v = array_to_value!(zeros_dec.clone())?;
            result.push((dec_k, dec_v));
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
        static_encoder_kv: &Vec<(Value<'static>, Value<'static>)>,  // 静态 encoder KV 占位符（每次步骤都使用相同的值）
    ) -> anyhow::Result<(Array1<f32>, DecoderState)> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        // 打印调试信息
        println!(
            "[decoder_step] step input_ids_len={}, use_cache_branch={}, has_decoder_kv={}",
            state.input_ids.len(),
            state.use_cache_branch,
            state.decoder_kv_cache.is_some(),
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

        // 4. KV cache：准备输入 KV cache（根据 marian_nmt_interface_spec.md）
        // - Decoder KV: 使用 state.decoder_kv_cache（如果可用）或零占位符
        // - Encoder KV: 始终使用静态占位符（每次步骤都相同）
        let encoder_seq_len = encoder_hidden_states.shape()[1];
        
        // 准备 decoder KV cache
        let decoder_kv = if state.use_cache_branch && state.decoder_kv_cache.is_some() {
            // 正常模式：使用历史 decoder KV cache
            state.decoder_kv_cache.take().unwrap()
        } else {
            // 第一步：使用零占位符
            self.build_zero_decoder_kv()?
        };
        
        // 构建完整的 KV cache 输入（模型需要 4 个值：dec_k, dec_v, enc_k, enc_v）
        // 根据 marian_nmt_interface_spec.md：Encoder KV 始终使用静态占位符
        // 注意：由于 Value 不支持 Clone，我们需要在每次步骤中重新创建 encoder KV
        // 但由于 encoder KV 是静态的（全零），每次创建相同的值
        let static_enc_kv = self.build_static_encoder_kv(encoder_seq_len)?;
        let mut decoder_kv_iter = decoder_kv.into_iter();
        let mut static_enc_kv_iter = static_enc_kv.into_iter();
        
        for layer_idx in 0..Self::NUM_LAYERS {
            // Decoder KV
            let (dec_k, dec_v) = decoder_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient decoder KV cache for layer {}", layer_idx))?;
            input_values.push(dec_k);
            input_values.push(dec_v);
            
            // Encoder KV: 使用静态占位符（每次步骤都相同）
            let (enc_k, enc_v) = static_enc_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient static encoder KV for layer {}", layer_idx))?;
            input_values.push(enc_k);
            input_values.push(enc_v);
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

        // KV cache：处理 present.* 输出（根据 marian_nmt_interface_spec.md）
        // 只提取 decoder KV cache，encoder KV 始终使用静态占位符
        if state.use_cache_branch {
            // 正常模式（第二步及以后）：只提取 decoder KV cache
            let mut next_decoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
            
            for _layer in 0..Self::NUM_LAYERS {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                iter.next(); // 跳过 present.*.encoder.key（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                iter.next(); // 跳过 present.*.encoder.value（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                
                next_decoder_kv.push((dec_k, dec_v));
            }
            
            state.decoder_kv_cache = Some(next_decoder_kv);
        } else {
            // 第一步（use_cache_branch=false）：提取 decoder KV cache
            // 注意：根据 spec，我们不需要提取 encoder KV cache，因为它始终使用静态占位符
            let mut next_decoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
            
            for _layer in 0..Self::NUM_LAYERS {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                iter.next(); // 跳过 present.*.encoder.key（不需要）
                iter.next(); // 跳过 present.*.encoder.value（不需要）
                
                next_decoder_kv.push((dec_k, dec_v));
            }
            
            state.decoder_kv_cache = Some(next_decoder_kv);
            state.use_cache_branch = true;  // 下一步启用 KV cache
        }

        // 返回 state（保持 generated_ids 不变，因为我们在 translate() 中管理它）
        Ok((last_step_logits, state))
    }
}

