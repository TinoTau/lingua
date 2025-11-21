# M2M100 直接切换实施计划

**日期**: 2025-11-21  
**目标**: 直接替换 Marian NMT 为 M2M100 NMT，不保留并行支持  
**策略**: 完全替换，快速切换

---

## 执行摘要

**总体评估**: ✅ **可行，但需要系统性的代码重构**

直接切换 M2M100 需要修改多个模块，但架构兼容性好，可以快速完成。预计工作量：**5-7 天开发 + 2-3 天测试**。

---

## 1. 切换策略

### 1.1 核心原则

1. **完全替换**: 移除所有 Marian 相关代码，直接使用 M2M100
2. **保持接口**: `NmtIncremental` trait 保持不变，只替换实现
3. **最小改动**: 尽量复用现有架构，只修改必要的部分

### 1.2 切换范围

| 组件 | 操作 | 说明 |
|------|------|------|
| **MarianNmtOnnx** | 替换为 `M2M100NmtOnnx` | 重命名或新建 |
| **MarianTokenizer** | 替换为 `M2M100Tokenizer` | 完全重写 |
| **模型文件** | 替换为 M2M100 模型 | 需要导出和部署 |
| **测试脚本** | 更新模型路径 | 修改测试用例 |
| **Bootstrap** | 更新默认配置 | 修改默认模型路径 |

---

## 2. 详细实施步骤

### 2.1 Phase 1: 模型导出和准备（1 天）

#### 步骤 1.1: 导出 M2M100 模型

```bash
# 导出 encoder
python docs/models/export_m2m100_encoder.py \
    --output_dir core/engine/models/nmt/m2m100-en-zh \
    --model_id facebook/m2m100_418M

# 导出 decoder
python docs/models/export_m2m100_decoder_kv.py \
    --output_dir core/engine/models/nmt/m2m100-en-zh \
    --model_id facebook/m2m100_418M
```

#### 步骤 1.2: 下载 tokenizer 文件

```bash
# 从 HuggingFace 下载
huggingface-cli download facebook/m2m100_418M \
    tokenizer.json \
    sentencepiece.model \
    config.json \
    --local-dir core/engine/models/nmt/m2m100-en-zh
```

#### 步骤 1.3: 验证模型文件

- [ ] `encoder.onnx` 存在且可加载
- [ ] `decoder.onnx` 存在且可加载
- [ ] `tokenizer.json` 存在
- [ ] `sentencepiece.model` 存在
- [ ] `config.json` 存在

---

### 2.2 Phase 2: Tokenizer 实现（2 天）

#### 步骤 2.1: 创建 M2M100Tokenizer

**文件**: `core/engine/src/nmt_incremental/m2m100_tokenizer.rs`

**关键功能**:
```rust
pub struct M2M100Tokenizer {
    tokenizer: tokenizers::Tokenizer,
    lang_id_map: HashMap<String, i64>,  // lang -> lang_id
    pad_token_id: i64,
    eos_token_id: i64,
}

impl M2M100Tokenizer {
    pub fn from_model_dir(model_dir: &Path) -> Result<Self>;
    pub fn encode(&self, text: &str, src_lang: &str, add_special_tokens: bool) -> Vec<i64>;
    pub fn decode(&self, ids: &[i64], skip_special_tokens: bool) -> String;
    pub fn get_lang_id(&self, lang: &str) -> i64;
}
```

**实现要点**:
1. 使用 `tokenizers::Tokenizer::from_file()` 加载 `tokenizer.json`
2. 从 `config.json` 解析语言 ID 映射
3. 编码时在文本前添加语言 token（如 `<en> ` + text）
4. 解码时过滤语言 token 和其他特殊 token

#### 步骤 2.2: 更新 Tokenizer Trait

**文件**: `core/engine/src/nmt_incremental/tokenizer.rs`

**修改**:
- 创建 `NmtTokenizer` trait（或扩展现有接口）
- 添加 `src_lang` 参数到 `encode` 方法
- 添加 `skip_special_tokens` 参数到 `decode` 方法
- 添加 `get_lang_id` 方法

