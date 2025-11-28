# 为什么本地服务会用到 HF_TOKEN？

## 原因说明

即使模型已经下载到本地，`transformers` 库的 `from_pretrained()` 方法仍然会：

### 1. 检查模型元数据
- 验证模型文件完整性
- 检查模型配置和版本信息
- 可能需要从 Hugging Face Hub 获取最新的模型元数据

### 2. 使用缓存的 Token
- `transformers` 库会自动读取以下位置的 token：
  - **Windows**: `%USERPROFILE%\.cache\huggingface\token`
  - **Linux/WSL**: `~/.cache/huggingface/token`
  - 或者通过 `huggingface-cli login` 存储的 token

### 3. 自动认证尝试
- 即使模型是公开的，库也会尝试使用缓存的 token
- 如果 token 过期，会导致 401 错误

## 解决方案

### 方案 1：禁用自动 token 使用（推荐）

修改代码，明确禁用 token：

```python
# 明确不使用 token
extra = {
    "use_safetensors": True,
    "local_files_only": False,  # 允许从缓存加载，但不使用 token
}
# 不传递 token 参数
```

### 方案 2：使用 local_files_only=True（如果模型已完全下载）

如果模型已经完全下载到本地，可以使用：

```python
extra = {
    "use_safetensors": True,
    "local_files_only": True,  # 只使用本地文件，不访问网络
}
```

### 方案 3：清除过期的 token

```powershell
# Windows
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface\token" -Force

# Linux/WSL
rm ~/.cache/huggingface/token
```

### 方案 4：设置环境变量禁用 token

```python
# 在代码中设置
os.environ["HF_HUB_DISABLE_IMPLICIT_TOKEN"] = "1"
```

## 最佳实践

对于公开模型（如 `facebook/m2m100_418M`）：
1. **不要设置 HF_TOKEN**（或设置为空）
2. **清除过期的 token 缓存**
3. **使用 `local_files_only=False`**（允许从缓存加载，但不需要 token）

