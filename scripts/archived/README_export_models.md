# Marian NMT 模型导出指南

本目录包含用于从 HuggingFace Marian 模型导出 ONNX 格式的脚本。

## 当前情况

你的项目已经有 `decoder_model.onnx`（即 `model.onnx`），但缺少 `encoder_model.onnx`。
因此，**推荐使用方法 1（只导出 encoder）**。

## 需要的依赖

```bash
pip install -r scripts/requirements_export.txt
# 或者手动安装：
pip install torch transformers onnx onnxruntime "numpy>=1.24.0,<2.2.0" sentencepiece
```

**重要依赖说明**：
- **numpy**: 如果遇到版本冲突（例如与 numba 不兼容），请确保 numpy 版本在 `>=1.24.0,<2.2.0` 范围内
- **sentencepiece**: 必需，用于加载 MarianTokenizer

## 使用方法

### 方法 1：只导出 Encoder（推荐）

如果你已经有 decoder 模型，只需要导出 encoder：

```bash
# 从 HuggingFace 导出 encoder
python scripts/export_marian_encoder.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --verify

# 从本地模型目录导出 encoder
python scripts/export_marian_encoder.py \
    --model_path /path/to/local/marian-model \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --verify
```

这会生成 `encoder_model.onnx`，配合现有的 `model.onnx`（decoder）使用。

### 方法 2：导出完整的 Encoder 和 Decoder

如果你想重新导出 encoder 和 decoder：

```bash
# 导出 en-zh 模型
python scripts/export_marian_onnx.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh

# 导出其他语言对
python scripts/export_marian_onnx.py \
    --model_name Helsinki-NLP/opus-mt-zh-en \
    --output_dir core/engine/models/nmt/marian-zh-en

python scripts/export_marian_onnx.py \
    --model_name Helsinki-NLP/opus-mt-en-es \
    --output_dir core/engine/models/nmt/marian-en-es
```

### 方法 3：使用简化脚本（如果主脚本失败）

```bash
python scripts/export_marian_onnx_simple.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh
```

## 导出的文件

### 使用 `export_marian_encoder.py`（推荐）

只导出 encoder，生成：
- **`encoder_model.onnx`** - Encoder 模型
  - 输入：`input_ids`, `attention_mask`
  - 输出：`last_hidden_state` (encoder_hidden_states)
  - 配合现有的 `model.onnx`（decoder）使用

### 使用 `export_marian_onnx.py`（完整导出）

会生成以下文件：

1. **`encoder_model.onnx`** - Encoder 模型
   - 输入：`input_ids`, `attention_mask`
   - 输出：`last_hidden_state` (encoder_hidden_states)

2. **`decoder_model.onnx`** - Decoder 模型（支持增量解码）
   - 输入：`input_ids`, `encoder_hidden_states`, `encoder_attention_mask`, `past_key_values.*`, `use_cache_branch`
   - 输出：`logits`, `present.*` (更新的 KV cache)

3. **`model_full.onnx`** (可选) - 完整模型（用于验证）

## 可选：量化模型

添加 `--quantize` 参数可以生成 int8 量化版本（更小更快）：

```bash
python scripts/export_marian_onnx.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --quantize
```

这会生成：
- `encoder_model_int8.onnx`
- `decoder_model_int8.onnx`

## 验证导出的模型

### 方法 1：使用脚本自动验证

```bash
# export_marian_encoder.py 会自动验证（使用 --verify 参数）
python scripts/export_marian_encoder.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --verify
```

### 方法 2：使用检查脚本

```bash
# 检查 encoder 模型
python scripts/check_onnx_model.py core/engine/models/nmt/marian-en-zh/encoder_model.onnx

# 检查 decoder 模型（现有的）
python scripts/check_onnx_model.py core/engine/models/nmt/marian-en-zh/model.onnx
```

### 方法 3：使用 Python 手动验证

```bash
python -c "
import onnx
model = onnx.load('core/engine/models/nmt/marian-en-zh/encoder_model.onnx')
print('Encoder inputs:', [inp.name for inp in model.graph.input])
print('Encoder outputs:', [out.name for out in model.graph.output])
"
```

## 注意事项

1. **模型格式**：导出的模型格式可能与当前代码不完全匹配，可能需要调整
2. **ONNX Opset**：脚本使用 opset 14，如果遇到问题可以尝试其他版本
3. **内存需求**：导出大模型可能需要较多内存
4. **时间**：导出过程可能需要几分钟

## 故障排除

### 问题 1：导出失败，提示缺少依赖

```bash
pip install --upgrade torch transformers onnx onnxruntime
```

### 问题 2：导出失败，提示模型不支持

尝试使用简化脚本 `export_marian_onnx_simple.py`

### 问题 3：导出的模型与代码不匹配

可能需要：
1. 调整导出脚本中的输入/输出名称
2. 修改代码以匹配导出的模型格式
3. 使用不同的导出方法

## 导出后的文件结构

导出成功后，模型目录应该包含：

```
core/engine/models/nmt/marian-en-zh/
├── config.json
├── encoder_model.onnx          # 新导出的 encoder
├── model.onnx                  # 现有的 decoder（保持不变）
├── model.onnx_data
├── source.spm
├── target.spm
├── tokenizer_config.json
├── tokenizer.json
└── vocab.json
```

## 下一步

导出成功后：

1. **验证 encoder 模型**：使用 `check_onnx_model.py` 或 `--verify` 参数
2. **更新 Rust 代码**：
   - 在 `MarianNmtOnnx` 中添加 encoder session
   - 实现 encoder 推理逻辑
   - 更新 `translate()` 方法使用真实的 encoder 输出
3. **测试完整流程**：
   - 运行 encoder 获取 `encoder_hidden_states`
   - 使用 `encoder_hidden_states` 运行 decoder
   - 验证翻译结果

## 快速开始示例

```bash
# 1. 安装依赖
pip install -r scripts/requirements_export.txt

# 2. 导出 encoder（推荐，因为你已经有 decoder）
python scripts/export_marian_encoder.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --verify

# 3. 验证导出的模型
python scripts/check_onnx_model.py core/engine/models/nmt/marian-en-zh/encoder_model.onnx

# 4. 更新 Rust 代码以使用 encoder_model.onnx
# 5. 测试完整的翻译流程
```

