# VAD 自适应策略设计

## 问题分析

当前 SileroVad 使用固定参数：
- `silence_threshold = 0.5`（固定）
- `min_silence_duration_ms = 600ms`（固定）

这会导致在不同场景下表现不佳：
- **多人交谈**：需要更短的静音时长来快速检测说话者切换
- **快语速**：说话者停顿时间短，需要更短的阈值
- **慢语速**：说话者停顿时间长，需要更长的阈值
- **不同语言**：不同语言的停顿习惯不同（中文停顿较短，英文停顿较长）

## 解决方案

### 方案 1：配置预设（推荐，简单易用）

为不同场景提供预设配置，用户可以根据实际情况选择。

**优点**：
- 实现简单
- 用户可控
- 无需复杂算法

**缺点**：
- 需要手动选择
- 无法自动适应

**实现**：

```toml
[vad]
type = "silero"
model_path = "models/vad/silero/silero_vad.onnx"

# 预设模式：fast（快语速/多人）、normal（标准）、slow（慢语速）、custom（自定义）
preset = "normal"  # 默认使用 normal

# 自定义配置（仅在 preset = "custom" 时生效）
[silence_threshold = 0.5]
min_silence_duration_ms = 600

# 预设配置
[vad.presets.fast]
silence_threshold = 0.4
min_silence_duration_ms = 400  # 400ms，适合快语速和多人交谈

[vad.presets.normal]
silence_threshold = 0.5
min_silence_duration_ms = 600  # 600ms，标准配置

[vad.presets.slow]
silence_threshold = 0.6
min_silence_duration_ms = 800  # 800ms，适合慢语速
```

### 方案 2：基于语速的自适应（中等复杂度）

根据检测到的语速动态调整阈值。

**原理**：
- 计算平均语速（字符/秒或音节/秒）
- 根据语速调整 `min_silence_duration_ms`
- 快语速 → 更短的阈值
- 慢语速 → 更长的阈值

**实现思路**：

```rust
struct AdaptiveVadConfig {
    base_threshold: f32,
    base_duration_ms: u64,
    // 语速自适应参数
    speech_rate_adaptive: bool,
    min_speech_rate: f32,  // 字符/秒
    max_speech_rate: f32,
}

impl AdaptiveVadConfig {
    fn adjust_for_speech_rate(&self, speech_rate: f32) -> (f32, u64) {
        if !self.speech_rate_adaptive {
            return (self.base_threshold, self.base_duration_ms);
        }
        
        // 归一化语速到 [0, 1]
        let normalized_rate = (speech_rate - self.min_speech_rate) 
            / (self.max_speech_rate - self.min_speech_rate);
        let normalized_rate = normalized_rate.clamp(0.0, 1.0);
        
        // 快语速（normalized_rate > 0.7）→ 更短的阈值
        // 慢语速（normalized_rate < 0.3）→ 更长的阈值
        let duration_multiplier = if normalized_rate > 0.7 {
            0.7  // 快语速：阈值缩短到 70%
        } else if normalized_rate < 0.3 {
            1.3  // 慢语速：阈值延长到 130%
        } else {
            1.0  // 正常语速：保持原值
        };
        
        let adjusted_duration = (self.base_duration_ms as f32 * duration_multiplier) as u64;
        (self.base_threshold, adjusted_duration)
    }
}
```

### 方案 3：基于说话者数量的自适应（中等复杂度）

根据检测到的说话者数量调整阈值。

**原理**：
- 多人交谈时，说话者切换频繁，需要更短的阈值
- 单人说话时，可以使用标准阈值

**实现思路**：

```rust
struct AdaptiveVadConfig {
    base_threshold: f32,
    base_duration_ms: u64,
    // 说话者数量自适应
    speaker_count_adaptive: bool,
}

impl AdaptiveVadConfig {
    fn adjust_for_speaker_count(&self, speaker_count: usize) -> (f32, u64) {
        if !self.speaker_count_adaptive {
            return (self.base_threshold, self.base_duration_ms);
        }
        
        // 多人交谈（>= 2 人）→ 更短的阈值
        let duration_multiplier = if speaker_count >= 2 {
            0.6  // 多人：阈值缩短到 60%（360ms）
        } else {
            1.0  // 单人：保持原值
        };
        
        let adjusted_duration = (self.base_duration_ms as f32 * duration_multiplier) as u64;
        (self.base_threshold, adjusted_duration)
    }
}
```

### 方案 4：基于历史统计的自适应（复杂，最智能）

根据最近检测到的边界间隔动态调整阈值。

**原理**：
- 统计最近 N 次边界检测的间隔时间
- 如果间隔时间短 → 降低阈值（更敏感）
- 如果间隔时间长 → 提高阈值（更保守）

**实现思路**：

