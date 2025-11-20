# VITS 中文模型实现计划

## 模型信息

- **模型名称**: `csukuangfj/vits-zh-aishell3`
- **模型类型**: VITS 中文多说话人 TTS
- **ONNX 文件**: `vits-aishell3.onnx` 或 `vits-aishell3.int8.onnx`

---

## 模型输入输出

### 输入：
1. **x**: `tensor(int64)`, shape `[N, L]` - 文本 token IDs
2. **x_length**: `tensor(int64)`, shape `[N]` - 序列长度
3. **noise_scale**: `tensor(float)`, shape `[1]` - 噪声缩放（默认 0.667）
4. **length_scale**: `tensor(float)`, shape `[1]` - 长度缩放（默认 1.0，>1.0 变慢，<1.0 变快）
5. **noise_scale_w**: `tensor(float)`, shape `[1]` - 噪声缩放 w（默认 0.8）
6. **sid**: `tensor(int64)`, shape `[1]` - 说话人 ID（0-N，N 为说话人数量）

### 输出：
- **y**: `tensor(float)`, shape `[N, 1, L]` - 音频波形（需要 squeeze 成 [L]）

---

## 实现步骤

### 1. 检查 tokens.txt 格式
- 确认 token 到 ID 的映射关系
- 确认是否需要拼音转换

### 2. 实现中文 Tokenizer
- 根据 tokens.txt 创建 token 映射
- 实现文本到 token IDs 的转换
- 可能需要：
  - 中文分词
  - 拼音转换（如果 tokens 是拼音）
  - 音素转换（如果 tokens 是音素）

### 3. 修改 VitsTtsEngine
- 添加对 vits-zh-aishell3 格式的支持
- 实现新的 `run_inference_zh_aishell3` 方法
- 处理不同的输入格式（x, x_length, noise_scale 等）

### 4. 多说话人支持
- 从模型或配置中获取说话人数量
- 支持在 TtsRequest 中指定说话人 ID

### 5. 测试
- 创建测试用例
- 验证音频输出质量

---

## 代码结构

### 选项 1：扩展现有 VitsTtsEngine
- 在 `VitsTtsEngine` 中添加模型类型检测
- 根据模型类型选择不同的推理方法

### 选项 2：创建新的引擎
- 创建 `VitsZhAishell3Engine` 专门处理中文模型
- 实现 `TtsStreaming` trait

**推荐选项 1**，因为都是 VITS 模型，可以共享大部分代码。

---

## 待确认信息

1. **tokens.txt 格式**：需要手动检查
2. **说话人数量**：需要从模型或配置中获取
3. **文本预处理**：是否需要拼音转换工具

---

## 参考

- 模型仓库：https://huggingface.co/csukuangfj/vits-zh-aishell3
- 可能需要参考原始实现或文档

