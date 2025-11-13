// tests/asr_whisper_cli.rs

use std::path::PathBuf;

use core_engine::asr_whisper::cli::{WhisperCliConfig, WhisperCliEngine};
use core_engine::asr_whisper::AsrEngine;

#[test]
fn test_whisper_cli_on_jfk_sample() {
    //
    // === 关键点 ===
    //
    // 集成测试运行时的当前目录是：
    //     D:\Programs\github\lingua   （workspace 根目录）
    //
    // 所以相对路径必须从 “lingua 根目录” 出发。
    //
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()            // D:\Programs\github\lingua\core
        .unwrap()
        .parent()            // D:\Programs\github\lingua
        .unwrap()
        .to_path_buf();

    // 相对于项目根目录的相对路径
    let exe = project_root.join("third_party/whisper.cpp/build/bin/whisper-cli.exe");
    let model = project_root.join("third_party/whisper.cpp/models/ggml-base.en.bin");
    let wav = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

    // 配置
    let cfg = WhisperCliConfig {
        exe_path: exe.to_string_lossy().into_owned(),
        model_path: model.to_string_lossy().into_owned(),
    };

    let engine = WhisperCliEngine::new(cfg);

    // 执行
    let result = engine
        .transcribe_wav_file(&wav)
        .expect("failed to run whisper-cli");

    let text = result.text.to_lowercase();
    println!("ASR RESULT:\n{}", text);

    assert!(text.contains("ask not what your country can do for you"));
    assert!(text.contains("what you can do for your country"));
}
