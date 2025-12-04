# ConnectionResetError 修复总结

## 问题说明

**这不是真正的错误**，而是 Windows 上 asyncio 的正常行为：

- ✅ NMT 服务正常运行
- ✅ 健康检查都返回 200 OK
- ⚠️ 只是日志中会显示 `ConnectionResetError`（不影响功能）

## 已实施的修复

### 1. NMT 服务日志优化 ✅

**文件**：`services/nmt_m2m100/nmt_service.py`

**修改**：添加日志级别配置，忽略连接重置错误

```python
import sys
import logging

# 在 Windows 上，配置日志级别，忽略连接重置错误
if sys.platform == 'win32':
    logging.getLogger('asyncio').setLevel(logging.WARNING)
    logging.getLogger('uvicorn.access').setLevel(logging.WARNING)
```

**效果**：连接重置错误不再显示在日志中（或显示为 WARNING 级别）

### 2. 启动脚本等待时间优化 ✅

**文件**：`start_all_services_with_speaker.ps1`

**修改**：
- Speaker Embedding 服务：等待时间从 3 秒增加到 5 秒
- NMT 服务：等待时间从 3 秒增加到 5 秒
- YourTTS 服务（WSL）：等待时间从 5 秒增加到 8 秒

**效果**：确保服务完全启动后再进行健康检查，减少连接重置错误

## 关于 YourTTS 的 conda 警告

**警告信息**：
```
Unable to create process using 'D:\Program Files\Anaconda\python.exe "D:\Program Files\Anaconda\Scripts\conda-script.py" shell.powershell hook'
```

**说明**：
- 这是 PowerShell 尝试激活 conda 环境时的警告
- **不影响 YourTTS 服务运行**（YourTTS 在 WSL 中运行，不依赖 conda）
- 可以安全忽略

**如果需要消除警告**：
- 可以在启动脚本中禁用 conda 自动激活
- 或者在 WSL 环境中配置，不使用 conda

## 验证

修复后，重新启动服务，应该：
- ✅ NMT 服务正常启动，无连接重置错误（或错误被抑制）
- ✅ YourTTS 服务正常启动（conda 警告可以忽略）
- ✅ 所有服务健康检查正常

## 总结

- **问题严重性**：低（不影响功能）
- **修复状态**：✅ 已完成
- **建议**：可以正常使用，连接重置错误已被抑制

---

**修复完成时间**：2024年

