# VITS TTS 中文模型下载指南

## 当前状态

- ✅ **英文模型**：已下载并测试通过（`mms-tts-eng`）
- ❌ **中文模型**：尚未下载

## 中文模型下载

### 方法 1：使用 Hugging Face（推荐）

MMS TTS 中文模型可能位于：
- `facebook/mms-tts-zho`（如果存在）
- `Xenova/mms-tts-zho`（ONNX 版本，如果存在）

**下载命令**：

```powershell
# 安装 git-lfs（如果还没有）
git lfs install

# 克隆中文模型仓库
cd D:\Programs\github\lingua\core\engine\models\tts
git clone https://huggingface.co/Xenova/mms-tts-zho mms-tts-zho
```

### 方法 2：检查 MMS TTS 官方仓库

1. 访问 https://huggingface.co/facebook/mms-tts
2. 查找中文（zho/cmn）模型
3. 下载对应的 ONNX 版本

### 方法 3：使用其他中文 TTS 模型

如果 MMS TTS 没有中文模型，可以考虑：
- 使用其他 VITS 中文模型（如 VITS-CN）
- 或继续使用 FastSpeech2 + HiFiGAN（如果模型可用）

## 模型目录结构

下载后，目录结构应该是：

```
core/engine/models/tts/
├── mms-tts-eng/          # 英文模型（已下载）
│   ├── onnx/
│   │   └── model.onnx
│   ├── tokenizer.json
│   └── config.json
└── mms-tts-zho/          # 中文模型（待下载）
    ├── onnx/
    │   └── model.onnx
    ├── tokenizer.json
    └── config.json
```

## 验证模型

下载后，运行 Python 测试脚本验证：

```powershell
python scripts/test_mms_tts_onnx.py
```

修改脚本中的模型路径为中文模型路径进行测试。

## 下一步

下载中文模型后，需要：
1. 修改 `VitsTtsEngine` 支持多语言模型选择
2. 根据 `TtsRequest.locale` 选择对应的模型
3. 测试中文语音合成

