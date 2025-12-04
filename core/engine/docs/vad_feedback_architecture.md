# VAD 反馈机制架构说明

## 关键问题回答

### Q1: 反馈逻辑是独立进程吗？

**答案：不是独立进程，而是在同一个 `CoreEngine` 中同步执行的。**

反馈逻辑完全集成在主处理流程中，不需要额外的进程或线程。

### Q2: 得到NMT的检测结果后如何通知ASR服务？

**答案：不需要直接通知ASR服务。反馈调整的是VAD的阈值，通过VAD间接影响ASR。**

## 架构设计

### 同步执行模式（当前实现）

```
音频帧到达
  ↓
CoreEngine::process_audio_frame()
  ├─> VAD::detect() [使用当前阈值]
  │   └─> 检测到边界 → 触发ASR
  ├─> ASR::infer_on_boundary()
  │   └─> 返回识别结果
  ├─> NMT::translate()
  │   └─> 返回翻译结果 + 质量指标
  ├─> adjust_vad_threshold_by_feedback() [同步调用]
  │   └─> 评估质量指标
  │       └─> 调整VAD阈值 [立即生效]
  └─> 返回处理结果

下次音频帧到达时：
  └─> VAD::detect() [使用新阈值]
      └─> 自动使用调整后的阈值
```

### 关键点

1. **同步执行**：反馈逻辑在 `process_audio_frame()` 中同步执行，不阻塞主流程
2. **无需通知**：不需要通知ASR，因为：
   - VAD阈值调整后，下次边界检测时自动使用新阈值
   - ASR只在VAD检测到边界时才会被触发
   - 所以VAD阈值的改变会自动影响ASR的触发时机

## 数据流和通信方式

### 组件关系

```
┌─────────────────────────────────────────────────────────┐
│                    CoreEngine                            │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐          │
│  │   VAD    │───>│   ASR    │───>│   NMT    │          │
│  │ (阈值)   │    │          │    │ (质量指标)│          │
│  └────┬─────┘    └──────────┘    └────┬─────┘          │
│       │                                │                │
│       │                                │                │
│       └──────────┬─────────────────────┘                │
│                  │                                      │
│                  ▼                                      │
│         adjust_vad_threshold_by_feedback()             │
│                  │                                      │
│                  ▼                                      │
│         VAD.adjust_threshold_by_feedback()             │
│                  │                                      │
│                  ▼                                      │
│         更新 VAD 内部阈值状态                           │
└─────────────────────────────────────────────────────────┘
```

### 通信方式

#### 1. VAD → ASR（通过边界检测）

```rust
// 在 process_audio_frame() 中
let vad_result = self.vad.detect(frame).await?;  // VAD检测边界

if vad_result.is_boundary {
    // VAD检测到边界，触发ASR
    let asr_result = whisper_asr.infer_on_boundary().await?;
}
```

**通信方式**：通过返回值（`DetectionOutcome`）传递边界信息

#### 2. ASR → NMT（通过识别结果）

```rust
// ASR返回识别结果
let asr_result = whisper_asr.infer_on_boundary().await?;

// 将识别结果传递给NMT
let translation_result = self.translate_and_publish(&transcript, ...).await?;
```

**通信方式**：通过函数参数和返回值传递数据

#### 3. NMT → VAD反馈（通过质量指标）

```rust
// NMT返回翻译结果和质量指标
let translation_result = self.translate_and_publish(...).await?;
// translation_result.quality_metrics 包含质量指标

// 反馈评估和阈值调整
self.adjust_vad_threshold_by_feedback(
    &asr_result,
    translation_stable.as_ref(),
    translation_result.as_ref(),  // 包含质量指标
    ...
);
```

**通信方式**：通过函数参数传递质量指标

#### 4. VAD反馈 → VAD阈值（通过内部状态）

```rust
// 在 adjust_vad_threshold_by_feedback() 中
self.apply_vad_feedback(feedback_type, adjustment_factor);

// 在 apply_vad_feedback() 中
silero_vad.adjust_threshold_by_feedback(feedback_type, adjustment_factor);

// 在 SileroVad::adjust_threshold_by_feedback() 中
let mut state = self.adaptive_state.lock().unwrap();
state.adjusted_duration_ms = new_threshold;  // 更新内部状态
```

**通信方式**：通过共享的 `Arc<Mutex<SpeakerAdaptiveState>>` 更新内部状态

#### 5. VAD阈值 → 下次边界检测（自动生效）

