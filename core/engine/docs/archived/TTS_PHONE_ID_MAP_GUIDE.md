# TTS Phone ID Map 获取指南

## 问题

FastSpeech2 模型需要 `phone_id_map.txt` 文件来将文本转换为音素 ID 序列。当前模型目录中缺少此文件。

## 什么是 phone_id_map.txt？

`phone_id_map.txt` 是一个音素到 ID 的映射文件，格式如下：

```
<unk> 1
<pad> 0
HH 2
EH 3
L 4
OW 5
...
```

## 获取方法

### 方法 1：从原始模型仓库获取（推荐）

1. **查找模型来源**：
   - 如果模型来自 HuggingFace，检查模型仓库的 `Files` 标签
   - 如果模型来自 GitHub 项目，检查 `preprocess` 或 `config` 目录

2. **常见位置**：
   - `phone_id_map.txt`
   - `preprocessed_data/phone_id_map.txt`
   - `config/phone_id_map.txt`
   - `statistics/phone_id_map.txt`

### 方法 2：从训练代码导出

如果模型是您自己训练的，可以从训练代码中导出：

```python
# 示例：从训练代码导出 phone_id_map.txt
import json

# 假设您有 phone_to_id 字典
phone_to_id = {
    "<unk>": 1,
    "<pad>": 0,
    "HH": 2,
    "EH": 3,
    # ... 更多音素
}

# 写入文件
with open("phone_id_map.txt", "w", encoding="utf-8") as f:
    for phone, phone_id in sorted(phone_to_id.items(), key=lambda x: x[1]):
        f.write(f"{phone} {phone_id}\n")
```

### 方法 3：使用标准音素集（临时方案）

如果无法获取原始映射，可以使用标准音素集创建临时映射：

#### 英文（CMUdict 音素集）

标准英文音素集包含约 40 个音素，如：
- 元音：AA, AE, AH, AO, AW, AY, EH, ER, EY, IH, IY, OW, OY, UH, UW
- 辅音：B, CH, D, DH, F, G, HH, JH, K, L, M, N, NG, P, R, S, SH, T, TH, V, W, Y, Z, ZH

#### 中文（拼音音素集）

中文音素通常基于拼音，如：
- 声母：b, p, m, f, d, t, n, l, g, k, h, j, q, x, zh, ch, sh, r, z, c, s
- 韵母：a, o, e, i, u, ü, ai, ei, ao, ou, an, en, ang, eng, ong, etc.

## 临时解决方案

我已经创建了一个脚本 `scripts/create_phone_id_map.py`，可以生成基本的音素映射文件。

### 使用步骤

1. **运行脚本生成英文音素映射**：
   ```bash
   python scripts/create_phone_id_map.py --lang en --output models/tts/fastspeech2-lite/phone_id_map_en.txt
   ```

2. **运行脚本生成中文音素映射**：
   ```bash
   python scripts/create_phone_id_map.py --lang zh --output models/tts/fastspeech2-lite/phone_id_map_zh.txt
   ```

3. **根据实际模型选择正确的映射文件**：
   - 如果模型是英文的，使用 `phone_id_map_en.txt`
   - 如果模型是中文的，使用 `phone_id_map_zh.txt`
   - 或者重命名为 `phone_id_map.txt`

## 验证

生成映射文件后，运行测试验证：

```bash
cargo test test_tts_synthesize_english -- --nocapture
cargo test test_tts_synthesize_chinese -- --nocapture
```

## 注意事项

⚠️ **重要**：临时生成的音素映射可能与实际模型训练时使用的映射不完全一致，可能导致：
- 音质下降
- 某些音素无法正确识别
- 需要调整文本预处理逻辑

**最佳实践**：尽可能从原始模型仓库或训练代码中获取准确的 `phone_id_map.txt`。

