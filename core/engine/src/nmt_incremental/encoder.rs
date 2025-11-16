use anyhow::{Result, anyhow};
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ndarray::{Array2, Array3, IxDyn, Ix3};
use super::marian_onnx::MarianNmtOnnx;

impl MarianNmtOnnx {
    /// 运行 encoder 模型，获取 encoder_hidden_states
    /// 
    /// # Arguments
    /// * `input_ids` - 编码后的输入 token IDs
    /// 
    /// # Returns
    /// (encoder_hidden_states, encoder_attention_mask)
    pub(crate) fn run_encoder(&self, input_ids: &[i64]) -> Result<(Array3<f32>, Array2<i64>)> {
        use ort::value::Value;

        let batch_size = 1usize;
        let seq_len = input_ids.len();

        // 准备输入
        let input_ids_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids.to_vec(),
        )?;

        let attention_mask: Array2<i64> = Array2::ones((batch_size, seq_len));

        // 转换为 ONNX Value
        // ort 1.16.3 使用 from_array 需要 allocator 和 array（需要 CowRepr 类型）
        use std::ptr;
        use ndarray::CowArray;
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                // 转换为动态维度和 CowRepr 类型，使用 null allocator（让 ORT 使用默认 allocator）
                // 注意：需要确保数据是 owned 的，这样 Value 可以拥有数据的所有权
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                // Value::from_array 会复制数据，但返回的 Value 仍然有生命周期限制
                // 使用 transmute 转换为 'static 生命周期（因为数据已经被复制到 ORT 内部）
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let input_ids_value: ort::Value<'static> = array_to_value!(input_ids_array, i64)?;
        let attention_mask_value: ort::Value<'static> = array_to_value!(attention_mask.clone(), i64)?;

        // 运行 encoder
        // ort 1.16.3: session.run() 接受 Vec<Value>，按输入顺序排列
        // 注意：需要按照模型定义的输入顺序传递
        let encoder_session = self.encoder_session.lock().unwrap();
        let inputs = vec![input_ids_value, attention_mask_value];
        let outputs: Vec<ort::Value> = encoder_session.run(inputs)
            .map_err(|e| anyhow!("failed to run encoder model: {e}"))?;

        // 提取 encoder_hidden_states (last_hidden_state)
        // ort 1.16.3: 当使用 HashMap 构建的 inputs 时，outputs 是 Vec<Value>，按输出顺序排列
        // encoder 只有一个输出 last_hidden_state，索引为 0
        let hidden_states_value = &outputs[0];

        // ort 1.16.3: 使用 try_extract() + view() + into_dimensionality()
        let tensor: OrtOwnedTensor<f32, IxDyn> = hidden_states_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract encoder hidden states: {e}"))?;
        let view = tensor.view();
        let encoder_hidden_states: Array3<f32> = view
            .to_owned()
            .into_dimensionality::<Ix3>()
            .map_err(|e| anyhow!("failed to convert to Array3: {e}"))?;

        Ok((encoder_hidden_states, attention_mask))
    }
}

