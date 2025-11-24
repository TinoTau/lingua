// ⚠️ 已废弃：此测试使用 ONNX decoder，已不再使用。
// 当前系统已切换为 Python NMT 服务（HTTP 调用）。

use std::path::PathBuf;

use core_engine::nmt_incremental::MarianNmtOnnx;

/// 测试：能否成功加载 Marian NMT 的 ONNX 模型并打印 I/O 信息
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_load_marian_onnx_model() {
    // 通过 CARGO_MANIFEST_DIR 找到 core/engine 目录
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 使用 core/engine/models/nmt/marian-en-zh/ 目录
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    assert!(
        model_dir.exists(),
        "NMT model directory not found at {:?}",
        model_dir
    );

    // 调用 new_from_dir 会打印模型的 I/O 信息
    let _nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("failed to load MarianNmtOnnx from directory");

    println!("✓ MarianNmtOnnx loaded successfully");
}