---

### 2.3 Phase 3: M2M100NmtOnnx 实现（2 天）

#### 步骤 3.1: 创建 M2M100NmtOnnx 结构

**文件**: `core/engine/src/nmt_incremental/m2m100_onnx.rs`

**关键修改**:
```rust
pub struct M2M100NmtOnnx {
    pub encoder_session: std::sync::Mutex<Session>,
    pub decoder_session: std::sync::Mutex<Session>,
    pub tokenizer: M2M100Tokenizer,
    pub decoder_start_token_id: i64,  // 从 tokenizer 获取
    pub eos_token_id: i64,
    pub pad_token_id: i64,
    pub max_length: usize,
}

impl M2M100NmtOnnx {
    // 模型常量（M2M100 418M）
    pub(crate) const NUM_LAYERS: usize = 12;  // 从 6 改为 12
    pub(crate) const NUM_HEADS: usize = 16;   // 从 8 改为 16
    pub(crate) const HEAD_DIM: usize = 64;    // 保持不变
    pub(crate) const HIDDEN_SIZE: usize = 1024; // 从 512 改为 1024
}
```

#### 步骤 3.2: 适配 Encoder 实现

**文件**: `core/engine/src/nmt_incremental/encoder.rs`

**修改**:
- 输出维度从 `[batch, seq, 512]` 改为 `[batch, seq, 1024]`
- 文件路径从 `encoder_model.onnx` 改为 `encoder.onnx`
- 逻辑保持不变

#### 步骤 3.3: 适配 Decoder 实现

**文件**: `core/engine/src/nmt_incremental/decoder.rs`

**修改**:
- 使用动态常量 `Self::NUM_LAYERS` 而不是硬编码 6
- KV Cache 构建逻辑保持不变（只是层数不同）
- 文件路径从 `model.onnx` 改为 `decoder.onnx`

#### 步骤 3.4: 适配 Translation 实现

**文件**: `core/engine/src/nmt_incremental/translation.rs`

**修改**:
- 编码时传入 `src_lang` 参数
- 获取 `decoder_start_token_id` 从 `tokenizer.get_lang_id(tgt_lang)`
- 解码时使用 `skip_special_tokens=true`

---

### 2.4 Phase 4: 更新集成代码（1 天）

#### 步骤 4.1: 更新 Bootstrap

**文件**: `core/engine/src/bootstrap.rs`

**修改**:
```rust
// 重命名方法
pub fn nmt_with_default_m2m100_onnx(mut self) -> EngineResult<Self> {
    let model_dir = crate_root.join("models/nmt/m2m100-en-zh");
    let nmt_impl = M2M100NmtOnnx::new_from_dir(&model_dir)?;
    self.nmt = Some(Box::new(nmt_impl));
    Ok(self)
}

// 或者直接替换默认方法
pub fn nmt_with_default_marian_onnx(mut self) -> EngineResult<Self> {
    // 内部调用 M2M100
    self.nmt_with_default_m2m100_onnx()
}
```

#### 步骤 4.2: 更新测试脚本

**文件**: `core/engine/examples/test_s2s_full_simple.rs`

**修改**:
```rust
// 更新模型路径
let nmt_model_name = match direction {
    TranslationDirection::EnToZh => "m2m100-en-zh",
    TranslationDirection::ZhToEn => "m2m100-zh-en",  // 需要导出
};
```

#### 步骤 4.3: 更新模块导出

**文件**: `core/engine/src/nmt_incremental/mod.rs`

**修改**:
```rust
mod m2m100_onnx;
mod m2m100_tokenizer;

pub use m2m100_tokenizer::M2M100Tokenizer;
pub use m2m100_onnx::M2M100NmtOnnx;

// 保留 Marian 相关代码（标记为 deprecated）或直接删除
```

---

### 2.5 Phase 5: 语言对支持（1 天）

#### 步骤 5.1: 更新 LanguagePair

**文件**: `core/engine/src/nmt_incremental/language_pair.rs`

