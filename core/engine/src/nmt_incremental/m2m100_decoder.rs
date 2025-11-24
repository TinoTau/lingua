// M2M100 Decoder 实现
// 与 Marian decoder 类似，但使用 M2M100 的常量（12 层，16 头，1024 维度）

use anyhow::Result;
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ndarray::{Array1, Array2, Array3, Array4, IxDyn};
use super::m2m100_onnx::M2M100NmtOnnx;
// ✅ 工程版实时翻译改造：decoder_step 不再使用 DecoderState
// 但保留导入，因为 decoder_step 函数仍然存在（已废弃，仅用于参考）
use super::decoder_state::DecoderState;

impl M2M100NmtOnnx {
    /// 构造静态 encoder KV 占位符
    /// 
    /// # Arguments
    /// * `encoder_seq_len` - encoder 序列长度（源序列长度）
    /// 
    /// # Returns
    /// 返回一个包含所有层的 encoder KV cache 占位符，每层有 2 个 Value：enc_k, enc_v
    /// 
    /// 注意：正确的 encoder KV cache 形状是 [batch, num_heads, src_seq_len, head_dim]
    /// 即 [1, 16, encoder_seq_len, 64]
    /// - encoder_seq_len 必须在第 3 个维度（索引 2），而不是第 4 个维度
    /// - 最后一维必须是 HEAD_DIM = 64
    pub(crate) fn build_static_encoder_kv(
        &self,
        encoder_seq_len: usize,
    ) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        // 正确形状: [batch, num_heads, src_seq_len, head_dim]
        // 即 [1, 16, encoder_seq_len, 64]
        let zeros_enc = Array4::<f32>::zeros((
            1,                      // batch_size
            Self::NUM_HEADS,        // 16
            encoder_seq_len,        // 源序列长度（第 3 个维度）
            Self::HEAD_DIM,         // 64（最后一维）
        ));

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
            let enc_k = array_to_value!(zeros_enc.clone())?;
            let enc_v = array_to_value!(zeros_enc.clone())?;
            result.push((enc_k, enc_v));
        }

        Ok(result)
    }

    /// 非增量解码：解码下一个 token（句级非增量版本）
    /// 
    /// 每次调用都传入全零 KV cache，不维护状态
    /// 
    /// # Arguments
    /// * `generated_ids` - 当前已生成的目标 token 序列（包含起始 token）
    /// * `encoder_hidden_states` - Encoder 输出的隐藏状态
    /// * `encoder_attention_mask` - Encoder 的 attention mask
    /// * `encoder_seq_len` - Encoder 序列长度
    /// * `use_new_format` - 是否使用新格式（没有 use_cache_branch）
    /// 
    /// # Returns
    /// 返回下一个 token 的 logits（vocab_size 维度）
    pub(crate) fn decode_next_token_non_incremental(
        &self,
        generated_ids: &[i64],
        encoder_hidden_states: &Array3<f32>,
        encoder_attention_mask: &Array2<i64>,
        encoder_seq_len: usize,
        use_new_format: bool,
    ) -> anyhow::Result<Array1<f32>> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;
        use ort::tensor::OrtOwnedTensor;

        let batch_size = 1usize;
        let tgt_seq_len = generated_ids.len();
        
        // 1. 将 generated_ids 转为 [batch, tgt_seq] 形状
        let decoder_input_ids = Array2::<i64>::from_shape_vec(
            (batch_size, tgt_seq_len),
            generated_ids.to_vec(),
        )?;

        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {e}"))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        // 2. 构造 3 个核心输入
        let input_ids_value = array_to_value!(decoder_input_ids.clone(), i64)?;
        let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
        let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;

        // 3. 构造全零 KV cache（每次都是新的）
        // 注意：非增量解码时，decoder KV 的形状应该是 [1, 16, tgt_seq_len, 64]
        // 而不是固定的 [1, 16, 1, 64]
        let decoder_kv = self.build_zero_decoder_kv_for_seq_len(tgt_seq_len)?;
        let encoder_kv = self.build_static_encoder_kv(encoder_seq_len)?;

        // 4. 组织 ONNX 输入：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
        let mut input_values: Vec<Value<'static>> = Vec::new();
        input_values.push(encoder_mask_value);
        input_values.push(input_ids_value);
        input_values.push(encoder_states_value);

        // 5. 添加全零 KV cache（12 层 × 4 = 48 个）
        let mut decoder_kv_iter = decoder_kv.into_iter();
        let mut encoder_kv_iter = encoder_kv.into_iter();
        
        for _layer_idx in 0..Self::NUM_LAYERS {
            // Decoder KV
            let (dec_k, dec_v) = decoder_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient decoder KV cache"))?;
            input_values.push(dec_k);
            input_values.push(dec_v);
            
            // Encoder KV
            let (enc_k, enc_v) = encoder_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient encoder KV cache"))?;
            input_values.push(enc_k);
            input_values.push(enc_v);
        }

        // 6. 如果是旧格式，添加 use_cache_branch（设为 false，因为我们不使用缓存）
        if !use_new_format {
            let use_cache_array = Array1::<bool>::from_vec(vec![false]);
            let use_cache_value = array_to_value!(use_cache_array, bool)?;
            input_values.push(use_cache_value);
        }

        // 7. 运行 decoder
        let decoder_session = self.decoder_session.lock().unwrap();
        let outputs: Vec<Value<'static>> = decoder_session.run(input_values)
            .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

        // 8. 提取 logits
        let logits_value = outputs.first()
            .ok_or_else(|| anyhow!("missing logits output"))?;

        let logits_tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract logits: {e}"))?;
        let logits_view = logits_tensor.view();
        let logits_array = logits_view.to_owned();

        // 9. 取最后一个 step 的 logits
        let shape = logits_array.shape();
        let seq_len = shape[1];
        let last_step_logits = logits_array
            .slice(ndarray::s![0, seq_len - 1, ..])
            .to_owned();

        Ok(last_step_logits)
    }

    /// 构造指定序列长度的 decoder KV 占位符（用于非增量解码）
    pub(crate) fn build_zero_decoder_kv_for_seq_len(
        &self,
        seq_len: usize,
    ) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        let batch = 1usize;
        let zeros_dec = Array4::<f32>::zeros((batch, Self::NUM_HEADS, seq_len, Self::HEAD_DIM));

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
            let dec_k = array_to_value!(zeros_dec.clone())?;
            let dec_v = array_to_value!(zeros_dec.clone())?;
            result.push((dec_k, dec_v));
        }

        Ok(result)
    }

    /// 构造第一步用的 decoder KV 占位符（用于增量解码，固定长度为 1）
    pub(crate) fn build_zero_decoder_kv(
        &self,
    ) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
        self.build_zero_decoder_kv_for_seq_len(1)
    }

    /// 执行 decoder 的单次步进（增量解码版本）
    /// 
    /// ⚠️ **已废弃**：工程版实时翻译改造后，M2M100 使用非增量解码
    /// 此函数保留用于参考，不再被 `m2m100_translation.rs` 调用
    /// 
    /// 新格式：51 个输入（3 base + 48 KV，没有 use_cache_branch）
    /// 旧格式：52 个输入（3 base + 48 KV + 1 flag）
    #[allow(dead_code)]  // 保留用于参考，但不再使用
    pub(crate) fn decoder_step(
        &self,
        encoder_hidden_states: &Array3<f32>,
        encoder_attention_mask: &Array2<i64>,
        mut state: DecoderState,
        encoder_seq_len: usize,
        use_new_format: bool,
    ) -> anyhow::Result<(Array1<f32>, DecoderState)> {
        use std::ptr;
        use ndarray::CowArray;
        use anyhow::anyhow;

        let batch_size = 1usize;
        let cur_len = state.input_ids.len();
        let decoder_input_ids = Array2::<i64>::from_shape_vec(
            (batch_size, cur_len),
            state.input_ids.clone(),
        )?;

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

        let input_ids_value = array_to_value!(decoder_input_ids.clone(), i64)?;
        let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
        let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
        
        // 如果是旧格式，准备 use_cache_branch 值（但先不添加，等 KV cache 添加后再添加）
        let use_cache_value = if !use_new_format {
            let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);
            Some(array_to_value!(use_cache_array, bool)?)
        } else {
            None
        };

        // 组织输入顺序：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
        let mut input_values: Vec<Value<'static>> = Vec::new();

        input_values.push(encoder_mask_value);
        input_values.push(input_ids_value);
        input_values.push(encoder_states_value);

        // KV cache：准备输入 KV cache
        let decoder_kv = if state.use_cache_branch && state.decoder_kv_cache.is_some() {
            let kv = state.decoder_kv_cache.take().unwrap();
            // 调试：检查 KV cache 的形状
            if let Some((first_k, _)) = kv.first() {
                if let Ok(tensor) = first_k.try_extract::<f32>() {
                    let view = tensor.view();
                    let shape = view.shape();
                    println!("[decoder_step] 使用现有 KV cache, shape: {:?}", shape);
                }
            }
            kv
        } else {
            println!("[decoder_step] 使用零 KV cache (首次或非缓存模式)");
            self.build_zero_decoder_kv()?
        };
        
        // 准备 encoder KV cache
        // ✅ 优先使用从 decoder 输出中提取的真实 encoder KV cache
        // 如果还没有（第一步），则使用全零占位符
        // 注意：不能直接 clone Value，需要重新创建或使用 take
        // 但是，我们需要保留 encoder_kv_cache 以便后续步骤使用，所以先检查但不 take
        let encoder_kv = if state.encoder_kv_cache.is_some() {
            println!("[decoder_step] 使用缓存的 encoder KV cache（从 decoder 输出中提取）");
            // 临时取出，使用后会在最后保存回去
            state.encoder_kv_cache.take().unwrap()
        } else {
            println!("[decoder_step] 使用全零 encoder KV cache 占位符（第一步）");
            // 先解包 Result
            self.build_static_encoder_kv(encoder_seq_len)?
        };
        
        // 调试：检查 encoder KV cache 的形状
        if let Some((first_enc_k, _)) = encoder_kv.first() {
            if let Ok(tensor) = first_enc_k.try_extract::<f32>() {
                let view = tensor.view();
                let shape = view.shape();
                println!("[Encoder KV Cache] Static encoder KV cache shape: {:?}", shape);
            }
        }
        
        let mut decoder_kv_iter = decoder_kv.into_iter();
        let mut encoder_kv_iter = encoder_kv.into_iter();
        
        for _layer_idx in 0..Self::NUM_LAYERS {
            // Decoder KV
            let (dec_k, dec_v) = decoder_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient decoder KV cache"))?;
            input_values.push(dec_k);
            input_values.push(dec_v);
            
            // Encoder KV: 使用全零占位符
            let (enc_k, enc_v) = encoder_kv_iter.next()
                .ok_or_else(|| anyhow!("insufficient encoder KV cache"))?;
            input_values.push(enc_k);
            input_values.push(enc_v);
        }

        // 如果是旧格式，添加 use_cache_branch
        if let Some(use_cache_val) = use_cache_value {
            input_values.push(use_cache_val);
        }

        // 调用 session.run
        let decoder_session = self.decoder_session.lock().unwrap();
        let outputs: Vec<Value<'static>> = decoder_session.run(input_values)
            .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

        // 从输出中提取 logits + 新 KV
        let mut iter = outputs.into_iter();
        let logits_value = iter.next().expect("missing logits output");

        let logits_tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract logits: {e}"))?;
        let logits_view = logits_tensor.view();
        let logits_array = logits_view.to_owned();

        // 取最后一个 step 的 logits
        let shape = logits_array.shape();
        let seq_len = shape[1];
        let last_step_logits = logits_array
            .slice(ndarray::s![0, seq_len - 1, ..])
            .to_owned();

        // 提取 decoder KV cache
        // ✅ 修复：直接使用整个 present KV，不截断
        // 根据 Python 测试，past_seq 维度是动态的，可以接受任意长度
        // 直接将上一步的 present KV 作为下一步的 past KV，保留完整的历史信息
        let mut next_decoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
        
        // ✅ 提取 encoder KV cache（从 decoder 输出中获取真实的 encoder KV）
        // 第一次调用后，decoder 会输出 encoder KV cache，形状应该是 [1, 16, encoder_seq_len, 64]
        // 后续步骤中复用这个 encoder KV cache，而不是使用全零占位符
        let mut next_encoder_kv: Vec<(Value<'static>, Value<'static>)> = Vec::with_capacity(Self::NUM_LAYERS);
        
        for layer_idx in 0..Self::NUM_LAYERS {
            let dec_k_value = iter.next().expect("missing present.*.decoder.key");
            let dec_v_value = iter.next().expect("missing present.*.decoder.value");
            let enc_k_value = iter.next().expect("missing present.*.encoder.key");
            let enc_v_value = iter.next().expect("missing present.*.encoder.value");
            
            // ✅ 直接使用整个 present KV，不截断
            // 提取 tensor 以获取形状信息（用于调试）并重新创建 Value
            let dec_k_tensor: OrtOwnedTensor<f32, IxDyn> = dec_k_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract decoder key tensor: {e}"))?;
            let dec_k_view = dec_k_tensor.view();
            let kv_shape = dec_k_view.shape();
            let kv_seq_len = kv_shape[2];
            
            // 调试日志（前几步）
            if layer_idx == 0 {
                println!("[KV Cache] Layer 0, present KV shape: {:?}, seq_len: {}", kv_shape, kv_seq_len);
                // 检查 KV cache 的内容（检查最后一个位置的 K 值，这应该是新添加的）
                if kv_seq_len >= 2 {
                    let last_k = dec_k_view.slice(ndarray::s![0, 0, kv_seq_len - 1, 0..5.min(Self::HEAD_DIM)]);
                    let first_k = dec_k_view.slice(ndarray::s![0, 0, 0, 0..5.min(Self::HEAD_DIM)]);
                    println!("[KV Cache] Layer 0, first K values (seq=0): {:?}", first_k);
                    println!("[KV Cache] Layer 0, last K values (seq={}): {:?}", kv_seq_len - 1, last_k);
                    // 检查是否有非零值
                    let has_nonzero = dec_k_view.iter().any(|&v| v.abs() > 1e-6);
                    println!("[KV Cache] Layer 0, has non-zero values: {}", has_nonzero);
                }
            }
            
            // 重新创建 Value（因为需要 'static 生命周期）
            // 但保持完整的序列长度，不截断
            let dec_k_array = dec_k_view.to_owned();
            let dec_k_dyn = dec_k_array.into_dyn();
            let dec_k_cow = ndarray::CowArray::from(dec_k_dyn);
            
            let dec_v_tensor: OrtOwnedTensor<f32, IxDyn> = dec_v_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract decoder value tensor: {e}"))?;
            let dec_v_view = dec_v_tensor.view();
            let dec_v_array = dec_v_view.to_owned();
            let dec_v_dyn = dec_v_array.into_dyn();
            let dec_v_cow = ndarray::CowArray::from(dec_v_dyn);
            
            let dec_k_new = Value::from_array(ptr::null_mut(), &dec_k_cow)
                .map_err(|e| anyhow!("failed to convert decoder key array to Value: {e}"))?;
            let dec_v_new = Value::from_array(ptr::null_mut(), &dec_v_cow)
                .map_err(|e| anyhow!("failed to convert decoder value array to Value: {e}"))?;
            
            next_decoder_kv.push((
                unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(dec_k_new) },
                unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(dec_v_new) },
            ));
            
            // 提取 encoder KV cache
            let enc_k_tensor: OrtOwnedTensor<f32, IxDyn> = enc_k_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract encoder key tensor: {e}"))?;
            let enc_k_view = enc_k_tensor.view();
            let enc_k_shape = enc_k_view.shape();
            
            // 调试日志（第一层）
            if layer_idx == 0 {
                println!("[Encoder KV Cache] Layer 0, present encoder KV shape: {:?}", enc_k_shape);
                // 检查是否有非零值
                let has_nonzero = enc_k_view.iter().any(|&v| v.abs() > 1e-6);
                println!("[Encoder KV Cache] Layer 0, has non-zero values: {}", has_nonzero);
                if has_nonzero && enc_k_shape.len() >= 3 {
                    let seq_len = enc_k_shape[2];
                    if seq_len > 0 {
                        let first_k = enc_k_view.slice(ndarray::s![0, 0, 0, 0..5.min(Self::HEAD_DIM)]);
                        let last_k = enc_k_view.slice(ndarray::s![0, 0, seq_len - 1, 0..5.min(Self::HEAD_DIM)]);
                        println!("[Encoder KV Cache] Layer 0, first K values (seq=0): {:?}", first_k);
                        println!("[Encoder KV Cache] Layer 0, last K values (seq={}): {:?}", seq_len - 1, last_k);
                    }
                }
            }
            
            let enc_k_array = enc_k_view.to_owned();
            let enc_k_dyn = enc_k_array.into_dyn();
            let enc_k_cow = ndarray::CowArray::from(enc_k_dyn);
            
            let enc_v_tensor: OrtOwnedTensor<f32, IxDyn> = enc_v_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract encoder value tensor: {e}"))?;
            let enc_v_view = enc_v_tensor.view();
            let enc_v_array = enc_v_view.to_owned();
            let enc_v_dyn = enc_v_array.into_dyn();
            let enc_v_cow = ndarray::CowArray::from(enc_v_dyn);
            
            let enc_k_new = Value::from_array(ptr::null_mut(), &enc_k_cow)
                .map_err(|e| anyhow!("failed to convert encoder key array to Value: {e}"))?;
            let enc_v_new = Value::from_array(ptr::null_mut(), &enc_v_cow)
                .map_err(|e| anyhow!("failed to convert encoder value array to Value: {e}"))?;
            
            next_encoder_kv.push((
                unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(enc_k_new) },
                unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(enc_v_new) },
            ));
        }
        
        state.decoder_kv_cache = Some(next_decoder_kv);
        // ✅ 保存 encoder KV cache，后续步骤中复用
        state.encoder_kv_cache = Some(next_encoder_kv);
        if !state.use_cache_branch {
            state.use_cache_branch = true;
        }

        Ok((last_step_logits, state))
    }
}

