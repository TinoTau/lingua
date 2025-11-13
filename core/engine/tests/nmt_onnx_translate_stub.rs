use std::path::PathBuf;

use core_engine::nmt_incremental::translate_full_sentence_stub;

/// 测试：调用 NMT 模块的“整句翻译入口”（目前是 stub 实现）
#[test]
fn test_translate_full_sentence_stub() {
    // 通过 CARGO_MANIFEST_DIR 定位到 core/engine
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 项目根目录：D:\Programs\github\lingua
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

    // 使用你刚才已经验证过的模型路径
    let model_path = project_root.join("third_party/nmt/marian-en-zh/model.onnx");

    let input = "Hello world";
    let output = translate_full_sentence_stub(input, &model_path)
        .expect("NMT stub translation failed");

    println!("NMT STUB OUTPUT: {}", output);

    // 确认输出有我们预期的前缀和原文
    assert!(output.contains("[NMT stub en→zh]"));
    assert!(output.contains(input));
}
