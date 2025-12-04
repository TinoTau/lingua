# 修复 HuggingFace Hub 版本兼容性问题

## 问题描述

错误信息：
```
TypeError: hf_hub_download() got an unexpected keyword argument 'use_auth_token'
```

## 原因

- `huggingface_hub` 1.1.6 版本太新，已移除 `use_auth_token` 参数
- SpeechBrain 1.0.3 仍在使用旧的 `use_auth_token` 参数
- 版本不兼容

## 解决方案

### 方案 1：降级 huggingface_hub（推荐）

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install "huggingface_hub<0.20.0"
```

这会安装 `huggingface_hub` 0.19.x 版本，支持 `use_auth_token` 参数。

### 方案 2：升级 SpeechBrain（如果可用）

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install --upgrade speechbrain
```

新版本的 SpeechBrain 可能已经修复了这个问题。

### 方案 3：使用兼容版本组合

```powershell
# 安装兼容的版本组合
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install "huggingface_hub==0.19.4" "speechbrain==1.0.3"
```

## 验证修复

修复后，重新启动服务：

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu
```

应该能正常加载模型了。

## 关于 token 验证

如果模型是本地下载的，不需要 token 验证。降级 `huggingface_hub` 后，即使传递 `use_auth_token=None` 也不会报错。

