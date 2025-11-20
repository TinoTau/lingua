# Piper-TTS 集成技术方案（PC 本地部署 + 移动端在线调用，含私有模型预留）

**面向对象：** CoreEngine / 后端服务 / 桌面端 / 移动端 / Chrome 插件开发团队  
**当前前提：**  
- 尚未开始训练自有 TTS 模型；  
- 中文 TTS 选用 Piper-TTS；  
- 仅考虑**联网场景**：  
  - 桌面 PC 端：Piper 模型和依赖打包在本地，以提高响应速度；  
  - 移动端：通过网络调用后端 TTS 服务；  
- 离线（无网络）完整自研 TTS 留给后续版本。  

---

## 1. 总体思路

在现有“Whisper + NMT + Emotion + CoreEngine”架构基础上：

1. **PC 端（含 Chrome 插件场景）**  
   - 在用户 PC 上安装一个本地组件（可与 CoreEngine/B 端进程一起或独立）  
   - 本地组件内包含：  
     - 嵌入式 Python / Piper 运行环境；  
     - Piper 中文模型文件；  
   - Piper 以 **本地 HTTP 服务（127.0.0.1）** 的方式运行，供本机 CoreEngine 或本机网关调用。  
   - 音频合成无需经过外网，响应速度高且不受网络抖动影响。  

2. **移动端**  
   - App 调用云端后端 API；  
   - 云端后端调用部署在服务器上的 Piper-TTS 服务（同样是 HTTP）；  
   - 模型与依赖部署在服务器，移动端无需本地 TTS 能力。  

3. **统一抽象层**  
   - CoreEngine 内部通过统一的 `TtsBackend` 接口与 `TtsRouter` 路由 TTS 请求；  
   - 对 CoreEngine 来说，“PC 本地 Piper”与“云端 Piper”只是不同的 `endpoint` 配置；  
   - 为未来自研/付费 TTS 模型预留可插拔后端。  

---

## 2. 调整后的整体架构

### 2.1 逻辑视图（统一抽象）

```text
┌────────────────────────────────────────────┐
│                CoreEngine (Rust)          │
│ ┌───────────┐  ┌──────────┐  ┌──────────┐ │
│ │ ASR       │  │  NMT     │  │ Emotion  │ │
│ └───────────┘  └──────────┘  └──────────┘ │
│              ┌──────────────────────────┐ │
│              │        TTS Router        │ │
│              │  ┌────────────────────┐  │ │
│              │  │ EnglishTtsBackend │  │ │ (本地 ONNX)
│              │  └────────────────────┘  │ │
│              │  ┌────────────────────┐  │ │
│              │  │ ChineseTtsBackend │  │ │ (HTTP → Piper)
│              │  └────────────────────┘  │ │
│              └──────────────────────────┘ │
└────────────────────────────────────────────┘
```

### 2.2 部署视图

#### 2.2.1 PC 端（本地 TTS）

```text
Chrome 插件 / 桌面 UI
        │
        ▼ localhost:PORT / WebSocket / Native bridge
  ┌───────────────────────┐
  │   本地 Core/B 端组件   │  ← 安装在 PC 上，可包含 CoreEngine
  │  ┌──────────────────┐ │
  │  │ CoreEngine (Rust)│ │
  │  └──────────────────┘ │
  │    │              │    │
  │    │ HTTP(127.0.0.1)   │
  │    ▼              │    │
  │  ┌────────────────────┐│
  │  │ Piper-TTS Service  ││  ← 本地 Python/Piper + 中文模型
  │  └────────────────────┘│
  └────────────────────────┘
```

- Chrome 插件可以：  
  - 直接调用本地 Core/B 端（如 `http://127.0.0.1:8080/api/...`）；  
  - 或通过现有 Electron/桌面应用与本地 CoreEngine 通信。  
- CoreEngine 调用本机 Piper 服务：`http://127.0.0.1:5005/tts`。  

#### 2.2.2 移动端（云端 TTS）

```text
Mobile App
   │ HTTPS
   ▼
API Gateway / B 端云服务
   │
   ▼
CoreEngine (Rust, 云上)
   │
   ▼ HTTP(内网)
Piper-TTS Service (云上，Python/Piper + 模型)
```