**修改**:
```rust
impl LanguagePair {
    /// 转换为模型目录名（支持 M2M100）
    pub fn to_model_dir_name(&self, backend: &str) -> String {
        match backend {
            "m2m100" => format!("m2m100-{}-{}", self.source.to_dir_name(), self.target.to_dir_name()),
            "marian" => format!("marian-{}-{}", self.source.to_dir_name(), self.target.to_dir_name()),
            _ => format!("{}-{}-{}", backend, self.source.to_dir_name(), self.target.to_dir_name()),
        }
    }
}
```

#### 步骤 5.2: 导出双向模型

- [ ] 导出 `m2m100-en-zh`（英文→中文）
- [ ] 导出 `m2m100-zh-en`（中文→英文）

---

## 3. 代码改动清单

### 3.1 新建文件

| 文件 | 说明 | 优先级 |
|------|------|--------|
| `core/engine/src/nmt_incremental/m2m100_tokenizer.rs` | M2M100 Tokenizer 实现 | P0 |
| `core/engine/src/nmt_incremental/m2m100_onnx.rs` | M2M100 ONNX 实现 | P0 |

### 3.2 修改文件

| 文件 | 改动类型 | 改动量 | 优先级 |
|------|---------|--------|--------|
| `core/engine/src/nmt_incremental/decoder.rs` | 适配 | 小 | P0 |
| `core/engine/src/nmt_incremental/encoder.rs` | 适配 | 小 | P0 |
| `core/engine/src/nmt_incremental/translation.rs` | 适配 | 中 | P0 |
| `core/engine/src/nmt_incremental/mod.rs` | 导出 | 小 | P0 |
| `core/engine/src/bootstrap.rs` | 集成 | 小 | P0 |
| `core/engine/examples/test_s2s_full_simple.rs` | 测试 | 小 | P1 |
| `core/engine/src/nmt_incremental/language_pair.rs` | 扩展 | 小 | P1 |

### 3.3 可选：删除/归档文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `core/engine/src/nmt_incremental/marian_onnx.rs` | 归档 | 保留作为参考 |
| `core/engine/src/nmt_incremental/tokenizer.rs` | 归档 | 保留作为参考 |

---

## 4. 关键技术点

### 4.1 Tokenizer 实现关键点

#### 语言 Token 处理

```rust
// 编码时添加语言 token
pub fn encode(&self, text: &str, src_lang: &str, add_special_tokens: bool) -> Vec<i64> {
    // 1. 构建带语言 token 的文本
    let lang_token = format!("<{}>", src_lang);  // 如 "<en>"
    let text_with_lang = format!("{} {}", lang_token, text);
    
    // 2. 使用 tokenizer 编码
    let encoding = self.tokenizer.encode(&text_with_lang, add_special_tokens)?;
    encoding.get_ids().iter().map(|&id| id as i64).collect()
}
```

#### 语言 ID 获取

```rust
// 从 config.json 解析
pub fn get_lang_id(&self, lang: &str) -> i64 {
    self.lang_id_map.get(lang)
        .copied()
        .unwrap_or_else(|| {
            // 如果找不到，尝试从 tokenizer 获取
            // 或使用默认值
            self.eos_token_id
        })
}
```

### 4.2 模型常量更新

```rust
impl M2M100NmtOnnx {
    // M2M100 418M 配置
    pub(crate) const NUM_LAYERS: usize = 12;      // Marian: 6
    pub(crate) const NUM_HEADS: usize = 16;       // Marian: 8
    pub(crate) const HEAD_DIM: usize = 64;        // 相同
    pub(crate) const HIDDEN_SIZE: usize = 1024;   // Marian: 512
}
```

### 4.3 KV Cache 构建（无需改动）

KV Cache 构建逻辑保持不变，只需使用动态常量：

```rust
// 之前（硬编码）
for _ in 0..6 {  // ❌

// 之后（动态）
for _ in 0..Self::NUM_LAYERS {  // ✅
```

---

## 5. 测试计划

### 5.1 单元测试

- [ ] Tokenizer 编码/解码测试
- [ ] 语言 ID 获取测试
- [ ] Encoder 运行测试
- [ ] Decoder 单步测试
- [ ] KV Cache 构建测试

