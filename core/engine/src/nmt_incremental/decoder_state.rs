use ort::value::Value;

/// 单句翻译时 Decoder 的状态
/// 方案 C：将 encoder KV cache 和 decoder KV cache 分开存储
pub(crate) struct DecoderState {
    /// 当前 decoder 的 input_ids（最后一个 token 是本步要解码的）
    pub input_ids: Vec<i64>,
    /// 已经生成的 token IDs（不包括起始的 decoder_start_token_id）
    pub generated_ids: Vec<i64>,
    /// Decoder KV cache（每层有 2 个 Value：decoder.key, decoder.value）
    /// - `None` 代表第一步（没有历史 KV）
    pub decoder_kv_cache: Option<Vec<(Value<'static>, Value<'static>)>>,
    /// Encoder KV cache（每层有 2 个 Value：encoder.key, encoder.value）
    /// - 只在 Step 0 时提取一次，之后保持不变
    /// - `None` 代表还没有提取
    pub encoder_kv_cache: Option<Vec<(Value<'static>, Value<'static>)>>,
    /// 控制 `use_cache_branch` 输入
    pub use_cache_branch: bool,
}