- 移动端只需调用云端统一 API；  
- CoreEngine 在云端调用内网的 Piper 服务（逻辑与 PC 本地相同，只是 endpoint 不同）。  

---

## 3. Piper-TTS 服务设计

### 3.1 统一 HTTP 接口规范

无论是 PC 本地 Piper 还是云端 Piper，接口统一为：

**Request**

```http
POST /tts
Content-Type: application/json

{
  "text": "你好，欢迎使用语音翻译系统。",
  "lang": "zh-CN",
  "voice": "zh_female_1",   // 可选，不传则用默认
  "sample_rate": 16000      // 可选，默认 22050 或模型默认
}
```

**Response（推荐返回 WAV 二进制）**

- HTTP 200 + `Content-Type: audio/wav`：WAV 文件字节流。  

如需要也可以扩展为 JSON + base64，但 CoreEngine 使用 WAV 更简单。

### 3.2 模型与目录结构

#### PC 端本地部署（示例目录）

假设安装根目录为：`C:\Program Files\LinguaTTS\piper`

```text
C:\Program Files\LinguaTTS\piper  ├─ python_env\           # 嵌入式 Python 或虚拟环境（可选）
  ├─ models  │   └─ zh  │       └─ zh_female_1.onnx   # 中文女声模型（示例）
  ├─ tts_service.py        # Piper HTTP 服务脚本
  └─ run_piper_service.bat # 启动脚本（安装后注册为服务/自启）
```

#### 云端部署（Linux 示例）

```text
/opt/piper/
  ├─ models/
  │   └─ zh/
  │       └─ zh_female_1.onnx
  └─ tts_service.py
```

**具体模型选择**：  
从 Piper 官方 `zh_Hans` 声音中选一套质量较好的模型（男声或女声），统一命名为 `zh_female_1.onnx`（项目内部约定即可）。  

### 3.3 Python 服务端示例（FastAPI + Piper CLI）

> PC 和云端可以复用同一份服务代码，只在部署方式上区分（本机 vs 服务器）。

#### 安装依赖（示意）

```bash
# 安装 fastapi 和 uvicorn
pip install fastapi uvicorn

# 安装 piper 可执行文件（参考 Piper 官方文档）
# 例如下载 piper 可执行文件并添加到 PATH
```

#### `tts_service.py` 示例

```python
from fastapi import FastAPI, HTTPException
from fastapi.responses import Response
from pydantic import BaseModel
import subprocess
import tempfile
from pathlib import Path

app = FastAPI()

# 简单配置：语言 -> 模型路径映射
MODEL_MAP = {
    "zh-CN": "C:/Program Files/LinguaTTS/piper/models/zh/zh_female_1.onnx",
    # 云端部署时改成 /opt/piper/models/zh/zh_female_1.onnx
    # 或通过环境变量控制路径
}

class TtsRequest(BaseModel):
    text: str
    lang: str = "zh-CN"
    voice: str | None = None
    sample_rate: int | None = None  # 可选，Piper 本身有默认采样率

@app.post("/tts")
def tts(req: TtsRequest):
    if not req.text.strip():
        raise HTTPException(status_code=400, detail="Empty text")

    model_path = MODEL_MAP.get(req.lang, MODEL_MAP["zh-CN"])

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_wav = Path(tmpdir) / "out.wav"

        cmd = [
            "piper",
            "--model", str(model_path),
            "--output_file", str(tmp_wav),
        ]

        # 根据需要追加采样率等参数
        # if req.sample_rate:
        #     cmd.extend(["--sample_rate", str(req.sample_rate)])

        try:
            proc = subprocess.run(
                cmd,
                input=req.text.encode("utf-8"),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )
        except subprocess.CalledProcessError as e:
            raise HTTPException(
                status_code=500,
                detail=f"TTS generation failed: {e.stderr.decode('utf-8', errors='ignore')}",
            )

        if not tmp_wav.exists():
            raise HTTPException(status_code=500, detail="TTS output wav not found")

        data = tmp_wav.read_bytes()
        return Response(content=data, media_type="audio/wav")
```

