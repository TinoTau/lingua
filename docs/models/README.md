# M2M100 NMT 模型切换文档索引

**最后更新**: 2025-11-21

---

## 📚 文档导航

### 🎯 开始开发
**👉 [M2M100_DEVELOPMENT_READY.md](./M2M100_DEVELOPMENT_READY.md)** ⭐ **开发准备清单**

这是开发人员应该首先阅读的文档，包含：
- 完整的开发步骤
- 关键参数对比
- 常见问题快速参考
- 开发前检查清单

---

### 📖 核心文档

#### 1. **M2M100_Switch_Implementation_Document.md** ⭐ **主实施文档**
- **用途**: 完整的技术实施文档
- **内容**:
  - 背景与目标
  - 模型架构参数
  - Tokenizer 说明
  - ONNX Decoder 调用流程
  - 全量替换步骤
  - 模型导出步骤
  - 常见问题与解决方案
  - 验收标准
- **适合**: 需要了解完整技术细节的开发人员

#### 2. **M2M100_DIRECT_SWITCH_PLAN.md**
- **用途**: 详细的实施计划和时间表
- **内容**:
  - 切换策略
  - 详细实施步骤（Phase 1-5）
  - 代码改动清单
  - 关键技术点
  - 测试计划
  - 风险评估
- **适合**: 项目管理和进度跟踪

#### 3. **M2M100_Tokenizer_Implementation_Guide.md**
- **用途**: Tokenizer 实现详细指南
- **内容**:
  - 基本概念
  - 所需文件
  - 编码/解码流程
  - 实现方案（Python 服务 vs Rust）
  - 实现检查清单
- **适合**: 负责 Tokenizer 实现的开发人员

#### 4. **M2M100_MIGRATION_IMPACT_ASSESSMENT.md**
- **用途**: 迁移影响评估
- **内容**:
  - 架构影响分析
  - 代码改动评估
  - 功能影响分析
  - 风险评估
  - 推荐方案
- **适合**: 技术决策者和架构师

#### 5. **NMT_Model_Upgrade_Technical_Plan.md**
- **用途**: 原始升级技术计划
- **内容**:
  - 升级背景
  - 推荐模型（M2M100-418M）
  - 目录布局
  - 迁移策略
  - 工作项和交付物
- **适合**: 了解升级背景和决策过程

---

### 🛠️ 工具脚本

#### 1. **export_m2m100_encoder.py**
- **用途**: 导出 M2M100 Encoder 到 ONNX
- **用法**:
  ```bash
  python export_m2m100_encoder.py \
      --output_dir core/engine/models/nmt/m2m100-en-zh \
      --model_id facebook/m2m100_418M
  ```
- **输出**: `encoder.onnx`

#### 2. **export_m2m100_decoder_kv.py**
- **用途**: 导出 M2M100 Decoder + KV Cache 到 ONNX
- **用法**:
  ```bash
  python export_m2m100_decoder_kv.py \
      --output_dir core/engine/models/nmt/m2m100-en-zh \
      --model_id facebook/m2m100_418M
  ```
- **输出**: `decoder.onnx`（52 输入，49 输出）

---

## 🗺️ 文档阅读路径

### 对于开发人员
1. **开始**: [M2M100_DEVELOPMENT_READY.md](./M2M100_DEVELOPMENT_READY.md) - 开发准备清单
2. **深入**: [M2M100_Switch_Implementation_Document.md](./M2M100_Switch_Implementation_Document.md) - 主实施文档
3. **专项**: [M2M100_Tokenizer_Implementation_Guide.md](./M2M100_Tokenizer_Implementation_Guide.md) - Tokenizer 指南

### 对于项目经理
1. **概览**: [M2M100_DIRECT_SWITCH_PLAN.md](./M2M100_DIRECT_SWITCH_PLAN.md) - 实施计划
2. **影响**: [M2M100_MIGRATION_IMPACT_ASSESSMENT.md](./M2M100_MIGRATION_IMPACT_ASSESSMENT.md) - 影响评估

### 对于技术决策者
1. **背景**: [NMT_Model_Upgrade_Technical_Plan.md](./NMT_Model_Upgrade_Technical_Plan.md) - 升级计划
2. **影响**: [M2M100_MIGRATION_IMPACT_ASSESSMENT.md](./M2M100_MIGRATION_IMPACT_ASSESSMENT.md) - 影响评估

---

## 📊 关键信息速查

### 参数对比
| 配置项 | Marian | M2M100 |
|--------|--------|--------|
| 层数 | 6 | **12** |
| 注意力头数 | 8 | **16** |
| 隐藏维度 | 512 | **1024** |
| Decoder 输入 | 28 | **52** |
| Decoder 输出 | 25 | **49** |

### 开发时间估算
- **Phase 0**: 模型导出（1 天）
- **Phase 1**: Tokenizer 实现（2 天）
- **Phase 2**: M2M100NmtOnnx 实现（2 天）
- **Phase 3**: 集成代码更新（1 天）
- **Phase 4**: 测试和验证（2-3 天）
- **总计**: 7-10 天

---

## ⚠️ 重要提醒

1. **输入/输出数量**: M2M100 Decoder 是 52 输入/49 输出（不是 28/25）
2. **维度变化**: Encoder 输出是 `[batch, seq, 1024]`（不是 512）
3. **KV Cache**: 必须使用动态常量 `Self::NUM_LAYERS`（不能硬编码）
4. **语言 Token**: 必须使用 `get_lang_id()`，不能手写字符串
5. **ONNX IR**: 必须 <= 9（兼容 ort 1.16.3）

---

## 🔗 相关文档

- [系统架构文档](../../architecture/SYSTEM_ARCHITECTURE_OVERVIEW.md)
- [项目概览](../../PROJECT_OVERVIEW.md)
- [集成测试文档](../../architecture/S2S_INTEGRATION_TEST_PASSED.md)

---

**状态**: ✅ **文档已就绪，可以开始开发**

