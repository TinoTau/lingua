use ort::value::Value;

/// 单句翻译时 Decoder 的状态
/// 根据 marian_nmt_interface_spec.md：只维护 decoder KV cache，encoder KV 作为静态占位符
/// 注意：新模型格式可能需要使用 decoder 输出的 encoder KV cache
pub(crate) struct DecoderState {
    /// 当前 decoder 的 input_ids（最后一个 token 是本步要解码的）
    pub input_ids: Vec<i64>,
    /// 已经生成的 token IDs（不包括起始的 decoder_start_token_id）
    pub generated_ids: Vec<i64>,
    /// Decoder KV cache（每层有 2 个 Value：decoder.key, decoder.value）
    /// - `None` 代表第一步（没有历史 KV）
    pub decoder_kv_cache: Option<Vec<(Value<'static>, Value<'static>)>>,
    /// Encoder KV cache（每层有 2 个 Value：encoder.key, encoder.value）
    /// - `None` 代表第一步（使用全零占位符）
    /// - 从第二步开始，使用上一步 decoder 输出的 encoder KV cache
    pub encoder_kv_cache: Option<Vec<(Value<'static>, Value<'static>)>>,
    /// 控制 `use_cache_branch` 输入
    pub use_cache_branch: bool,
}

