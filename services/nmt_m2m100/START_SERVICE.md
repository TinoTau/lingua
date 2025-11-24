# M2M100 NMT 服务启动指南

## 快速启动

### 方式1：使用 uvicorn 直接启动（推荐）

```bash
cd services/nmt_m2m100
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

### 方式2：使用 Python 直接运行

```bash
cd services/nmt_m2m100
python nmt_service.py
```

## 启动参数说明

### 基本参数

- `--host 127.0.0.1`：服务监听地址（默认：127.0.0.1，仅本地访问）
- `--port 5008`：服务端口（默认：5008）
- `--reload`：开发模式，代码修改后自动重启（仅开发时使用）

### 完整启动命令示例

```bash
# 开发模式（自动重载）
uvicorn nmt_service:app --host 127.0.0.1 --port 5008 --reload

# 生产模式（多进程）
uvicorn nmt_service:app --host 0.0.0.0 --port 5008 --workers 4

# 仅本地访问
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

## 启动步骤

### 1. 确保依赖已安装

```bash
pip install -r requirements.txt
```

### 2. 启动服务

```bash
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

### 3. 验证服务运行

服务启动后，你会看到类似以下的输出：

```
INFO:     Started server process [xxxxx]
INFO:     Waiting for application startup.
[NMT Service] Loading model: facebook/m2m100_418M
[NMT Service] Device: cpu
[NMT Service] Model loaded successfully
INFO:     Application startup complete.
INFO:     Uvicorn running on http://127.0.0.1:5008 (Press CTRL+C to quit)
```

### 4. 测试服务

在另一个终端中测试：

```bash
# 健康检查
curl http://127.0.0.1:5008/health

# 翻译测试
curl -X POST http://127.0.0.1:5008/v1/translate \
  -H "Content-Type: application/json" \
  -d '{"src_lang": "zh", "tgt_lang": "en", "text": "你好，欢迎参加测试。"}'
```

## 环境变量

### HF_TOKEN（可选）

如果需要访问私有模型或提高下载速度，可以设置 HuggingFace 访问令牌：

```bash
# Windows PowerShell
$env:HF_TOKEN="your_token_here"
uvicorn nmt_service:app --host 127.0.0.1 --port 5008

# Linux/Mac
export HF_TOKEN="your_token_here"
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

## 常见问题

### 问题1：端口已被占用

**错误信息**：`Address already in use`

**解决方案**：
- 更换端口：`--port 5009`
- 或关闭占用端口的进程

### 问题2：模型下载失败

**错误信息**：`Failed to load model`

**解决方案**：
- 检查网络连接
- 设置 `HF_TOKEN` 环境变量
- 手动下载模型到本地

### 问题3：内存不足

**错误信息**：`Out of memory`

**解决方案**：
- 使用 CPU 模式（自动检测）
- 或使用更小的模型
- 增加系统内存

## 停止服务

按 `Ctrl+C` 停止服务。

## 后台运行（Linux/Mac）

```bash
# 使用 nohup
nohup uvicorn nmt_service:app --host 127.0.0.1 --port 5008 > nmt_service.log 2>&1 &

# 查看日志
tail -f nmt_service.log

# 停止服务
pkill -f "uvicorn nmt_service"
```

## Windows 后台运行

```powershell
# 使用 Start-Process
Start-Process python -ArgumentList "-m", "uvicorn", "nmt_service:app", "--host", "127.0.0.1", "--port", "5008" -WindowStyle Hidden

# 或使用任务计划程序
```

## 服务地址

启动成功后，服务将在以下地址运行：

- **本地访问**：http://127.0.0.1:5008
- **API 文档**：http://127.0.0.1:5008/docs（FastAPI 自动生成）
- **健康检查**：http://127.0.0.1:5008/health
- **翻译接口**：http://127.0.0.1:5008/v1/translate

