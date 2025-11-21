# Marian zh-en IR 7 模型运行时问题报告

**日期**: 2025-11-21  
**报告人**: 开发团队  
**状态**: 🔴 运行时错误，需要进一步调试

---

## 1. 执行摘要

### 1.1 背景

为了解决 Marian NMT 模型的 ONNX IR 版本兼容性问题（IR 10 vs IR 9），我们成功导出了 IR 7 版本的 `marian-zh-en` 模型。模型导出和加载均成功，但在实际运行时遇到了访问违规错误。

### 1.2 问题概述

- **问题**: Decoder 模型在第一步运行时发生访问违规错误（STATUS_ACCESS_VIOLATION）
- **影响**: 无法完成完整的 S2S（语音转语音）翻译流程
- **严重程度**: 🔴 高 - 阻塞核心功能

---

## 2. 已完成的工作

### 2.1 模型导出 ✅

1. **环境准备**
   - 创建 Python 3.10 虚拟环境
   - 安装依赖：torch==1.13.1+cpu, transformers==4.40.0, onnx==1.14.0
   - 成功导出 Encoder 和 Decoder 模型

2. **模型验证**
   - ✅ IR 版本: 7（兼容 ort 1.16.3）
   - ✅ Opset 版本: 12
   - ✅ Encoder 模型: 209 MB，成功加载
   - ✅ Decoder 模型: 368 MB，成功加载
   - ✅ 输入输出数量: 28 输入，25 输出（匹配代码期望）
   - ✅ 输入输出名称: 完全匹配现有 `marian-en-zh` 模型格式

3. **模型部署**
   - ✅ 备份原有 IR 10 模型到 `marian-zh-en.ir10.backup`
   - ✅ 部署新 IR 7 模型到 `core/engine/models/nmt/marian-zh-en`
   - ✅ 恢复必要的配置文件（tokenizer, vocab, config 等）

### 2.2 系统集成测试 ✅

1. **组件验证**
   - ✅ Piper HTTP 服务: 正常运行
   - ✅ Whisper ASR: 成功加载和识别
   - ✅ Marian NMT Encoder: 成功加载和运行
   - ✅ Marian NMT Decoder: 成功加载（模型结构验证通过）

2. **ASR 识别**
   - ✅ 成功识别英文音频
   - ✅ 识别结果: "Hello welcome. Hello welcome to the video. Welcome to the video. Hello welcome to the video of the video. Hello, welcome to the \"Lenlo Yu\" movie series."

3. **NMT 翻译**
   - ✅ Encoder 成功运行
   - ✅ 源文本成功编码
   - ✅ Encoder 输出形状正确: [1, 29, 512]

---

## 3. 遇到的问题

### 3.1 错误详情

**错误类型**: 访问违规错误（Access Violation）  
**错误代码**: `STATUS_ACCESS_VIOLATION (0xc0000005)`  
**发生位置**: Decoder 模型的第一步推理（`decoder_step`）  
**发生时机**: `use_cache_branch=false`（第一步，无历史 KV cache）

### 3.2 错误日志

```
[6/7] 执行 NMT 翻译...
Source text: 'Hello welcome. Hello welcome to the video. Welcome to the video. Hello welcome to the video of the video. Hello, welcome to the \"Lenlo Yu\" movie series.'
Encoded source IDs: [0, 3833, 0, 3833, 2904, 8, 3, 0, 12904, 8, 3, 0, 3833, 2904, 8, 3, 9011, 4, 3, 0, 0, 2904, 8, 3, 0, 0, 15807, 0, 0] (length: 29)
Encoder output shape: [1, 29, 512]
[DEBUG] Step 0: decoder_input_ids=[65000] (length: 1), use_cache_branch=false, has_decoder_kv=false
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_decoder_kv=false
[decoder_step] input_ids shape: [1, 1]
error: process didn't exit successfully: `target\debug\examples\test_s2s_full_simple.exe ..\..\test_output\s2s_flow_test.wav` (exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)
```

### 3.3 错误分析

错误发生在以下位置：
- **函数**: `decoder_step`（`core/engine/src/nmt_incremental/decoder.rs`）
- **步骤**: 第一步推理（`use_cache_branch=false`）
- **操作**: 调用 `decoder_session.run(input_values)` 时

