use anyhow::{Result, anyhow};

/// 初始化全局 ONNX Runtime 环境。
/// 后面所有使用 ONNX 模型（Marian NMT、Emotion 等）都应该先调用一次这个函数。
pub fn init_onnx_runtime() -> Result<()> {
    // ort::init().commit() 返回 Result<_, ort::Error>
    // 手动把它转换成 anyhow::Error 即可
    ort::init()
        .commit()
        .map_err(|e| anyhow!("failed to init ONNX Runtime: {e}"))?;

    Ok(())
}
