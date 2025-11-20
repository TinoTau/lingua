# VITS 中文 AISHELL3 下一步行动

**日期**: 2024-12-19  
**状态**: 模型可以正常推理，但生成的音频无法识别

---

## 当前状态

### 已验证
- ✅ 模型输入输出格式正确
- ✅ 模型可以正常加载和推理
- ✅ 生成的音频文件存在且长度合理
- ✅ Tokenizer 编码逻辑看起来正确
- ❌ **所有格式的音频都无法识别任何一个字**

### 模型信息
- 输入: `x` (int64, [N, L]), `x_length` (int64, [N]), `noise_scale`, `length_scale`, `noise_scale_w`, `sid`
- 输出: `y` (float32, [N, 1, L])
- 输出范围: min=-0.015681, max=0.012289, mean=0.000155（看起来正常）

---

## 可能的问题

1. **Tokenizer 编码方式不对**
   - 虽然编码逻辑看起来正确，但可能与模型训练时使用的格式不一致
   - 需要查看原始实现的 tokenizer 代码

2. **模型本身的问题**
   - 模型可能不适合直接使用
   - 模型可能需要特定的预处理或后处理

3. **模型文件问题**
   - ONNX 模型可能导出有问题
   - 可能需要使用原始 PyTorch 模型进行对比

---

## 下一步行动

### 方案 1: 检查原始实现（推荐）

1. **获取原始 tokenizer 代码**
   ```powershell
   python scripts/fetch_original_vits_tokenizer.py
   ```
   这会尝试从 GitHub 获取原始实现的 tokenizer 代码。

2. **对比我们的实现和原始实现**
   - 检查 tokenizer 的编码逻辑
   - 确认是否有遗漏的预处理步骤
   - 确认 token 格式是否正确

3. **如果找到差异，修复我们的实现**

### 方案 2: 暂时只支持英文 TTS

如果原始实现无法获取或问题无法解决：
1. 暂时禁用中文 TTS
2. 只支持英文 TTS（MMS TTS 已验证可用）
3. 后续再添加中文 TTS 支持

### 方案 3: 寻找其他中文 TTS 模型

如果原始实现无法解决问题：
1. 搜索其他中文 TTS 模型
2. 测试其他模型
3. 如果可用，集成到现有系统

---

## 建议

**推荐**: **方案 1（检查原始实现）**

**理由**:
- 模型可以正常推理，说明模型本身可能没问题
- 问题很可能在 tokenizer 编码方式上
- 查看原始实现可以快速定位问题

---

## 参考信息

- 原始仓库: https://github.com/csukuangfj/vits_chinese
- 模型来源: https://huggingface.co/jackyqs/vits-aishell3-175-chinese
- ONNX 导出脚本: `export_onnx_aishell3.py`

