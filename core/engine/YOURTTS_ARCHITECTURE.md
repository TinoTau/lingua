# YourTTS 架构适配说明

## 当前系统架构

### 已确认的配置

1. **torchaudio 版本**：已降级到 2.9 以下 ✅
   - 在下载 speaker_embedding 模型时已处理
   - 兼容性问题已解决

2. **TTS 服务运行环境**：WSL2 ✅
   - Piper TTS 在 WSL2 中运行
   - YourTTS 也在 WSL2 中运行（已通过集成测试）

3. **GPU 支持**：所有模块都使用 GPU ✅
   - 其他模块已通过 GPU 测试
   - YourTTS 在 WSL2 中使用 GPU（通过 WSL2 GPU 直通）

## YourTTS 架构适配

### 当前实现

YourTTS 服务架构与 Piper TTS 完全一致：

1. **服务运行位置**：WSL2
2. **端口映射**：WSL 端口自动映射到 Windows localhost
3. **客户端连接**：Rust 客户端连接 `http://127.0.0.1:5004`
4. **GPU 支持**：通过 WSL2 GPU 直通

### 配置说明

#### 服务端配置（WSL）

```bash
# 在 WSL 中启动服务
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 0.0.0.0
```

**关键参数**：
- `--host 0.0.0.0`：允许从 Windows 访问
- `--gpu`：使用 GPU（通过 WSL2 GPU 直通）
- `--port 5004`：服务端口

#### 客户端配置（Rust）

```rust
let yourtts = YourTtsHttp::new(YourTtsHttpConfig {
    endpoint: "http://127.0.0.1:5004".to_string(),  // WSL 端口自动映射
    timeout_ms: 10000,
})?;
```

**说明**：
- 端点使用 `127.0.0.1:5004`（Windows localhost）
- WSL2 自动将 WSL 中的端口映射到 Windows
- 无需特殊配置

## 与现有架构的兼容性

### ✅ 完全兼容

1. **与 Piper TTS 一致**：
   - 相同的运行环境（WSL2）
   - 相同的端口映射机制
   - 相同的客户端连接方式

2. **与 Speaker Embedding 兼容**：
   - Speaker Embedding 在 Windows 中运行（GPU 支持更好）
   - YourTTS 在 WSL2 中运行（与 Piper 一致）
   - 两者通过 HTTP 通信，互不干扰

3. **GPU 支持**：
   - Windows 服务直接使用 GPU
   - WSL2 服务通过 GPU 直通使用 GPU
   - 所有模块都支持 GPU 加速

## 启动流程

### 推荐启动方式

1. **Speaker Embedding 服务**（Windows，GPU）：
   ```powershell
   python core/engine/scripts/speaker_embedding_service.py --gpu
   ```

2. **YourTTS 服务**（WSL，GPU）：
   ```bash
   # 在 WSL 中
   python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
   ```

3. **Piper TTS 服务**（WSL，已运行）：
   - 继续使用现有配置

### 使用启动脚本

**统一启动**：
```powershell
.\core\engine\scripts\start_all_services.ps1
```

**单独启动 YourTTS**：
```powershell
.\core\engine\scripts\start_yourtts_wsl.ps1
```

或在 WSL 中：
```bash
./core/engine/scripts/start_yourtts_wsl.sh
```

## 验证配置

### 1. 检查服务运行

**Windows 中**：
```powershell
netstat -an | findstr :5003  # Speaker Embedding
netstat -an | findstr :5004  # YourTTS
```

**WSL 中**：
```bash
netstat -tuln | grep 5004  # YourTTS
```

### 2. 健康检查

```powershell
# 从 Windows
curl http://127.0.0.1:5003/health  # Speaker Embedding
curl http://127.0.0.1:5004/health  # YourTTS
```

### 3. 集成测试

```bash
cd core/engine
cargo test --test speaker_services_integration_test -- --ignored
```

## 架构优势

### 1. 环境隔离
- Windows 服务：Speaker Embedding（GPU 支持更好）
- WSL 服务：TTS 服务（Python 依赖更友好）

### 2. 资源利用
- GPU 资源在 Windows 和 WSL 之间共享
- 服务可以独立启动和停止

### 3. 开发便利
- WSL 环境对 Python 依赖更友好
- Windows 环境对 GPU 支持更好
- 通过 HTTP 通信，解耦服务

## 总结

YourTTS 已完全适配当前系统架构：

✅ **运行环境**：WSL2（与 Piper TTS 一致）  
✅ **GPU 支持**：通过 WSL2 GPU 直通  
✅ **端口映射**：自动映射到 Windows localhost  
✅ **客户端连接**：使用标准 HTTP 端点  
✅ **集成测试**：已通过测试  

无需额外配置，直接使用即可。

