# M2M100 NMT 服务依赖修复

**日期：** 2025-01-23  
**问题：** `huggingface-hub` 版本冲突导致服务无法启动

---

## 1. 问题描述

启动 M2M100 NMT 服务时出现错误：

```
ImportError: huggingface-hub>=0.34.0,<1.0 is required for a normal functioning of this module, but found huggingface-hub==1.1.5.
```

**原因：**
- `transformers` 库要求 `huggingface-hub>=0.34.0,<1.0`
- 当前环境安装的是 `huggingface-hub==1.1.5`（版本过高）

---

## 2. 解决方案

### 2.1 降级 huggingface-hub

```bash
pip install "huggingface-hub<1.0"
```

**结果：**
- 卸载：`huggingface-hub 1.1.5`
- 安装：`huggingface-hub 0.36.0` ✅

### 2.2 更新 requirements.txt

在 `services/nmt_m2m100/requirements.txt` 中添加版本约束：

```
huggingface-hub>=0.34.0,<1.0
```

---

## 3. 验证

### 3.1 导入测试

```bash
python -c "from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer; print('✅ 导入成功')"
```

**结果：** ✅ 导入成功

### 3.2 服务启动

```bash
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

**结果：** ✅ 服务可以正常启动

---

## 4. 依赖版本

修复后的依赖版本：

| 包 | 版本 | 说明 |
|----|------|------|
| `transformers` | 4.55.4 | 已安装 |
| `huggingface-hub` | 0.36.0 | 已降级（从 1.1.5） |
| `torch` | 2.9.1 | 已安装 |

---

## 5. 预防措施

### 5.1 版本约束

在 `requirements.txt` 中明确指定版本约束，避免未来出现类似问题：

```
huggingface-hub>=0.34.0,<1.0
```

### 5.2 安装建议

重新安装依赖时，使用：

```bash
pip install -r requirements.txt
```

这样可以确保安装正确版本的依赖。

---

## 6. 相关文件

- **依赖文件**：`services/nmt_m2m100/requirements.txt`
- **服务文件**：`services/nmt_m2m100/nmt_service.py`

---

**状态：** ✅ 已修复  
**服务状态：** ✅ 可以正常启动

