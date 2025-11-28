# 修复 NMT 服务 Hugging Face Token 过期问题

## 问题描述

启动 NMT 服务时出现错误：
```
User Access Token "mms-tts-zho-download" is expired
401 Client Error: Unauthorized
```

## 为什么本地服务会用到 HF_TOKEN？

即使模型已经下载到本地，`transformers` 库的 `from_pretrained()` 方法仍然会：
1. **检查模型元数据**：验证模型文件完整性，可能需要从 Hugging Face Hub 获取最新信息
2. **使用缓存的 Token**：自动读取 `~/.cache/huggingface/token` 或通过 `huggingface-cli login` 存储的 token
3. **自动认证尝试**：即使模型是公开的，库也会尝试使用缓存的 token，如果过期会导致 401 错误

详见：[为什么需要HF_TOKEN说明.md](./为什么需要HF_TOKEN说明.md)

## 解决方案

### 方法 1：清除过期的 Token（推荐）

`facebook/m2m100_418M` 是公开模型，不需要 token。清除过期的 token：

**Windows PowerShell**：
```powershell
# 清除 Hugging Face token 缓存
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface\hub\*" -Recurse -Force -ErrorAction SilentlyContinue

# 或者只清除 token 文件
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface\token" -Force -ErrorAction SilentlyContinue
```

**或者手动删除**：
1. 打开文件资源管理器
2. 导航到：`C:\Users\<你的用户名>\.cache\huggingface\`
3. 删除 `token` 文件（如果存在）
4. 或者删除整个 `hub` 目录下的缓存

### 方法 2：设置空的 HF_TOKEN 环境变量

在启动 NMT 服务时，明确设置空的 HF_TOKEN：

**修改启动脚本**（`start_all_services_simple.ps1`）：
```powershell
$nmtCommand = "cd '$nmtServicePath'; .\venv\Scripts\Activate.ps1; `$env:HF_TOKEN=''; Write-Host '=== NMT Service (GPU) ===' -ForegroundColor Green; uvicorn nmt_service:app --host 127.0.0.1 --port 5008"
```

### 方法 3：使用本地已下载的模型

如果之前已经下载过模型，可以指定本地路径：

1. 查找本地模型位置（通常在 `C:\Users\<用户名>\.cache\huggingface\hub\models--facebook--m2m100_418M`）
2. 修改 `nmt_service.py` 中的 `MODEL_NAME` 为本地路径

### 方法 4：登录 Hugging Face CLI 并清除 token

```powershell
# 安装 huggingface-cli（如果未安装）
pip install huggingface_hub

# 登录（会提示输入 token，可以输入新的或留空）
huggingface-cli login

# 或者直接登出
huggingface-cli logout
```

## 快速修复命令

**最简单的方法**（在 PowerShell 中运行）：

```powershell
# 清除 token 缓存
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface\token" -Force -ErrorAction SilentlyContinue

# 然后重新启动 NMT 服务
cd D:\Programs\github\lingua\services\nmt_m2m100
.\venv\Scripts\Activate.ps1
$env:HF_TOKEN = ""
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

## 验证修复

启动服务后，应该看到：
```
[NMT Service] Loading model: facebook/m2m100_418M
[NMT Service] Device: cuda
[NMT Service] ✓ CUDA available: True
[NMT Service] Failed to load model: ...
```

如果仍然报错，检查：
1. 网络连接是否正常
2. 模型是否已下载（检查缓存目录）
3. 是否有防火墙阻止访问 huggingface.co

