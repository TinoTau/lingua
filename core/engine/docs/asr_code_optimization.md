# ASR 代码优化总结

## 已完成的优化

### 1. **提取音频预处理逻辑**

**优化前**：在 `infer_partial`、`infer_on_boundary` 和 `infer` 中都有重复的音频预处理代码（约 10 行）

**优化后**：提取为独立方法 `get_and_preprocess_audio()`

```rust
fn get_and_preprocess_audio(&self) -> EngineResult<Vec<f32>> {
    // 获取当前缓冲区中的所有帧
    let frames = { ... };
    
    if frames.is_empty() {
        return Ok(Vec::new());
    }
    
    // 预处理所有累积的帧
    let mut audio_buffer = Vec::new();
    for frame in &frames {
        let preprocessed = preprocess_audio_frame(frame)?;
        audio_buffer.extend_from_slice(&preprocessed);
    }
    
    Ok(audio_buffer)
}
```

**效果**：
- 减少了约 30 行重复代码
- 统一了音频预处理逻辑
- 更容易维护和修改

### 2. **提取上下文缓存获取逻辑**

**优化前**：在 `infer_on_boundary` 中有约 25 行的上下文获取和日志输出代码

**优化后**：提取为独立方法 `get_context_prompt()`

```rust
fn get_context_prompt(&self) -> EngineResult<Option<String>> {
    let cache = self.context_cache.lock()?;
    
    if !cache.is_empty() {
        // 拼接前 2 句作为上下文
        let context_sentences: Vec<String> = cache.iter()
            .rev().take(2).rev().cloned().collect();
        let context: String = context_sentences.join(" ");
        
        // 详细的日志输出
        eprintln!("[ASR] 📚 Context Cache: Found {} previous sentence(s)", ...);
        // ...
        
        Ok(Some(context))
    } else {
        eprintln!("[ASR] 📚 Context Cache: Empty (no previous sentences)");
        Ok(None)
    }
}
```

**效果**：
- 减少了约 25 行重复代码
- 统一了上下文获取逻辑
- 日志输出更一致

### 3. **提取上下文缓存更新逻辑**

**优化前**：在 `infer_on_boundary` 中有约 30 行的缓存更新和日志输出代码

**优化后**：提取为独立方法 `update_context_cache()`

```rust
fn update_context_cache(&self, text: &str) -> EngineResult<()> {
    let trimmed_text = text.trim();
    if trimmed_text.is_empty() {
        eprintln!("[ASR] ⚠️  Context Cache: Skipped update (empty transcript)");
        return Ok(());
    }
    
    let mut cache = self.context_cache.lock()?;
    let old_cache_size = cache.len();
    
    // 添加到缓存
    cache.push(trimmed_text.to_string());
    
    // 只保留最近 2 句
    if cache.len() > 2 {
        let removed_text = cache.remove(0);
        eprintln!("[ASR] 💾 Context Cache: Removed oldest sentence (cache full)");
        // ...
    }
    
    // 详细的日志输出
    eprintln!("[ASR] 💾 Context Cache: Added new sentence (cache size: {} -> {})", ...);
    // ...
    
    Ok(())
}
```

**效果**：
- 减少了约 30 行重复代码
- 统一了缓存更新逻辑
- 日志输出更一致

## 优化统计

- **减少重复代码**：约 85 行
- **提取的方法**：3 个
- **代码可维护性**：显著提升
- **代码可读性**：显著提升

## 进一步优化（已完成）

### 1. **提取 spawn_blocking 推理逻辑** ✅

已提取为 `transcribe_audio_async()` 方法：

```rust
async fn transcribe_audio_async(
    &self,
    audio_data: Vec<f32>,
    context_prompt: Option<String>,
    use_segments: bool,
) -> EngineResult<(String, Vec<String>, Option<String>)> {
    // 统一的异步推理逻辑
    // 支持是否返回 segments
    // 支持上下文 prompt
}
```

**效果**：
- 消除了 3 处重复的 `spawn_blocking` 调用
- 统一了错误处理逻辑
- 减少了约 40 行重复代码

### 2. **提取空结果构造** ✅

已提取为 `empty_asr_result()` 方法：

```rust
fn empty_asr_result() -> AsrResult {
    AsrResult {
        partial: None,
        final_transcript: None,
    }
}
```

**效果**：
- 消除了多处重复的空结果构造
- 统一了空结果的定义
- 减少了约 15 行重复代码

## 优化统计（更新）

- **减少重复代码**：约 140 行
- **提取的方法**：5 个
  - `get_and_preprocess_audio()` - 音频预处理
  - `get_context_prompt()` - 上下文获取
  - `update_context_cache()` - 缓存更新
  - `transcribe_audio_async()` - 异步转录
  - `empty_asr_result()` - 空结果构造
- **代码可维护性**：显著提升
- **代码可读性**：显著提升

## 总结

通过提取重复逻辑，代码变得更加：
- **简洁**：减少了大量重复代码
- **可维护**：修改逻辑只需要在一个地方
- **可读**：方法名清晰表达了功能
- **可测试**：独立的方法更容易测试

