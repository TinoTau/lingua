# VITS 中文 AISHELL3 Tokenizer 格式问题

**日期**: 2024-12-19  
**问题**: 生成的音频无法辨认任何一个字，可能是 tokenizer 编码格式不对

---

## 问题确认

### Tokenizer 编码验证
- ✅ 每个字符都正确映射到了声母和韵母
- ✅ 所有 token 都在 tokens.txt 中
- ✅ 编码格式看起来正确：`sil + 声母 + 韵母 + sp + ... + eos`
- ❌ 但生成的音频无法辨认任何一个字

### 可能的原因

1. **Token 顺序问题**
   - 虽然 token 本身正确，但顺序可能不对
   - 可能需要不同的 token 排列方式

2. **分隔符 (sp) 的使用**
   - 当前实现：字之间加 sp
   - 可能需要：每个音节后加 sp，或者不加 sp

3. **特殊 token 的使用**
   - 当前实现：开头 sil，结尾 eos
   - 可能需要：不同的特殊 token 使用方式

4. **模型输入格式**
   - 虽然 tokenizer 编码看起来正确，但模型可能期望不同的格式

---

## 测试方案

创建了 `scripts/test_different_tokenizer_formats.py` 来测试不同的编码格式：

1. **格式1**: 当前实现（字之间加 sp，开头 sil）
2. **格式2**: 每个音节后加 sp
3. **格式3**: 不加 sp（只在开头和结尾）
4. **格式4**: 字之间加 sp，但不在开头加 sil

请运行测试脚本，检查哪种格式能生成正确的音频。

---

## 下一步

1. **运行格式测试脚本**，找出正确的编码格式
2. **检查原始实现的 tokenizer 代码**，确认正确的格式
3. **如果问题仍无法解决**，考虑使用其他中文 TTS 模型

---

## 参考信息

- 原始仓库: https://github.com/csukuangfj/vits_chinese
- 模型来源: https://huggingface.co/jackyqs/vits-aishell3-175-chinese
- ONNX 导出脚本: `export_onnx_aishell3.py`

