# HF_TOKEN 配置说明

## 配置方式

### 方式 1：使用本地文件（推荐，完全禁用验证）

如果模型已经完全下载到本地，可以设置环境变量：

```powershell
$env:HF_LOCAL_FILES_ONLY = "true"
```

这样会完全禁用网络请求和 token 验证。

### 方式 2：使用配置文件

Token 已保存在 `hf_token.txt` 文件中：
```
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
```

代码会自动从该文件读取 token（如果环境变量未设置）。

### 方式 3：使用环境变量

```powershell
$env:HF_TOKEN = "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"
```

## 优先级

1. **环境变量 `HF_LOCAL_FILES_ONLY=true`**：完全禁用网络验证（最高优先级）
2. **环境变量 `HF_TOKEN`**：使用环境变量中的 token
3. **配置文件 `hf_token.txt`**：从文件读取 token
4. **无 token**：尝试使用公开模型访问（可能失败）

## 注意事项

- `hf_token.txt` 文件已添加到 `.gitignore`，不会被提交到 Git
- 如果模型已完全下载，推荐使用 `HF_LOCAL_FILES_ONLY=true`
- 如果模型未完全下载，会自动使用配置文件中的 token