#### 启动服务

PC 端（本地）：

```bash
uvicorn tts_service:app --host 127.0.0.1 --port 5005
```

- 可由安装器创建 Windows 服务 / 后台进程；  
- 也可由本地 Core/B 端进程在启动时对子进程进行拉起和健康检查。  

云端：

```bash
uvicorn tts_service:app --host 0.0.0.0 --port 5005
```

- 建议放入 Docker 镜像，由 K8s / supervisor 管理；  
- 只在内网暴露（例如 `http://piper-tts:5005/tts`）。  

---

## 4. CoreEngine 侧改造方案

### 4.1 抽象 TTS 接口与路由器

在 `core/engine` 中引入统一接口（示意）：

```rust
#[derive(Clone, Copy, Debug)]
pub enum LanguageId {
    En,
    Zh,
    // 未来可扩展更多语言
}

pub struct TtsRequest {
    pub text: String,
    pub lang: LanguageId,
}

pub struct TtsResult {
    pub wav_bytes: Vec<u8>, // audio/wav 二进制
}

#[async_trait::async_trait]
pub trait TtsBackend: Send + Sync {
    async fn synthesize(&self, req: TtsRequest) -> anyhow::Result<TtsResult>;
}
```

定义 TTS 路由器：

```rust
use std::sync::Arc;

pub struct TtsRouter {
    english_backend: Arc<dyn TtsBackend>,
    chinese_backend: Arc<dyn TtsBackend>,
}

impl TtsRouter {
    pub fn new(
        english_backend: Arc<dyn TtsBackend>,
        chinese_backend: Arc<dyn TtsBackend>,
    ) -> Self {
        Self { english_backend, chinese_backend }
    }

    pub async fn synthesize(&self, req: TtsRequest) -> anyhow::Result<TtsResult> {
        match req.lang {
            LanguageId::En => self.english_backend.synthesize(req).await,
            LanguageId::Zh => self.chinese_backend.synthesize(req).await,
        }
    }
}
```

### 4.2 英文 TTS Backend（保留现有 ONNX 实现）

```rust
pub struct EnglishOnnxTtsBackend {
    // ORT session、tokenizer 等字段
}

#[async_trait::async_trait]
impl TtsBackend for EnglishOnnxTtsBackend {
    async fn synthesize(&self, req: TtsRequest) -> anyhow::Result<TtsResult> {
        // 1. 文本编码
        // 2. ONNX 推理
        // 3. 生成 WAV/PCM，封装到 TtsResult
        unimplemented!()
    }
}
```

### 4.3 中文 Piper Backend（调用 HTTP 服务）

对于 PC 本地和云端，仅 endpoint 不同：  

- PC 本地：`http://127.0.0.1:5005/tts`  
- 云端：`http://piper-tts:5005/tts`

实现示例：

```rust
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct PiperTtsHttpRequest<'a> {
    text: &'a str,
    lang: &'a str,      // "zh-CN"
    voice: Option<&'a str>,
    sample_rate: Option<u32>,
}

pub struct ChinesePiperTtsBackend {
    http: Client,
    endpoint: String, // 如 "http://127.0.0.1:5005/tts" 或云端内网 URL
    default_voice: String,
}

impl ChinesePiperTtsBackend {
    pub fn new(endpoint: String, default_voice: String) -> Self {
        Self {
            http: Client::new(),
            endpoint,
            default_voice,
        }
    }
}

#[async_trait::async_trait]
impl TtsBackend for ChinesePiperTtsBackend {
    async fn synthesize(&self, req: TtsRequest) -> anyhow::Result<TtsResult> {
        let body = PiperTtsHttpRequest {
            text: &req.text,
            lang: "zh-CN",
            voice: Some(&self.default_voice),
            sample_rate: None,
        };

        let resp = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Piper TTS failed: {} - {}", status, text);
        }

        let wav_bytes = resp.bytes().await?.to_vec();
        Ok(TtsResult { wav_bytes })
    }
}
```

### 4.4 CoreEngine 初始化时选择本地 / 云端 Piper

