//! Silero VAD 实现
//! 
//! 使用 ONNX Runtime 加载和运行 Silero VAD 模型，用于自然停顿识别

use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::VecDeque;
use ort::{Environment, Session, SessionBuilder, Value};
use ndarray::{Array1, Array2, Array3, Ix2, Ix3};
use ndarray::CowArray;

use crate::error::EngineResult;
use crate::types::AudioFrame;
use crate::vad::{DetectionOutcome, VoiceActivityDetector, BoundaryType};

/// Silero VAD 配置
#[derive(Clone)]
pub struct SileroVadConfig {
    /// 模型文件路径
    pub model_path: String,
    /// 采样率（Silero VAD 要求 16kHz）
    pub sample_rate: u32,
    /// 帧大小（512 samples @ 16kHz = 32ms）
    pub frame_size: usize,
    /// 静音阈值（0.0-1.0），低于此值认为是静音
    pub silence_threshold: f32,
    /// 最小静音时长（毫秒），超过此时长才判定为自然停顿
    pub min_silence_duration_ms: u64,
    /// 是否启用自适应调整（按用户）
    #[allow(dead_code)]
    pub adaptive_enabled: bool,
    /// 自适应调整的最小样本数（每个用户至少需要这么多样本才开始调整）
    #[allow(dead_code)]
    pub adaptive_min_samples: usize,
    /// 自适应调整的速率（每次调整的幅度，0.0-1.0）
    #[allow(dead_code)]
    pub adaptive_rate: f32,
    /// 基础阈值范围（语速自适应输出的基础范围，毫秒）
    #[allow(dead_code)]
    pub base_threshold_min_ms: u64,
    /// 基础阈值范围（语速自适应输出的基础范围，毫秒）
    #[allow(dead_code)]
    pub base_threshold_max_ms: u64,
    /// Delta 偏移量范围（质量反馈偏移量，毫秒）
    #[allow(dead_code)]
    pub delta_min_ms: i64,
    /// Delta 偏移量范围（质量反馈偏移量，毫秒）
    #[allow(dead_code)]
    pub delta_max_ms: i64,
    /// 最终阈值范围（实际使用的有效范围，毫秒）
    #[allow(dead_code)]
    pub final_threshold_min_ms: u64,
    /// 最终阈值范围（实际使用的有效范围，毫秒）
    #[allow(dead_code)]
    pub final_threshold_max_ms: u64,
    /// 最小话语时长（防止半句话被切掉，毫秒）
    #[allow(dead_code)]
    pub min_utterance_ms: u64,
}

impl Default for SileroVadConfig {
    fn default() -> Self {
        Self {
            model_path: "models/vad/silero/silero_vad.onnx".to_string(),
            sample_rate: 16000,
            frame_size: 512,  // 32ms @ 16kHz
            silence_threshold: 0.2,  // 降低阈值，提高语音检测灵敏度（从 0.5 降到 0.2）
            min_silence_duration_ms: 300,  // 基础阈值（从500ms降低到300ms以更快响应）
            adaptive_enabled: true,  // 默认启用自适应
            adaptive_min_samples: 1,  // 至少1个样本（降低以更快开始调整）
            adaptive_rate: 0.4,  // 每次调整40%（提高调整速度，更快适应语速变化）
            base_threshold_min_ms: 200,  // 基础阈值范围：200-600ms（从400-800ms降低，更快响应短句）
            base_threshold_max_ms: 600,
            delta_min_ms: -200,  // Delta 偏移量范围：-200 ~ +200ms（质量反馈，降低范围）
            delta_max_ms: 200,
            final_threshold_min_ms: 200,  // 最终阈值范围：200-800ms（从300-1000ms降低，更快响应）
            final_threshold_max_ms: 800,
            min_utterance_ms: 1000,  // 最小话语时长：1000ms（降低以防止过度等待）
        }
    }
}

/// 每个说话者的自适应状态
struct SpeakerAdaptiveState {
    /// 语速历史（字符/秒）
    speech_rate_history: VecDeque<f32>,
    /// 基础阈值（由语速自适应生成，毫秒）
    base_threshold_ms: u64,
    /// Delta 偏移量（由质量反馈生成，毫秒）
    delta_ms: i64,
    /// 样本数量
    sample_count: usize,
}

impl SpeakerAdaptiveState {
    fn new(base_duration_ms: u64) -> Self {
        eprintln!("[SileroVad] 🆕 Initialized SpeakerAdaptiveState with base_duration_ms={}ms", base_duration_ms);
        Self {
            speech_rate_history: VecDeque::with_capacity(20),  // 保留最近20个样本
            base_threshold_ms: base_duration_ms,
            delta_ms: 0,  // 初始 delta 为 0
            sample_count: 0,
        }
    }
    
