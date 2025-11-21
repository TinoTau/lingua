# Marian zh-en IR 9 模型导出操作指南

**日期**: 2025-11-21  
**目标**: 在 Python 3.10 环境中导出 IR ≤ 9, opset 12 的 Marian NMT 模型

---

## 1. 准备 Python 3.10 环境

### 1.1 创建虚拟环境（推荐）

```bash
# 创建 Python 3.10 虚拟环境
python3.10 -m venv venv_marian_export

# 激活虚拟环境
# Windows:
venv_marian_export\Scripts\activate
# Linux/Mac:
source venv_marian_export/bin/activate
```

### 1.2 安装依赖

```bash
pip install torch==1.13.1+cpu -f https://download.pytorch.org/whl/torch_stable.html
pip install transformers==4.40.0
pip install onnx==1.14.0
pip install onnxruntime==1.16.3  # 可选，用于验证
```

**注意**: 
- 如果无法安装 `torch==1.13.1+cpu`，可以尝试 `torch==1.13.1`（CPU 版本）
- 确保 `onnx` 版本为 `1.14.0`，以确保导出的模型 IR 版本 ≤ 9

---

## 2. 需要复制的文件

### 2.1 必需文件

将以下文件复制到 Python 3.10 环境中（或直接在项目目录中执行）：

1. **`export_marian_encoder_ir9.py`** - Encoder 导出脚本
2. **`export_marian_decoder_ir9_fixed.py`** - Decoder 导出脚本（修复版）

### 2.2 文件位置

这些文件位于项目根目录：
```
D:\Programs\github\lingua\
├── export_marian_encoder_ir9.py
└── export_marian_decoder_ir9_fixed.py
```

**建议**: 直接在项目目录中执行，这样导出的模型会直接保存到正确的位置。

---

## 3. 执行步骤

### 3.1 步骤 1: 导出 Encoder

```bash
# 确保在项目根目录
cd D:\Programs\github\lingua

# 激活 Python 3.10 虚拟环境
# Windows:
venv_marian_export\Scripts\activate
# Linux/Mac:
source venv_marian_export/bin/activate

# 执行 Encoder 导出
python export_marian_encoder_ir9.py --output_dir core/engine/models/nmt/marian-zh-en
```

**预期输出**:
```
[INFO] Using model_id: Helsinki-NLP/opus-mt-zh-en
[INFO] Output dir: core/engine/models/nmt/marian-zh-en
[1/4] Loading tokenizer and model ...
[2/4] Preparing dummy inputs ...
[3/4] Exporting encoder ONNX (opset_version=12) ...
[INFO] Encoder ONNX model saved to: core/engine/models/nmt/marian-zh-en/encoder_model.onnx
[4/4] Inspecting ONNX encoder model IR/opset ...
    IR version: 7
    Opset imports:
      domain='' version=12
[DONE] Encoder export finished and model is valid.
```

**验证**:
- 检查文件是否存在: `core/engine/models/nmt/marian-zh-en/encoder_model.onnx`
- 检查 IR 版本应该是 ≤ 9

### 3.2 步骤 2: 导出 Decoder

```bash
# 继续在项目根目录，确保虚拟环境已激活

# 执行 Decoder 导出
python export_marian_decoder_ir9_fixed.py --output_dir core/engine/models/nmt/marian-zh-en
```

**预期输出**:
```
[INFO] Using model_id: Helsinki-NLP/opus-mt-zh-en
[INFO] Output dir: core/engine/models/nmt/marian-zh-en
[1/5] Loading tokenizer and model ...
[INFO] Using num_kv_layers = 6
[2/5] Preparing dummy inputs ...
[3/5] Building input/output names ...
    #inputs = 28, #outputs = 25
[4/5] Exporting decoder ONNX (opset_version=12) ...
[INFO] Decoder ONNX model saved to: core/engine/models/nmt/marian-zh-en/model.onnx
[5/5] Inspecting ONNX decoder model IR/opset ...
    IR version: 7
    Opset imports:
      domain='' version=12
[DONE] Decoder export with KV cache finished and model is valid.
```

**验证**:
- 检查文件是否存在: `core/engine/models/nmt/marian-zh-en/model.onnx`
- 检查 IR 版本应该是 ≤ 9
- 检查输入输出数量：28 个输入，25 个输出

---

## 4. 验证导出的模型

### 4.1 检查 IR 版本和 Opset

```bash
python -c "import onnx; m = onnx.load('core/engine/models/nmt/marian-zh-en/encoder_model.onnx'); print(f'Encoder - IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
python -c "import onnx; m = onnx.load('core/engine/models/nmt/marian-zh-en/model.onnx'); print(f'Decoder - IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
```

**预期结果**:
- IR version: ≤ 9（通常是 7 或 8）
- Opset version: 12

### 4.2 检查模型结构

