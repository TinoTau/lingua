# 说话者音色一致性说明

## 当前实现状态

### ❌ 音色不一致（当前实现）

**当前流程**：
1. **Speaker Identifier** → 只识别说话者（分配 `speaker_id`），**不提取音色特征**
2. **Speaker Voice Mapper** → 根据 `speaker_id` **轮询分配**预定义的 voice 列表
3. **TTS** → 使用分配的 voice ID 合成，**与输入音频音色无关**

**示例**：
- 输入：说话者A（男声，低音）→ `speaker_1`
- TTS 输出：可能使用 `zh_CN-huayan-medium`（女声，高音）❌

### ✅ 已实现的改进（为音色一致性做准备）

我已经扩展了 `SpeakerIdentificationResult`，添加了：
- `voice_embedding: Option<Vec<f32>>` - 音色特征向量（用于 Voice Cloning）
- `reference_audio: Option<Vec<f32>>` - 参考音频片段（用于 zero-shot TTS）

**Embedding 模式**会提取这些信息，但**当前 TTS 还不支持使用它们**。

## 实现音色一致的方案

### 方案 1：扩展 TTS 支持 Voice Embedding（推荐）

**需要修改**：
1. 扩展 `TtsRequest` 添加 `voice_embedding` 字段
2. 集成支持 zero-shot 的 TTS 模型（如 YourTTS、VALL-E）
3. 修改 `synthesize_and_publish()` 传递 `voice_embedding`

**优点**：
- 音色完全一致
- 支持任意说话者
- 无需预定义 voice 列表

**缺点**：
- 需要集成新的 TTS 模型
- 计算开销较大

### 方案 2：使用参考音频（Zero-shot TTS）

**需要修改**：
1. 扩展 `TtsRequest` 添加 `reference_audio` 字段
2. 集成支持 zero-shot 的 TTS（如 YourTTS）
3. 将参考音频传递给 TTS

**优点**：
- 音色一致
- 实现相对简单

**缺点**：
- 需要存储参考音频（内存占用）
- 需要支持 zero-shot 的 TTS 模型

### 方案 3：混合方案（当前推荐）

**当前状态**：
- ✅ Speaker Identifier 已提取 `voice_embedding` 和 `reference_audio`
- ❌ TTS 还不支持使用这些信息
- ✅ 使用预定义 voice 列表作为后备方案

**下一步**：
1. 集成支持 zero-shot 的 TTS 模型
2. 修改 TTS 合成逻辑，优先使用 `voice_embedding`，如果没有则使用预定义 voice

## 当前使用方式

### VAD 模式（免费用户）
- **音色**：使用预定义 voice 列表（轮询分配）
- **一致性**：❌ 不一致（随机分配）

### Embedding 模式（付费用户）
- **音色特征**：✅ 已提取 `voice_embedding` 和 `reference_audio`
- **TTS 使用**：❌ 当前仍使用预定义 voice（因为 TTS 还不支持 embedding）
- **一致性**：❌ 当前不一致，但已为未来实现做好准备

## 总结

**当前答案**：**不一致**。TTS 为每个说话者分配的音色是**预定义的 voice 列表**，与输入音频的音色**无关**。

**未来实现**：当集成支持 zero-shot 的 TTS 模型后，可以使用提取的 `voice_embedding` 或 `reference_audio` 实现音色一致。

