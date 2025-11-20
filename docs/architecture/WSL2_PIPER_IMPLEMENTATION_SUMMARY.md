# WSL2 Piper 中文 TTS 实现总结

**日期**: 2025-11-21  
**状态**: ✅ 已完成并测试通过

## 实现概述

成功在 Windows 环境下通过 WSL2 部署了 Piper 中文 TTS HTTP 服务，解决了之前 Windows 版本 Piper 无法使用的问题。

## 完成的工作

### 1. 环境准备 ✅
- 安装并配置 WSL2
- 安装 Ubuntu 发行版
- 创建用户账户

### 2. Piper TTS 安装 ✅
- 在 WSL2 中创建 Python 虚拟环境
- 安装 `piper-tts[http]` 包
- 安装 FastAPI、Uvicorn 等 HTTP 服务依赖

### 3. 模型下载 ✅
- 下载中文模型：`zh_CN-huayan-medium`
- 模型位置：`~/piper_models/zh/`
- 包含模型文件（.onnx）和配置文件（.onnx.json）

### 4. HTTP 服务实现 ✅
- 创建了 `piper_http_server.py` - FastAPI 实现的 HTTP 服务包装器
- 解决了文本编码问题（UTF-8 编码处理）
- 使用 `--input-file` 参数替代 stdin，提高可靠性
- 服务监听地址：`0.0.0.0:5005`

### 5. 测试验证 ✅
- ✅ 健康检查接口正常
- ✅ 语音列表接口正常
- ✅ TTS 合成接口正常
- ✅ 生成的音频质量清晰可识别
- ✅ 从 Windows 侧访问服务正常

## 文件结构

```
scripts/wsl2_piper/
├── setup_wsl2.ps1              # WSL2 环境设置脚本
├── install_piper_in_wsl.sh     # Piper 安装脚本
├── download_piper_model.sh     # 模型下载脚本
├── piper_http_server.py        # HTTP 服务实现
├── start_piper_service.sh      # 服务启动脚本
├── test_piper_service.ps1      # Windows 侧测试脚本
└── README.md                   # 使用说明文档
```

## API 接口

### POST /tts
**请求体**:
```json
{
  "text": "你好，欢迎使用 Lingua 语音翻译系统。",
  "voice": "zh_CN-huayan-medium"
}
```

**响应**: WAV 音频文件（二进制）

### GET /health
**响应**:
```json
{
  "status": "ok",
  "service": "piper-tts"
}
```

### GET /voices
**响应**:
```json
{
  "voices": [
    {
      "name": "zh_CN-huayan-medium",
      "path": "/home/tinot/piper_models/zh/zh_CN-huayan-medium.onnx"
    }
  ]
}
```

## 使用方法

### 启动服务

在 WSL2 中执行：
```bash
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/start_piper_service.sh
```

### 测试服务

在 Windows PowerShell 中执行：
```powershell
.\scripts\wsl2_piper\test_piper_service.ps1
```

### 在 Rust 代码中使用

已在 `core/engine/src/tts_streaming/piper_http.rs` 中实现了 `PiperHttpTts`，可以通过以下方式使用：

```rust
use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::tts_streaming::PiperHttpConfig;

let engine = CoreEngineBuilder::new()
    .tts_with_default_piper_http()?  // 使用默认配置
    // 或
    .tts_with_piper_http(PiperHttpConfig {
        endpoint: "http://127.0.0.1:5005/tts".to_string(),
        default_voice: "zh_CN-huayan-medium".to_string(),
        timeout_ms: 8000,
    })?
    .build()?;
```

## 解决的问题

1. **Windows Piper 兼容性问题**
   - Windows 版本的 `piper.exe` 在用户环境中崩溃（Stack Buffer Overrun）
   - 解决方案：使用 WSL2 运行 Linux 版本的 Piper

2. **文本编码问题**
   - PowerShell 发送中文文本时编码损坏
   - 解决方案：使用 UTF-8 字节数组确保编码正确

3. **文本传递方式**
   - 使用 stdin 传递文本时可能存在问题
   - 解决方案：使用 `--input-file` 参数，将文本写入临时文件

## 性能指标

- **音频生成时间**: ~1-2 秒（取决于文本长度）
- **音频文件大小**: ~130KB（测试文本："你好，欢迎使用 Lingua 语音翻译系统。"）
- **音频格式**: WAV, 16 bit, mono, 22050 Hz
- **服务响应时间**: < 3 秒（包含音频生成）

## 下一步工作

1. **集成到 CoreEngine** ✅（代码已实现，待测试）
   - 在完整的 S2S 流程中测试 Piper HTTP TTS
   - 验证与 ASR、NMT 模块的集成

2. **优化和扩展**
   - 支持更多中文语音模型
   - 添加语音缓存机制
   - 优化服务性能（并发处理）

3. **部署文档**
   - 编写完整的部署指南
   - 添加故障排查文档

## 参考文档

- [WSL2_Piper_ZH_TTS_Deployment_Guide.md](./WSL2_Piper_ZH_TTS_Deployment_Guide.md) - 完整部署指南
- [scripts/wsl2_piper/README.md](../../scripts/wsl2_piper/README.md) - 使用说明
- [PIPER_TTS_IMPLEMENTATION_STEPS.md](../core/engine/docs/PIPER_TTS_IMPLEMENTATION_STEPS.md) - 实现步骤

## 总结

WSL2 Piper 中文 TTS 服务已成功部署并通过测试。该方案解决了 Windows 环境下 Piper TTS 的兼容性问题，为语音转语音翻译系统提供了可靠的中文 TTS 支持。
