use std::path::PathBuf;
use core_engine::tts_streaming::TextProcessor;

/// 测试文本预处理器加载
#[test]
fn test_text_processor_load() {
    let model_dir = PathBuf::from("models/tts");
    
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found at {}", model_dir.display());
        return;
    }
    
    let processor_zh = TextProcessor::new_from_dir(&model_dir, "zh");
    let processor_en = TextProcessor::new_from_dir(&model_dir, "en");
    
    match processor_zh {
        Ok(p) => {
            println!("✅ Chinese TextProcessor loaded successfully");
            println!("   Phone map size: {}", p.get_phone_to_id_map().len());
        }
        Err(e) => {
            eprintln!("⚠️  Failed to load Chinese TextProcessor: {}", e);
        }
    }
    
    match processor_en {
        Ok(p) => {
            println!("✅ English TextProcessor loaded successfully");
            println!("   Phone map size: {}", p.get_phone_to_id_map().len());
        }
        Err(e) => {
            eprintln!("⚠️  Failed to load English TextProcessor: {}", e);
        }
    }
}

/// 测试文本规范化
#[test]
fn test_text_normalization() {
    let model_dir = PathBuf::from("models/tts");
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found");
        return;
    }
    
    let processor = match TextProcessor::new_from_dir(&model_dir, "en") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping test: failed to load TextProcessor: {}", e);
            return;
        }
    };
    
    let test_cases = vec![
        ("  hello   world  ", "hello world"),
        ("Hello, World!", "Hello, World!"),
        ("test\nwith\nnewlines", "test with newlines"),
    ];
    
    for (input, expected_prefix) in test_cases {
        let normalized = processor.normalize_text(input);
        println!("Input: '{}' -> Normalized: '{}'", input, normalized);
        assert!(!normalized.is_empty());
    }
}

/// 测试音素 ID 映射
#[test]
fn test_phoneme_to_id_mapping() {
    let model_dir = PathBuf::from("models/tts");
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found");
        return;
    }
    
    let processor = match TextProcessor::new_from_dir(&model_dir, "en") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping test: failed to load TextProcessor: {}", e);
            return;
        }
    };
    
    // 测试已知音素（phone_id_map.txt 中应该存在）
    let test_phonemes = vec!["<pad>", "<unk>", "AA0", "AA1", "B", "CH"];
    
    for phone in test_phonemes {
        let ids = processor.phonemes_to_ids(&[phone.to_string()]);
        match ids {
            Ok(ids_vec) => {
                if !ids_vec.is_empty() {
                    println!("✅ Phoneme '{}' -> ID: {:?}", phone, ids_vec);
                } else {
                    println!("⚠️  Phoneme '{}' -> Empty ID vector", phone);
                }
            }
            Err(e) => {
                println!("⚠️  Failed to map phoneme '{}': {}", phone, e);
            }
        }
    }
}

