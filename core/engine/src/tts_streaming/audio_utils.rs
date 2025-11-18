use anyhow::Result;
use std::io::Write;
use std::fs::File;
use std::path::Path;

/// 将 PCM 16-bit 音频保存为 WAV 文件（用于测试和验证）
/// 
/// # Arguments
/// * `pcm_data` - PCM 16-bit 音频数据（小端字节序）
/// * `output_path` - 输出 WAV 文件路径
/// * `sample_rate` - 采样率（默认 16000 Hz）
/// * `channels` - 声道数（默认 1 = mono）
pub fn save_pcm_to_wav(
    pcm_data: &[u8],
    output_path: &Path,
    sample_rate: u32,
    channels: u16,
) -> Result<()> {
    let mut file = File::create(output_path)?;
    
    // WAV 文件头
    // RIFF header
    file.write_all(b"RIFF")?;
    let file_size = 36 + pcm_data.len() as u32;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    
    // fmt chunk
    file.write_all(b"fmt ")?;
    let fmt_chunk_size = 16u32;
    file.write_all(&fmt_chunk_size.to_le_bytes())?;
    let audio_format = 1u16; // PCM
    file.write_all(&audio_format.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate as u32 * channels as u32 * 2; // 16-bit = 2 bytes
    file.write_all(&byte_rate.to_le_bytes())?;
    let block_align = channels * 2; // 16-bit = 2 bytes
    file.write_all(&block_align.to_le_bytes())?;
    let bits_per_sample = 16u16;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    
    // data chunk
    file.write_all(b"data")?;
    let data_size = pcm_data.len() as u32;
    file.write_all(&data_size.to_le_bytes())?;
    file.write_all(pcm_data)?;
    
    Ok(())
}

/// 验证 PCM 音频数据格式
pub fn validate_pcm_audio(pcm_data: &[u8], expected_sample_rate: u32) -> Result<()> {
    // PCM 16-bit = 2 字节/样本
    if pcm_data.len() % 2 != 0 {
        return Err(anyhow::anyhow!("PCM data length must be even (16-bit = 2 bytes per sample)"));
    }
    
    let num_samples = pcm_data.len() / 2;
    let duration_sec = num_samples as f32 / expected_sample_rate as f32;
    
    if duration_sec < 0.01 {
        return Err(anyhow::anyhow!("Audio too short: {} seconds", duration_sec));
    }
    
    Ok(())
}

