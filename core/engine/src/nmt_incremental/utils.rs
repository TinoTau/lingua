use anyhow::{Result, anyhow, Context};
use std::path::{Path, PathBuf};

use super::m2m100_onnx::M2M100NmtOnnx;

/// 仅用于测试：尝试加载 Marian NMT 的 ONNX 模型，确认文件与 ORT 兼容。
pub fn load_marian_onnx_for_smoke_test(model_path: &Path) -> Result<()> {
    // 1. 初始化全局 ORT 环境
    crate::onnx_utils::init_onnx_runtime()?;

    // 2. 检查文件存在
    if !model_path.exists() {
        return Err(anyhow!(
            "NMT ONNX model not found at: {}",
            model_path.display()
        ));
    }

    // 4. 使用 SessionBuilder 加载模型（文件模式）
    use ort::{SessionBuilder, Environment};
    use std::sync::Arc;
    let env = Arc::new(
        Environment::builder()
            .with_name("marian_nmt_test")
            .build()?
    );
    let _session = SessionBuilder::new(&env)
        .map_err(|e| anyhow!("failed to create Session builder: {e}"))?
        .with_model_from_file(model_path)
        .map_err(|e| anyhow!("failed to load NMT model: {e}"))?;

    // 能走到这里，说明模型格式至少是 ORT 能识别的
    Ok(())
}

/// 整句翻译入口（调用实际的 Marian ONNX 推理）
pub fn translate_full_sentence_stub(input: &str, model_path: &Path) -> Result<String> {
    let model_dir = resolve_model_dir(model_path)
        .with_context(|| format!("无法定位 M2M100 模型目录: {}", model_path.display()))?;

    let translator = M2M100NmtOnnx::new_from_dir(&model_dir)
        .with_context(|| format!("加载 M2M100 模型失败: {}", model_dir.display()))?;

    translator
        .translate(input)
        .with_context(|| "M2M100 翻译失败".to_string())
}

fn resolve_model_dir(model_path: &Path) -> Result<PathBuf> {
    if model_path.is_dir() {
        return Ok(model_path.to_path_buf());
    }

    if let Some(parent) = model_path.parent() {
        if parent.exists() {
            return Ok(parent.to_path_buf());
        }
    }

    Err(anyhow!(
        "模型路径既不是目录，也找不到上层目录: {}",
        model_path.display()
    ))
}

