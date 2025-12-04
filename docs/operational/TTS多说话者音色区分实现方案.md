# TTS 多说话者音色区分实现方案

## 一、需求分析

### 1.1 新增需求（第二阶段目标附加内容）

**目标**：在多人轮流发言场景中，为每个用户分配不同的 TTS 音色，使用户能够通过声音区分不同的发言者。

**核心要求**：
1. 为每个用户生成/分配 speaker embedding 或 voice ID
2. TTS 生成时根据用户 ID 选择对应的音色
3. 支持两种方案：
   - (A) 预训练多说话者模型（固定音色）
   - (B) Zero-shot / Voice cloning（用户自定义音色）

## 二、技术可行性分析

### 2.1 当前系统状态 ✅

**已实现的功能**：
- ✅ TTS 系统已有 `voice` 字段，可以指定不同的 voice 模型
- ✅ 使用 Piper TTS，支持多个预训练 voice 模型（如 `zh_CN-huayan-medium`、`en_US-lessac-medium`）
- ✅ `TtsRequest` 结构包含 `voice` 字段
- ✅ 可以根据 locale 自动选择默认 voice

**当前架构**：
```rust
pub struct TtsRequest {
    pub text: String,
    pub voice: String,  // 已支持指定 voice
    pub locale: String,
}
```

### 2.2 实现可行性评估

#### ✅ **完全可行** - 方案 A：预训练多说话者模型

**技术路径**：
1. **Piper TTS 多 voice 支持**：
   - Piper 已经支持多个预训练 voice 模型
   - 每个 voice 对应不同的音色
   - 可以通过 `voice` 字段切换

2. **用户 ID → Voice ID 映射**：
   - 在 Speaker Embedding 模块识别用户后，为每个用户分配一个 voice ID
   - 建立用户 ID 到 voice ID 的映射表
   - TTS 请求时根据用户 ID 查找对应的 voice ID

3. **实现步骤**：
   ```
   Speaker Embedding → 用户ID → Voice ID映射表 → TtsRequest.voice → Piper TTS
   ```

**优点**：
- ✅ 实现简单，无需额外模型
- ✅ 延迟低，无需额外推理
- ✅ 音色稳定，预训练模型质量高
- ✅ 与现有系统完全兼容

**缺点**：
- ⚠️ 音色选择有限（取决于可用的 voice 模型数量）
- ⚠️ 不支持用户自定义音色

#### ⚠️ **需要额外开发** - 方案 B：Zero-shot / Voice cloning

**技术路径**：
1. **模型选择**：
   - YourTTS（推荐，开源，支持多语言）
   - VALL-E X（需要 API 或自行部署）
   - StyleSpeech / Meta-Voice

2. **实现步骤**：
   - 用户上传参考音频（5-10 秒）
   - 提取 speaker embedding
   - 存储用户 ID → speaker embedding 映射
   - TTS 生成时使用 speaker embedding

**优点**：
- ✅ 支持用户自定义音色
- ✅ 音色选择无限
- ✅ 用户体验更好

**缺点**：
- ❌ 需要额外的模型和推理
- ❌ 延迟可能增加（需要 embedding 提取）
- ❌ 需要存储用户参考音频或 embedding
- ❌ 实现复杂度较高

## 三、推荐实现方案

### 3.1 阶段 1：快速实现（方案 A - 预训练多说话者）

**目标**：快速实现基础的多说话者音色区分功能

**实现步骤**：

#### Step 1: 扩展 TtsRequest 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub locale: String,
    // 新增字段
    pub speaker_id: Option<String>,  // 用户 ID（可选，用于映射到 voice）
}
```

#### Step 2: 创建 Speaker-to-Voice 映射管理器

```rust
// core/engine/src/speaker_voice_mapper.rs
pub struct SpeakerVoiceMapper {
    // 用户 ID → Voice ID 映射
    mapping: Arc<RwLock<HashMap<String, String>>>,
    // 可用 voice 列表（用于轮询分配）
    available_voices: Vec<String>,
}

impl SpeakerVoiceMapper {
    pub fn new(available_voices: Vec<String>) -> Self {
        Self {
            mapping: Arc::new(RwLock::new(HashMap::new())),
            available_voices,
        }
    }
    
    /// 为新的用户分配 voice
    pub async fn assign_voice(&self, speaker_id: &str) -> String {
        let mut mapping = self.mapping.write().await;
        
        // 如果用户已有 voice，直接返回
        if let Some(voice) = mapping.get(speaker_id) {
            return voice.clone();
        }
        
        // 为新用户分配 voice（轮询方式）
        let voice_index = mapping.len() % self.available_voices.len();
        let voice = self.available_voices[voice_index].clone();
        mapping.insert(speaker_id.to_string(), voice.clone());
        
        voice
    }
    
    /// 获取用户的 voice
    pub async fn get_voice(&self, speaker_id: &str) -> Option<String> {
        let mapping = self.mapping.read().await;
        mapping.get(speaker_id).cloned()
    }
}
```

#### Step 3: 集成到 CoreEngine

```rust
// 在 CoreEngine 中添加
pub struct CoreEngine {
    // ... 现有字段 ...
    speaker_voice_mapper: Option<Arc<SpeakerVoiceMapper>>,
}

