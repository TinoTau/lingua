# M2M100 模型手动导出步骤

如果 PowerShell 脚本有问题，可以按照以下步骤手动导出：

## 前提条件

- Python 3.10 环境已激活
- 依赖已安装：torch 1.13.1+cpu, transformers 4.40.0, onnx 1.14.0, numpy 1.26.4

## 导出步骤

### 1. 导出 en-zh 模型

```bash
# 进入项目根目录
cd D:\Programs\github\lingua

# 导出 Encoder
python docs/models/export_m2m100_encoder.py --output_dir core/engine/models/nmt/m2m100-en-zh --model_id facebook/m2m100_418M

# 导出 Decoder
python docs/models/export_m2m100_decoder_kv.py --output_dir core/engine/models/nmt/m2m100-en-zh --model_id facebook/m2m100_418M

# 下载 Tokenizer 文件
huggingface-cli download facebook/m2m100_418M tokenizer.json sentencepiece.model config.json --local-dir core/engine/models/nmt/m2m100-en-zh
```

### 2. 导出 zh-en 模型

```bash
# 导出 Encoder
python docs/models/export_m2m100_encoder.py --output_dir core/engine/models/nmt/m2m100-zh-en --model_id facebook/m2m100_418M

# 导出 Decoder
python docs/models/export_m2m100_decoder_kv.py --output_dir core/engine/models/nmt/m2m100-zh-en --model_id facebook/m2m100_418M

# 下载 Tokenizer 文件
huggingface-cli download facebook/m2m100_418M tokenizer.json sentencepiece.model config.json --local-dir core/engine/models/nmt/m2m100-zh-en
```

### 3. 验证模型

```bash
python scripts/verify_m2m100_models.py core/engine/models/nmt/m2m100-en-zh core/engine/models/nmt/m2m100-zh-en
```

## 注意事项

1. 确保在项目根目录（`D:\Programs\github\lingua`）执行命令
2. 如果 HuggingFace 需要 token，设置环境变量：`$env:HF_TOKEN="your_token"`
3. 导出可能需要 5-10 分钟，请耐心等待