```rust
struct AdaptiveVadState {
    recent_boundary_intervals: VecDeque<u64>,  // 最近 N 次边界间隔
    window_size: usize,  // 统计窗口大小
    min_interval_ms: u64,  // 最小边界间隔
    max_interval_ms: u64,  // 最大边界间隔
}

impl AdaptiveVadState {
    fn update_boundary(&mut self, current_timestamp: u64, last_boundary_timestamp: Option<u64>) {
        if let Some(last) = last_boundary_timestamp {
            let interval = current_timestamp - last;
            self.recent_boundary_intervals.push_back(interval);
            if self.recent_boundary_intervals.len() > self.window_size {
                self.recent_boundary_intervals.pop_front();
            }
        }
    }
    
    fn get_adjusted_duration(&self, base_duration_ms: u64) -> u64 {
        if self.recent_boundary_intervals.is_empty() {
            return base_duration_ms;
        }
        
        // 计算平均边界间隔
        let avg_interval: u64 = self.recent_boundary_intervals.iter().sum::<u64>() 
            / self.recent_boundary_intervals.len() as u64;
        
        // 如果平均间隔短，说明需要更短的阈值
        // 如果平均间隔长，说明可以使用更长的阈值
        let multiplier = if avg_interval < self.min_interval_ms {
            0.7  // 间隔太短，降低阈值
        } else if avg_interval > self.max_interval_ms {
            1.2  // 间隔太长，提高阈值
        } else {
            1.0  // 正常范围，保持原值
        };
        
        (base_duration_ms as f32 * multiplier) as u64
    }
}
```

### 方案 5：混合自适应（推荐，平衡智能和简单）

结合多种因素进行自适应调整。

**实现思路**：

```rust
struct AdaptiveVadConfig {
    base_threshold: f32,
    base_duration_ms: u64,
    
    // 自适应模式
    adaptive_mode: AdaptiveMode,
    
    // 各因素的权重
    speech_rate_weight: f32,
    speaker_count_weight: f32,
    history_weight: f32,
}

enum AdaptiveMode {
    Fixed,           // 固定参数
    Preset(String),  // 使用预设
    Adaptive,        // 完全自适应
    Hybrid,          // 混合模式（预设 + 自适应微调）
}

impl AdaptiveVadConfig {
    fn get_adjusted_params(
        &self,
        speech_rate: Option<f32>,
        speaker_count: Option<usize>,
        recent_intervals: &[u64],
    ) -> (f32, u64) {
        match self.adaptive_mode {
            AdaptiveMode::Fixed => {
                (self.base_threshold, self.base_duration_ms)
            }
            AdaptiveMode::Preset(ref preset_name) => {
                // 使用预设配置
                self.get_preset_params(preset_name)
            }
            AdaptiveMode::Adaptive | AdaptiveMode::Hybrid => {
                // 根据多个因素调整
                let mut duration = self.base_duration_ms;
                
                // 1. 语速调整
                if let Some(rate) = speech_rate {
                    let rate_adjustment = self.adjust_for_speech_rate(rate);
                    duration = (duration as f32 * (1.0 + rate_adjustment * self.speech_rate_weight)) as u64;
                }
                
                // 2. 说话者数量调整
                if let Some(count) = speaker_count {
                    let count_adjustment = self.adjust_for_speaker_count(count);
                    duration = (duration as f32 * (1.0 + count_adjustment * self.speaker_count_weight)) as u64;
                }
                
                // 3. 历史统计调整
                if !recent_intervals.is_empty() {
                    let history_adjustment = self.adjust_for_history(recent_intervals);
                    duration = (duration as f32 * (1.0 + history_adjustment * self.history_weight)) as u64;
                }
                
                // 限制在合理范围内
                duration = duration.clamp(200, 2000);  // 200ms - 2s
                
                (self.base_threshold, duration)
            }
        }
    }
}
```

## 推荐实现方案

### 阶段 1：配置预设（立即实现）

**优点**：
- 实现简单
- 用户可控
- 无需复杂算法

**实现步骤**：
1. 在 `SileroVadConfig` 中添加 `preset` 字段
2. 定义预设配置（fast、normal、slow）
3. 在初始化时根据预设设置参数

### 阶段 2：混合自适应（后续优化）

**优点**：
- 自动适应不同场景
- 平衡智能和简单
- 可以基于预设进行微调

**实现步骤**：
1. 添加自适应状态跟踪
2. 实现语速检测
3. 实现说话者数量统计
4. 实现历史统计
5. 实现混合调整算法

## 配置示例

### 当前配置（固定）

```toml
[vad]
type = "silero"
silence_threshold = 0.5
min_silence_duration_ms = 600
```

### 预设配置（阶段 1）

```toml
[vad]
type = "silero"
preset = "fast"  # fast, normal, slow

# 或者自定义
[vad]
type = "silero"
preset = "custom"
silence_threshold = 0.5
min_silence_duration_ms = 600
```

### 自适应配置（阶段 2）

```toml
[vad]
type = "silero"
adaptive_mode = "hybrid"  # fixed, preset, adaptive, hybrid
base_silence_threshold = 0.5
base_min_silence_duration_ms = 600

# 自适应参数
[vad.adaptive]
speech_rate_adaptive = true
speaker_count_adaptive = true
history_adaptive = true
speech_rate_weight = 0.3
speaker_count_weight = 0.4
history_weight = 0.3
```

## 使用建议

1. **初期**：使用预设配置，根据场景手动选择
2. **中期**：启用混合自适应，自动微调参数
3. **长期**：根据实际使用数据优化自适应算法

## 参考

- [WebRTC VAD 自适应策略](https://webrtc.org/)
- [语音活动检测最佳实践](https://github.com/snakers4/silero-vad)