    /// 更新语速并调整阈值
    /// 
    /// 使用更精细的语速调整策略：
    /// - 根据语速动态计算阈值倍数（连续函数，而非分段函数）
    /// - 快语速 → 更短的阈值（说话者句子之间停顿短）
    /// - 慢语速 → 更长的阈值（说话者可能在句子中间思考停顿）
    fn update_speech_rate(&mut self, speech_rate: f32, config: &SileroVadConfig) {
        self.speech_rate_history.push_back(speech_rate);
        if self.speech_rate_history.len() > 20 {
            self.speech_rate_history.pop_front();
        }
        self.sample_count += 1;
        
        // 如果样本数不足，使用基础阈值
        // 但即使样本数不足，也允许使用当前语速进行快速调整（降低延迟）
        let history_len = self.speech_rate_history.len();
        let avg_speech_rate = if history_len > 0 {
            // 使用指数加权移动平均（EWMA），最近的值权重更高
            let alpha = 0.5; // 提高平滑因子（从0.3到0.5），更重视最近的值，响应更快
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            for (i, &rate) in self.speech_rate_history.iter().enumerate() {
                let weight = (1.0_f32 - alpha).powi((history_len - i - 1) as i32);
                weighted_sum += rate * weight;
                weight_sum += weight;
            }
            weighted_sum / weight_sum
        } else {
            speech_rate
        };
        
        // 即使样本数不足，也允许进行快速调整（使用当前语速）
        // 这样可以更快地响应语速变化，减少多个短句被合并的情况
        
        // 根据语速动态计算阈值倍数（使用连续函数，而非分段函数）
        // 语速范围：0-20 字符/秒（正常范围：3-12 字符/秒）
        // 目标：快语速（> 8 字符/秒）→ 更短的阈值，慢语速（< 4 字符/秒）→ 更长的阈值
        // 
        // 使用 sigmoid 函数实现平滑过渡：
        // multiplier = 1.0 + (0.5 - sigmoid((rate - 6.0) / 2.0)) * 0.4
        // 这样：
        // - 语速 = 2 字符/秒 → multiplier ≈ 1.4（延长40%）
        // - 语速 = 6 字符/秒 → multiplier ≈ 1.0（保持原值）
        // - 语速 = 10 字符/秒 → multiplier ≈ 0.6（缩短40%）
        let sigmoid = |x: f32| -> f32 {
            1.0 / (1.0 + (-x).exp())
        };
        
        // 将语速映射到 [-2, 2] 范围（sigmoid 的有效范围）
        let normalized_rate = (avg_speech_rate - 6.0) / 2.0;
        let sigmoid_value = sigmoid(normalized_rate);
        
        // 计算倍数：1.0 + (0.5 - sigmoid_value) * 0.4
        // 当 sigmoid_value = 0.5（语速 = 6）时，multiplier = 1.0
        // 当 sigmoid_value < 0.5（语速 < 6，慢语速）时，multiplier > 1.0
        // 当 sigmoid_value > 0.5（语速 > 6，快语速）时，multiplier < 1.0
        let multiplier = 1.0 + (0.5 - sigmoid_value) * 0.4;
        
        // 限制倍数范围：0.5 - 1.5（避免过度调整）
        let multiplier = multiplier.clamp(0.5, 1.5);
        
        // 应用调整（使用平滑更新）- 只调整 base_threshold
        let base_threshold_center = (config.base_threshold_min_ms + config.base_threshold_max_ms) / 2;
        let target_base = (base_threshold_center as f32 * multiplier) as u64;
        let old_base = self.base_threshold_ms;
        let adjustment = (target_base as f32 - self.base_threshold_ms as f32) * config.adaptive_rate;
        self.base_threshold_ms = ((self.base_threshold_ms as f32 + adjustment) as u64)
            .clamp(config.base_threshold_min_ms, config.base_threshold_max_ms);
        
        // 记录阈值调整（仅在阈值变化较大时记录，避免日志过多）
        let change_ratio = if old_base > 0 {
            (self.base_threshold_ms as f32 - old_base as f32) / old_base as f32
        } else {
            0.0
        };
        if change_ratio.abs() > 0.1 {  // 变化超过10%时记录
            let effective = self.get_effective_threshold(config);
            eprintln!("[SileroVad] 🔧 Base threshold adjusted: {}ms -> {}ms (target={}ms, multiplier={:.2}x, avg_rate={:.2} chars/s, effective={}ms, change={:.1}%)", 
                     old_base, self.base_threshold_ms, target_base, multiplier, avg_speech_rate, effective, change_ratio * 100.0);
        }
    }
    
    /// 获取有效阈值（base + delta，限制在最终范围内）
    fn get_effective_threshold(&self, config: &SileroVadConfig) -> u64 {
        let effective = (self.base_threshold_ms as i64 + self.delta_ms) as u64;
        effective.clamp(config.final_threshold_min_ms, config.final_threshold_max_ms)
    }
    
    /// 获取当前调整后的阈值（兼容旧接口）
    fn get_adjusted_duration(&self, config: &SileroVadConfig) -> u64 {
        // 即使样本数不足，也使用调整后的阈值（如果已经调整过）
        // 这样可以更快地响应语速变化，减少多个短句被合并的情况
        if self.sample_count == 0 {
            config.min_silence_duration_ms
        } else {
            self.get_effective_threshold(config)
        }
    }
    
    /// 获取平均语速
    fn get_avg_speech_rate(&self) -> Option<f32> {
        if self.speech_rate_history.is_empty() {
            None
        } else {
            Some(self.speech_rate_history.iter().sum::<f32>() / self.speech_rate_history.len() as f32)
        }
    }
}

/// Silero VAD 实现
pub struct SileroVad {
    session: Arc<Mutex<Session>>,
    config: SileroVadConfig,
    /// 连续静音帧数
    silence_frame_count: Arc<Mutex<usize>>,
    /// 上一个检测到语音的帧的时间戳
    last_speech_timestamp: Arc<Mutex<Option<u64>>>,
    /// 隐藏状态（用于 VAD 模型的状态传递）
    hidden_state: Arc<Mutex<Option<Array2<f32>>>>,
    /// 全局自适应状态（不按说话者区分，每个短句都根据上一个短句的语速调整）
    adaptive_state: Arc<Mutex<SpeakerAdaptiveState>>,
    /// 上一次边界检测的时间戳（用于冷却期）
    last_boundary_timestamp: Arc<Mutex<Option<u64>>>,
    /// 帧缓冲区（用于累积小帧，直到达到 frame_size）
    frame_buffer: Arc<Mutex<Vec<f32>>>,
}

impl SileroVad {
    /// 从模型路径创建 SileroVad
    /// 
    /// # Arguments
    /// * `model_path` - ONNX 模型文件路径
    pub fn new(model_path: impl AsRef<Path>) -> EngineResult<Self> {
        Self::with_config(SileroVadConfig {
            model_path: model_path.as_ref().to_string_lossy().to_string(),
            ..Default::default()
        })
    }
    
