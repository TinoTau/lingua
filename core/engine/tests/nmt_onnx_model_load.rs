use std::path::PathBuf;

use core_engine::nmt_incremental::load_marian_onnx_for_smoke_test;

/// 测试：能否成功加载 Marian NMT 的 ONNX 模型（仅检查 Session 创建成功）
#[test]
fn test_load_marian_onnx_model() {
    // 通过 CARGO_MANIFEST_DIR 找到 core/engine 目录
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 项目根目录：D:\Programs\github\lingua
    let project_root = crate_root
        .parent()  // ...\lingua\core
        .and_then(|p| p.parent())  // ...\lingua
        .expect("failed to resolve project root");

    // 按我们约定的路径拼出 model.onnx 的位置
    let model_path = project_root.join("third_party/nmt/marian-en-zh/model.onnx");

    load_marian_onnx_for_smoke_test(&model_path)
        .expect(&format!("failed to load Marian ONNX model at {}", model_path.display()));
}