---

## 4. 可能的原因分析

### 4.1 模型兼容性问题 ⚠️

**假设**: 新导出的 IR 7 模型与代码期望不完全匹配

**证据**:
- 模型结构验证通过（输入输出数量、名称匹配）
- 但运行时发生内存访问错误

**可能原因**:
1. 模型内部操作（operators）与 ort 1.16.3 不完全兼容
2. 某些操作在 opset 12 下的行为与预期不同
3. KV cache 的形状或布局与代码期望不匹配

### 4.2 KV Cache 输入问题 ⚠️

**假设**: KV cache 输入的形状或类型不正确

**证据**:
- 错误发生在第一步（`use_cache_branch=false`）
- 此时使用零占位符作为 KV cache 输入

**可能原因**:
1. 零占位符的形状与模型期望不匹配
2. KV cache 的维度顺序或数据类型不正确
3. 静态 encoder KV 占位符的形状问题

### 4.3 输入数据格式问题 ⚠️

**假设**: 输入数据的格式或类型不正确

**证据**:
- Encoder 运行成功，但 Decoder 失败
- 输入数据从 Encoder 输出转换而来

**可能原因**:
1. `encoder_hidden_states` 的数据格式不正确
2. `encoder_attention_mask` 的类型或形状问题
3. `decoder_input_ids` 的格式问题

### 4.4 ONNX Runtime 版本问题 ⚠️

**假设**: ort 1.16.3 与 IR 7 模型存在兼容性问题

**证据**:
- 模型导出使用 onnx 1.14.0
- 运行时使用 ort 1.16.3

**可能原因**:
1. ort 1.16.3 对某些 opset 12 操作的支持不完整
2. 内存管理或生命周期问题

---

## 5. 对比分析

### 5.1 现有工作模型（marian-en-zh）

- **IR 版本**: 10
- **Opset 版本**: 14
- **状态**: ✅ 正常工作
- **导出方式**: 使用 `scripts/export_marian_onnx.py`（opset 14）

### 5.2 新导出模型（marian-zh-en IR 7）

- **IR 版本**: 7
- **Opset 版本**: 12
- **状态**: ❌ 运行时错误
- **导出方式**: 使用修复后的导出脚本（opset 12）

### 5.3 关键差异

| 项目 | marian-en-zh (工作) | marian-zh-en IR 7 (失败) |
|------|---------------------|--------------------------|
| IR 版本 | 10 | 7 |
| Opset 版本 | 14 | 12 |
| 导出脚本 | `export_marian_onnx.py` | `export_marian_decoder_ir9_fixed.py` |
| 导出环境 | 未知 | Python 3.10, torch 1.13.1 |
| 运行时 | ✅ 正常 | ❌ 访问违规 |

---

## 6. 影响评估

### 6.1 功能影响

- **阻塞功能**: 完整的 S2S 翻译流程（中文→英文）
- **受影响模块**: Marian NMT Decoder
- **其他功能**: ASR、Encoder、TTS 均正常

### 6.2 时间影响

- **已投入时间**: 约 1 天（环境准备、模型导出、测试）
- **预计修复时间**: 未知（需要进一步调试）

### 6.3 风险影响

- **技术风险**: 中等 - 可能需要调整导出脚本或代码
- **进度风险**: 中等 - 可能影响项目进度
- **质量风险**: 低 - 不影响其他已工作功能

---

## 7. 建议的解决方案

### 7.1 方案 1: 调试和修复当前模型 ⭐ 推荐

**步骤**:
1. 使用调试工具（如 WinDbg）分析访问违规的具体位置
2. 对比新模型与工作模型的内部结构差异
3. 检查 KV cache 输入的形状和类型
4. 验证输入数据的格式

**优点**:
- 保持 IR 7 兼容性
- 不需要升级 ort

**缺点**:
- 需要深入调试
- 时间不确定

**预计时间**: 1-3 天

### 7.2 方案 2: 使用现有工作模型（marian-en-zh）

**步骤**:
1. 使用 `marian-en-zh` 模型（英文→中文）
2. 调整测试流程为英文→中文

**优点**:
- 立即可用
- 已验证工作

