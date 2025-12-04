# Speaker Identifier 手动测试指南

## 快速测试命令

### 1. 运行所有单元测试

```bash
cd core/engine
cargo test --lib speaker_identifier -- --nocapture
```

这会运行：
- `VadBasedSpeakerIdentifier` 的所有单元测试
- `EmbeddingBasedSpeakerIdentifier` 的所有单元测试

### 2. 运行特定模式的测试

#### 测试 VAD 基于边界的模式

```bash
cargo test --lib speaker_identifier::vad_based::tests -- --nocapture
```

#### 测试 Embedding 基于的模式

```bash
cargo test --lib speaker_identifier::embedding_based::tests -- --nocapture
```

### 3. 运行集成测试（如果已创建）

```bash
cargo test --test speaker_identifier_test -- --nocapture
```

## 测试场景说明

### VAD 基于边界的模式测试场景

1. **第一个边界**：应该创建 `speaker_1`
2. **短间隔插话**（< 1000ms）：应该创建新说话者（`speaker_2`）
3. **中等间隔**（1000-5000ms）：应该是同一说话者继续
4. **长间隔**（> 5000ms）：应该是新说话者

### Embedding 基于的模式测试场景

1. **第一个边界**：应该创建 `speaker_1` 并保存 embedding
2. **相似音频**：如果相似度 > 阈值，认为是同一说话者
3. **不相似音频**：如果相似度 < 阈值，创建新说话者

## 手动测试示例

### 测试 VAD 模式的基本功能

```rust
use core_engine::*;

#[tokio::test]
async fn manual_test() {
    let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
    
    // 场景：轮流说话
    let r1 = identifier.identify_speaker(&[], 0).await.unwrap();
    println!("0ms: {}", r1.speaker_id);  // speaker_1
    
    let r2 = identifier.identify_speaker(&[], 3000).await.unwrap();
    println!("3000ms: {}", r2.speaker_id);  // speaker_1 (同一人)
    
    // 场景：插话
    let r3 = identifier.identify_speaker(&[], 3500).await.unwrap();
    println!("3500ms: {}", r3.speaker_id);  // speaker_2 (插话)
    
    // 场景：新说话者
    let r4 = identifier.identify_speaker(&[], 6000).await.unwrap();
    println!("6000ms: {}", r4.speaker_id);  // speaker_3 (新说话者)
}
```

## 性能测试

### 测试识别延迟

```bash
# 运行性能测试（如果有）
cargo test --lib speaker_identifier --release -- --nocapture
```

## 调试技巧

### 查看详细信息

使用 `--nocapture` 参数可以看到 `println!` 和 `eprintln!` 的输出：

```bash
cargo test --lib speaker_identifier -- --nocapture
```

### 运行单个测试

```bash
cargo test --lib speaker_identifier::vad_based::tests::test_short_interval_interruption -- --nocapture
```

## 预期结果

### VAD 模式测试应该全部通过

- ✅ `test_first_boundary` - 第一个边界创建 speaker_1
- ✅ `test_short_interval_interruption` - 短间隔识别为插话
- ✅ `test_medium_interval_same_speaker` - 中等间隔识别为同一说话者
- ✅ `test_long_interval_new_speaker` - 长间隔识别为新说话者
- ✅ `test_reset` - 重置功能正常

### Embedding 模式测试应该全部通过

- ✅ `test_first_speaker` - 第一个说话者创建
- ✅ `test_reset` - 重置功能正常
- ✅ `test_cosine_similarity` - 余弦相似度计算正确

