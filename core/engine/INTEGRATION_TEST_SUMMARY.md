# 集成测试总结

## 测试文件

### Python 服务测试
- `core/engine/scripts/test_speaker_embedding_service.py` - 使用真实 WAV 文件测试
- `core/engine/scripts/test_yourtts_service.py` - 使用真实 WAV 文件测试

### Rust 集成测试
- `core/engine/tests/speaker_services_integration_test.rs` - 确保服务不影响其他模块

## 测试音频文件

测试使用 `test_output` 目录中的 WAV 文件：
- `chinese.wav`
- `english.wav`
- `s2s_pipeline_output_zh.wav`
- `s2s_pipeline_output.wav`

## 运行测试

### Python 测试（不需要服务运行）

```bash
# Speaker Embedding 服务测试
python core/engine/scripts/test_speaker_embedding_service.py

# YourTTS 服务测试
python core/engine/scripts/test_yourtts_service.py
```

### Rust 集成测试（需要服务运行）

```bash
# 1. 启动服务（使用启动脚本）
# Windows:
.\core\engine\scripts\start_services.ps1

# Linux/Mac:
chmod +x core/engine/scripts/start_services.sh
./core/engine/scripts/start_services.sh

# 2. 运行集成测试
cd core/engine
cargo test --test speaker_services_integration_test -- --ignored
```

## 服务启动

### 快速启动（GPU 模式）

**Windows**：
```powershell
.\core\engine\scripts\start_services.ps1
```

**Linux/Mac**：
```bash
chmod +x core/engine/scripts/start_services.sh
./core/engine/scripts/start_services.sh
```

### 手动启动

**终端 1 - Speaker Embedding 服务**：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu
```

**终端 2 - YourTTS 服务**：
```bash
python core/engine/scripts/yourtts_service.py --gpu
```

## 验证服务运行

### 健康检查

```bash
# Speaker Embedding
curl http://127.0.0.1:5003/health

# YourTTS
curl http://127.0.0.1:5004/health
```

### Python 验证

```python
import requests

# 检查 Speaker Embedding
r = requests.get("http://127.0.0.1:5003/health")
print("Speaker Embedding:", r.json())

# 检查 YourTTS
r = requests.get("http://127.0.0.1:5004/health")
print("YourTTS:", r.json())
```

## 集成测试覆盖

### 1. 服务独立性测试
- ✅ 服务客户端创建不影响其他模块
- ✅ 不同服务的配置是独立的

### 2. 服务健康检查
- ✅ Speaker Embedding 服务健康检查
- ✅ YourTTS 服务健康检查

### 3. 功能测试
- ✅ Speaker Embedding 提取功能
- ✅ YourTTS 合成功能

### 4. 错误处理
- ✅ 服务未运行时的处理
- ✅ 网络错误处理

## 确保不影响其他服务

### 测试策略

1. **隔离测试**：每个服务独立测试
2. **配置隔离**：不同服务使用不同端口和配置
3. **错误处理**：服务不可用时，不影响其他功能
4. **可选依赖**：服务是可选的，不强制要求

### 集成测试验证

```rust
// 测试：确保服务客户端创建不会影响其他模块
#[tokio::test]
async fn test_services_do_not_affect_other_modules() {
    // 创建客户端不应该影响其他功能
    let embedding_client = SpeakerEmbeddingClient::with_default_config();
    let tts_client = YourTtsHttp::with_default_config();
    
    // 应该成功创建，不影响其他模块
    assert!(embedding_client.is_ok());
    assert!(tts_client.is_ok());
}
```

## GPU 模式

### 检查 GPU 是否可用

```python
import torch
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"CUDA device: {torch.cuda.get_device_name(0)}")
```

### 启动 GPU 模式

添加 `--gpu` 参数：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu
python core/engine/scripts/yourtts_service.py --gpu
```

### GPU 模式优势

- **Speaker Embedding**：5-10 倍性能提升
- **YourTTS**：10-20 倍性能提升

## 常见问题

### 1. 服务启动失败

**检查**：
- 模型文件是否存在
- 端口是否被占用
- Python 依赖是否安装

### 2. GPU 不可用

**现象**：服务使用 CPU 模式
**解决**：检查 CUDA 安装和 PyTorch 配置

### 3. 测试失败

**检查**：
- 服务是否运行
- 模型是否加载成功
- 网络连接是否正常

## 文档

- `SERVICE_STARTUP_GUIDE.md` - 详细的服务启动指南
- `BUG_FIXES.md` - Bug 修复记录
- `TESTING_GUIDE.md` - 测试指南