// 在 synthesize_and_publish 方法中
async fn synthesize_and_publish(
    &self,
    translation: &TranslationResponse,
    timestamp_ms: u64,
) -> EngineResult<TtsStreamChunk> {
    // 从 translation 中获取 speaker_id（如果 Speaker Embedding 已实现）
    let speaker_id = translation.speaker_id.clone();
    
    // 确定使用的 voice
    let voice = if let Some(ref mapper) = self.speaker_voice_mapper {
        if let Some(sid) = speaker_id {
            // 根据 speaker_id 获取或分配 voice
            mapper.get_voice(&sid).await
                .unwrap_or_else(|| mapper.assign_voice(&sid).await)
        } else {
            // 没有 speaker_id，使用默认 voice
            None
        }
    } else {
        None
    };
    
    // 构建 TTS 请求
    let tts_request = TtsRequest {
        text: translation.translated_text.clone(),
        voice: voice.unwrap_or_else(|| self.get_default_voice(&translation.language)),
        locale: translation.language.clone(),
        speaker_id,
    };
    
    // ... 执行 TTS 合成 ...
}
```

#### Step 4: 配置可用 voice 列表

```rust
// 在 CoreEngineBuilder 中
pub fn with_speaker_voice_mapping(
    mut self,
    available_voices: Vec<String>,
) -> Self {
    let mapper = SpeakerVoiceMapper::new(available_voices);
    self.speaker_voice_mapper = Some(Arc::new(mapper));
    self
}
```

**使用示例**：
```rust
let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_speaker_voice_mapping(vec![
        "zh_CN-huayan-medium".to_string(),  // 用户1
        "zh_CN-xiaoyan-medium".to_string(), // 用户2
        "en_US-lessac-medium".to_string(),  // 用户3
    ])
    .build()?;
```

### 3.2 阶段 2：增强实现（方案 B - Zero-shot）

**目标**：支持用户自定义音色

**实现步骤**：

1. **集成 YourTTS 或其他 zero-shot 模型**
2. **实现 speaker embedding 提取**
3. **存储用户参考音频或 embedding**
4. **修改 TTS 请求以支持 speaker embedding**

**注意**：这需要额外的模型和推理时间，建议在阶段 1 验证后再实现。

## 四、与现有系统的集成

### 4.1 与 Speaker Embedding 模块的集成

**流程**：
```
音频输入 → VAD → Speaker Embedding → 用户ID → Voice映射 → TTS → 输出
```

**数据流**：
1. Speaker Embedding 模块识别用户，生成 `speaker_id`
2. `speaker_id` 传递给翻译结果（`TranslationResponse`）
3. TTS 模块根据 `speaker_id` 查找对应的 voice
4. 使用对应的 voice 生成 TTS 音频

### 4.2 与连续输入输出系统的兼容性

**完全兼容**：
- ✅ 每个音频片段独立处理
- ✅ 每个片段都有对应的 `speaker_id`
- ✅ TTS 生成时根据 `speaker_id` 选择 voice
- ✅ 多个片段可以并发处理，每个使用不同的 voice

### 4.3 与现有 TTS 系统的兼容性

**向后兼容**：
- ✅ 如果 `speaker_id` 为 `None`，使用默认 voice（现有行为）
- ✅ 如果未配置 `SpeakerVoiceMapper`，使用默认 voice（现有行为）
- ✅ 现有代码无需修改即可继续工作

## 五、实现优先级

### Phase 1：基础实现（1-2 天）✅ 高优先级

- [ ] 扩展 `TtsRequest` 添加 `speaker_id` 字段
- [ ] 实现 `SpeakerVoiceMapper`
- [ ] 集成到 `CoreEngine`
- [ ] 配置可用 voice 列表
- [ ] 测试多用户场景

### Phase 2：与 Speaker Embedding 集成（待 Speaker Embedding 实现后）

- [ ] 修改 `TranslationResponse` 添加 `speaker_id` 字段
- [ ] 在 TTS 请求中使用 `speaker_id`
- [ ] 端到端测试

### Phase 3：Zero-shot 支持（可选，2-3 周）

- [ ] 集成 YourTTS 或其他 zero-shot 模型
- [ ] 实现 speaker embedding 提取
- [ ] 实现用户参考音频上传和管理
- [ ] 修改 TTS 请求以支持 speaker embedding

## 六、技术难点和解决方案

### 6.1 Voice 数量限制

**问题**：如果用户数量超过可用 voice 数量怎么办？

**解决方案**：
- 使用轮询方式分配 voice（已实现）
- 或者为每个用户生成唯一的 voice ID（需要 zero-shot 模型）

### 6.2 音色一致性

**问题**：如何确保同一用户在不同会话中使用相同的音色？

**解决方案**：
- 持久化存储用户 ID → Voice ID 映射（数据库或配置文件）
- 会话开始时加载映射
- 新用户自动分配 voice

### 6.3 延迟影响

**问题**：Voice 映射是否会影响 TTS 延迟？

**解决方案**：
- Voice 映射是内存操作，延迟可忽略（<1ms）
- 不影响 TTS 推理延迟
- 可以异步预加载映射

## 七、性能预期

- **Voice 映射延迟**：<1ms（内存查找）
- **TTS 延迟**：与现有系统相同（无额外开销）
- **内存占用**：每个映射项 ~100 bytes，1000 用户约 100KB
- **并发支持**：完全支持，无锁设计

## 八、总结

### 可行性结论

✅ **完全可行** - 方案 A（预训练多说话者）可以快速实现，与现有系统完全兼容。

⚠️ **需要额外开发** - 方案 B（Zero-shot）需要额外的模型和开发工作，建议分阶段实现。

### 推荐实施路径

1. **立即实施**：方案 A（预训练多说话者）
   - 实现简单，1-2 天即可完成
   - 与现有系统完全兼容
   - 可以立即验证多说话者场景

2. **后续增强**：方案 B（Zero-shot）
   - 在方案 A 验证后实施
   - 需要额外的模型和开发工作
   - 提供更好的用户体验

### 风险评估

- **低风险**：方案 A 实现简单，不会影响现有功能
- **中风险**：方案 B 需要额外的模型和资源
- **建议**：先实现方案 A，验证效果后再考虑方案 B

