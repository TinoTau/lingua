//! TTS 音频增强模块
//! 
//! 用于改善增量播放的听感：fade in/out、停顿插入等

use crate::error::EngineResult;

/// 音频增强配置
#[derive(Debug, Clone)]
pub struct AudioEnhancementConfig {
    /// 是否启用 fade in/out
    pub enable_fade: bool,
    /// Fade 时长（毫秒）
    pub fade_duration_ms: u32,
    /// 是否在句末插入停顿
    pub enable_pause: bool,
    /// 句末停顿时长（毫秒）
    pub pause_duration_ms: u32,
    /// 采样率（用于计算样本数）
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
}

impl Default for AudioEnhancementConfig {
    fn default() -> Self {
        Self {
            enable_fade: true,
            fade_duration_ms: 20,  // 20ms fade
            enable_pause: true,
            pause_duration_ms: 100,  // 100ms 停顿
            sample_rate: 22050,  // Piper TTS 默认采样率
            channels: 1,
        }
    }
}

/// 音频增强器
pub struct AudioEnhancer {
    config: AudioEnhancementConfig,
}

impl AudioEnhancer {
    /// 创建新的音频增强器
    pub fn new(config: AudioEnhancementConfig) -> Self {
        Self { config }
    }

    /// 处理音频数据（添加 fade in/out 和停顿）
    /// 
    /// # Arguments
    /// * `audio_data` - WAV 格式的音频数据
    /// * `is_first` - 是否为第一段（决定是否添加 fade in）
    /// * `is_last` - 是否为最后一段（决定是否添加 fade out 和停顿）
    /// * `has_sentence_end` - 是否包含句子结束标点（决定是否添加停顿）
    pub fn enhance_audio(
        &self,
        audio_data: &[u8],
        is_first: bool,
        is_last: bool,
        has_sentence_end: bool,
    ) -> EngineResult<Vec<u8>> {
        if !self.config.enable_fade && !self.config.enable_pause {
            return Ok(audio_data.to_vec());
        }

        // 解析 WAV 文件
        let (mut samples, sample_rate, channels) = self.parse_wav(audio_data)?;
        
        // 应用 fade in/out
        if self.config.enable_fade {
            self.apply_fade(&mut samples, is_first, is_last, sample_rate)?;
        }

        // 添加停顿
        if self.config.enable_pause && (is_last || has_sentence_end) {
            self.add_pause(&mut samples, sample_rate, channels)?;
        }

        // 重新编码为 WAV
        self.encode_wav(&samples, sample_rate, channels)
    }

    /// 解析 WAV 文件，提取 PCM 样本
    fn parse_wav(&self, wav_data: &[u8]) -> EngineResult<(Vec<i16>, u32, u16)> {
        // 简单的 WAV 解析（假设标准 PCM WAV 格式）
        if wav_data.len() < 44 {
            return Err(crate::error::EngineError::new("Invalid WAV file: too short".to_string()));
        }

        // 检查 RIFF 头
        if &wav_data[0..4] != b"RIFF" {
            return Err(crate::error::EngineError::new("Invalid WAV file: missing RIFF header".to_string()));
        }

        // 检查 WAVE 标识
        if &wav_data[8..12] != b"WAVE" {
            return Err(crate::error::EngineError::new("Invalid WAV file: missing WAVE identifier".to_string()));
        }

        // 查找 fmt chunk
        let mut offset = 12;
        let mut sample_rate = 22050u32;
        let mut channels = 1u16;
        let mut bits_per_sample = 16u16;
        let mut data_offset = 0usize;

        while offset < wav_data.len() - 8 {
            let chunk_id = &wav_data[offset..offset + 4];
            let chunk_size = u32::from_le_bytes([
                wav_data[offset + 4],
                wav_data[offset + 5],
                wav_data[offset + 6],
                wav_data[offset + 7],
            ]) as usize;

            if chunk_id == b"fmt " {
                // 解析 fmt chunk
                let audio_format = u16::from_le_bytes([wav_data[offset + 8], wav_data[offset + 9]]);
                if audio_format != 1 {
                    return Err(crate::error::EngineError::new(
                        format!("Unsupported audio format: {}", audio_format)
                    ));
                }
                channels = u16::from_le_bytes([wav_data[offset + 10], wav_data[offset + 11]]);
                sample_rate = u32::from_le_bytes([
                    wav_data[offset + 12],
                    wav_data[offset + 13],
                    wav_data[offset + 14],
                    wav_data[offset + 15],
                ]);
                bits_per_sample = u16::from_le_bytes([wav_data[offset + 22], wav_data[offset + 23]]);
            } else if chunk_id == b"data" {
                data_offset = offset + 8;
                break;
            }

            offset += 8 + chunk_size;
        }

        if data_offset == 0 {
            return Err(crate::error::EngineError::new("Invalid WAV file: missing data chunk".to_string()));
        }

        // 读取 PCM 数据
        let pcm_data = &wav_data[data_offset..];
        let num_samples = pcm_data.len() / (bits_per_sample as usize / 8) / channels as usize;
        let mut samples = Vec::with_capacity(num_samples * channels as usize);

        if bits_per_sample == 16 {
            for i in 0..num_samples * channels as usize {
                let sample = i16::from_le_bytes([pcm_data[i * 2], pcm_data[i * 2 + 1]]);
                samples.push(sample);
            }
        } else {
            return Err(crate::error::EngineError::new(
                format!("Unsupported bits per sample: {}", bits_per_sample)
            ));
        }

        Ok((samples, sample_rate, channels))
    }