通过配置文件区分 PC/云端部署：

```toml
[tts]
backend_zh = "piper"
backend_en = "onnx"

[tts.piper]
# PC 端本地部署时：
endpoint = "http://127.0.0.1:5005/tts"

# 云端部署时：
# endpoint = "http://piper-tts:5005/tts"

default_voice = "zh_female_1"
```

构建时：

```rust
fn build_zh_tts_backend(cfg: &Config) -> Arc<dyn TtsBackend> {
    Arc::new(ChinesePiperTtsBackend::new(
        cfg.tts.piper.endpoint.clone(),
        cfg.tts.piper.default_voice.clone(),
    ))
}
```

---

## 5. PC 端（Chrome 插件 / 桌面）的调用路径

### 5.1 PC 端安装内容

- 本地 Core/B 端组件（包含 CoreEngine，可选 Electron UI 或仅作为后台服务）；  
- 本地 Piper-TTS 服务及模型文件；  
- 安装器负责：  
  - 解压 Piper 模型与二进制；  
  - 安装 Python/虚拟环境（如果需要）；  
  - 注册 `run_piper_service.bat` 或 systemd/nssm/service，使 Piper 服务随系统 / 程序启动。  

### 5.2 Chrome 插件调用流程（PC）

```text
Chrome 插件
   │  HTTP/WebSocket (localhost)
   ▼
本地 Core/B 端组件 (包含 CoreEngine)
   │
   ▼
TTS Router (Rust)
   │
   ▼ HTTP 127.0.0.1:5005/tts
Piper-TTS Service (本地)
```

- 插件始终只调用本地 Core/B 端的统一 API，比如：  

  ```http
  POST http://127.0.0.1:8080/api/v1/s2s_translate
  ```

- Core/B 端负责：  
  1. 调用 Whisper → NMT；  
  2. 调用 `TtsRouter`；  
  3. 将 `wav_bytes` 返回给插件播放。  

（如果当前架构是插件 → 远程 B 端，也可以保持远程调用，只是 TTS 服务在远程服务器上，此时与移动端路径一致。）

---

## 6. 移动端接入方式（完全在线）

移动端只需要：

```http
POST https://api.xxx.com/v1/s2s_translate
{
  "audio": "...",
  "source_lang": "zh-CN",
  "target_lang": "en-US"
}
```

- 后端统一调用云端 CoreEngine；  
- CoreEngine 使用 `TtsRouter`：  
  - 中文目标语调用云端 Piper；  
  - 英文目标语用本地/云端 ONNX TTS；  
- 返回合成后的 WAV/PCM 给移动端播放。  

**移动端不需要本地 TTS 模型，也不需要 Piper。**

---

## 7. 为未来私有模型预留接口

未来可能会：

- 购买商用云端 TTS；  
- 或训练自研中文/英文 TTS 模型（可在云端或本地运行）；  
- 实现完全离线 TTS。  

### 7.1 配置层预留

```toml
[tts]
backend_zh = "piper"        # 将来可改为 "private_cloud" / "private_local"
backend_en = "onnx"

[tts.piper]
endpoint = "http://127.0.0.1:5005/tts"
default_voice = "zh_female_1"

[tts.private_cloud]
endpoint = "https://api.my-tts.com/v1/tts"
api_key = "..."

[tts.private_local]
# 用于未来本地 ONNX / 自研引擎时的初始化参数
```

### 7.2 后端类型扩展（示意）

```rust
enum ZhTtsBackendKind {
    Piper,
    PrivateCloud,
    PrivateLocal,
}

fn build_zh_tts_backend(cfg: &Config) -> Arc<dyn TtsBackend> {
    match cfg.tts.backend_zh {
        ZhTtsBackendKind::Piper => Arc::new(ChinesePiperTtsBackend::new(
            cfg.tts.piper.endpoint.clone(),
            cfg.tts.piper.default_voice.clone(),
        )),
        ZhTtsBackendKind::PrivateCloud => Arc::new(PrivateCloudZhTtsBackend::new(
            cfg.tts.private_cloud.endpoint.clone(),
            cfg.tts.private_cloud.api_key.clone(),
        )),
        ZhTtsBackendKind::PrivateLocal => Arc::new(PrivateLocalZhTtsBackend::new(
            /* 本地 ONNX / 自研引擎初始化 */
        )),
    }
}
```

