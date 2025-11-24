// src/bin/nmt_stub_cli.rs

use std::path::PathBuf;

// 使用你库里的 NMT stub 函数
use core_engine::nmt_incremental::translate_full_sentence_stub;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("nmt_stub_cli error: {e}");
        std::process::exit(1);
    }
}

fn real_main() -> anyhow::Result<()> {
    // 1. 从命令行读取要翻译的文本
    //    用法示例：
    //      cargo run --bin nmt_stub_cli -- "Hello world from CLI"
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "Hello world".to_string()
    };

    // 2. 通过 CARGO_MANIFEST_DIR 定位到 core/engine，再推到项目根目录
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 3. 使用 core/engine 内部的 M2M100 模型目录
    let model_path = crate_root.join("models/nmt/m2m100-en-zh");

    // 4. 调用你已经写好的 stub 翻译函数
    let output = translate_full_sentence_stub(&input, &model_path)?;

    // 5. 打印结果
    println!("{output}");

    Ok(())
}
