# M2M100 TTS 增量播放实现方案

**日期：** 2025-01-23  
**目标：** 实现 TTS 增量播放，每个短句合成完就立刻播放，或缓存几个短句平滑衔接

---

## 1. 当前实现分析

### 1.1 当前流程

**当前实现：**
- `synthesize_and_publish()` 一次性合成整个翻译文本
- 返回一个 `TtsStreamChunk`，包含完整的音频数据
- 客户端需要等待整个文本合成完成后才能播放

**问题：**
- 长文本合成时间长，用户等待时间长
- 无法实现"边合成边播放"的流畅体验
- 无法实现增量播放

---

## 2. 实现方案

### 2.1 方案概述

**核心思路：**
1. 将翻译文本分割成短句（按标点符号）
2. 对每个短句分别调用 TTS 合成
3. 每个短句合成完成后立即发布事件
4. 支持两种模式：
   - **立即播放模式**：每个短句合成完就立刻播放
   - **缓冲模式**：缓存几个短句，平滑衔接播放

### 2.2 文本分割策略

**分割规则：**
- 按句号、问号、感叹号分割（中英文）
- 最大句子长度限制（例如 50 字符）
- 保留标点符号

**示例：**
```
输入："Hello, world. How are you? I'm fine!"
分割：
  - "Hello, world."
  - "How are you?"
  - "I'm fine!"
```

---

## 3. 实现步骤

### 3.1 创建文本分割模块

**文件：** `core/engine/src/text_segmentation.rs`

**功能：**
- 将文本分割成短句
- 支持中英文标点
- 支持最大长度限制

### 3.2 修改 TTS 合成方法

**文件：** `core/engine/src/bootstrap.rs`

**修改：**
- 将 `synthesize_and_publish()` 改为 `synthesize_and_publish_incremental()`
- 支持增量合成和发布
- 支持缓冲模式配置

### 3.3 添加缓冲管理

**功能：**
- 维护一个音频缓冲区
- 控制缓冲的短句数量
- 平滑衔接播放

---

## 4. 详细设计

### 4.1 文本分割模块

```rust
pub struct TextSegmenter {
    max_sentence_length: usize,
}

impl TextSegmenter {
    pub fn new(max_sentence_length: usize) -> Self {
        Self { max_sentence_length }
    }
    
    pub fn segment(&self, text: &str) -> Vec<String> {
        // 按标点符号分割
        // 处理中英文标点：. ! ? 。！？
        // 如果句子过长，按逗号或空格进一步分割
    }
}
```

### 4.2 增量合成方法

```rust
async fn synthesize_and_publish_incremental(
    &self,
    translation: &TranslationResponse,
    timestamp_ms: u64,
    buffer_size: usize,  // 缓冲的短句数量（0 = 立即播放）
) -> EngineResult<Vec<TtsStreamChunk>> {
    // 1. 分割文本
    let segments = self.segment_text(&translation.translated_text);
    
    // 2. 对每个短句进行 TTS 合成
    let mut chunks = Vec::new();
    let mut buffer = Vec::new();
    
    for (idx, segment) in segments.iter().enumerate() {
        // 合成短句
        let chunk = self.synthesize_segment(segment, idx == segments.len() - 1).await?;
        
        if buffer_size == 0 {
            // 立即播放模式：立即发布
            self.publish_tts_event(&chunk, timestamp_ms).await?;
        } else {
            // 缓冲模式：加入缓冲区
            buffer.push(chunk);
            
            // 如果缓冲区满了，发布最早的短句
            if buffer.len() >= buffer_size {
                let first_chunk = buffer.remove(0);
                self.publish_tts_event(&first_chunk, timestamp_ms).await?;
            }
        }
        
        chunks.push(chunk);
    }
    
    // 3. 缓冲模式：发布剩余的短句
    if buffer_size > 0 {
        for chunk in buffer {
            self.publish_tts_event(&chunk, timestamp_ms).await?;
        }
    }
    
    Ok(chunks)
}
```

### 4.3 配置选项

**在 `config.toml` 中添加：**

```toml
[tts]
# TTS 增量播放配置
incremental_playback = true
# 缓冲模式：0 = 立即播放，> 0 = 缓冲的短句数量
buffer_sentences = 2
# 最大句子长度（字符）
max_sentence_length = 50
```

---

## 5. 实现细节

### 5.1 文本分割实现

**需要考虑：**
- 中英文标点符号
- 缩写处理（如 "Dr.", "Mr."）
- 数字和标点的组合
- 引号内的内容

### 5.2 音频衔接

**平滑衔接：**
- 在短句之间添加短暂静音（可选）
- 确保音频格式一致
- 处理采样率和声道

### 5.3 错误处理

**需要考虑：**
- 某个短句合成失败时的处理
- 网络中断时的恢复
- 部分播放失败时的降级

---

## 6. 使用方式

### 6.1 立即播放模式

```rust
// buffer_size = 0，每个短句合成完就立刻播放
let chunks = engine.synthesize_and_publish_incremental(
    &translation,
    timestamp_ms,
    0,  // 立即播放
).await?;
```

### 6.2 缓冲模式

```rust
// buffer_size = 2，缓存 2 个短句后开始播放
let chunks = engine.synthesize_and_publish_incremental(
    &translation,
    timestamp_ms,
    2,  // 缓冲 2 个短句
).await?;
```

---

## 7. 测试计划

### 7.1 单元测试

- 文本分割功能测试
- 不同长度的文本测试
- 中英文混合文本测试

### 7.2 集成测试

- 立即播放模式测试
- 缓冲模式测试
- 长文本增量播放测试

### 7.3 性能测试

- 增量播放的延迟对比
- 缓冲模式的流畅度测试

---

## 8. 后续优化

### 8.1 智能缓冲

- 根据网络延迟动态调整缓冲大小
- 根据合成速度调整缓冲策略

### 8.2 音频预处理

- 在短句之间添加自然停顿
- 调整音量平衡
- 音频淡入淡出

---

## 9. 实施优先级

1. **P0（高优先级）**：
   - 文本分割模块
   - 增量合成方法
   - 立即播放模式

2. **P1（中优先级）**：
   - 缓冲模式
   - 配置选项
   - 错误处理

3. **P2（低优先级）**：
   - 智能缓冲
   - 音频预处理
   - 性能优化

---

**方案状态：** 📋 待实施  
**预计工作量：** 1-2 天

