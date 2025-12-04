# Faster-Whisper ASR 迁移指南

## 概述

已成功实现从 `whisper-rs` 到 `faster-whisper`（通过 Python 微服务）的迁移。新的实现支持：

- ✅ `initial_prompt`（上下文支持）
- ✅ `condition_on_previous_text`（连续识别）
- ✅ 上下文缓存机制
- ✅ 与现有 VAD、NMT、TTS 逻辑兼容

## 已完成的工作

### 1. Python ASR 微服务

**文件**：`core/engine/scripts/asr_service.py`

- 使用 `faster-whisper` 实现 ASR 服务
- 支持 `initial_prompt` 和 `condition_on_previous_text`
- 提供 HTTP API (`/asr` 和 `/health`)
- 支持 GPU/CPU 加速

**启动方式**：
```powershell
cd core/engine/scripts
.\start_asr_service.ps1
```

### 2. Rust HTTP 客户端

**模块**：`core/engine/src/asr_http_client/`

- `client.rs`：HTTP 客户端实现
- `types.rs`：请求/响应类型定义
- 支持超时和错误处理

### 3. FasterWhisperAsrStreaming 实现

**文件**：`core/engine/src/asr_whisper/faster_whisper_streaming.rs`

- 实现 `AsrStreaming` trait
- 实现 `AsrStreamingExt` trait（支持 `accumulate_frame`、`infer_on_boundary` 等）
- 保留上下文缓存机制
- 与 `WhisperAsrStreaming` API 兼容

### 4. CoreEngineBuilder 支持

**文件**：`core/engine/src/bootstrap/builder.rs`

新增方法：
```rust
pub fn asr_with_faster_whisper(mut self, service_url: String, timeout_secs: u64) -> EngineResult<Self>
```

### 5. AsrStreamingExt Trait

**文件**：`core/engine/src/asr_streaming/ext.rs`

统一了两种 ASR 实现的扩展方法：
- `accumulate_frame`
- `get_accumulated_frames`
- `clear_buffer`
- `set_language` / `get_language`
- `infer_on_boundary`

## 使用方法

### 方式 1：使用 Faster-Whisper（推荐）

```rust
let builder = CoreEngineBuilder::new()
    .asr_with_faster_whisper("http://127.0.0.1:6006".to_string(), 30)?;
```

### 方式 2：使用原有的 Whisper-rs

```rust
let builder = CoreEngineBuilder::new()
    .asr_with_default_whisper()?;
```

## 配置

### Python 服务环境变量

- `ASR_MODEL_PATH`：模型路径（默认：`model/whisper-large-v3`）
- `ASR_DEVICE`：设备（`cpu` 或 `cuda`，默认：`cpu`）
- `ASR_COMPUTE_TYPE`：计算类型（`float32`、`float16`、`int8`，默认：`float32`）
- `ASR_SERVICE_PORT`：服务端口（默认：`6006`）

### 安装依赖

```bash
pip install -r core/engine/scripts/asr_service_requirements.txt
```

## 优势

1. **上下文支持**：真正支持 `initial_prompt`，提高识别准确度
2. **连续识别**：支持 `condition_on_previous_text`，适合连续对话
3. **GPU 加速**：支持 GPU 加速，提高推理速度
4. **更好的模型**：可以使用 `whisper-large-v3` 等更大的模型
5. **向后兼容**：保留原有 `WhisperAsrStreaming` 实现

## 注意事项

1. **服务依赖**：需要先启动 Python ASR 服务
2. **网络延迟**：HTTP 调用会有网络延迟，建议在同一台机器上运行
3. **模型下载**：首次运行需要下载模型（可通过 `faster-whisper` 自动下载）

## 下一步

1. 更新 `engine.rs` 以使用 `AsrStreamingExt` trait（当前仍使用 `unsafe` 指针转换）
2. 添加单元测试
3. 性能测试和优化
4. 文档完善

## 相关文件

- `core/engine/docs/ASR_FASTER_WHISPER_MIGRATION.md`：原始迁移方案
- `core/engine/scripts/asr_service.py`：Python ASR 服务
- `core/engine/src/asr_http_client/`：Rust HTTP 客户端
- `core/engine/src/asr_whisper/faster_whisper_streaming.rs`：新的 ASR 实现

