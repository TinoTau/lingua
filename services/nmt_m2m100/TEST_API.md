# M2M100 NMT 服务 API 测试指南

## PowerShell 测试命令

### 健康检查

```powershell
# 方式1：使用 Invoke-WebRequest
Invoke-WebRequest -Uri http://127.0.0.1:5008/health | Select-Object -ExpandProperty Content

# 方式2：使用 curl（PowerShell 别名）
curl http://127.0.0.1:5008/health
```

### 翻译接口

```powershell
# 方式1：使用 Invoke-WebRequest（推荐）
$body = @{
    src_lang = "zh"
    tgt_lang = "en"
    text = "你好"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://127.0.0.1:5008/v1/translate `
    -Method POST `
    -ContentType "application/json" `
    -Body $body | Select-Object -ExpandProperty Content

# 方式2：使用 Invoke-RestMethod（更简洁，自动解析 JSON）
$body = @{
    src_lang = "zh"
    tgt_lang = "en"
    text = "你好"
} | ConvertTo-Json

Invoke-RestMethod -Uri http://127.0.0.1:5008/v1/translate `
    -Method POST `
    -ContentType "application/json" `
    -Body $body
```

### 完整示例

```powershell
# 测试中文到英文翻译
$requestBody = @{
    src_lang = "zh"
    tgt_lang = "en"
    text = "你好，欢迎参加测试。"
} | ConvertTo-Json

$response = Invoke-RestMethod -Uri http://127.0.0.1:5008/v1/translate `
    -Method POST `
    -ContentType "application/json" `
    -Body $requestBody

Write-Host "翻译结果: $($response.text)"
Write-Host "耗时: $($response.extra.elapsed_ms)ms"
```

## Linux/Mac 测试命令

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
    "text": "你好"
  }'
```

## Python 测试脚本

```python
import requests

# 健康检查
response = requests.get("http://127.0.0.1:5008/health")
print(response.json())

# 翻译
response = requests.post(
    "http://127.0.0.1:5008/v1/translate",
    json={
        "src_lang": "zh",
        "tgt_lang": "en",
        "text": "你好，欢迎参加测试。"
    }
)
print(response.json())
```

## 浏览器测试

访问以下地址查看 API 文档和交互式测试界面：

- **API 文档**：http://127.0.0.1:5008/docs
- **替代文档**：http://127.0.0.1:5008/redoc

在文档页面可以直接测试 API 接口。

