use core_engine::persona_adapter::{PersonaAdapter, PersonaContext, PersonaStub, RuleBasedPersonaAdapter};
use core_engine::types::StableTranscript;

/// 测试 PersonaStub（不依赖任何配置）
#[tokio::test]
async fn test_persona_stub() {
    let stub = PersonaStub::new();
    
    let transcript = StableTranscript {
        text: "Hello, this is a test.".to_string(),
        speaker_id: None,
        language: "en".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "formal".to_string(),
        culture: "en".to_string(),
    };
    
    let result = stub.personalize(transcript.clone(), context).await.unwrap();
    
    // Stub 应该直接返回原始 transcript
    assert_eq!(result.text, transcript.text);
    assert_eq!(result.speaker_id, transcript.speaker_id);
    assert_eq!(result.language, transcript.language);
    
    println!("✅ PersonaStub test passed: text={}", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 正式语调（中文）
#[tokio::test]
async fn test_rule_based_formal_chinese() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "帮我做这个".to_string(),
        speaker_id: None,
        language: "zh".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "formal".to_string(),
        culture: "zh".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 正式语调应该添加"请"
    assert!(result.text.contains("请") || result.text.starts_with("请"));
    println!("✅ Formal Chinese test passed: original='帮我做这个', personalized='{}'", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 随意语调（中文）
#[tokio::test]
async fn test_rule_based_casual_chinese() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "请您帮我做这个".to_string(),
        speaker_id: None,
        language: "zh".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "casual".to_string(),
        culture: "zh".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 随意语调应该移除"请"和"您"
    assert!(!result.text.contains("请") || !result.text.contains("您"));
    println!("✅ Casual Chinese test passed: original='请您帮我做这个', personalized='{}'", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 友好语调（中文）
#[tokio::test]
async fn test_rule_based_friendly_chinese() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "你好".to_string(),
        speaker_id: None,
        language: "zh".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "friendly".to_string(),
        culture: "zh".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 友好语调应该添加"哦"或"呢"
    assert!(result.text.ends_with("哦") || result.text.ends_with("呢") || result.text.ends_with("！"));
    println!("✅ Friendly Chinese test passed: original='你好', personalized='{}'", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 正式语调（英文）
#[tokio::test]
async fn test_rule_based_formal_english() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "I don't want to do this".to_string(),
        speaker_id: None,
        language: "en".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "formal".to_string(),
        culture: "en".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 正式语调应该使用完整形式
    assert!(!result.text.contains("don't"));
    assert!(result.text.contains("do not"));
    println!("✅ Formal English test passed: original='I don't want to do this', personalized='{}'", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 随意语调（英文）
#[tokio::test]
async fn test_rule_based_casual_english() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "I do not want to do this".to_string(),
        speaker_id: None,
        language: "en".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "casual".to_string(),
        culture: "en".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 随意语调应该使用缩写
    assert!(result.text.contains("don't"));
    println!("✅ Casual English test passed: original='I do not want to do this', personalized='{}'", result.text);
}

/// 测试 RuleBasedPersonaAdapter - 友好语调（英文）
#[tokio::test]
async fn test_rule_based_friendly_english() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let transcript = StableTranscript {
        text: "Hello".to_string(),
        speaker_id: None,
        language: "en".to_string(),
    };
    
    let context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "friendly".to_string(),
        culture: "en".to_string(),
    };
    
    let result = adapter.personalize(transcript, context).await.unwrap();
    
    // 友好语调应该添加"!"或":)"
    assert!(result.text.ends_with("!") || result.text.ends_with(":)"));
    println!("✅ Friendly English test passed: original='Hello', personalized='{}'", result.text);
}

/// 测试多个不同的 tone 和 culture 组合
#[tokio::test]
async fn test_rule_based_multiple_combinations() {
    let adapter = RuleBasedPersonaAdapter::new();
    
    let test_cases = vec![
        ("formal", "zh", "帮我", "请帮我"),
        ("casual", "zh", "请您帮我", "你帮我"),
        ("friendly", "en", "Hi", "Hi!"),
        ("professional", "en", "This is a test", "This is a test"),
    ];
    
    for (tone, culture, input, _expected_prefix) in test_cases {
        let transcript = StableTranscript {
            text: input.to_string(),
            speaker_id: None,
            language: culture.to_string(),
        };
        
        let context = PersonaContext {
            user_id: "test_user".to_string(),
            tone: tone.to_string(),
            culture: culture.to_string(),
        };
        
        let result = adapter.personalize(transcript, context).await.unwrap();
        println!("✅ Test case: tone={}, culture={}, input='{}', output='{}'", 
            tone, culture, input, result.text);
    }
}

