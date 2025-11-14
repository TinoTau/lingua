// tests/nmt_onnx_decoder_step.rs

use anyhow::{Result, anyhow};
use ndarray::{Array1, Array2, Array3, Array4};
use ort::session::Session;
use std::fs;

/// 用 Marian decoder ONNX 做一次"单步"推理，
/// 只为了验证：所有输入名字、shape、类型是否正确，
/// 模型能不能正常跑出 logits。
#[test]
fn test_marian_decoder_single_step() -> Result<()> {
    // 1. 初始化 ONNX Runtime
    core_engine::onnx_utils::init_onnx_runtime()?;

    // 2. 模型路径：使用相对路径
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/nmt/marian-en-zh/model.onnx");

    // 3. 创建 Session（使用文件模式，ort 1.16.3）
    use ort::{SessionBuilder, Environment};
    use std::sync::Arc;
    let env = Arc::new(
        Environment::builder()
            .with_name("marian_nmt_test")
            .build()?
    );
    let mut session = SessionBuilder::new(&env)
        .map_err(|e| anyhow!("failed to create Session builder: {e}"))?
        .with_model_from_file(&model_path)
        .map_err(|e| anyhow!("failed to load ONNX model: {e}"))?;

    println!("✓ Marian decoder model loaded.");

    // 3. 准备一批“假数据”，只要 shape/类型对上即可：
    //
    //    - batch_size = 1
    //    - encoder_sequence_length = 4（随便选一个 >0 的长度）
    //    - past_decoder_sequence_length = 0（第一次 decode，没有 past）
    //    - num_heads = 8
    //    - head_dim = 64
    //
    let batch_size = 1usize;
    let encoder_seq_len = 4usize;
    let past_decoder_seq_len = 0usize;
    let num_heads = 8usize;
    let head_dim = 64usize;

    // 3.1 encoder_attention_mask: int64[batch, encoder_seq_len]
    let encoder_attention_mask: Array2<i64> =
        Array2::ones((batch_size, encoder_seq_len));

    // 3.2 input_ids: int64[batch, decoder_seq_len]
    //
    // 这里选 decoder_seq_len = 1，只喂一个 token。
    // 根据你的 config.json：
    //   "decoder_start_token_id": 65000
    // 我们就用 65000 作为第一个 decoder token。
    let decoder_seq_len = 1usize;
    let start_id: i64 = 65000;

    let input_ids: Array2<i64> = Array2::from_shape_vec(
        (batch_size, decoder_seq_len),
        vec![start_id],
    )?;

    // 3.3 encoder_hidden_states: float32[batch, encoder_seq_len, 512]
    //
    // 真正的系统里，这个应该是 encoder ONNX 的输出。
    // 这里先用全 0 占位，只为验证 decoder 图能否跑通。
    let encoder_hidden_states: Array3<f32> =
        Array3::zeros((batch_size, encoder_seq_len, 512));

    // 3.4 past_key_values.*: float32[batch, num_heads, past_decoder_seq_len, head_dim]
    //
    // 第一次调用时，past_decoder_seq_len = 0，但 ORT 不允许维度为 0 的 tensor。
    // 我们需要使用最小维度 1，或者使用特殊的处理方式。
    // 对于 decoder 的 past_key_values，当 past_len = 0 时，我们使用 shape [batch, num_heads, 1, head_dim] 但数据全为 0
    fn empty_kv(
        batch: usize,
        num_heads: usize,
        past_len: usize,
        head_dim: usize,
    ) -> Array4<f32> {
        // ORT 不允许维度为 0，所以使用 max(1, past_len)
        let actual_len = past_len.max(1);
        Array4::<f32>::zeros((batch, num_heads, actual_len, head_dim))
    }

    let pkv0_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv0_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv0_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv0_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    let pkv1_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv1_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv1_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv1_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    let pkv2_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv2_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv2_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv2_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    let pkv3_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv3_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv3_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv3_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    let pkv4_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv4_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv4_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv4_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    let pkv5_dec_key = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv5_dec_val = empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim);
    let pkv5_enc_key = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);
    let pkv5_enc_val = empty_kv(batch_size, num_heads, encoder_seq_len, head_dim);

    // 3.5 use_cache_branch: bool[1]
    let use_cache_branch: Array1<bool> = Array1::from_vec(vec![true]);

    // 4. 将 ndarray 转换为 ort::Value
    // ort 1.16.3: Value::from_array 需要 allocator 和 CowArray
    use ort::value::Value;
    use std::ptr;
    use ndarray::CowArray;
    
    // 辅助函数：将 ndarray 转换为 ort::Value
    macro_rules! array_to_value {
        ($arr:expr, $t:ty) => {{
            let arr_dyn = $arr.into_dyn();
            let arr_owned = arr_dyn.to_owned();
            let cow_arr = CowArray::from(arr_owned);
            let value = Value::from_array(ptr::null_mut(), &cow_arr)
                .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
            Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
        }};
    }
    
    let encoder_attention_mask_value = array_to_value!(encoder_attention_mask, i64)?;
    let input_ids_value = array_to_value!(input_ids, i64)?;
    let encoder_hidden_states_value = array_to_value!(encoder_hidden_states, f32)?;
    let use_cache_branch_value = array_to_value!(use_cache_branch, bool)?;

    // 转换所有 past_key_values
    let pkv0_dec_key_value = array_to_value!(pkv0_dec_key, f32)?;
    let pkv0_dec_val_value = array_to_value!(pkv0_dec_val, f32)?;
    let pkv0_enc_key_value = array_to_value!(pkv0_enc_key, f32)?;
    let pkv0_enc_val_value = array_to_value!(pkv0_enc_val, f32)?;

    let pkv1_dec_key_value = array_to_value!(pkv1_dec_key, f32)?;
    let pkv1_dec_val_value = array_to_value!(pkv1_dec_val, f32)?;
    let pkv1_enc_key_value = array_to_value!(pkv1_enc_key, f32)?;
    let pkv1_enc_val_value = array_to_value!(pkv1_enc_val, f32)?;

    let pkv2_dec_key_value = array_to_value!(pkv2_dec_key, f32)?;
    let pkv2_dec_val_value = array_to_value!(pkv2_dec_val, f32)?;
    let pkv2_enc_key_value = array_to_value!(pkv2_enc_key, f32)?;
    let pkv2_enc_val_value = array_to_value!(pkv2_enc_val, f32)?;

    let pkv3_dec_key_value = array_to_value!(pkv3_dec_key, f32)?;
    let pkv3_dec_val_value = array_to_value!(pkv3_dec_val, f32)?;
    let pkv3_enc_key_value = array_to_value!(pkv3_enc_key, f32)?;
    let pkv3_enc_val_value = array_to_value!(pkv3_enc_val, f32)?;

    let pkv4_dec_key_value = array_to_value!(pkv4_dec_key, f32)?;
    let pkv4_dec_val_value = array_to_value!(pkv4_dec_val, f32)?;
    let pkv4_enc_key_value = array_to_value!(pkv4_enc_key, f32)?;
    let pkv4_enc_val_value = array_to_value!(pkv4_enc_val, f32)?;

    let pkv5_dec_key_value = array_to_value!(pkv5_dec_key, f32)?;
    let pkv5_dec_val_value = array_to_value!(pkv5_dec_val, f32)?;
    let pkv5_enc_key_value = array_to_value!(pkv5_enc_key, f32)?;
    let pkv5_enc_val_value = array_to_value!(pkv5_enc_val, f32)?;

    // 5. 调用 session.run，ort 1.16.3 使用 Vec<Value>，按输入顺序排列
    let inputs: Vec<ort::Value> = vec![
        encoder_attention_mask_value,
        input_ids_value,
        encoder_hidden_states_value,
        use_cache_branch_value,
        pkv0_dec_key_value,
        pkv0_dec_val_value,
        pkv0_enc_key_value,
        pkv0_enc_val_value,
        pkv1_dec_key_value,
        pkv1_dec_val_value,
        pkv1_enc_key_value,
        pkv1_enc_val_value,
        pkv2_dec_key_value,
        pkv2_dec_val_value,
        pkv2_enc_key_value,
        pkv2_enc_val_value,
        pkv3_dec_key_value,
        pkv3_dec_val_value,
        pkv3_enc_key_value,
        pkv3_enc_val_value,
        pkv4_dec_key_value,
        pkv4_dec_val_value,
        pkv4_enc_key_value,
        pkv4_enc_val_value,
        pkv5_dec_key_value,
        pkv5_dec_val_value,
        pkv5_enc_key_value,
        pkv5_enc_val_value,
    ];
    let outputs: Vec<ort::Value> = session.run(inputs)
        .map_err(|e| anyhow!("failed to run session: {e}"))?;

    // 6. 取出 logits，查看形状
    // ort 1.16.3: outputs 是 Vec<Value>，logits 是第一个输出（索引 0）
    use ort::tensor::OrtOwnedTensor;
    use ndarray::{IxDyn, Ix3};
    let logits_value = &outputs[0];
    let tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
        .try_extract::<f32>()
        .map_err(|e| anyhow!("failed to extract logits tensor: {e}"))?;
    let view = tensor.view();
    let logits_arr: ndarray::Array3<f32> = view
        .to_owned()
        .into_dimensionality::<Ix3>()
        .map_err(|e| anyhow!("failed to convert logits to Array3: {e}"))?;
    println!("Decoder logits shape: {:?}", logits_arr.shape());

    // 再看一下一部分值，确认不是全 0（虽然因为 encoder_hidden_states 是 0，语义肯定是乱的）
    let flat: Vec<f32> = logits_arr.iter().cloned().collect();
    println!("First few logit values: {:?}", &flat[..flat.len().min(10)]);

    // 7. 同时也可以看一下某一层 present cache 的 shape，确认 KV cache 正常返回
    // present.0.decoder.key 应该在索引 1（logits 是 0）
    let present0_dec_key_value = &outputs[1];
    let present0_tensor: OrtOwnedTensor<f32, IxDyn> = present0_dec_key_value
        .try_extract::<f32>()
        .map_err(|e| anyhow!("failed to extract present.0.decoder.key tensor: {e}"))?;
    let present0_view = present0_tensor.view();
    let present0_arr = present0_view.to_owned();
    println!("present.0.decoder.key shape: {:?}", present0_arr.shape());

    Ok(())
}