当你训练好自己的模型或接入商用 TTS，只需：

1. 新增一个 `PrivateCloudZhTtsBackend` / `PrivateLocalZhTtsBackend` 实现 `TtsBackend`；  
2. 更新配置 `backend_zh = "private_cloud"` 或 `"private_local"`；  

其余业务代码、客户端协议和路由逻辑都保持不变。  

---

## 8. 总结

在“PC 端本地打包 Piper 模型与依赖 + 移动端通过网络调用”的前提下：

1. **PC 端**：  
   - 安装本地 Core/B 端组件 + Piper-TTS 服务；  
   - Chrome 插件 / 桌面应用通过本地 CoreEngine 完成 S2S 处理；  
   - TTS 请求由 CoreEngine 调用本地 `http://127.0.0.1:5005/tts`，响应快速、稳定。  

2. **移动端**：  
   - 调用云端统一 API；  
   - 云端 CoreEngine 调用云端 Piper 服务；  
   - 移动端无需本地模型，仅需联网。  

3. **架构层面**：  
   - CoreEngine 内通过统一的 `TtsBackend` + `TtsRouter` 实现多语言 TTS 路由；  
   - Piper 只是一种中文 TTS 的后端实现；  
   - 为未来自研/付费私有模型预留接口，只需新增 Backend 并调整配置即可。  

该方案兼顾：

- 当前阶段快速落地可用的中文 TTS；  
- PC 端高响应速度（本地模型）；  
- 移动端统一在线方案；  
- 未来可平滑迁移到自研或商用 TTS 模型。  



# 9. 分步实施与验证方案（可独立测试）

为方便开发团队逐项验收，本方案拆分成 8 个可独立测试的阶段。每一步都可以单独验证，通过后再进入下一步。

---

## 步骤 1：在本地命令行直接验证 Piper + 模型

**目标：** 在 PC 上用 Piper 命令行生成中文 WAV。

### 测试方式

```powershell
echo 你好 | piper --model "path/to/zh_female_1.onnx" --output_file test_zh.wav
```

### 成功标准
- 生成 test_zh.wav
- 能正常播放且内容清晰

---

## 步骤 2：启动本地 Piper HTTP 服务，使用 HTTP 调用验证

**目标：** 启动 `http://127.0.0.1:5005/tts`

### 测试方式

```bash
curl -X POST "http://127.0.0.1:5005/tts" ^
     -H "Content-Type: application/json" ^
     -d "{"text":"你好","lang":"zh-CN"}" ^
     --output test_api.wav
```

### 成功标准
- HTTP 200
- 返回 WAV 正常播放

---

## 步骤 3：用独立 Rust 小程序调用 Piper HTTP（不接 CoreEngine）

**目标：** 验证 Future Rust backend 的调用逻辑。

### 成功标准
- Rust 代码成功保存 test_rust.wav
- 播放正常

---

## 步骤 4：CoreEngine 内实现 ChinesePiperTtsBackend（单元测试级验证）

**目标：** 使用 reqwest 调用本地 Piper 并在单测中验证。

### 成功标准
- 单测通过
- WAV 字节长度 > 1024

---

## 步骤 5：接入 TtsRouter，在 CoreEngine 中测试文本→语音（不走完整 S2S）

### 成功标准
- `coreengine_zh_test.wav` 成功生成并能播放

---

## 步骤 6：完整 S2S 流集成测试（Whisper → NMT → Piper TTS）

### 成功标准
- API 返回 200
- 返回目标语音正确且可理解

---

## 步骤 7：移动端路径验证（云端 Piper）

### 成功标准
- 移动端通过后端 API 获得音频
- 日志显示 CoreEngine 调用云端 Piper

---

## 步骤 8：PC 端安装流程验证（工程化）

### 成功标准
- 在干净环境中安装后可直接使用
- Piper 服务自动随程序启动
- Chrome 插件/桌面端调用完整链路成功