```rust
// 在 VAD::detect() 中
let effective_threshold = self.get_adjusted_duration_ms();  // 获取最新阈值

if silence_duration_ms >= effective_threshold {
    // 使用新阈值进行边界检测
    return Ok(DetectionOutcome { is_boundary: true, ... });
}
```

**通信方式**：通过读取内部状态获取最新阈值

## 为什么不需要直接通知ASR？

### 原因分析

1. **ASR是被动触发的**：
   - ASR只在VAD检测到边界时才会被调用
   - ASR不主动监听阈值变化
   - ASR不需要知道阈值是多少

2. **VAD是主动检测的**：
   - VAD持续监听音频帧
   - VAD使用阈值判断是否触发边界
   - 阈值改变后，VAD自动使用新阈值

3. **间接影响机制**：
   ```
   阈值调整 → VAD使用新阈值 → 边界检测时机改变 → ASR触发时机改变
   ```

### 示例说明

```
场景：阈值从400ms调整到340ms

调整前：
T0: 用户说话
T1: 停顿 400ms
T2: VAD检测到边界（阈值=400ms）
T3: 触发ASR

调整后（下次检测）：
T0: 用户说话
T1: 停顿 340ms
T2: VAD检测到边界（阈值=340ms，已自动使用新值）
T3: 触发ASR（更早触发）

ASR不需要知道阈值改变了，它只是在VAD检测到边界时被调用
```

## 如果需要独立进程会怎样？

### 独立进程方案（未采用）

如果使用独立进程，架构会是这样：

```
┌─────────────────┐         ┌─────────────────┐
│  CoreEngine     │         │  Feedback       │
│  (主进程)       │         │  Monitor        │
│                 │         │  (独立进程)     │
│  VAD → ASR → NMT│────────>│                 │
│                 │ 事件    │  监听质量指标   │
│                 │<────────│  调整VAD阈值    │
│                 │ 阈值    │                 │
└─────────────────┘         └─────────────────┘
```

**缺点**：
1. 需要进程间通信（IPC），增加复杂度
2. 需要事件总线或消息队列
3. 延迟更高（进程间通信开销）
4. 同步问题（需要确保阈值更新的原子性）

### 当前方案的优势

1. **零延迟**：同步执行，立即生效
2. **简单可靠**：不需要进程间通信
3. **原子操作**：使用 `Mutex` 保证线程安全
4. **低开销**：只是简单的函数调用和状态更新

## 线程安全

### 当前实现

```rust
// VAD内部使用 Arc<Mutex<SpeakerAdaptiveState>> 保护状态
pub struct SileroVad {
    adaptive_state: Arc<Mutex<SpeakerAdaptiveState>>,  // 线程安全的状态
    ...
}

// 阈值调整时加锁
pub fn adjust_threshold_by_feedback(...) {
    let mut state = self.adaptive_state.lock().unwrap();  // 加锁
    state.adjusted_duration_ms = new_threshold;  // 更新
    // 锁自动释放
}

// 读取阈值时也加锁
pub fn get_adjusted_duration_ms(&self) -> u64 {
    let state = self.adaptive_state.lock().unwrap();  // 加锁
    state.get_adjusted_duration(&self.config)  // 读取
    // 锁自动释放
}
```

**线程安全保证**：
- 使用 `Mutex` 保护共享状态
- 读写操作都是原子性的
- 多个线程可以安全地同时访问VAD

## 总结

### 反馈逻辑的工作方式

1. **不是独立进程**：在主处理流程中同步执行
2. **不需要通知ASR**：通过调整VAD阈值间接影响ASR
3. **自动生效**：阈值调整后，下次边界检测自动使用新值
4. **线程安全**：使用 `Mutex` 保护共享状态

### 数据流向

```
NMT质量指标 
  ↓ (函数参数)
adjust_vad_threshold_by_feedback()
  ↓ (函数调用)
VAD.adjust_threshold_by_feedback()
  ↓ (更新内部状态)
VAD.adaptive_state.adjusted_duration_ms
  ↓ (下次检测时读取)
VAD.get_adjusted_duration_ms()
  ↓ (用于边界检测)
VAD.detect() → 触发ASR
```

### 关键设计决策

- ✅ **同步执行**：简单、可靠、低延迟
- ✅ **间接影响**：通过VAD阈值影响ASR，无需直接通信
- ✅ **状态共享**：使用 `Arc<Mutex<>>` 实现线程安全的状态共享
- ✅ **立即生效**：阈值调整后立即影响下次检测

这种设计既简单又高效，避免了复杂的进程间通信，同时保证了线程安全和实时性。

