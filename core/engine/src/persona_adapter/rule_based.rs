use async_trait::async_trait;

use crate::error::EngineResult;
use crate::types::StableTranscript;
use super::{PersonaAdapter, PersonaContext};

/// 基于规则的 Persona 适配器实现
/// 
/// 根据 PersonaContext 中的 tone 和 culture 信息，对文本进行个性化处理
pub struct RuleBasedPersonaAdapter {
    // 可以添加配置或规则数据
}

impl RuleBasedPersonaAdapter {
    /// 创建新的 RuleBasedPersonaAdapter
    pub fn new() -> Self {
        Self {}
    }

    /// 根据 tone 和 culture 对文本进行个性化处理
    fn apply_persona_rules(&self, text: &str, context: &PersonaContext) -> String {
        let mut result = text.to_string();

        // 根据 tone（语调）进行个性化
        match context.tone.as_str() {
            "formal" => {
                // 正式语调：保持原样，可能添加敬语
                result = self.make_formal(&result, &context.culture);
            }
            "casual" => {
                // 随意语调：简化表达
                result = self.make_casual(&result, &context.culture);
            }
            "friendly" => {
                // 友好语调：添加友好表达
                result = self.make_friendly(&result, &context.culture);
            }
            "professional" => {
                // 专业语调：使用专业术语
                result = self.make_professional(&result, &context.culture);
            }
            _ => {
                // 默认：保持原样
            }
        }

        result
    }

    /// 转换为正式语调
    fn make_formal(&self, text: &str, culture: &str) -> String {
        match culture {
            "zh" | "chinese" => {
                // 中文正式语调：添加"您"、"请"等
                let mut result = text.to_string();
                // 简单的规则：在句首添加"请"
                if !result.starts_with("请") && !result.starts_with("Please") {
                    result = format!("请{}", result);
                }
                result
            }
            "en" | "english" => {
                // 英文正式语调：使用完整形式
                let mut result = text.to_string();
                // 简单的规则：确保使用完整形式
                result = result.replace("don't", "do not");
                result = result.replace("can't", "cannot");
                result = result.replace("won't", "will not");
                result
            }
            _ => text.to_string(),
        }
    }

    /// 转换为随意语调
    fn make_casual(&self, text: &str, culture: &str) -> String {
        match culture {
            "zh" | "chinese" => {
                // 中文随意语调：简化表达
                let mut result = text.to_string();
                // 移除"请"等正式用语
                result = result.replace("请", "");
                result = result.replace("您", "你");
                result
            }
            "en" | "english" => {
                // 英文随意语调：使用缩写
                let mut result = text.to_string();
                result = result.replace("do not", "don't");
                result = result.replace("cannot", "can't");
                result = result.replace("will not", "won't");
                result
            }
            _ => text.to_string(),
        }
    }

    /// 转换为友好语调
    fn make_friendly(&self, text: &str, culture: &str) -> String {
        match culture {
            "zh" | "chinese" => {
                // 中文友好语调：添加友好表达
                let mut result = text.to_string();
                // 在句尾添加"哦"、"呢"等
                if !result.ends_with("哦") && !result.ends_with("呢") && !result.ends_with("！") {
                    result = format!("{}哦", result);
                }
                result
            }
            "en" | "english" => {
                // 英文友好语调：添加友好表达
                let mut result = text.to_string();
                // 在句尾添加"!"或" :)"
                if !result.ends_with("!") && !result.ends_with(":)") {
                    result = format!("{}!", result);
                }
                result
            }
            _ => text.to_string(),
        }
    }

    /// 转换为专业语调
    fn make_professional(&self, text: &str, culture: &str) -> String {
        // 专业语调：保持原样，但确保使用专业术语
        // 这里可以添加专业术语替换规则
        text.to_string()
    }
}

#[async_trait]
impl PersonaAdapter for RuleBasedPersonaAdapter {
    async fn personalize(
        &self,
        transcript: StableTranscript,
        context: PersonaContext,
    ) -> EngineResult<StableTranscript> {
        // 应用个性化规则
        let personalized_text = self.apply_persona_rules(&transcript.text, &context);

        // 返回个性化后的 transcript
        Ok(StableTranscript {
            text: personalized_text,
            speaker_id: transcript.speaker_id,
            language: transcript.language,
        })
    }
}

