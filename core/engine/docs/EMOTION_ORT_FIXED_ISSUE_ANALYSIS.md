# Emotion 适配器问题分析（ort 不能升级的情况）

## 🔴 核心问题

在 `ort` 版本固定为 1.16.3（不支持 IR 10）的情况下，我们遇到以下问题：

---

## 问题 1: IR 版本不兼容

### 问题描述

- **当前 ort 版本**: 1.16.3（只支持 IR version ≤ 9）
- **原始模型**: IR version 10（opset 18）
- **结果**: 原始模型无法被 ort 1.16.3 加载

### 错误信息

```
Unsupported model IR version: 10, max supported IR version: 9
```

---

## 问题 2: 手动降级失败

### 问题描述

我们尝试手动将模型从 IR 10 降级到 IR 9：
- ✅ 成功修改了元数据（IR version: 9, opset version: 12）
- ❌ **但模型加载失败**

### 错误信息

```
[ONNXRuntimeError] : 10 : INVALID_GRAPH : 
Load model from model_ir9.onnx failed:
This is an invalid model. 
In Node, ("node_Shape_0", Shape, "", -1) : 
("input_ids": tensor(int64),) -> ("val_0": tensor(int64),) , 
Error Unrecognized attribute: start for operator Shape
```

### 根本原因

**手动降级只修改了元数据，但模型内部的操作定义仍然使用了 opset 18 的特性**：

1. **Shape 操作**:
   - opset 18: 支持 `start` 和 `end` 属性
   - opset 12: **不支持** `start` 和 `end` 属性

2. **其他可能的操作**:
   - 可能还有其他操作使用了 opset 18 的特性
   - 手动降级无法自动转换这些操作定义

---

## 问题 3: 自动版本转换失败

### 问题描述

尝试使用 ONNX 的 `version_converter` 自动降级：
- ❌ 转换失败

### 错误信息

```
RuntimeError: No Previous Version of LayerNormalization exists
```

### 原因

某些操作（如 LayerNormalization）在 opset 18 中的实现与 opset 12 不兼容，无法自动转换。

---

## 📊 问题总结

| 问题 | 状态 | 影响 |
|------|------|------|
| IR 版本不兼容 | ❌ 未解决 | 原始模型无法加载 |
| 手动降级失败 | ❌ 失败 | 模型元数据正确但操作定义不兼容 |
| 自动转换失败 | ❌ 失败 | 某些操作无法自动转换 |

---

## 🎯 在 ort 不能升级的情况下，可能的解决方案

### 方案 1: 使用旧版本 PyTorch 重新导出模型 ⭐（推荐）

**思路**: 使用支持 opset 12 的旧版本 PyTorch 导出模型，确保模型从一开始就是 IR 9 兼容的。

**优点**:
- 模型从源头就是 IR 9 兼容的
- 操作定义与 opset 12 完全匹配
- 不需要手动降级

**缺点**:
- 需要安装旧版本 PyTorch（可能与其他依赖冲突）
- 可能无法使用最新模型特性

**实施步骤**:
1. 安装 PyTorch 1.x（支持 opset 12）
2. 使用旧版本重新导出模型
3. 验证模型兼容性

---

### 方案 2: 使用 ONNX Simplifier 优化模型

**思路**: 使用 ONNX Simplifier 简化模型，可能能够移除不兼容的操作。

**优点**:
- 可能简化模型结构
- 可能移除不兼容的操作

**缺点**:
- 不一定能解决兼容性问题
- 可能改变模型行为

**实施步骤**:
```bash
pip install onnx-simplifier
python -m onnxsim model.onnx model_simplified.onnx
```

---

### 方案 3: 使用其他模型（兼容 IR 9）

**思路**: 寻找或训练一个兼容 IR 9 的情感分析模型。

**优点**:
- 完全兼容 ort 1.16.3
- 无需处理兼容性问题

**缺点**:
- 可能需要重新训练或寻找模型
- 可能性能不如 XLM-R

---

### 方案 4: 暂时使用 EmotionStub

**思路**: 在找到解决方案之前，使用 EmotionStub 占位符。

**优点**:
- 不影响其他功能开发
- 可以继续开发其他模块

**缺点**:
- Emotion 功能不完整
- 需要后续补充

---

## 🔍 推荐方案

**推荐方案 1（使用旧版本 PyTorch 重新导出）**，原因：
1. 最有可能成功
2. 模型从源头就是兼容的
3. 不需要复杂的转换步骤

---

## 📋 下一步行动

### 立即执行

1. **尝试方案 1**: 使用旧版本 PyTorch 重新导出模型
   - 安装 PyTorch 1.13 或更早版本
   - 使用 opset 12 导出模型
   - 验证模型兼容性

2. **如果方案 1 失败**: 尝试方案 2（ONNX Simplifier）

3. **如果都失败**: 考虑方案 3 或方案 4

---

## ⚠️ 注意事项

1. **PyTorch 版本冲突**: 如果项目中其他部分需要新版本 PyTorch，可能需要使用虚拟环境

2. **模型性能**: 使用旧版本导出的模型可能性能略有差异

3. **长期方案**: 如果 Emotion 功能很重要，建议未来升级 ort 版本

---

**最后更新**: 2024-12-19  
**状态**: ort 不能升级的情况下，需要找到其他方法使模型兼容 IR 9

