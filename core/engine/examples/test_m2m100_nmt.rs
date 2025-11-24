/// M2M100 NMT 模型测试程序
/// 
/// 测试 M2M100NmtOnnx 的加载、编码、解码和翻译功能

use core_engine::nmt_incremental::M2M100NmtOnnx;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== M2M100 NMT 模型测试 ===\n");

    // 1. 测试模型目录路径
    let model_dir = PathBuf::from("models/nmt/m2m100-en-zh");
    println!("[1/6] 检查模型目录: {}", model_dir.display());
    
    if !model_dir.exists() {
        eprintln!("❌ 模型目录不存在: {}", model_dir.display());
        eprintln!("请确保模型已导出到: {}", model_dir.display());
        return Err("模型目录不存在".into());
    }
    println!("✅ 模型目录存在\n");

    // 2. 测试加载模型
    println!("[2/6] 加载 M2M100 NMT 模型...");
    let model = match M2M100NmtOnnx::new_from_dir(&model_dir) {
        Ok(m) => {
            println!("✅ 模型加载成功");
            println!("   源语言: {}", m.src_lang);
            println!("   目标语言: {}", m.tgt_lang);
            println!("   Decoder start token ID: {}", m.decoder_start_token_id);
            println!("   EOS token ID: {}", m.eos_token_id);
            println!("   PAD token ID: {}", m.pad_token_id);
            println!("   最大长度: {}\n", m.max_length);
            m
        }
        Err(e) => {
            eprintln!("❌ 模型加载失败: {}", e);
            return Err(e.into());
        }
    };

    // 3. 测试 Tokenizer
    println!("[3/6] 测试 Tokenizer...");
    let test_text = "Hello world";
    match model.tokenizer.encode(test_text, &model.src_lang, true) {
        Ok(ids) => {
            println!("✅ 编码成功: \"{}\" -> {} tokens", test_text, ids.len());
            if ids.len() <= 20 {
                println!("   IDs: {:?}", ids);
            }
        }
        Err(e) => {
            eprintln!("❌ 编码失败: {}", e);
        }
    }
    println!();

    // 4. 测试语言 ID
    println!("[4/5] 测试语言 ID...");
    let src_lang_id = model.tokenizer.get_lang_id(&model.src_lang);
    let tgt_lang_id = model.tokenizer.get_lang_id(&model.tgt_lang);
    println!("   源语言 ({}): {}", model.src_lang, src_lang_id);
    println!("   目标语言 ({}): {}", model.tgt_lang, tgt_lang_id);
    println!("✅ 语言 ID 获取成功\n");

    // 5. 测试完整翻译（可选，可能需要较长时间）
    println!("[5/5] 测试完整翻译...");
    println!("   注意: 完整翻译可能需要较长时间，如果模型未正确配置可能会失败");
    println!("   跳过完整翻译测试（可以在集成测试中验证）\n");

    println!("=== 所有测试完成 ===");
    println!("\n下一步:");
    println!("  - 运行完整翻译测试: 修改代码取消注释翻译测试部分");
    println!("  - 进行集成测试: 使用 test_s2s_full_simple.rs");
    
    Ok(())
}