    /// 应用 fade in/out
    fn apply_fade(
        &self,
        samples: &mut [i16],
        is_first: bool,
        is_last: bool,
        sample_rate: u32,
    ) -> EngineResult<()> {
        if samples.is_empty() {
            return Ok(());
        }

        let fade_samples = (self.config.fade_duration_ms as f32 * sample_rate as f32 / 1000.0) as usize;
        let fade_samples = fade_samples.min(samples.len() / 2);

        // Fade in（第一段）
        if is_first && fade_samples > 0 {
            for i in 0..fade_samples.min(samples.len()) {
                let factor = i as f32 / fade_samples as f32;
                samples[i] = (samples[i] as f32 * factor) as i16;
            }
        }

        // Fade out（最后一段）
        if is_last && fade_samples > 0 {
            let start = samples.len().saturating_sub(fade_samples);
            for i in start..samples.len() {
                let factor = (samples.len() - i) as f32 / fade_samples as f32;
                samples[i] = (samples[i] as f32 * factor) as i16;
            }
        }

        Ok(())
    }

    /// 添加停顿（静音）
    fn add_pause(
        &self,
        samples: &mut Vec<i16>,
        sample_rate: u32,
        channels: u16,
    ) -> EngineResult<()> {
        let pause_samples = (self.config.pause_duration_ms as f32 * sample_rate as f32 / 1000.0) as usize;
        let pause_samples = pause_samples * channels as usize;
        
        // 添加静音样本
        samples.extend(vec![0i16; pause_samples]);
        
        Ok(())
    }

    /// 将 PCM 样本编码为 WAV 格式
    fn encode_wav(&self, samples: &[i16], sample_rate: u32, channels: u16) -> EngineResult<Vec<u8>> {
        let mut wav_data = Vec::new();
        
        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        let file_size = 36 + samples.len() * 2;
        wav_data.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav_data.extend_from_slice(b"WAVE");
        
        // fmt chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes());  // fmt chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes());   // audio format (PCM)
        wav_data.extend_from_slice(&channels.to_le_bytes());
        wav_data.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * channels as u32 * 2;
        wav_data.extend_from_slice(&byte_rate.to_le_bytes());
        wav_data.extend_from_slice(&(channels * 2).to_le_bytes());  // block align
        wav_data.extend_from_slice(&16u16.to_le_bytes());  // bits per sample
        
        // data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&(samples.len() * 2).to_le_bytes());
        
        // PCM data
        for &sample in samples {
            wav_data.extend_from_slice(&sample.to_le_bytes());
        }
        
        Ok(wav_data)
    }
}

impl Default for AudioEnhancer {
    fn default() -> Self {
        Self {
            config: AudioEnhancementConfig::default(),
        }
    }
}

