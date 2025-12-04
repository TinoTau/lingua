# 多用户隔离架构分析

## 当前架构问题

### 1. 全局共享的 CoreEngine 实例

**位置**：`core/engine/src/bin/core_engine.rs:110-114`

```rust
struct AppState {
    engine: Arc<CoreEngine>,  // ❌ 所有 WebSocket 连接共享同一个实例
    config: RuntimeConfig,
    simple_config: Arc<SimpleConfig>,
}
```

**问题**：
- 所有用户共享同一个 `CoreEngine` 实例
- 所有状态都是全局的，没有用户隔离

### 2. 需要隔离的状态组件

#### 2.1 ASR Context Cache
**位置**：`core/engine/src/asr_whisper/faster_whisper_streaming.rs:43`

```rust
pub struct FasterWhisperAsrStreaming {
    context_cache: Arc<Mutex<Vec<String>>>,  // ❌ 全局共享
    // ...
}
```

**问题**：
- 所有用户的 ASR 上下文缓存共享
- 用户 A 的对话历史会被用户 B 看到（作为上下文）

#### 2.2 VAD Speech Rate History
**位置**：`core/engine/src/vad/silero_vad.rs`

```rust
pub struct SpeakerAdaptiveState {
    speech_rate_history: VecDeque<f32>,  // ❌ 全局共享
    // ...
}
```

**问题**：
- 所有用户的语速历史共享
- 用户 A 的语速会影响用户 B 的边界检测

#### 2.3 Speaker Identifier State
**位置**：`core/engine/src/speaker_identifier/mod.rs`

**问题**：
- 说话者识别状态可能共享（需要检查具体实现）
- 用户 A 识别的说话者可能被用户 B 使用

#### 2.4 Audio Buffer (Continuous Mode)
**位置**：`core/engine/src/bootstrap/core.rs:47`

```rust
pub struct CoreEngine {
    audio_buffer: Option<Arc<AudioBufferManager>>,  // ❌ 全局共享
    // ...
}
```

**问题**：
- 连续模式下的音频缓冲区共享
- 用户 A 的音频可能被用户 B 处理

#### 2.5 Config (Language Settings)
**位置**：`core/engine/src/bin/core_engine.rs:113`

```rust
struct AppState {
    simple_config: Arc<SimpleConfig>,  // ❌ 全局共享
    // ...
}
```

**问题**：
- 语言配置全局共享
- 用户 A 修改语言会影响用户 B

## 解决方案

### 方案 1：Session-Based Isolation（推荐）

为每个 WebSocket 连接创建独立的会话（Session），每个会话拥有自己的状态。

#### 架构设计

```
WebSocket Connection
    ↓
Session (session_id: UUID)
    ↓
SessionState {
    asr_context: Vec<String>,
    vad_state: SpeakerAdaptiveState,
    speaker_state: SpeakerIdentifierState,
    audio_buffer: AudioBufferManager,
    config: SessionConfig,
}
    ↓
Shared CoreEngine (无状态组件)
    - ASR Service (HTTP)
    - NMT Service (HTTP)
    - TTS Service (HTTP)
    - VAD Model (只读)
```

#### 需要修改的地方

##### 1. 创建 Session 结构

```rust
// core/engine/src/session/mod.rs
pub struct Session {
    pub session_id: String,
    pub asr_context: Arc<Mutex<Vec<String>>>,
    pub vad_state: Arc<Mutex<SpeakerAdaptiveState>>,
    pub speaker_state: Arc<Mutex<SpeakerIdentifierState>>,
    pub audio_buffer: Option<Arc<AudioBufferManager>>,
    pub config: Arc<SessionConfig>,
    pub created_at: Instant,
    pub last_activity: Arc<Mutex<Instant>>,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    cleanup_interval: Duration,
}
```

##### 2. 修改 AppState

```rust
struct AppState {
    engine: Arc<CoreEngine>,  // 共享的无状态组件
    session_manager: Arc<SessionManager>,  // 新增：会话管理器
}
```

##### 3. 修改 WebSocket Handler

```rust
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 创建新会话
    let session = state.session_manager.create_session().await;
    let session_id = session.session_id.clone();
    
    // 清理函数（连接断开时）
    let cleanup = {
        let manager = Arc::clone(&state.session_manager);
        let sid = session_id.clone();
        move || {
            manager.remove_session(&sid).await;
        }
    };
    
    // 处理消息时使用会话状态
    while let Some(msg) = socket.recv().await {
        // 使用 session.asr_context, session.vad_state 等
    }
    
    cleanup().await;
}
```

