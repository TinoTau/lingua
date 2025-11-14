use anyhow::Result;

/// 初始化全局 ONNX Runtime 环境。
/// 后面所有使用 ONNX 模型（Marian NMT、Emotion 等）都应该先调用一次这个函数。
/// 
/// 注意：在 ort 1.16.3 中，Environment 的创建方式已改变。
/// 这个函数现在只是一个占位符，实际的 Environment 创建在需要时进行。
pub fn init_onnx_runtime() -> Result<()> {
    // ort 1.16.3: 不再需要全局初始化
    // Environment 在创建 Session 时自动创建
    Ok(())
}
