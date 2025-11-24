use std::path::PathBuf;

use core_engine::nmt_incremental::translate_full_sentence_stub;

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| matches!(c as u32, 0x4E00..=0x9FFF))
}

/// 测试：调用 NMT 模块的整句翻译入口（真实 Marian 推理）
#[test]
fn test_translate_full_sentence_stub() {
    // 通过 CARGO_MANIFEST_DIR 定位到 core/engine
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 使用 core/engine 提供的 M2M100 模型
    let model_path = crate_root.join("models/nmt/m2m100-en-zh");
    if !model_path.exists() {
        eprintln!("⚠ 跳过测试：M2M100 模型目录缺失 {}", model_path.display());
        return;
    }

    let input = "Hello world";
    let output = translate_full_sentence_stub(input, &model_path)
        .expect("NMT stub translation failed");

    println!("NMT STUB OUTPUT: {}", output);

    assert!(
        contains_cjk(&output),
        "M2M100 翻译未生成中文文本: {}",
        output
    );
}
