use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 说话者到语音的映射管理器
/// 
/// 用于在多人场景中，为每个说话者分配不同的 TTS 音色（voice）
/// 实现第二阶段目标：TTS 多说话者音色区分
pub struct SpeakerVoiceMapper {
    /// 用户 ID → Voice ID 映射
    mapping: Arc<RwLock<HashMap<String, String>>>,
    /// 可用 voice 列表（用于轮询分配）
    available_voices: Vec<String>,
    /// 下一个分配的 voice 索引（用于轮询）
    next_voice_index: Arc<RwLock<usize>>,
}

impl SpeakerVoiceMapper {
    /// 创建新的 SpeakerVoiceMapper
    /// 
    /// # Arguments
    /// * `available_voices` - 可用的 voice 列表（例如：["zh_CN-huayan-medium", "zh_CN-xiaoyan-medium", "en_US-lessac-medium"]）
    pub fn new(available_voices: Vec<String>) -> Self {
        if available_voices.is_empty() {
            panic!("SpeakerVoiceMapper: available_voices cannot be empty");
        }
        
        Self {
            mapping: Arc::new(RwLock::new(HashMap::new())),
            available_voices: available_voices.clone(),
            next_voice_index: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 从 voice 列表中查找男性声音
    /// 根据 voice 名称推断（包含 "male", "man", "zh_CN" 等可能是男性）
    fn find_male_voice(voices: &[String]) -> Option<String> {
        // 优先级：明确包含 "male" 或 "man" 的 > 中文男性名字 > 第一个
        for voice in voices {
            let lower = voice.to_lowercase();
            if lower.contains("male") || lower.contains("man") {
                return Some(voice.clone());
            }
        }
        // 检查中文男性名字（常见的中文男性 TTS 声音）
        for voice in voices {
            let lower = voice.to_lowercase();
            if lower.contains("huayan") || lower.contains("xiaoyi") || lower.contains("xiaofeng") {
                return Some(voice.clone());
            }
        }
        None
    }
    
    /// 从 voice 列表中查找女性声音
    /// 根据 voice 名称推断（包含 "female", "woman", "xiaoyan" 等可能是女性）
    fn find_female_voice(voices: &[String]) -> Option<String> {
        // 优先级：明确包含 "female" 或 "woman" 的 > 中文女性名字 > 第二个
        for voice in voices {
            let lower = voice.to_lowercase();
            if lower.contains("female") || lower.contains("woman") {
                return Some(voice.clone());
            }
        }
        // 检查中文女性名字（常见的中文女性 TTS 声音）
        for voice in voices {
            let lower = voice.to_lowercase();
            if lower.contains("xiaoyan") || lower.contains("xiaoxiao") || lower.contains("xiaomei") {
                return Some(voice.clone());
            }
        }
        None
    }
    
    /// 为新的用户分配 voice
    /// 
    /// 如果用户已有 voice，直接返回；否则使用轮询方式分配新的 voice
    /// 
    /// # Arguments
    /// * `speaker_id` - 用户 ID
    /// 
    /// # Returns
    /// 返回分配给该用户的 voice ID
    pub async fn assign_voice(&self, speaker_id: &str) -> String {
        let mut mapping = self.mapping.write().await;
        
        // 如果用户已有 voice，直接返回
        if let Some(voice) = mapping.get(speaker_id) {
            return voice.clone();
        }
        
        // 为新用户分配 voice（轮询方式）
        let mut next_index = self.next_voice_index.write().await;
        let voice_index = *next_index % self.available_voices.len();
        let voice = self.available_voices[voice_index].clone();
        
        // 更新索引（为下一个用户准备）
        *next_index = (*next_index + 1) % self.available_voices.len();
        
        // 保存映射
        mapping.insert(speaker_id.to_string(), voice.clone());
        
        voice
    }
    
    /// 获取用户的 voice
    /// 
    /// # Arguments
    /// * `speaker_id` - 用户 ID
    /// 
    /// # Returns
    /// 如果用户已有 voice，返回 Some(voice_id)；否则返回 None
    pub async fn get_voice(&self, speaker_id: &str) -> Option<String> {
        let mapping = self.mapping.read().await;
        mapping.get(speaker_id).cloned()
    }
    
    /// 获取或分配 voice（如果用户没有 voice，自动分配）
    /// 
    /// # Arguments
    /// * `speaker_id` - 用户 ID
    /// 
    /// # Returns
    /// 返回用户的 voice ID
    pub async fn get_or_assign_voice(&self, speaker_id: &str) -> String {
        // 检查是否是默认说话者 ID
        match speaker_id {
            "default_male" => {
                // 查找或分配男性声音
                if let Some(voice) = self.get_voice(speaker_id).await {
                    return voice;
                }
                let male_voice = Self::find_male_voice(&self.available_voices)
                    .unwrap_or_else(|| self.available_voices[0].clone());
                self.set_voice(speaker_id, male_voice.clone()).await;
                male_voice
            }
            "default_female" => {
                // 查找或分配女性声音
                if let Some(voice) = self.get_voice(speaker_id).await {
                    return voice;
                }
                let female_voice = Self::find_female_voice(&self.available_voices)
                    .unwrap_or_else(|| {
                        if self.available_voices.len() >= 2 {
                            self.available_voices[1].clone()
                        } else {
                            self.available_voices[0].clone()
                        }
                    });
                self.set_voice(speaker_id, female_voice.clone()).await;
                female_voice
            }
            "default_speaker" => {
                // 未知性别，使用男性声音作为默认
                if let Some(voice) = self.get_voice(speaker_id).await {
                    return voice;
                }
                let default_voice = Self::find_male_voice(&self.available_voices)
                    .unwrap_or_else(|| self.available_voices[0].clone());
                self.set_voice(speaker_id, default_voice.clone()).await;
                default_voice
            }
            _ => {
                // 普通说话者，使用原有逻辑
                if let Some(voice) = self.get_voice(speaker_id).await {
                    voice
                } else {
                    self.assign_voice(speaker_id).await
                }
            }
        }
    }
    
    /// 手动设置用户的 voice
    /// 
    /// # Arguments
    /// * `speaker_id` - 用户 ID
    /// * `voice_id` - Voice ID
    pub async fn set_voice(&self, speaker_id: &str, voice_id: String) {
        let mut mapping = self.mapping.write().await;
        mapping.insert(speaker_id.to_string(), voice_id);
    }
    
    /// 清除所有映射（用于新的会话）
    pub async fn clear(&self) {
        let mut mapping = self.mapping.write().await;
        let mut next_index = self.next_voice_index.write().await;
        mapping.clear();
        *next_index = 0;
    }
    
    /// 获取映射数量
    pub async fn count(&self) -> usize {
        let mapping = self.mapping.read().await;
        mapping.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_assign_voice() {
        let mapper = SpeakerVoiceMapper::new(vec![
            "voice1".to_string(),
            "voice2".to_string(),
            "voice3".to_string(),
        ]);
        
        // 为第一个用户分配 voice
        let voice1 = mapper.assign_voice("user1").await;
        assert_eq!(voice1, "voice1");
        
        // 为第二个用户分配 voice
        let voice2 = mapper.assign_voice("user2").await;
        assert_eq!(voice2, "voice2");
        
        // 第一个用户应该还是 voice1
        let voice1_again = mapper.get_voice("user1").await;
        assert_eq!(voice1_again, Some("voice1".to_string()));
    }
    
    #[tokio::test]
    async fn test_round_robin() {
        let mapper = SpeakerVoiceMapper::new(vec![
            "voice1".to_string(),
            "voice2".to_string(),
        ]);
        
        // 分配 4 个用户，应该轮询
        assert_eq!(mapper.assign_voice("user1").await, "voice1");
        assert_eq!(mapper.assign_voice("user2").await, "voice2");
        assert_eq!(mapper.assign_voice("user3").await, "voice1");
        assert_eq!(mapper.assign_voice("user4").await, "voice2");
    }
    
    #[tokio::test]
    async fn test_get_or_assign() {
        let mapper = SpeakerVoiceMapper::new(vec!["voice1".to_string()]);
        
        // 第一次调用应该分配
        let voice1 = mapper.get_or_assign_voice("user1").await;
        assert_eq!(voice1, "voice1");
        
        // 第二次调用应该返回已分配的
        let voice1_again = mapper.get_or_assign_voice("user1").await;
        assert_eq!(voice1_again, "voice1");
    }
    
    #[tokio::test]
    async fn test_clear() {
        let mapper = SpeakerVoiceMapper::new(vec!["voice1".to_string()]);
        
        mapper.assign_voice("user1").await;
        assert_eq!(mapper.count().await, 1);
        
        mapper.clear().await;
        assert_eq!(mapper.count().await, 0);
    }
}