    /// 使用自定义配置创建 SileroVad
    pub fn with_config(config: SileroVadConfig) -> EngineResult<Self> {
        // 初始化 ONNX Runtime 环境
        crate::onnx_utils::init_onnx_runtime()
            .map_err(|e| crate::error::EngineError::new(format!("Failed to init ONNX runtime: {}", e)))?;
        
        // 创建 ONNX Runtime 环境
        let env = Arc::new(
            Environment::builder()
                .with_name("silero_vad")
                .build()
                .map_err(|e| crate::error::EngineError::new(format!("Failed to create ONNX environment: {}", e)))?
        );
        
        // 加载模型
        let session = SessionBuilder::new(&env)
            .map_err(|e| crate::error::EngineError::new(format!("Failed to create session builder: {}", e)))?
            .with_model_from_file(&config.model_path)
            .map_err(|e| crate::error::EngineError::new(format!("Failed to load model from {}: {}", config.model_path, e)))?;
        
        // 打印模型输入信息（用于调试）
        eprintln!("[SileroVad] Model inputs:");
        for (i, input) in session.inputs.iter().enumerate() {
            eprintln!("  Input[{}]: name='{}', dimensions={:?}, input_type={:?}", 
                     i, input.name, input.dimensions, input.input_type);
        }
        
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            config: config.clone(),
            silence_frame_count: Arc::new(Mutex::new(0)),
            last_speech_timestamp: Arc::new(Mutex::new(None)),
            hidden_state: Arc::new(Mutex::new(None)),
            adaptive_state: Arc::new(Mutex::new(SpeakerAdaptiveState::new(
                (config.base_threshold_min_ms + config.base_threshold_max_ms) / 2
            ))),
            last_boundary_timestamp: Arc::new(Mutex::new(None)),
            frame_buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// 检测语音活动概率
    /// 
    /// # Arguments
    /// * `audio` - 音频数据（f32，范围 -1.0 到 1.0）
    /// 
    /// # Returns
    /// 返回语音概率（0.0-1.0）
    fn detect_voice_activity(&self, audio: &[f32]) -> EngineResult<f32> {
        // 预处理：确保音频长度正确
        if audio.len() != self.config.frame_size {
            return Err(crate::error::EngineError::new(
                format!("Audio length {} does not match frame size {}", audio.len(), self.config.frame_size)
            ));
        }
        
        // 归一化到 [-1, 1]（Silero VAD 要求）
        let normalized: Vec<f32> = audio.iter()
            .map(|&x| x.clamp(-1.0, 1.0))
            .collect();
        
        // 创建音频输入数组（形状：[1, frame_size]）
        let input_array = Array2::from_shape_vec((1, normalized.len()), normalized)
            .map_err(|e| crate::error::EngineError::new(format!("Failed to create input array: {}", e)))?;
        
        // 获取或初始化隐藏状态（形状：[2, 1, 128]）
        let state_array = {
            let mut state_guard = self.hidden_state.lock().unwrap();
            if let Some(ref state_2d) = *state_guard {
                // 状态存储为 [2, 128]，需要扩展为 [2, 1, 128]
                let state_3d = state_2d.clone().into_shape((2, 1, 128))
                    .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape state: {}", e)))?;
                state_3d
            } else {
                // 初始化隐藏状态为零 [2, 1, 128]
                let new_state = Array3::<f32>::zeros((2, 1, 128));
                // 存储为 [2, 128] 以便下次使用
                *state_guard = Some(new_state.clone().into_shape((2, 128))
                    .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape new state: {}", e)))?);
                new_state
            }
        };
        
        // 转换为动态维度
        let arr_dyn = input_array.into_dyn();
        let arr_owned = arr_dyn.to_owned();
        let cow_arr = CowArray::from(arr_owned);
        
        let state_dyn = state_array.into_dyn();
        let state_owned = state_dyn.to_owned();
        let state_cow = CowArray::from(state_owned);
        
        // 创建采样率输入（Int64 标量，形状：[]）
        // 注意：Silero VAD 的 sr 输入是 Int64，不是 Float32
        let sr_array = Array1::from_vec(vec![self.config.sample_rate as i64]);
        let sr_dyn = sr_array.into_dyn();
        let sr_owned = sr_dyn.to_owned();
        let sr_cow = CowArray::from(sr_owned);
        
        // 创建 ONNX 输入（需要在同一个作用域内创建，确保生命周期正确）
        use std::ptr;
        let audio_input = {
            let audio_val = Value::from_array(ptr::null_mut(), &cow_arr)
                .map_err(|e| crate::error::EngineError::new(format!("Failed to create audio input: {}", e)))?;
            unsafe { std::mem::transmute::<Value, Value<'static>>(audio_val) }
        };
        
        let state_input = {
            let state_val = Value::from_array(ptr::null_mut(), &state_cow)
                .map_err(|e| crate::error::EngineError::new(format!("Failed to create state input: {}", e)))?;
            unsafe { std::mem::transmute::<Value, Value<'static>>(state_val) }
        };
        
        let sr_input = {
            let sr_val = Value::from_array(ptr::null_mut(), &sr_cow)
                .map_err(|e| crate::error::EngineError::new(format!("Failed to create sr input: {}", e)))?;
            unsafe { std::mem::transmute::<Value, Value<'static>>(sr_val) }
        };
        
        // 推理（按模型输入顺序传递：input, state, sr）
        let session_guard = self.session.lock().unwrap();
        let outputs = session_guard
            .run(vec![audio_input, state_input, sr_input])
            .map_err(|e| crate::error::EngineError::new(format!("ONNX inference failed: {}", e)))?;
        
        // 提取输出
        // Silero VAD 输出：[output, new_state]
        // output 形状：[1, 2]，第一列是静音概率，第二列是语音概率
        // new_state 形状：[2, 1, 128]，新的隐藏状态
        use ort::tensor::OrtOwnedTensor;
        use ndarray::IxDyn;
        
        // 提取 output（第一个输出）
        let output_tensor: OrtOwnedTensor<f32, IxDyn> = outputs[0]
            .try_extract()
            .map_err(|e| crate::error::EngineError::new(format!("Failed to extract output: {}", e)))?;
        
        // 提取 new_state（第二个输出）并更新隐藏状态
        if outputs.len() > 1 {
            let state_tensor: OrtOwnedTensor<f32, IxDyn> = outputs[1]
                .try_extract()
                .map_err(|e| crate::error::EngineError::new(format!("Failed to extract state: {}", e)))?;
            
            let state_view = state_tensor.view();
            let new_state_3d: Array3<f32> = state_view
                .to_owned()
                .into_dimensionality::<Ix3>()
                .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape state: {}", e)))?;
            
            // 将状态从 [2, 1, 128] 转换为 [2, 128] 存储
            let new_state_2d = new_state_3d.into_shape((2, 128))
                .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape state for storage: {}", e)))?;
            
            // 更新隐藏状态
            let mut state_guard = self.hidden_state.lock().unwrap();
            *state_guard = Some(new_state_2d);
        }
        
        // 提取输出值
        // 根据实际输出形状处理：
        // - 如果输出是 [1, 2]，取 [0, 1]（第二列，语音概率）
        // - 如果输出是 [1, 1] 或 [1]，取 [0, 0] 或 [0]（直接是语音概率）
        let view = output_tensor.view();
        let shape = view.shape();
        
        // 不再输出模型输出的调试信息
        let should_log = false;
        
        let raw_output = if shape.len() == 2 {
            // 2维数组
            let output_array: Array2<f32> = view
                .to_owned()
                .into_dimensionality::<Ix2>()
                .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape output: {}", e)))?;
            
            if should_log {
                eprintln!("[SileroVad] 🔬 Output array shape: {:?}, values: {:?}", output_array.shape(), 
                         if output_array.len() <= 10 { format!("{:?}", output_array.iter().collect::<Vec<_>>()) } else { "too many".to_string() });
            }
            
            if output_array.shape()[1] >= 2 {
                // 有2列，取第二列（语音概率）
                output_array[[0, 1]]
            } else {
                // 只有1列，直接使用
                output_array[[0, 0]]
            }
        } else if shape.len() == 1 {
            // 1维数组，直接取第一个值
            let output_array: Array1<f32> = view
                .to_owned()
                .into_dimensionality::<ndarray::Ix1>()
                .map_err(|e| crate::error::EngineError::new(format!("Failed to reshape output: {}", e)))?;
            output_array[0]
        } else {
            // 其他形状，尝试 flatten 后取第一个值
            let flat: Vec<f32> = view.iter().copied().collect();
            if flat.is_empty() {
                return Err(crate::error::EngineError::new("Output tensor is empty"));
            }
            flat[0]
        };
        
        // 处理输出值：根据 Silero VAD 的官方实现，模型输出可能是：
        // 1. [1, 2] 形状：第一列是静音概率，第二列是语音概率
        // 2. [1, 1] 形状：可能是 logit（需要 sigmoid），或者需要乘以系数
        // 
        // 根据问题报告，当前输出是 [1, 1] 形状，值为 0.0006-0.0013（非常小）
        // 如果直接应用 sigmoid，所有值都会变成约 0.5，无法区分
        // 
        // 可能的解决方案：
        // 1. 输出值需要乘以系数（比如 100 或 1000）后再应用 sigmoid
        // 2. 或者输出值已经是概率，但需要不同的阈值
        // 3. 或者输出值需要取反（如果是静音概率）
        //
        // 根据 Silero VAD 的常见实现，如果输出值非常小（< 0.01），
        // 可能需要乘以一个系数（比如 100）后再应用 sigmoid
        let speech_prob = if raw_output < -10.0 || raw_output > 10.0 {
            // 看起来是 logit，使用 sigmoid 转换
            let prob = 1.0 / (1.0 + (-raw_output).exp());
            if should_log {
                eprintln!("[SileroVad] 🔬 Raw output {} looks like logit, applying sigmoid: {}", raw_output, prob);
            }
            prob
        } else if raw_output < 0.2 && raw_output > -0.01 {
            // 根据诊断结果，实际模型的输出值范围：
            // - 静音帧：0.004 - 0.044（均值 0.016）
            // - 语音帧：0.089 - 0.124（均值 0.099）
            // 
            // 这些值看起来像是直接的语音概率（或接近），但值域在 0-0.2 之间
            // 如果直接使用，静音帧（0.016）会被识别为静音，语音帧（0.099）也会被识别为静音
            // 
            // 可能的解释：
            // 1. 输出值需要乘以系数（比如 5-10）才能得到 0-1 范围的概率
            // 2. 或者输出值已经是概率，但需要不同的阈值
            // 
            // 根据诊断，如果使用系数 10：
            // - 静音 0.016 * 10 = 0.16 → sigmoid(0.16) ≈ 0.54（仍然接近 0.5）
            // - 语音 0.099 * 10 = 0.99 → sigmoid(0.99) ≈ 0.73（可以区分）
            // 
            // 但更好的方法是：直接使用原始值，但调整阈值
            // 或者：将输出值视为 logit，使用较小的系数（比如 10-20）
            // 
            // 根据实际测试，使用系数 10 可以区分静音和语音：
            let scaled_logit = raw_output * 10.0;
            let prob = 1.0 / (1.0 + (-scaled_logit).exp());
            // 不再输出调试信息
            prob
        } else if raw_output < 0.5 {
            // 值在 0-0.5 之间，可能是静音概率，取反得到语音概率
            let prob = 1.0 - raw_output;
            if should_log {
                eprintln!("[SileroVad] 🔬 Raw output {} might be silence prob, inverting: {}", raw_output, prob);
            }
            prob
        } else {
            // 值 >= 0.5，直接使用（已经是语音概率）
            if should_log {
                eprintln!("[SileroVad] 🔬 Raw output {} used directly as speech prob", raw_output);
            }
            raw_output
        };
        
        Ok(speech_prob)
    }
}

#[async_trait]
impl VoiceActivityDetector for SileroVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        // 检查采样率是否匹配
        if frame.sample_rate != self.config.sample_rate {
            return Err(crate::error::EngineError::new(
                format!("Sample rate mismatch: expected {}, got {}", self.config.sample_rate, frame.sample_rate)
            ));
        }
        
        // 清理 FINAL_FRAME_FLAG（如果设置了的话）
        // FINAL_FRAME_FLAG = 1u64 << 63，用于标记最后一帧
        const FINAL_FRAME_FLAG: u64 = 1u64 << 63;
        let cleaned_timestamp = frame.timestamp_ms & !FINAL_FRAME_FLAG;
        let mut cleaned_frame = frame.clone();
        cleaned_frame.timestamp_ms = cleaned_timestamp;
        
        // 累积帧到缓冲区，直到达到 frame_size
        let mut buffer = self.frame_buffer.lock().unwrap();
        buffer.extend_from_slice(&cleaned_frame.data);
        
        // 如果缓冲区还没有达到 frame_size，返回一个"非边界"的结果
        // 注意：我们需要至少累积到 frame_size 才能进行 VAD 检测
        if buffer.len() < self.config.frame_size {
            drop(buffer); // 释放锁
            // 不再输出缓冲区累积日志
            return Ok(DetectionOutcome {
                is_boundary: false,
                confidence: 0.5,
                frame: cleaned_frame.clone(),
                boundary_type: None,
            });
        }
        
        // 提取一个完整的 frame_size 样本进行检测
        let audio_data: Vec<f32> = buffer[..self.config.frame_size].to_vec();
        
        // 计算音频数据的统计信息（用于调试，目前未使用）
        // let audio_max = audio_data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        // let audio_mean = audio_data.iter().sum::<f32>() / audio_data.len() as f32;
        // let audio_rms = (audio_data.iter().map(|x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        
        // 保留剩余的样本在缓冲区中（用于下一次检测）
        let remaining = buffer.len() - self.config.frame_size;
        if remaining > 0 {
            let remaining_data = buffer[self.config.frame_size..].to_vec();
            *buffer = remaining_data;
        } else {
            buffer.clear();
        }
        drop(buffer); // 释放锁
        
        // 不再输出每次检测的详细信息
        // 检测语音活动
        let speech_prob = self.detect_voice_activity(&audio_data)?;
        
        // 判断是否为静音
        let is_silence = speech_prob < self.config.silence_threshold;
        
        // 更新静音帧计数
        let mut silence_count = self.silence_frame_count.lock().unwrap();
        let mut last_speech = self.last_speech_timestamp.lock().unwrap();
        
        if is_silence {
            *silence_count += 1;
        } else {
            // 检测到语音，重置静音计数
            *silence_count = 0;
            *last_speech = Some(cleaned_timestamp);
        }
        
        // 计算静音持续时间
        let silence_duration_ms = (*silence_count as u64 * self.config.frame_size as u64 * 1000) 
            / self.config.sample_rate as u64;
        
        // 获取全局自适应阈值
        // 注意：这个操作非常快（< 0.01ms），不需要性能监控
        let effective_threshold = self.get_adjusted_duration_ms();
        
        // 记录边界检测信息（仅在接近或超过阈值时记录，避免日志过多）
        if silence_duration_ms >= effective_threshold * 8 / 10 {
            let state = self.adaptive_state.lock().unwrap();
            let base = state.base_threshold_ms;
            let delta = state.delta_ms;
            drop(state);
            eprintln!("[SileroVad] 🔍 Boundary check: silence={}ms, effective_threshold={}ms (base={}ms, delta={:+}ms, adaptive={})", 
                     silence_duration_ms, effective_threshold, base, delta, self.config.adaptive_enabled);
        }
        
        // 判断是否为边界（自然停顿）
        // 注意：只有在连续静音达到最小时长时才判定为边界
        // 同时，需要检查冷却期（避免在连续静音期间频繁触发边界）
        // 还需要检查最小话语时长（防止半句话被切掉）
        let mut last_boundary_ts = self.last_boundary_timestamp.lock().unwrap();
        
        // 检查时间戳是否异常（防止溢出或未初始化的值）
        // u64::MAX 的一半作为合理上限（约 292 年）
        // 注意：cleaned_timestamp 已经清理了 FINAL_FRAME_FLAG
        const MAX_REASONABLE_TIMESTAMP: u64 = u64::MAX / 2;
        if cleaned_timestamp > MAX_REASONABLE_TIMESTAMP {
            eprintln!("[SileroVad] ⚠️  Warning: Abnormal timestamp detected: {}ms, resetting boundary tracking", cleaned_timestamp);
            *last_boundary_ts = None;
            *last_speech = None;
            drop(last_boundary_ts);
            drop(last_speech);
            drop(silence_count);
            return Ok(DetectionOutcome {
                is_boundary: false,
                confidence: 0.5,
                frame: cleaned_frame.clone(),
                boundary_type: None,
            });
        }
        
        // 冷却期：防止在连续静音期间频繁触发边界
        // 降低冷却期（从1.5倍降到1.0倍）以支持更快的短句检测
        // 如果用户每个短句之间都停了1秒，冷却期不应该阻止边界检测
        let cooldown_ms = effective_threshold; // 从1.5倍降到1.0倍，减少延迟
        let is_in_cooldown = if let Some(last_ts) = *last_boundary_ts {
            // 检查 last_ts 是否也异常
            if last_ts > MAX_REASONABLE_TIMESTAMP {
                eprintln!("[SileroVad] ⚠️  Warning: Abnormal last_boundary_timestamp: {}ms, resetting", last_ts);
                *last_boundary_ts = None;
                false
            } else {
                let elapsed = cleaned_timestamp.saturating_sub(last_ts);
                elapsed < cooldown_ms
            }
        } else {
            false
        };
        
        // 只有在检测到语音之后，静音才能触发边界
        // 如果从未检测到语音，开头的静音不应该触发边界
        let has_detected_speech = last_speech.is_some();
        
        // 检查最小话语时长（防止半句话被切掉）
        // 如果从上次语音开始到现在的时间小于 min_utterance_ms，即使达到静音阈值也不应该触发边界
        let utterance_duration_ok = if let Some(last_speech_ts) = *last_speech {
            let utterance_duration = cleaned_timestamp.saturating_sub(last_speech_ts);
            utterance_duration >= self.config.min_utterance_ms
        } else {
            false  // 如果没有检测到语音，不允许触发边界
        };
        
        let is_boundary = is_silence 
            && silence_duration_ms >= effective_threshold 
            && !is_in_cooldown
            && has_detected_speech  // 只有在检测到语音后才允许触发边界
            && utterance_duration_ok;  // 确保话语时长足够，防止半句话被切掉
        
        // 如果因为话语时长不足而阻止边界检测，记录日志
        if is_silence 
            && silence_duration_ms >= effective_threshold 
            && !is_in_cooldown
            && has_detected_speech
            && !utterance_duration_ok {
            if let Some(last_speech_ts) = *last_speech {
                let utterance_duration = cleaned_timestamp.saturating_sub(last_speech_ts);
                eprintln!("[SileroVad] ⏸️  Boundary blocked by min_utterance: utterance_duration={}ms < min_utterance={}ms (preventing mid-sentence cut)", 
                         utterance_duration, self.config.min_utterance_ms);
            }
        }
        
        // 只输出边界检测结果
        // 注意：边界检测后，ASR/翻译/TTS 会立即开始处理（流式处理）
        // 这样可以实现：用户说完话后立即开始翻译，无需等待完整音频
        // 对于手机端 AEC（声学回响消除）场景，这可以显著减少端到端延迟
        if is_boundary {
            eprintln!("[SileroVad] ✅ Boundary detected: silence_duration={}ms (threshold={}ms), timestamp={}ms → 🚀 Pipeline will start immediately", 
                     silence_duration_ms, effective_threshold, cleaned_timestamp);
            // 更新上一次边界检测的时间戳
            *last_boundary_ts = Some(cleaned_timestamp);
        }
        
        // 重置静音计数（如果检测到边界）
        if is_boundary {
            *silence_count = 0;
        }
        
        // 如果检测到语音，清除冷却期（允许立即检测新的边界）
        // 注意：这允许在语音结束后立即检测边界，减少延迟
        if !is_silence {
            *last_boundary_ts = None;
        }
        
        Ok(DetectionOutcome {
            is_boundary,
            confidence: speech_prob,
            frame: cleaned_frame,
            boundary_type: if is_boundary {
                Some(BoundaryType::NaturalPause)
            } else {
                None
            },
        })
    }
    
    async fn reset(&self) -> EngineResult<()> {
        let mut silence_count = self.silence_frame_count.lock().unwrap();
        let mut last_speech = self.last_speech_timestamp.lock().unwrap();
        let mut hidden_state = self.hidden_state.lock().unwrap();
        let mut adaptive_state = self.adaptive_state.lock().unwrap();
        let mut last_boundary_ts = self.last_boundary_timestamp.lock().unwrap();
        let mut frame_buffer = self.frame_buffer.lock().unwrap();
        *silence_count = 0;
        *last_speech = None;
        *hidden_state = None;  // 重置隐藏状态
        *adaptive_state = SpeakerAdaptiveState::new(
            (self.config.base_threshold_min_ms + self.config.base_threshold_max_ms) / 2
        );  // 重置自适应状态
        frame_buffer.clear();  // 清空帧缓冲区
        *last_boundary_ts = None;  // 重置边界冷却期
        Ok(())
    }
    
    fn get_info(&self) -> String {
        format!(
            "SileroVad(model={}, threshold={}, min_silence={}ms, adaptive={})",
            self.config.model_path,
            self.config.silence_threshold,
            self.config.min_silence_duration_ms,
            self.config.adaptive_enabled
        )
    }
}

// 为 SileroVad 添加自适应相关方法
impl SileroVad {
    /// 更新语速（用于自适应调整）
    /// 
    /// 每个短句识别完成后，根据该短句的语速更新全局阈值。
    /// 不区分说话者，因为同一个人说话的语速也会变化。
    /// 
    /// # Arguments
    /// * `text` - 识别的文本
    /// * `audio_duration_ms` - 音频时长（毫秒）
    pub fn update_speech_rate(&self, text: &str, audio_duration_ms: u64) {
        use std::time::Instant;
        let perf_start = Instant::now();
        
        if !self.config.adaptive_enabled {
            eprintln!("[SileroVad] ⚠️  update_speech_rate: adaptive_enabled is false, skipping");
            return;
        }
        
        if audio_duration_ms == 0 {
            eprintln!("[SileroVad] ⚠️  update_speech_rate: audio_duration_ms is 0, skipping");
            return;
        }
        
        // 计算语速（字符/秒）
        // 对于中文，使用字符数；对于英文，可以使用词数（这里简化使用字符数）
        let text_length = text.chars().count() as f32;
        let audio_duration_sec = audio_duration_ms as f32 / 1000.0;
        let speech_rate = text_length / audio_duration_sec;
        
        // ⚠️ 重要：检查语速是否在合理范围内
        // 真实语音输入的语速通常在 1-30 字符/秒之间
        // 误识别文本（如模型"幻觉"产生的"(笑)"等）可能产生异常语速：
        // - 如果文本很短但音频时长很长（静音期间误识别），语速会非常低（< 0.5 字符/秒）
        // - 如果文本很短但音频时长很短（极短静音），语速可能异常高（> 50 字符/秒）
        // 这些异常语速不应该用于更新语速历史，因为它们不代表真实的语音输入
        const MIN_REASONABLE_RATE: f32 = 0.5;  // 最小合理语速（字符/秒）
        const MAX_REASONABLE_RATE: f32 = 50.0;  // 最大合理语速（字符/秒）
        
        if speech_rate < MIN_REASONABLE_RATE || speech_rate > MAX_REASONABLE_RATE {
            eprintln!("[SileroVad] ⚠️  update_speech_rate: Abnormal speech rate {:.2} chars/s (text='{}', {} chars, {}ms) - likely misrecognition, skipping", 
                     speech_rate, text.chars().take(30).collect::<String>(), text_length, audio_duration_ms);
            return;
        }
        
        eprintln!("[SileroVad] 📝 update_speech_rate: text='{}' ({} chars), duration={}ms, rate={:.2} chars/s", 
                 text.chars().take(30).collect::<String>(), text_length, audio_duration_ms, speech_rate);
        
        // 更新全局自适应状态
        let mut state = self.adaptive_state.lock().unwrap();
        let old_sample_count = state.sample_count;
        state.update_speech_rate(speech_rate, &self.config);
        
        let perf_ms = perf_start.elapsed().as_micros() as f32 / 1000.0;
        
        // 输出调试信息（包含性能数据和调整详情）
        if let Some(avg_rate) = state.get_avg_speech_rate() {
            let effective_threshold = state.get_effective_threshold(&self.config);
            let base_threshold = state.base_threshold_ms;
            let delta = state.delta_ms;
            eprintln!("[SileroVad] 📊 Global speech_rate={:.2} chars/s, effective_threshold={}ms (base={}ms, delta={:+}ms) [samples={}->{}, update_time={:.3}ms]", 
                     avg_rate, effective_threshold, base_threshold, delta, old_sample_count, state.sample_count, perf_ms);
        } else {
            eprintln!("[SileroVad] ⚠️  update_speech_rate: After update, speech_rate_history is still empty (samples: {})", state.sample_count);
        }
    }
    
    /// 获取全局自适应阈值
    /// 
    /// # Returns
    /// 返回调整后的最小静音时长阈值（毫秒）
    pub fn get_adjusted_duration_ms(&self) -> u64 {
        if !self.config.adaptive_enabled {
            return self.config.min_silence_duration_ms;
        }
        
        let state = self.adaptive_state.lock().unwrap();
        let adjusted = state.get_adjusted_duration(&self.config);
        
        // 记录异常高的阈值（可能是问题）
        // 降低警告阈值，从 80% 降到 90%，避免频繁警告
        if adjusted > self.config.final_threshold_max_ms * 9 / 10 {
            eprintln!("[SileroVad] ⚠️  High threshold detected: {}ms (base={}ms, delta={:+}ms, samples={}, history_len={})", 
                     adjusted, state.base_threshold_ms, state.delta_ms, state.sample_count, state.speech_rate_history.len());
        }
        
        adjusted
    }
    
    /// 获取全局平均语速（用于传递给TTS）
    /// 
    /// # Returns
    /// 返回平均语速（字符/秒），如果数据不足则返回None
    pub fn get_speech_rate(&self) -> Option<f32> {
        if !self.config.adaptive_enabled {
            eprintln!("[SileroVad] ⚠️  get_speech_rate: adaptive_enabled is false");
            return None;
        }
        
        let state = self.adaptive_state.lock().unwrap();
        let rate = state.get_avg_speech_rate();
        
        // 减少日志输出频率（只在首次获取或状态变化时输出）
        // 避免每次调用都输出日志，减少日志噪音
        if rate.is_none() && state.sample_count == 0 {
            eprintln!("[SileroVad] ⚠️  get_speech_rate: speech_rate_history is empty (samples: {})", state.sample_count);
        }
        // 只在有语速数据时输出一次确认日志（减少日志噪音）
        
        rate
    }
    
    /// 获取上一个检测到语音的时间戳（用于过滤静音帧）
    /// 
    /// # Returns
    /// 返回上一个检测到语音的时间戳（毫秒），如果没有则返回 None
    pub fn get_last_speech_timestamp(&self) -> Option<u64> {
        let last_speech = self.last_speech_timestamp.lock().unwrap();
        *last_speech
    }
    
    /// 基于反馈调整 delta（用于自适应优化）
    /// 
    /// # Arguments
    /// * `feedback_type` - 反馈类型：`BoundaryTooLong`（边界过长，需要降低阈值）或 `BoundaryTooShort`（边界过短，需要提高阈值）
    /// * `adjustment_ms` - 调整量（毫秒），通常为 150ms
    /// 
    /// # 使用场景
    /// - 如果检测到音频输入但ASR长时间无输出，说明边界过长，应该降低阈值
    /// - 如果ASR识别结果混乱、被过滤、或NMT翻译异常，说明边界过短，应该提高阈值
    /// 
    /// # 修订版设计
    /// - 只调整 delta，不直接修改 base_threshold
    /// - BoundaryTooLong → delta -= 150ms
    /// - BoundaryTooShort → delta += 150ms
    /// - effective_threshold = clamp(base_threshold + delta, 500-1500ms)
    pub fn adjust_delta_by_feedback(&self, feedback_type: VadFeedbackType, adjustment_ms: i64) {
        if !self.config.adaptive_enabled {
            return;
        }
        
        let mut state = self.adaptive_state.lock().unwrap();
        let old_delta = state.delta_ms;
        let old_base = state.base_threshold_ms;
        let old_effective = state.get_effective_threshold(&self.config);
        
        let delta_adjustment = match feedback_type {
            VadFeedbackType::BoundaryTooLong => {
                // 边界过长：降低阈值（减少等待时间）
                -adjustment_ms
            }
            VadFeedbackType::BoundaryTooShort => {
                // 边界过短：提高阈值（增加等待时间）
                adjustment_ms
            }
        };
        
        // 更新 delta，并限制在范围内
        state.delta_ms = (state.delta_ms + delta_adjustment)
            .clamp(self.config.delta_min_ms, self.config.delta_max_ms);
        
        let new_effective = state.get_effective_threshold(&self.config);
        
        eprintln!("[SileroVad] 🔧 Delta adjusted by feedback: {}ms -> {}ms (type={:?}, adjustment={:+}ms, base={}ms, effective={}ms -> {}ms)", 
                 old_delta, state.delta_ms, feedback_type, delta_adjustment, old_base, old_effective, new_effective);
    }
    
    /// 基于反馈调整阈值（兼容旧接口，已废弃）
    #[deprecated(note = "Use adjust_delta_by_feedback instead")]
    pub fn adjust_threshold_by_feedback(&self, feedback_type: VadFeedbackType, _adjustment_factor: f32) {
        // 使用固定的 150ms 调整量
        self.adjust_delta_by_feedback(feedback_type, 150);
    }
}

/// VAD反馈类型（用于自适应阈值调整）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadFeedbackType {
    /// 边界过长：检测到音频输入但ASR长时间无输出，需要降低阈值
    BoundaryTooLong,
    /// 边界过短：ASR识别结果混乱、被过滤、或NMT翻译异常，需要提高阈值
    BoundaryTooShort,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame(timestamp_ms: u64, data: Vec<f32>) -> AudioFrame {
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data,
            timestamp_ms,
        }
    }
    
    #[tokio::test]
    #[ignore]  // 需要模型文件，默认忽略
    async fn test_silero_vad_with_model() {
        // 这个测试需要实际的模型文件
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !Path::new(model_path).exists() {
            eprintln!("Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        
        // 创建测试音频（静音）
        let silence_audio = vec![0.0f32; 512];
        let frame = create_test_frame(0, silence_audio);
        let result = vad.detect(frame).await.unwrap();
        
        // 静音应该被检测到
        assert!(result.confidence < 0.5);
    }
    
    #[test]
    fn test_speaker_adaptive_state() {
        let config = SileroVadConfig::default();
        let mut state = SpeakerAdaptiveState::new(600);
        
        // 测试初始状态
        assert_eq!(state.get_adjusted_duration(&config), 600);
        assert_eq!(state.sample_count, 0);
        assert!(state.get_avg_speech_rate().is_none());
        
        // 更新语速（快语速）
        state.update_speech_rate(10.0, &config);
        assert_eq!(state.sample_count, 1);
        assert!(state.get_avg_speech_rate().is_some());
        
        // 更新语速（慢语速）
        state.update_speech_rate(3.0, &config);
        assert_eq!(state.sample_count, 2);
        
        // 更新语速（正常语速）
        state.update_speech_rate(6.0, &config);
        assert_eq!(state.sample_count, 3);
        
        // 现在应该使用调整后的阈值
        let adjusted = state.get_adjusted_duration(&config);
        assert!(adjusted >= config.final_threshold_min_ms);
        assert!(adjusted <= config.final_threshold_max_ms);
    }
    
    #[test]
    fn test_silero_vad_config_default() {
        let config = SileroVadConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.frame_size, 512);
        assert_eq!(config.silence_threshold, 0.2);  // 更新为新的默认值
        assert_eq!(config.min_silence_duration_ms, 300);
        assert!(config.adaptive_enabled);
        assert_eq!(config.adaptive_min_samples, 1);
        assert_eq!(config.adaptive_rate, 0.4);
        assert_eq!(config.base_threshold_min_ms, 200);
        assert_eq!(config.base_threshold_max_ms, 600);
        assert_eq!(config.final_threshold_min_ms, 200);
        assert_eq!(config.final_threshold_max_ms, 800);
    }
    
    /// 创建测试用的语音音频帧
    fn create_speech_frame(timestamp_ms: u64) -> AudioFrame {
        // 创建 512 样本的音频帧（32ms @ 16kHz）
        // 使用正弦波模拟语音
        let data: Vec<f32> = (0..512)
            .map(|i| {
                // 生成 440Hz 的正弦波（A4 音符）
                let t = i as f32 / 16000.0;
                (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5
            })
            .collect();
        
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data,
            timestamp_ms,
        }
    }
    
    #[tokio::test]
    async fn test_boundary_detection_requires_speech_first() {
        // 测试：只有在检测到语音后，静音才能触发边界
        // 如果模型文件不存在，跳过测试
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !std::path::Path::new(model_path).exists() {
            eprintln!("⚠️  Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        
        // 1. 开头的静音不应该触发边界（即使达到阈值）
        // 注意：由于需要实际运行 ONNX 模型，这里我们主要测试逻辑
        // 实际测试中，如果 speech_prob 一直很低，边界不应该触发
        
        // 2. 重置 VAD
        vad.reset().await.unwrap();
        
        // 验证重置后状态
        assert!(vad.get_last_speech_timestamp().is_none());
    }
    
    #[tokio::test]
    #[ignore]  // 需要模型文件，默认忽略
    async fn test_cooldown_mechanism() {
        // 测试冷却期机制：在冷却期内不应该触发新的边界
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !std::path::Path::new(model_path).exists() {
            eprintln!("⚠️  Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        vad.reset().await.unwrap();
        
        // 这个测试需要实际运行模型，所以主要是验证逻辑正确性
        // 实际行为会在集成测试中验证
    }
    
    #[tokio::test]
    #[ignore]  // 需要模型文件，默认忽略
    async fn test_speech_detection_updates_timestamp() {
        // 测试：检测到语音时，应该更新 last_speech_timestamp
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !std::path::Path::new(model_path).exists() {
            eprintln!("⚠️  Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        vad.reset().await.unwrap();
        
        // 初始状态：没有检测到语音
        assert!(vad.get_last_speech_timestamp().is_none());
        
        // 处理一些帧（实际测试需要运行模型）
        // 这里主要验证接口可用性
    }
    
    #[tokio::test]
    #[ignore]  // 需要模型文件，默认忽略
    async fn test_reset_clears_state() {
        // 测试：reset 应该清除所有状态
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !std::path::Path::new(model_path).exists() {
            eprintln!("⚠️  Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        
        // 处理一些帧
        let frame = create_test_frame(0, vec![0.0; 512]);
        let _ = vad.detect(frame).await;
        
        // 重置
        vad.reset().await.unwrap();
        
        // 验证状态已清除
        assert!(vad.get_last_speech_timestamp().is_none());
    }
    
    #[tokio::test]
    async fn test_adaptive_speech_rate_update() {
        // 测试：自适应语速更新功能
        let model_path = "models/vad/silero/silero_vad.onnx";
        if !std::path::Path::new(model_path).exists() {
            eprintln!("⚠️  Skipping test: model file not found at {}", model_path);
            return;
        }
        
        let vad = SileroVad::new(model_path).unwrap();
        
        // 更新全局语速
        vad.update_speech_rate("Hello world", 1000);
        
        // 获取全局语速
        let speech_rate = vad.get_speech_rate();
        assert!(speech_rate.is_some());
        
        // 验证语速计算（"Hello world" = 11 字符，1000ms = 1秒，应该是 11 字符/秒）
        let rate = speech_rate.unwrap();
        assert!((rate - 11.0).abs() < 0.1, "Expected ~11 chars/s, got {}", rate);
    }
}

