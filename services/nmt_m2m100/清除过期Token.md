# 清除过期的 Hugging Face Token

## 问题

即使设置了 `local_files_only=True`，transformers 库在 safetensors 自动转换时仍会尝试网络请求，并使用过期的缓存 token。

## 解决方案

### 方法 1：清除 Windows 缓存（推荐）

```powershell
# 清除 Hugging Face token 文件
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface\token" -Force -ErrorAction SilentlyContinue

# 清除整个缓存目录（如果上面的方法不行）
Remove-Item -Path "$env:USERPROFILE\.cache\huggingface" -Recurse -Force -ErrorAction SilentlyContinue
```

### 方法 2：使用配置文件中的 Token

代码已更新，会自动从 `hf_token.txt` 读取 token。如果 `local_files_only` 失败，会自动使用配置文件中的 token。

### 方法 3：禁用 local_files_only，直接使用 token

修改启动脚本，不使用 `local_files_only`：

```powershell
# 不使用 local_files_only，直接使用配置文件中的 token
$env:HF_LOCAL_FILES_ONLY = "false"
```

代码会自动从 `hf_token.txt` 读取 token。

## 当前配置

- Token 已保存在：`services/nmt_m2m100/hf_token.txt`
- 代码会自动读取该文件
- 如果 `local_files_only=True` 失败，会自动回退到使用 token

