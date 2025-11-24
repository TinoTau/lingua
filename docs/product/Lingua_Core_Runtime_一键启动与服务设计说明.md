# Lingua Core Runtime 一键启动与服务设计说明

## 1. 概述
Lingua Core Runtime 是提供端到端实时语音翻译（Speech-to-Speech, S2S）的核心服务集群。
各前端形态（Chrome 插件、Electron、移动端、PWA）均通过统一 API 与核心服务通信，实现真正的“一套核心，多种壳”。

## 2. 核心服务组成

### 2.1 CoreEngine Service（Rust）
- 职责：
  - Whisper ASR（可内置）
  - 音频分段、停顿检测
  - 调用 NMT 服务执行翻译
  - 调用 TTS 服务生成语音
  - 对外暴露统一 S2S API
- 接口：
  - POST /s2s
  - WS /stream
  - GET /health

### 2.2 NMT Service（Python + M2M100）
- 提供翻译能力，支持 en↔zh 等语言对
- 可替换为线上翻译服务（已预留接口）
- 接口：
  - POST /translate

### 2.3 TTS Service（Piper HTTP）
- 提供语音合成能力
- 接口：
  - POST /tts

## 3. 配置文件：lingua_core_config.toml

```toml
[nmt]
url = "http://127.0.0.1:9001/translate"

[tts]
url = "http://127.0.0.1:9002/tts"

[engine]
port = 9000
whisper_model_path = "models/whisper/medium"
```

## 4. 一键启动脚本

### Windows: start_lingua_core.ps1

```powershell
Write-Host "启动 Piper TTS..."
Start-Process -NoNewWindow "piper.exe" -ArgumentList "--server --port 9002"

Write-Host "启动 Python NMT 服务..."
& python -m venv venv
& venv\Scripts\activate
python nmt_service.py --port 9001

Write-Host "启动 CoreEngine..."
Start-Process -NoNewWindow "core_engine.exe" -ArgumentList "--config lingua_core_config.toml"
```

### Linux / macOS: start_lingua_core.sh

```bash
#!/bin/bash

nohup ./piper --server --port 9002 &
source venv/bin/activate
nohup uvicorn nmt_service:app --port 9001 &
nohup ./core_engine --config lingua_core_config.toml &
```

## 5. 对前端暴露的 API

### 5.1 整句翻译（同步 S2S）
POST /s2s

输入：
```json
{
  "audio": "<base64>",
  "src_lang": "zh",
  "tgt_lang": "en"
}
```

### 5.2 流式实时翻译（WebSocket）
- WS /stream/start
- WS /stream/audio
- WS /stream/output
- WS /stream/stop

## 6. 前端（壳）接入方式
所有壳无需关心 Whisper/NMT/TTS 模型，只需：
- 采集麦克风音频
- 推送到 CoreEngine
- 播放返回音频
- 显示字幕（可选）

## 7. 推荐项目结构
```
/lingua-core
  /engine
  /nmt_service
  /tts
  start_lingua_core.ps1
  start_lingua_core.sh
  lingua_core_config.toml
```
