# 音色识别与分配功能一键启动指南

## 概述

本指南说明如何使用一键启动脚本启动所有服务，实现用户音色识别以及生成语音时分配音色功能。

## 服务架构

### 服务列表

| 服务 | 端口 | 运行环境 | 功能 |
|------|------|----------|------|
| **Speaker Embedding** | 5003 | Windows (conda) | 提取说话者音色特征向量 |
| **YourTTS** | 5004 | WSL2 (venv) | 零样本音色克隆 TTS |
| **NMT (M2M100)** | 5008 | Windows (venv) | 神经机器翻译 |
| **Piper TTS** | 5005 | WSL2 (piper_env) | 传统 TTS（可选） |
| **CoreEngine** | 9000 | Windows (Rust) | 核心引擎（包含 VAD 和 ASR） |

### 服务调用关系

```
音频输入
    ↓
CoreEngine (VAD + ASR)
    ↓
Speaker Embedding Service (提取音色特征)
    ↓
Speaker Identifier (识别说话者)
    ↓
NMT Service (翻译)
    ↓
YourTTS Service (使用参考音频进行零样本合成)
    ↓
音频输出（保持原说话者音色）
```

## 一键启动

### 方法 1：使用一键启动脚本（推荐）

```powershell
# 在项目根目录执行
.\start_all_services_with_speaker.ps1
```

### 方法 2：手动启动各个服务

#### 1. 启动 Speaker Embedding 服务

```powershell
cd core\engine\scripts
.\start_speaker_embedding.ps1
```

#### 2. 启动 YourTTS 服务（WSL2）

```powershell
cd core\engine\scripts
.\start_yourtts_wsl.ps1
```

#### 3. 启动 NMT 服务

```powershell
cd core\engine\scripts
.\start_nmt.ps1
```

#### 4. 启动 CoreEngine

```powershell
cd core\engine\scripts
.\start_core_engine.ps1
```

## 配置文件说明

配置文件：`lingua_core_config.toml`

### 说话者识别配置

```toml
[speaker_identification]
# 模式：vad_based 或 embedding_based
mode = "embedding_based"
# Speaker Embedding 服务 URL（仅 embedding_based 模式需要）
service_url = "http://127.0.0.1:5003"
# 相似度阈值（0.0-1.0，越高越严格）
similarity_threshold = 0.7
```

**模式说明：**
- `vad_based`: 基于 VAD 边界的简单模式（免费用户）
  - 使用时间间隔判断说话者切换
  - 不需要 Speaker Embedding 服务
- `embedding_based`: 基于音色特征向量的准确模式（付费用户）
  - 使用 ECAPA-TDNN 模型提取音色特征
  - 通过相似度阈值判断是否为同一说话者
  - 需要 Speaker Embedding 服务运行

### 说话者音色映射配置

```toml
[speaker_voice_mapping]
# 可用的 TTS 音色列表（用于轮询分配）
available_voices = [
    "zh_CN-huayan-medium",
    "zh_CN-xiaoyan-medium",
    "en_US-lessac-medium",
    "en_US-libritts-high"
]
```

**说明：**
- 如果使用 YourTTS，这些音色将作为备选
- 优先使用参考音频（reference_audio）进行零样本合成
- 如果没有参考音频，则使用轮询方式分配音色

### TTS 配置

```toml
[tts]
url = "http://127.0.0.1:5005/tts"  # Piper TTS（可选）
yourtts_url = "http://127.0.0.1:5004"  # YourTTS（推荐，支持零样本）
```

**说明：**
- 如果配置了 `yourtts_url`，优先使用 YourTTS
- YourTTS 支持零样本音色克隆，可以保持原说话者音色
- Piper TTS 作为备选方案

## 工作流程

### 1. 音频输入处理

```
音频帧 → VAD 检测边界 → ASR 识别文本
```

### 2. 说话者识别

```
音频段 → Speaker Embedding 提取特征 → 与已有说话者比较 → 识别或创建新说话者
```

### 3. 翻译与合成

```
识别文本 → NMT 翻译 → YourTTS 合成（使用参考音频）→ 输出音频
```

### 4. 音色分配