**缺点**:
- 不符合原始需求（中文→英文）
- 只是临时方案

**预计时间**: 0.5 天

### 7.3 方案 3: 升级 ONNX Runtime

**步骤**:
1. 升级 ort 到 2.0+（支持 IR 10）
2. 使用原始 IR 10 模型

**优点**:
- 使用最新技术
- 支持更多模型

**缺点**:
- 高风险（历史内存安全问题）
- 影响范围大
- 需要全面测试

**预计时间**: 3-5 天（含测试）

### 7.4 方案 4: 重新导出模型（调整参数）

**步骤**:
1. 调整导出脚本的参数
2. 尝试不同的 opset 版本或导出选项
3. 重新导出和测试

**优点**:
- 可能解决兼容性问题
- 保持 IR 7

**缺点**:
- 需要多次尝试
- 时间不确定

**预计时间**: 1-2 天

---

## 8. 推荐行动

### 8.1 短期（立即）

1. **深入调试**（方案 1）
   - 使用调试工具分析访问违规
   - 对比工作模型与新模型的差异
   - 检查输入数据的格式和类型

2. **临时方案**（方案 2）
   - 使用 `marian-en-zh` 进行功能验证
   - 确保其他组件正常工作

### 8.2 中期（1-2 周）

1. **继续调试**
   - 如果方案 1 无进展，考虑方案 4
   - 尝试不同的导出参数

2. **评估升级**
   - 如果调试无果，评估方案 3 的风险和收益

### 8.3 长期（1 个月+）

1. **技术决策**
   - 根据调试结果决定最终方案
   - 考虑长期维护成本

---

## 9. 技术细节

### 9.1 模型信息

**Encoder 模型**:
- 文件: `encoder_model.onnx`
- 大小: 209 MB
- IR 版本: 7
- Opset 版本: 12
- 输入: `input_ids`, `attention_mask`
- 输出: `last_hidden_state`

**Decoder 模型**:
- 文件: `model.onnx`
- 大小: 368 MB
- IR 版本: 7
- Opset 版本: 12
- 输入: 28 个（3 基础 + 24 KV cache + 1 use_cache_branch）
- 输出: 25 个（1 logits + 24 present KV cache）

### 9.2 导出脚本

- **Encoder**: `export_marian_encoder_ir9.py`
- **Decoder**: `export_marian_decoder_ir9_fixed.py`
- **环境**: Python 3.10, torch 1.13.1+cpu, transformers 4.40.0, onnx 1.14.0

### 9.3 运行时环境

- **ONNX Runtime**: ort 1.16.3
- **Rust 版本**: 最新稳定版
- **操作系统**: Windows 10/11
- **架构**: x86_64

---

## 10. 结论

### 10.1 当前状态

- ✅ **模型导出**: 成功
- ✅ **模型加载**: 成功
- ✅ **系统集成**: 部分成功（ASR、Encoder 正常）
- ❌ **Decoder 推理**: 失败（访问违规错误）

### 10.2 关键发现

1. IR 7 模型可以成功导出和加载
2. 模型结构与代码期望匹配
3. 但运行时存在兼容性问题

### 10.3 下一步

1. **立即**: 深入调试访问违规错误
2. **备选**: 使用临时方案（marian-en-zh）进行功能验证
3. **评估**: 根据调试结果决定最终方案

---

## 11. 附录

### 11.1 相关文件

- 导出脚本: `export_marian_encoder_ir9.py`, `export_marian_decoder_ir9_fixed.py`
- 模型位置: `core/engine/models/nmt/marian-zh-en/`
- 备份位置: `core/engine/models/nmt/marian-zh-en.ir10.backup/`
- 测试程序: `core/engine/examples/test_s2s_full_simple.rs`

### 11.2 参考文档

- `MARIAN_ZH_EN_IR9_EXPORT_GUIDE.md` - 导出操作指南
- `MARIAN_ZH_EN_IR9_EXPORT_PLAN_V2_ANALYSIS.md` - 方案分析
- `MARIAN_ZH_EN_IR9_EXPORT_FIXED_ANALYSIS.md` - 修复版分析

---

**报告生成时间**: 2025-11-21  
**状态**: 🔴 待决策  
**优先级**: P0（阻塞核心功能）

