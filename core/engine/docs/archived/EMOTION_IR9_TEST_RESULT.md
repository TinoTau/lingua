# Emotion IR 9 模型测试结果

## ❌ 测试结果：IR 9 模型**不能满足功能需求**

### 测试详情

**测试时间**: 2024-12-19

**测试脚本**: `scripts/test_emotion_ir9.py`

**测试结果**:
- ✅ IR 版本: 9（正确）
- ✅ Opset 版本: 12（正确）
- ❌ **模型加载失败**: `Unrecognized attribute: start for operator Shape`

---

## 问题分析

### 根本原因

手动降级 IR 版本只是修改了模型的**元数据**（IR version 和 opset version），但模型内部的**操作定义**仍然使用了 opset 18 的特性。

**具体问题**:
- `Shape` 操作在 opset 18 中支持 `start` 属性
- `Shape` 操作在 opset 12 中**不支持** `start` 属性
- 手动降级无法自动转换这些操作定义

### 错误信息

```
[ONNXRuntimeError] : 10 : INVALID_GRAPH : 
Load model from model_ir9.onnx failed:
This is an invalid model. 
In Node, ("node_Shape_0", Shape, "", -1) : 
("input_ids": tensor(int64),) -> ("val_0": tensor(int64),) , 
Error Unrecognized attribute: start for operator Shape
```

---

## 结论

**IR 9 模型（手动降级）不能满足功能需求**，因为：
1. 模型无法被 `ort` 1.16.3 加载
2. 操作定义与 opset 版本不匹配
3. 手动降级方法不完整

---

## 解决方案

### 方案 1: 升级 `ort` 到支持 IR 10 的版本（推荐）

**优点**:
- 使用原始 IR 10 模型（无需降级）
- 模型完整，无功能缺失
- 代码修改最小

**缺点**:
- 需要测试是否影响 NMT 功能
- 可能需要处理 API 变化

**实施步骤**:
1. 升级 `ort` 版本
2. 测试 NMT 功能是否正常
3. 如果正常，直接使用 IR 10 模型

---

### 方案 2: 使用旧版本 PyTorch 导出模型

**优点**:
- 可以导出真正的 IR 9 模型
- 兼容 `ort` 1.16.3

**缺点**:
- 需要安装旧版本 PyTorch
- 可能无法使用最新模型特性

**实施步骤**:
1. 安装 PyTorch 1.x（支持 opset 12）
2. 使用旧版本导出模型
3. 验证模型兼容性

---

### 方案 3: 使用其他模型转换工具

**选项**:
- ONNX Simplifier
- ONNX Optimizer
- 其他第三方工具

**缺点**:
- 可能无法完全解决兼容性问题
- 需要额外工具和依赖

---

## 推荐方案

**推荐使用方案 1（升级 `ort`）**，原因：
1. 最简单直接
2. 使用原始模型，无功能缺失
3. 如果 NMT 不受影响，风险最小

---

## 下一步行动

1. **测试升级 `ort` 对 NMT 的影响** 🔴
   - 升级 `ort` 到支持 IR 10 的版本
   - 运行 NMT 测试
   - 如果通过，使用 IR 10 模型

2. **如果 NMT 受影响** 🟡
   - 考虑方案 2（旧版本 PyTorch）
   - 或方案 3（其他转换工具）

---

**最后更新**: 2024-12-19  
**状态**: IR 9 模型不能满足功能需求，需要采用其他方案

