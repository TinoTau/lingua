# Emotion Adapter 技术方案（面向开发人员）

**版本**: 1.0  
**适用模块**: Lingua Core Engine - Emotion Adapter  
**作者**: ChatGPT  
**文档类型**: 技术实现方案（可直接用于开发）

---

# 1. 背景与目标

Emotion Adapter 是 Lingua 多模态实时翻译系统中的情绪识别模块，负责在 ASR 后、NMT 前分析输入的文本情绪，用于：

- 改善 TTS 情绪风格
- 提供 UI 情绪显示
- 未来的 Persona 情绪调节

当前代码已实现全部逻辑，但存在 **ONNX Runtime 无法加载 IR10 模型** 的问题。该文档提供完整、可落地的解决方案：

> **使用旧版本 PyTorch（1.13.1）重新导出 IR9 + opset 12 的情绪识别模型，使其兼容 ORT 1.16.3。**

---

# 2. 情绪识别系统架构

```
Audio → VAD → ASR → Emotion Adapter → Persona Adapter → NMT → TTS
```

Emotion Adapter 的输入输出：

- 输入：ASR 文本（UTF-8 string）
- 输出：
  ```json
  {
    "emotion": "neutral | joy | sadness | anger | fear | surprise",
    "intensity": 0.0 - 1.0,
    "confidence": 0.0 - 1.0
  }
  ```

---

# 3. 当前问题总结

❌ 原模型为 IR10 + opset 18  
❌ ORT 1.16.3 仅支持 IR ≤ 9  
❌ 所有降级尝试失败  
➡ **必须重新导出模型**

---

# 4. 解决方案：使用 PyTorch 1.13 导出 IR9 模型

优点：

- 完全兼容 ORT 1.16.3  
- 不需要手动降级  
- 模型行为一致  
- 稳定可复现  

---

# 5. 实施步骤

## Step 1：创建虚拟环境
```
conda create -n emotion_ir9 python=3.10 -y
conda activate emotion_ir9
```

## Step 2：安装依赖
```
pip install torch==1.13.1 torchvision torchaudio
pip install transformers onnx
```

## Step 3：导出 IR9 模型
（脚本略，同前文）

## Step 4：验证 IR 版本
应输出：
```
IR: 9
Opset: 12
```

---

# 6. Emotion Adapter 接口定义（最终版）

```
pub struct EmotionRequest {
    pub text: String,
    pub lang: String,
}

pub struct EmotionResponse {
    pub primary: String,
    pub intensity: f32,
    pub confidence: f32,
}
```

---

# 7. 后处理规则

- 文本过短 → 强制 neutral  
- logits 差值过小 → neutral  
- confidence = softmax(top1)

---

# 8. Emotion → TTS 参数映射规范（含中英文差异）

（表格略，同前文）

---

# 9. 系统流程图

```
Audio → VAD → ASR → Emotion → Persona → NMT → TTS
```

---

# 10. Persona 映射规范（情绪驱动）

（表格略）

---

# 11. 单元测试示例

（测试代码略）

---

# 12. 未来扩展：Emotion-aware NMT（可选）

（说明略）

---

# 13. 业务架构图

```
Frontend ↔ Engine  
Engine = ASR + Emotion + Persona + NMT + TTS
```

---

# 14. 结语

本方案可直接交付开发团队使用。
