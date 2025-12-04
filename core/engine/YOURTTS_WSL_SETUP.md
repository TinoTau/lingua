# YourTTS WSL 集成指南

## 当前架构确认

### 现有 TTS 服务
- **Piper TTS**：在 WSL2 中运行 ✅
- **YourTTS**：需要适配 WSL 架构

### Speaker Embedding 服务
- 在 Windows 中运行（GPU 支持更好）
- 端口：5003

## YourTTS WSL 集成方案

### 推荐配置

**YourTTS 服务在 WSL2 中运行**（与 Piper TTS 一致）

**优势**：
- ✅ 与现有架构一致
- ✅ Linux 环境对 Python 依赖更友好
- ✅ WSL2 支持 GPU 直通
- ✅ 端口自动映射到 Windows

## 启动方式

### 方式 1：在 WSL 中直接启动

```bash
# 在 WSL 终端中
cd /mnt/d/Programs/github/lingua
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 0.0.0.0
```

**注意**：
- `--host 0.0.0.0` 允许从 Windows 访问
- WSL2 会自动将端口映射到 Windows localhost

### 方式 2：从 Windows 启动 WSL 服务

```powershell
# 使用启动脚本
.\core\engine\scripts\start_yourtts_wsl.ps1
```

### 方式 3：统一启动所有服务

```powershell
# 启动 Speaker Embedding (Windows) + YourTTS (WSL)
.\core\engine\scripts\start_all_services.ps1
```

## 端口映射

WSL2 自动端口映射：
- WSL 中监听 `0.0.0.0:5004` 
- Windows 可以通过 `127.0.0.1:5004` 访问
- 无需手动配置端口转发

## GPU 支持

### WSL2 GPU 检查

```bash
# 在 WSL 中检查 GPU
wsl nvidia-smi
```

### 启动 GPU 模式

```bash
# 在 WSL 中
python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

## 验证服务

### 从 Windows 验证

```powershell
# 健康检查
curl http://127.0.0.1:5004/health

# 或使用 PowerShell
Invoke-WebRequest -Uri http://127.0.0.1:5004/health
```

### 从 WSL 验证

```bash
curl http://localhost:5004/health
```

## 完整启动流程

### 推荐配置

1. **Speaker Embedding**（Windows，GPU）：
   ```powershell
   python core/engine/scripts/speaker_embedding_service.py --gpu
   ```

2. **YourTTS**（WSL，GPU）：
   ```bash
   # 在 WSL 中
   python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
   ```

3. **Piper TTS**（WSL，已运行）：
   - 继续使用现有配置

### 使用统一启动脚本

```powershell
.\core\engine\scripts\start_all_services.ps1
```

## 配置 Rust 客户端

YourTTS 客户端配置保持不变，因为 WSL 端口自动映射：

```rust
let yourtts = YourTtsHttp::new(YourTtsHttpConfig {
    endpoint: "http://127.0.0.1:5004".to_string(),  // 自动映射到 WSL
    timeout_ms: 10000,
})?;
```

## 故障排除

### 1. 无法从 Windows 访问 WSL 服务

**检查**：
- 服务是否监听 `0.0.0.0`（不是 `127.0.0.1`）
- WSL 端口是否正确映射

**解决**：
```bash
# 在 WSL 中检查端口
netstat -tuln | grep 5004

# 应该显示：0.0.0.0:5004
```

### 2. GPU 不可用

**检查**：
```bash
wsl nvidia-smi
```

**解决**：
- 确保 WSL2 已安装 GPU 驱动
- 或使用 CPU 模式（不添加 `--gpu`）

### 3. 路径问题

**WSL 路径转换**：
- Windows: `D:\Programs\github\lingua`
- WSL: `/mnt/d/Programs/github/lingua`

## 与现有架构的兼容性

### ✅ 完全兼容

- YourTTS 客户端使用 HTTP 连接，不关心服务运行位置
- WSL 端口自动映射，Windows 客户端可以直接连接
- 与 Piper TTS 的架构完全一致

### 配置示例

```rust
// 在 CoreEngineBuilder 中配置
let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_tts(Arc::new(YourTtsHttp::new(YourTtsHttpConfig {
        endpoint: "http://127.0.0.1:5004".to_string(),  // WSL 服务
        timeout_ms: 10000,
    })?))
    .build()?;
```