### 5.2 集成测试

- [ ] 完整翻译流程测试（en-zh）
- [ ] 完整翻译流程测试（zh-en）
- [ ] S2S 端到端测试
- [ ] 性能测试（延迟、内存）

### 5.3 质量验证

- [ ] 翻译质量对比测试
- [ ] 边界情况测试（空文本、长文本等）
- [ ] 错误处理测试

---

## 6. 风险评估和缓解

### 6.1 技术风险

| 风险 | 严重程度 | 可能性 | 缓解措施 |
|------|---------|--------|---------|
| **Tokenizer 实现错误** | 高 | 中 | 与 Python 实现对比验证 |
| **模型导出失败** | 高 | 低 | 提前验证导出脚本 |
| **性能下降** | 中 | 中 | 性能测试和优化 |
| **内存不足** | 低 | 低 | 监控内存使用 |

### 6.2 集成风险

| 风险 | 严重程度 | 可能性 | 缓解措施 |
|------|---------|--------|---------|
| **破坏现有功能** | 中 | 低 | 充分测试 |
| **配置错误** | 低 | 中 | 配置验证 |
| **模型文件缺失** | 高 | 低 | 部署前检查 |

---

## 7. 回滚方案

### 7.1 代码回滚

- 保留 Marian 代码在 Git 历史中
- 可以快速切换回 Marian（修改几行代码）

### 7.2 模型回滚

- 保留 Marian 模型文件
- 可以快速切换回 Marian 模型路径

---

## 8. 实施时间表

### 8.1 开发阶段（5-7 天）

| 阶段 | 任务 | 时间 | 负责人 |
|------|------|------|--------|
| Day 1 | 模型导出和准备 | 1 天 | AI 模块 |
| Day 2-3 | Tokenizer 实现 | 2 天 | Dev 团队 |
| Day 4-5 | M2M100NmtOnnx 实现 | 2 天 | Dev 团队 |
| Day 6 | 集成和测试脚本更新 | 1 天 | Dev 团队 |
| Day 7 | 语言对支持和双向模型 | 1 天 | Dev 团队 |

### 8.2 测试阶段（2-3 天）

| 阶段 | 任务 | 时间 |
|------|------|------|
| Day 8 | 单元测试和集成测试 | 1 天 |
| Day 9 | 端到端测试和质量验证 | 1 天 |
| Day 10 | 性能测试和优化 | 1 天 |

---

## 9. 检查清单

### 9.1 模型准备

- [ ] M2M100 encoder 模型已导出
- [ ] M2M100 decoder 模型已导出
- [ ] tokenizer.json 已下载
- [ ] sentencepiece.model 已下载
- [ ] config.json 已下载
- [ ] 模型文件验证通过

### 9.2 代码实现

- [ ] M2M100Tokenizer 实现完成
- [ ] M2M100NmtOnnx 实现完成
- [ ] Encoder 适配完成
- [ ] Decoder 适配完成
- [ ] Translation 适配完成
- [ ] Bootstrap 更新完成
- [ ] 测试脚本更新完成

### 9.3 测试验证

- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] S2S 端到端测试通过
- [ ] 翻译质量验证通过
- [ ] 性能测试通过

---

## 10. 总结

### 10.1 可行性

✅ **直接切换完全可行**

- 架构兼容性好
- 接口保持不变
- 改动范围可控

### 10.2 工作量

- **开发**: 5-7 天
- **测试**: 2-3 天
- **总计**: 7-10 天

### 10.3 风险

- **技术风险**: 中等（主要是 Tokenizer 实现）
- **集成风险**: 低（接口兼容）
- **回滚风险**: 低（可以快速回滚）

### 10.4 建议

✅ **推荐直接切换**

理由：
1. Marian 模型效果无法使用
2. 架构兼容性好，风险可控
3. 可以快速完成（7-10 天）
4. 有完整的回滚方案

---

**最后更新**: 2025-11-21  
**状态**: ✅ **推荐直接切换，预计 7-10 天完成**