```bash
python -c "
import onnxruntime as ort
sess = ort.InferenceSession('core/engine/models/nmt/marian-zh-en/model.onnx', providers=['CPUExecutionProvider'])
print('Decoder Inputs:', len(sess.get_inputs()))
print('Decoder Outputs:', len(sess.get_outputs()))
print('')
print('Input names (first 5):')
for i in sess.get_inputs()[:5]:
    print(f'  {i.name}: {i.shape}')
print('')
print('Output names (first 5):')
for o in sess.get_outputs()[:5]:
    print(f'  {o.name}: {o.shape}')
"
```

**预期结果**:
- Decoder Inputs: 28
- Decoder Outputs: 25
- 输入名称格式: `past_key_values.0.decoder.key`, `past_key_values.0.decoder.value`, 等
- 输出名称格式: `present.0.decoder.key`, `present.0.decoder.value`, 等

### 4.3 对比现有模型（可选）

```bash
python -c "
import onnxruntime as ort
sess_old = ort.InferenceSession('core/engine/models/nmt/marian-en-zh/model.onnx', providers=['CPUExecutionProvider'])
sess_new = ort.InferenceSession('core/engine/models/nmt/marian-zh-en/model.onnx', providers=['CPUExecutionProvider'])
print('Existing model (marian-en-zh):')
print(f'  Inputs: {len(sess_old.get_inputs())}, Outputs: {len(sess_old.get_outputs())}')
print('New model (marian-zh-en):')
print(f'  Inputs: {len(sess_new.get_inputs())}, Outputs: {len(sess_new.get_outputs())}')
print('')
print('Input names match:', [i.name for i in sess_old.get_inputs()] == [i.name for i in sess_new.get_inputs()])
print('Output names match:', [o.name for o in sess_old.get_outputs()] == [o.name for o in sess_new.get_outputs()])
"
```

---

## 5. 测试导出的模型

### 5.1 运行 S2S 测试

```bash
# 返回项目根目录，使用正常的 Rust 环境
cd D:\Programs\github\lingua

# 运行完整 S2S 测试
cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
```

**预期结果**:
- 模型成功加载
- 翻译功能正常
- 生成音频文件

---

## 6. 常见问题

### 6.1 模型下载失败

如果 Hugging Face 模型下载失败，可以：

1. **使用镜像**:
   ```bash
   export HF_ENDPOINT=https://hf-mirror.com
   ```

2. **手动下载**: 从 Hugging Face 网站下载模型到本地，然后使用 `--model_id` 参数指定本地路径

### 6.2 IR 版本不是 ≤ 9

如果导出的模型 IR 版本 > 9：

1. 检查 `onnx` 版本是否为 `1.14.0`
2. 检查 `torch` 版本是否为 `1.13.1`
3. 确保使用 `opset_version=12`

### 6.3 输入输出数量不匹配

如果 Decoder 模型的输入输出数量不对：

1. 检查 `num_kv_layers` 是否正确（应该是 6）
2. 检查脚本是否正确执行
3. 查看错误信息

### 6.4 模型加载失败

如果 Rust 代码无法加载模型：

1. 检查文件路径是否正确
2. 检查 IR 版本是否 ≤ 9
3. 检查输入输出名称是否匹配
4. 查看 Rust 错误信息

---

## 7. 文件清单

### 7.1 需要复制的文件

- ✅ `export_marian_encoder_ir9.py` - Encoder 导出脚本
- ✅ `export_marian_decoder_ir9_fixed.py` - Decoder 导出脚本

### 7.2 导出的文件

导出完成后，以下文件应该存在于 `core/engine/models/nmt/marian-zh-en/` 目录：

- ✅ `encoder_model.onnx` - Encoder 模型（IR ≤ 9, opset 12）
- ✅ `model.onnx` - Decoder 模型（IR ≤ 9, opset 12）
- ✅ `tokenizer.json` - Tokenizer（如果存在）
- ✅ `config.json` - 模型配置（如果存在）

---

## 8. 快速参考

### 8.1 完整命令序列

```bash
# 1. 创建并激活虚拟环境
python3.10 -m venv venv_marian_export
venv_marian_export\Scripts\activate  # Windows
# source venv_marian_export/bin/activate  # Linux/Mac

# 2. 安装依赖
pip install torch==1.13.1+cpu -f https://download.pytorch.org/whl/torch_stable.html
pip install transformers==4.40.0
pip install onnx==1.14.0

# 3. 进入项目目录
cd D:\Programs\github\lingua

# 4. 导出 Encoder
python export_marian_encoder_ir9.py --output_dir core/engine/models/nmt/marian-zh-en

# 5. 导出 Decoder
python export_marian_decoder_ir9_fixed.py --output_dir core/engine/models/nmt/marian-zh-en

# 6. 验证模型
python -c "import onnx; m = onnx.load('core/engine/models/nmt/marian-zh-en/model.onnx'); print(f'IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
```

---

**最后更新**: 2025-11-21  
**状态**: ✅ 操作指南已准备就绪

