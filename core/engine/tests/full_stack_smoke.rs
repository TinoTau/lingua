// tests/full_stack_smoke.rs

use std::path::PathBuf;

use core_engine::asr_whisper::cli::{WhisperCliConfig, WhisperCliEngine};
use core_engine::asr_whisper::AsrEngine;
use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::nmt_incremental::{
    load_marian_onnx_for_smoke_test,
    translate_full_sentence_stub,
    MarianNmtOnnx,
};

/// 1. ASR：用 whisper-cli 跑 JFK 示例音频，检查经典台词是否出现。
#[test]
fn test_fullstack_asr_whisper_jfk_sample() {
    // CARGO_MANIFEST_DIR = core/engine
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // 项目根目录：.../lingua
    let project_root = crate_root
        .parent()      // .../lingua/core
        .and_then(|p| p.parent()) // .../lingua
        .expect("failed to resolve project root");

    // whisper.cpp 可执行文件、模型和示例音频路径
    let exe = project_root.join("third_party/whisper.cpp/build/bin/whisper-cli.exe");
    let model = project_root.join("third_party/whisper.cpp/models/ggml-base.en.bin");
    let wav = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

    assert!(
        exe.exists(),
        "whisper-cli.exe not found at {:?}",
        exe
    );
    assert!(
        model.exists(),
        "whisper model not found at {:?}",
        model
    );
    assert!(
        wav.exists(),
        "JFK sample wav not found at {:?}",
        wav
    );

    let cfg = WhisperCliConfig {
        exe_path: exe.to_string_lossy().into_owned(),
        model_path: model.to_string_lossy().into_owned(),
    };

    let engine = WhisperCliEngine::new(cfg);

    let result = engine
        .transcribe_wav_file(&wav)
        .expect("failed to run whisper-cli on JFK sample");

    let text = result.text.to_lowercase();
    println!("ASR JFK RESULT:\n{}", text);

    assert!(
        text.contains("ask not what your country can do for you"),
        "ASR output does not contain expected JFK quote"
    );
    assert!(
        text.contains("what you can do for your country"),
        "ASR output does not contain second part of JFK quote"
    );
}

/// 2. NMT：加载 Marian ONNX 模型，并通过 stub 翻译一条简单的句子。
#[test]
fn test_fullstack_nmt_marian_stub_hello_world() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

    let model_path = project_root.join("third_party/nmt/marian-en-zh/model.onnx");

    assert!(
        model_path.exists(),
        "NMT ONNX model not found at {:?}",
        model_path
    );

    // 先确认 ORT 能正常加载模型
    load_marian_onnx_for_smoke_test(&model_path)
        .expect("failed to load Marian NMT ONNX model");

    let input = "Hello world";
    let output = translate_full_sentence_stub(input, &model_path)
        .expect("NMT stub translation failed");

    println!("NMT STUB OUTPUT: {}", output);

    // 只检查输出包含占位前缀和原始输入文本
    assert!(
        output.contains("[NMT stub en→zh]"),
        "NMT stub output missing expected prefix"
    );
    assert!(
        output.contains(input),
        "NMT stub output does not contain original input text"
    );
}

/// 3. CoreEngineBuilder：检查默认的 Marian NMT stub wiring 是否正常。
#[test]
fn test_fullstack_core_engine_builder_nmt_wiring() {
    // 这里只测试 NMT wiring，不构造完整 CoreEngine（因为其它依赖还没实现）。
    let builder = CoreEngineBuilder::new()
        .nmt_with_default_marian_stub()
        .expect("failed to attach MarianNmtStub via nmt_with_default_marian_stub");

    // 不能直接访问 builder.nmt（是私有字段），
    // 但如果路径不对或构造失败，上面的 expect 已经会 panic。
    //
    // 这里仅仅确保函数本身不会出错即可。
    let _ = builder;
}

/// 4. NMT ONNX：加载完整的 MarianNmtOnnx 并打印模型的 I/O 信息。
#[test]
fn test_fullstack_nmt_onnx_model_io_info() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

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
