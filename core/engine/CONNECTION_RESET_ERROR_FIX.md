# ConnectionResetError 问题分析与解决方案

## 问题描述

在 NMT 服务启动时，会出现以下错误日志：

```
Exception in callback _ProactorBasePipeTransport._call_connection_lost(None)
handle: <Handle _ProactorBasePipeTransport._call_connection_lost(None)>
Traceback (most recent call last):
  ...
ConnectionResetError: [WinError 10054] 远程主机强迫关闭了一个现有的连接。
```

## 问题分析

### 1. 错误原因

这是 **Windows 上 asyncio 的一个已知问题**，不是真正的错误：

1. **健康检查流程**：
   - CoreEngine 在启动时会定期检查 NMT 和 TTS 服务的健康状态
   - 健康检查请求完成后，客户端（CoreEngine）关闭连接
   - Windows 上的 asyncio 在清理连接时，如果服务器端已经关闭了连接，会抛出 `ConnectionResetError`

2. **为什么会出现**：
   - 健康检查请求已经成功（返回 200 OK）
   - 但在异步回调清理连接时，连接已经被服务器端关闭
   - Windows 的 `ProactorEventLoop` 会尝试调用 `shutdown()`，但连接已关闭，导致异常

3. **影响**：
   - ✅ **不影响服务运行**：所有健康检查都返回 200 OK
   - ✅ **不影响功能**：NMT 服务正常工作
   - ⚠️ **只是日志噪音**：会在日志中显示错误信息

### 2. 为什么在 YourTTS 启动之前出现？

- CoreEngine 在启动时会等待 NMT 和 TTS 服务就绪
- 健康检查是定期进行的（每 500ms 一次，最多 15 次）
- 在服务完全启动之前，连接可能不稳定，导致连接重置

## 解决方案

### 方案 1：在 NMT 服务中抑制连接重置错误（推荐）

在 NMT 服务的健康检查端点中，可以添加异常处理，但这实际上是在服务器端，而错误来自客户端。

**更好的方法**：在 NMT 服务的启动日志中，可以添加说明，告知这是正常的 Windows asyncio 行为。

### 方案 2：优化健康检查代码（推荐）

在 CoreEngine 的健康检查代码中，确保正确处理连接关闭：

```rust
// 在 health_check.rs 中，确保响应被完全读取后再关闭连接
match self.http.get(health_url.clone()).send().await {
    Ok(mut response) => {
        // 读取响应体，确保连接正常关闭
        let _ = response.text().await;
        if response.status().is_success() {
            // 服务健康
        }
    }
    Err(e) => {
        // 处理错误
    }
}
```

### 方案 3：增加启动等待时间（简单有效）

在启动脚本中，增加服务启动后的等待时间，确保服务完全启动后再进行健康检查：

```powershell
# 启动 NMT 服务后，等待更长时间
Start-Sleep -Seconds 5  # 从 3 秒增加到 5 秒
```

### 方案 4：忽略 Windows asyncio 的连接重置错误（最佳）

在 NMT 服务的 Python 代码中，可以配置 asyncio 忽略连接重置错误：

```python
import asyncio
import sys

# 在 Windows 上，忽略连接重置错误
if sys.platform == 'win32':
    # 设置事件循环策略，忽略连接重置错误
    asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    
    # 或者，在 uvicorn 启动时添加参数
    # uvicorn.run(..., log_level="warning")  # 降低日志级别
```

## 推荐方案

**建议采用方案 3 + 方案 4 的组合**：

1. **方案 3**：在启动脚本中增加等待时间，确保服务完全启动
2. **方案 4**：在 NMT 服务中配置 asyncio，忽略连接重置错误

这样可以：
- ✅ 减少连接重置错误的发生
- ✅ 即使出现错误，也不会在日志中显示
- ✅ 不影响服务功能

## 实施步骤

### 步骤 1：修改启动脚本

在 `start_all_services_with_speaker.ps1` 中，增加 NMT 服务启动后的等待时间：

```powershell
Start-Sleep -Seconds 5  # 从 3 秒增加到 5 秒
```

### 步骤 2：修改 NMT 服务（可选）

在 `services/nmt_m2m100/nmt_service.py` 中，添加 asyncio 配置：

```python
import asyncio
import sys
import logging

# 在 Windows 上，配置日志级别，忽略连接重置错误
if sys.platform == 'win32':
    # 设置 asyncio 日志级别为 WARNING，忽略 INFO 级别的连接重置错误
    logging.getLogger('asyncio').setLevel(logging.WARNING)
```

或者在启动 uvicorn 时：

```python
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "nmt_service:app",
        host="127.0.0.1",
        port=5008,
        log_level="warning"  # 降低日志级别，忽略连接重置错误
    )
```

## 验证

修改后，重新启动服务，应该：
- ✅ NMT 服务正常启动
- ✅ 健康检查正常（返回 200 OK）
- ✅ 连接重置错误不再显示（或显示为 WARNING 级别）

## 总结

**这个 `ConnectionResetError` 是 Windows 上 asyncio 的正常行为，不影响服务运行。**

如果不想看到这些错误日志，可以：
1. 增加启动等待时间
2. 降低日志级别
3. 或者在健康检查代码中优化连接关闭逻辑

---

**报告生成时间**：2024年
**问题严重性**：低（不影响功能，只是日志噪音）
**建议优先级**：低（可选优化）

