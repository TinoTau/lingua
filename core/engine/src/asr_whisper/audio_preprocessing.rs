// core/engine/src/asr_whisper/audio_preprocessing.rs
// Whisper 音频预处理模块

use crate::types::AudioFrame;
use anyhow::{Result, anyhow};

/// Whisper 音频格式要求
pub const WHISPER_SAMPLE_RATE: u32 = 16000;
pub const WHISPER_N_MEL: usize = 80;

/// 将 AudioFrame 转换为 Whisper 输入格式（16kHz 单声道 PCM f32）
/// 
/// # Arguments
/// * `frame` - 输入的音频帧
/// 
/// # Returns
/// 返回预处理后的音频数据（Vec<f32>），采样率为 16kHz，单声道
pub fn preprocess_audio_frame(frame: &AudioFrame) -> Result<Vec<f32>> {
    let mut audio_data = frame.data.clone();
    
    // 1. 转换为单声道（如果需要）
    if frame.channels > 1 {
        audio_data = convert_to_mono(&audio_data, frame.channels as usize);
    }
    
    // 2. 重采样到 16kHz（如果需要）
    if frame.sample_rate != WHISPER_SAMPLE_RATE {
        audio_data = resample_audio(
            &audio_data,
            frame.sample_rate,
            WHISPER_SAMPLE_RATE,
        )?;
    }
    
    // 3. 归一化（确保在 [-1.0, 1.0] 范围内）
    normalize_audio(&mut audio_data);
    
    Ok(audio_data)
}

/// 将多声道音频转换为单声道（取平均值）
pub fn convert_to_mono(audio: &[f32], num_channels: usize) -> Vec<f32> {
    audio.chunks(num_channels)
        .map(|chunk| {
            chunk.iter().sum::<f32>() / num_channels as f32
        })
        .collect()
}

/// 简单的线性重采样（用于测试，生产环境建议使用专业库）
/// 
/// # Arguments
/// * `audio` - 输入音频数据
/// * `from_rate` - 源采样率
/// * `to_rate` - 目标采样率
/// 
/// # Returns
/// 重采样后的音频数据
pub fn resample_audio(audio: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
    if from_rate == to_rate {
        return Ok(audio.to_vec());
    }
    
    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = (audio.len() as f64 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let src_idx = (i as f64 / ratio) as usize;
        if src_idx < audio.len() {
            resampled.push(audio[src_idx]);
        } else {
            // 超出范围，使用最后一个样本或零填充
            resampled.push(*audio.last().unwrap_or(&0.0));
        }
    }
    
    Ok(resampled)
}

/// 归一化音频数据到 [-1.0, 1.0] 范围
/// 
/// # Arguments
/// * `audio` - 音频数据（原地修改）
pub fn normalize_audio(audio: &mut [f32]) {
    // 找到最大绝对值
    let max_abs = audio.iter()
        .map(|&x| x.abs())
        .fold(0.0f32, f32::max);
    
    // 如果最大值超过 1.0，进行归一化
    if max_abs > 1.0 {
        let scale = 1.0 / max_abs;
        for sample in audio.iter_mut() {
            *sample *= scale;
        }
    }
}

/// 从多个 AudioFrame 累积音频数据
/// 
/// # Arguments
/// * `frames` - 音频帧序列
/// 
/// # Returns
/// 累积后的预处理音频数据
pub fn accumulate_audio_frames(frames: &[AudioFrame]) -> Result<Vec<f32>> {
    if frames.is_empty() {
        return Ok(Vec::new());
    }
    
    // 检查所有帧的格式是否一致
    let first_frame = &frames[0];
    for frame in frames.iter().skip(1) {
        if frame.sample_rate != first_frame.sample_rate {
            return Err(anyhow!("Inconsistent sample rates: {} vs {}", 
                frame.sample_rate, first_frame.sample_rate));
        }
        if frame.channels != first_frame.channels {
            return Err(anyhow!("Inconsistent channels: {} vs {}", 
                frame.channels, first_frame.channels));
        }
    }
    
    // 累积所有帧的数据
    let mut accumulated: Vec<f32> = Vec::new();
    for frame in frames {
        let preprocessed = preprocess_audio_frame(frame)?;
        accumulated.extend_from_slice(&preprocessed);
    }
    
    Ok(accumulated)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_to_mono() {
        // 立体声：左右声道
        let stereo: Vec<f32> = vec![1.0, 0.0, 0.5, 0.5, 0.0, 1.0];
        let mono = convert_to_mono(&stereo, 2);
        assert_eq!(mono, vec![0.5, 0.5, 0.5]);
    }
    
    #[test]
    fn test_resample_audio() {
        // 从 8kHz 重采样到 16kHz（2倍）
        let audio_8k: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let audio_16k = resample_audio(&audio_8k, 8000, 16000).unwrap();
        assert_eq!(audio_16k.len(), 8);
    }
    
    #[test]
    fn test_normalize_audio() {
        let mut audio = vec![0.5, 1.0, 1.5, 2.0];
        normalize_audio(&mut audio);
        assert_eq!(audio, vec![0.25, 0.5, 0.75, 1.0]);
    }
    
    #[test]
    fn test_preprocess_audio_frame() {
        let frame = AudioFrame {
            sample_rate: 44100,
            channels: 2,
            data: vec![1.0, -1.0, 0.5, -0.5, 0.0, 0.0],
            timestamp_ms: 0,
        };
        
        let preprocessed = preprocess_audio_frame(&frame).unwrap();
        
        // 应该是单声道
        // 采样率应该是 16kHz（重采样后长度会变化）
        assert!(!preprocessed.is_empty());
        assert!(preprocessed.iter().all(|&x| x.abs() <= 1.0));
    }
}