##### 4. 修改 CoreEngine 方法签名

**当前**：
```rust
async fn process_audio_frame(
    &self,
    frame: AudioFrame,
    language_hint: Option<String>,
) -> EngineResult<Option<ProcessResult>>
```

**修改后**：
```rust
async fn process_audio_frame(
    &self,
    frame: AudioFrame,
    language_hint: Option<String>,
    session: &Session,  // 新增：传入会话状态
) -> EngineResult<Option<ProcessResult>>
```

##### 5. 修改 ASR Context Cache

**当前**：
```rust
pub struct FasterWhisperAsrStreaming {
    context_cache: Arc<Mutex<Vec<String>>>,
}
```

**修改后**：
```rust
// 从 CoreEngine 中移除 context_cache
// 改为从 Session 中获取
async fn process_audio_frame(..., session: &Session) {
    let context = session.asr_context.lock().await;
    // 使用 session 的 context
}
```

##### 6. 修改 VAD State

**当前**：
```rust
pub struct SileroVad {
    adaptive_state: Arc<Mutex<SpeakerAdaptiveState>>,
}
```

**修改后**：
```rust
// 从 SileroVad 中移除 adaptive_state
// 改为从 Session 中获取
async fn detect(&self, frame: AudioFrame, session: &Session) {
    let state = session.vad_state.lock().await;
    // 使用 session 的 state
}
```

##### 7. 修改 Audio Buffer

**当前**：
```rust
pub struct CoreEngine {
    audio_buffer: Option<Arc<AudioBufferManager>>,
}
```

**修改后**：
```rust
// 从 CoreEngine 中移除 audio_buffer
// 改为从 Session 中获取
async fn process_audio_frame_continuous(..., session: &Session) {
    let buffer = session.audio_buffer.as_ref();
    // 使用 session 的 buffer
}
```

##### 8. 修改 Config

**当前**：
```rust
struct AppState {
    simple_config: Arc<SimpleConfig>,
}
```

**修改后**：
```rust
// 每个 Session 有自己的 config
pub struct SessionConfig {
    src_lang: String,
    tgt_lang: String,
    // ...
}
```

### 方案 2：Per-Connection CoreEngine（不推荐）

为每个 WebSocket 连接创建独立的 `CoreEngine` 实例。

**优点**：
- 完全隔离，实现简单

**缺点**：
- 内存开销大（每个连接都要加载模型）
- 初始化时间长
- 不适合高并发场景

## 改动评估

### 改动规模：**中等**

#### 需要修改的文件

1. **新增文件**：
   - `core/engine/src/session/mod.rs` - Session 管理
   - `core/engine/src/session/state.rs` - Session 状态定义

2. **修改文件**：
   - `core/engine/src/bin/core_engine.rs` - 添加 SessionManager
   - `core/engine/src/bootstrap/engine.rs` - 方法签名添加 session 参数
   - `core/engine/src/asr_whisper/faster_whisper_streaming.rs` - 移除 context_cache
   - `core/engine/src/vad/silero_vad.rs` - 移除 adaptive_state
   - `core/engine/src/bootstrap/core.rs` - 移除 audio_buffer
   - `core/engine/src/speaker_identifier/mod.rs` - 添加 session 支持

#### 工作量估算

- **Session 管理模块**：2-3 天
- **修改 CoreEngine 方法签名**：1-2 天
- **修改 ASR/VAD/Speaker 组件**：2-3 天
- **测试和调试**：2-3 天

**总计**：约 1-2 周

## 实现建议

### 阶段 1：基础 Session 框架
1. 创建 `Session` 和 `SessionManager`
2. 在 WebSocket handler 中创建会话
3. 添加会话清理机制（超时清理）

### 阶段 2：状态迁移
1. 将 ASR context 迁移到 Session
2. 将 VAD state 迁移到 Session
3. 将 Audio buffer 迁移到 Session

### 阶段 3：配置隔离
1. 将 Config 迁移到 Session
2. 支持每个会话独立的语言配置

### 阶段 4：测试和优化
1. 多用户并发测试
2. 内存泄漏检查
3. 性能优化

## 注意事项

1. **会话超时**：需要实现会话超时清理机制，避免内存泄漏
2. **并发安全**：确保 Session 状态的并发访问安全
3. **事件总线**：如果实现事件总线，需要支持按 session_id 过滤事件
4. **向后兼容**：考虑是否需要支持单用户模式（无 Session）

