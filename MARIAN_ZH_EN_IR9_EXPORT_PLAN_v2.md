# MARIAN_ZH_EN_IR9_EXPORT_PLAN_v2.md

本方案用于指导开发人员：在 **不升级 ONNX Runtime（固定 1.16.3）** 的前提下，  
将 `marian-zh-en` 模型重新导出为 **IR 版本 ≤ 9、opset 12** 的 **分离 encoder / decoder ONNX 模型**：

- `encoder_model.onnx`（Encoder，IR≤9，opset 12）
- `model.onnx`（Decoder+LM head，IR≤9，opset 12）

以兼容当前 NMT 架构（增量解码 / KV cache 由现有 Rust / Node 逻辑负责，导出脚本本身只提供标准前向图）。

目标目录结构：

```text
core/
  engine/
    models/
      nmt/
        marian-zh-en/
          encoder_model.onnx      # 新 IR9 Encoder
          model.onnx              # 新 IR9 Decoder+LM head
          tokenizer.json          # 可选：HF tokenizer
          config.json             # 可选：HF config
```

---

后续内容略（开发可根据实际需要补充说明）。