- **首次识别到说话者**：提取参考音频，保存到说话者记录
- **后续识别到同一说话者**：使用保存的参考音频进行零样本合成
- **新说话者**：分配新的音色，提取并保存参考音频

## 验证服务

### 检查所有服务是否运行

```powershell
# Speaker Embedding
curl http://127.0.0.1:5003/health

# YourTTS
curl http://127.0.0.1:5004/health

# NMT
curl http://127.0.0.1:5008/health

# CoreEngine
curl http://127.0.0.1:9000/health
```

### 预期响应

所有服务应返回 `200 OK` 状态码。

## 功能特性

### ✅ 已实现功能

1. **VAD（语音活动检测）**
   - 内置在 CoreEngine 中
   - 检测语音边界和自然停顿

2. **ASR（自动语音识别）**
   - 内置在 CoreEngine 中
   - 使用 Whisper 模型进行实时识别

3. **说话者识别**
   - 支持 VAD 模式和 Embedding 模式
   - Embedding 模式使用 ECAPA-TDNN 模型

4. **音色提取与分配**
   - 自动提取说话者音色特征
   - 为每个说话者分配唯一音色
   - 支持零样本音色克隆

5. **翻译与合成**
   - 使用 M2M100 进行神经机器翻译
   - 使用 YourTTS 进行零样本 TTS 合成

### 🔄 工作流程

1. **多人对话场景**
   - 系统自动识别每个说话者
   - 为每个说话者分配并保持独特音色
   - 翻译时保持原说话者音色特征

2. **轮流说话**
   - VAD 检测说话切换
   - Speaker Identifier 识别说话者变化
   - 自动切换对应的音色

3. **偶尔插嘴**
   - 快速识别新说话者
   - 分配新音色
   - 不影响其他说话者的音色

## 故障排除

### 服务无法启动

1. **检查端口占用**
   ```powershell
   netstat -ano | findstr "5003 5004 5005 5008 9000"
   ```

2. **检查 Python 环境**
   - Windows: 确认 conda 环境 `lingua-py310` 已激活
   - WSL2: 确认虚拟环境 `venv-wsl` 已激活

3. **检查 GPU 可用性**
   ```powershell
   # Windows
   python -c "import torch; print(torch.cuda.is_available())"
   
   # WSL2
   wsl nvidia-smi
   ```

### 服务间无法通信

1. **检查 WSL2 端口转发**
   ```powershell
   netsh interface portproxy show all
   ```

2. **检查防火墙设置**
   - 确保 Windows 防火墙允许本地连接

3. **检查服务 URL 配置**
   - 确认 `lingua_core_config.toml` 中的 URL 正确

### 音色识别不准确

1. **调整相似度阈值**
   ```toml
   [speaker_identification]
   similarity_threshold = 0.7  # 降低以提高敏感度，提高以降低误识别
   ```

2. **检查音频质量**
   - 确保输入音频清晰
   - 避免背景噪音过大

3. **检查 Speaker Embedding 服务**
   - 确认服务正常运行
   - 检查 GPU 是否可用

## 性能优化

### GPU 加速

所有服务都支持 GPU 加速：
- Speaker Embedding: 使用 CUDA
- YourTTS: 使用 CUDA
- NMT: 自动检测 GPU
- ASR (Whisper): 使用 CUDA（如果编译时启用）

### 内存优化

- Speaker Embedding: 约 500MB GPU 内存
- YourTTS: 约 2GB GPU 内存
- NMT: 约 1GB GPU 内存
- ASR: 约 1GB GPU 内存

**建议：** 至少 8GB GPU 内存

## 下一步

1. **测试多人对话场景**
   - 录制多人对话音频
   - 验证音色识别和分配功能

2. **优化参数**
   - 根据实际场景调整相似度阈值
   - 优化 VAD 参数

3. **性能监控**
   - 监控服务响应时间
   - 优化 GPU 使用率

## 相关文档

- [服务启动命令参考](core/engine/examples/SERVICE_STARTUP_COMMANDS.md)
- [虚拟环境设置指南](core/engine/VIRTUAL_ENVIRONMENT_SETUP.md)
- [故障排除指南](core/engine/TROUBLESHOOTING.md)

