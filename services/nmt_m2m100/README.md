# M2M100 NMT Service

基于 FastAPI 和 HuggingFace Transformers 的 M2M100 翻译服务。

## 安装依赖

```bash
pip install -r requirements.txt
```

## 启动服务

```bash
# 方式1：使用 uvicorn 直接启动
uvicorn nmt_service:app --host 127.0.0.1 --port 5008

# 方式2：使用 Python 直接运行
python nmt_service.py
```

## API 接口

### 健康检查

```bash
curl http://127.0.0.1:5008/health
```

### 翻译接口

```bash
curl -X POST http://127.0.0.1:5008/v1/translate \
  -H "Content-Type: application/json" \
  -d '{
    "src_lang": "zh",
    "tgt_lang": "en",
    "text": "你好，欢迎参加测试。"
  }'
```

## 环境变量

- `HF_TOKEN`: HuggingFace 访问令牌（如果需要访问私有模型）

## 测试服务

### 使用 Python 脚本（推荐）

```bash
python test_translate.py
```

### 使用浏览器（最简单）

访问 http://127.0.0.1:5008/docs 查看交互式 API 文档并测试。

### 使用 PowerShell

**注意**：PowerShell 直接传递中文 JSON 时可能有编码问题，建议使用：
- Python 脚本：`python test_translate.py`
- PowerShell 脚本：`.\test_translate.ps1`（已处理编码）
- 浏览器：http://127.0.0.1:5008/docs

