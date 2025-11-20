# WSL2 Piper 中文 TTS 服务部署说明

**文档用途：**  
本说明文档用于指导开发人员在 Windows 11/10（支持 WSL2）环境下，部署基于 **Piper-tts** 的本地中文 TTS HTTP 服务，供 Lingua 项目（CoreEngine / B 端 / 桌面端 / Chrome 插件）调用。

部署完成后，Windows 侧可以通过：

> `http://127.0.0.1:5005/tts`  

向 WSL2 里的 Piper 服务发送文本，获得中文语音（WAV）。

---

## 1. 环境要求

### 1.1 操作系统

- 推荐：**Windows 11**（已内置良好的 WSL2 支持）  
- 可选：Windows 10 2004 及以上版本（需手动启用 WSL2）

### 1.2 权限要求

- 需要在 Windows 上具备 **管理员权限**（用于启用 WSL、安装 Ubuntu）。

### 1.3 硬件建议

- CPU：建议 4 核以上（如 AMD Ryzen / Intel i5+）  
- 内存：≥ 8 GB（推荐 16 GB）  
- 磁盘空间：至少预留 10 GB 给 WSL（Ubuntu + 模型文件）

---

## 2. 步骤总览

1. 在 Windows 上启用 **WSL2** 并安装 Ubuntu。  
2. 在 Ubuntu 中安装 Python3 及虚拟环境。  
3. 通过 `pip` 安装 **Piper-tts（带 HTTP 支持）**。  
4. 下载至少一个 **中文 Piper 模型**（如 `zh_CN-huayan-medium`）。  
5. 启动 Piper HTTP 服务（监听 0.0.0.0:5005）。  
6. 在 Windows 侧通过 HTTP 请求测试 `/tts` 接口。  
7. 在 Lingua 项目中配置 TTS endpoint 为 `http://127.0.0.1:5005/tts`。

---

## 3. 启用 WSL2 并安装 Ubuntu

> 以下步骤在 **Windows PowerShell（管理员权限）** 下执行。

### 3.1 启用 WSL

1. 打开「开始菜单」，搜索 **PowerShell**，右键 → 以管理员身份运行。
2. 执行：

   ```powershell
   wsl --install
   ```

   - 如果是 Windows 11，通常会：  
     - 启用 WSL 所需组件；  
     - 自动安装默认的 Ubuntu 发行版。  
   - 如果提示已启用，可直接跳到 **3.3 设置默认版本为 WSL2**。

3. 安装完成后，按提示 **重启电脑**。

### 3.2 首次启动 Ubuntu

1. 重启后，在开始菜单中搜索 **Ubuntu**（例如“Ubuntu”、“Ubuntu 22.04 LTS”等）。  
2. 首次启动会提示设置：  
   - UNIX 用户名（如：`lingua`）  
   - 密码（记住，用于后续 `sudo`）。

### 3.3 确认 WSL2 版本

在 **PowerShell（普通权限即可）** 中执行：

```powershell
wsl -l -v
```

若看到类似输出：

```text
  NAME      STATE           VERSION
* Ubuntu    Running         2
```

说明已使用 WSL2。若 VERSION 为 1，可执行：

```powershell
wsl --set-version Ubuntu 2
```

（其中 `Ubuntu` 为你的发行版名称，以 `wsl -l -v` 输出为准）

---

## 4. 在 Ubuntu 中安装 Python 与 Piper

以下命令均在 **Ubuntu 终端** 中执行（即 WSL 窗口）。

### 4.1 更新系统与安装依赖

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y python3 python3-venv python3-pip git curl
```

### 4.2 创建虚拟环境（推荐）

```bash
mkdir -p ~/piper_env && cd ~/piper_env
python3 -m venv .venv
source .venv/bin/activate
```

- 之后需要使用 Piper 时，只需：

  ```bash
  cd ~/piper_env
  source .venv/bin/activate
  ```

### 4.3 安装 Piper-tts（带 HTTP 支持）

```bash
pip install --upgrade pip
pip install "piper-tts[http]"
```

> 说明：  
> - `piper-tts` 是 Piper 的官方 Python 包；  
> - `[http]` extra 将安装 HTTP 服务相关依赖（如 FastAPI/Uvicorn 等）。

---

## 5. 下载中文 Piper 模型

Piper 官方提供多种中文声音模型，例如：

- `zh_CN-huayan-medium`（女声，常用测试模型）  

### 5.1 创建模型目录

```bash
mkdir -p ~/piper_models/zh && cd ~/piper_models/zh
```

### 5.2 从官方列表下载模型

> 由于模型文件较大，建议在 **Windows 浏览器** 中打开官方 Piper 样例页面，然后将下载链接复制到 WSL 中使用 `wget` 或 `curl` 下载。

1. 在 Windows 浏览器中访问（示例链接）：  
   - Piper 声音及样例页面（含中文模型）：  
     - https://rhasspy.github.io/piper-samples/

2. 找到中文条目（如 `zh_CN-huayan-medium`），右键复制 ONNX 模型链接和配置文件链接：
   - `zh_CN-huayan-medium.onnx`  
   - `zh_CN-huayan-medium.onnx.json`（或类似命名的配置文件）

3. 在 Ubuntu 中使用 `wget` 下载（示例）：

   ```bash
   cd ~/piper_models/zh

   # 下面 URL 请替换为实际中文模型的下载地址
   wget "https://example.com/path/to/zh_CN-huayan-medium.onnx" -O zh_CN-huayan-medium.onnx
   wget "https://example.com/path/to/zh_CN-huayan-medium.onnx.json" -O zh_CN-huayan-medium.onnx.json
   ```

> 注：模型链接随官方发布更新，请以 Piper 官方页面为准。

---

## 6. 启动 Piper HTTP 服务

Piper-tts 提供 HTTP 服务组件，可在本地监听端口。具体命令以当前 piper-tts 版本文档为准，这里给出推荐模式（示例）：

### 6.1 设置环境变量

```bash
cd ~/piper_env
source .venv/bin/activate

export PIPER_MODEL_DIR=~/piper_models
export PIPER_DEFAULT_VOICE=zh_CN-huayan-medium
```

### 6.2 启动 HTTP 服务（示例命令）

> ⚠ 重要：实际启动命令请以 `piper-tts` 文档或 `piper1-gpl` 仓库中的 HTTP 服务说明为准。这里给出通用示例，供开发人员参考。

```bash
piper-http   --model-dir "$PIPER_MODEL_DIR"   --voice "$PIPER_DEFAULT_VOICE"   --host 0.0.0.0   --port 5005
```

或使用 Python 模块入口（视实际提供情况而定）：

```bash
python -m piper_http   --model-dir "$PIPER_MODEL_DIR"   --voice "$PIPER_DEFAULT_VOICE"   --host 0.0.0.0   --port 5005
```

> 建议：由开发人员参考官方 `piper-tts` / `piper1-gpl` 仓库文档，确认：  
> - 实际 HTTP 服务的模块名 / 命令名；  
> - 支持的参数（模型目录、voice 名称、并发设置等）；  
> - API 路径（例如 `/tts` 或 `/api/tts`）。

### 6.3 验证服务在 WSL 内监听

```bash
ss -tlnp | grep 5005
```

若看到类似：

```text
LISTEN 0      128      0.0.0.0:5005      0.0.0.0:* ...
```

说明服务已启动成功。

---

## 7. 在 Windows 侧测试 TTS HTTP 接口

### 7.1 通过 PowerShell 测试

在 Windows PowerShell 中执行：

```powershell
$body = @{
    text  = "你好，欢迎使用 Lingua 语音翻译系统。"
    voice = "zh_CN-huayan-medium"
} | ConvertTo-Json

Invoke-WebRequest `
  -Uri "http://127.0.0.1:5005/tts" `  # 具体路径以实际 API 文档为准
  -Method POST `
  -ContentType "application/json" `
  -Body $body `
  -OutFile "D:	emp\piper_test_zh.wav"
```

然后在 `D:	emp\` 下找到 `piper_test_zh.wav` 文件，用任意播放器测试是否为正常中文语音。

### 7.2 常见问题排查

- 如果请求超时或连接被拒绝：  
  - 检查 WSL 中的 Piper 服务是否正在运行；  
  - 确认端口和 host（0.0.0.0:5005）配置正确。

- 如果 HTTP 返回错误码：  
  - 在 WSL 终端查看 Piper 服务日志；  
  - 检查请求体 JSON 是否包含必须字段（如 `text`、`voice` 等）。

---

## 8. 与 Lingua 项目集成建议

### 8.1 配置文件示例（Rust CoreEngine）

在 Lingua 的配置文件（如 `config.toml`）中增加：

```toml
[tts]
backend = "piper_local"

[tts.piper_local]
endpoint = "http://127.0.0.1:5005/tts"  # 具体路径以 API 文档为准
default_voice = "zh_CN-huayan-medium"
language = "zh-CN"
timeout_ms = 8000
```

### 8.2 调用流程

1. ASR → 得到源文本。  
2. NMT / 情感分析 → 生成目标文本（中文）。  
3. CoreEngine 构造 TTS 请求：

   ```jsonc
   {
     "text": "你好，欢迎使用 Lingua 语音翻译系统。",
     "voice": "zh_CN-huayan-medium",
     "language": "zh-CN"
   }
   ```

4. 将 HTTP 返回的 WAV 字节：  
   - 写入文件供前端加载播放，或  
   - 直接通过 WebSocket / HTTP 流推到前端。

### 8.3 未来扩展示例

- 后续如需切换为：  
  - 云端 Piper 服务，或  
  - 自研中文 TTS 模型，或  
  - 商业 TTS API  
- 只需：  
  - 保持 `TtsBackend` 接口不变；  
  - 新增对应后端实现；  
  - 修改 `config.toml` 的 `tts.backend` 和 endpoint 即可。

---

## 9. 参考链接（交给开发部门）

> 以下为 **Piper / WSL / 模型下载** 的官方入口，开发人员可根据实际情况选用最新版本。

- WSL 官方安装说明（含 Windows 11/10）：  
  - https://learn.microsoft.com/windows/wsl/install  

- Ubuntu on WSL（Microsoft Store）：  
  - https://apps.microsoft.com/detail/9pn20msr04dw  

- Piper 官方仓库（当前社区维护）：  
  - https://github.com/OHF-Voice/piper1-gpl  

- Piper Python 包（piper-tts）：  
  - https://pypi.org/project/piper-tts/  

- Piper 声音与样例页面（包含中文模型）：  
  - https://rhasspy.github.io/piper-samples/  

---

## 10. 交付与验收建议

1. **基础验收**  
   - 在目标 Windows 机器上完成 WSL2 + Ubuntu 安装；  
   - 在 WSL 中成功安装 `piper-tts[http]`；  
   - 下载至少一个中文模型；  
   - 在本机（WSL 内）调用 Piper 合成语音成功。

2. **HTTP 服务验收**  
   - 在 WSL 内启动 Piper HTTP 服务；  
   - Windows PowerShell 能通过 `http://127.0.0.1:5005/tts` 获取 WAV 文件；  
   - 验证返回音频语音清晰、内容正确。

3. **项目集成验收**  
   - Lingua CoreEngine 使用配置化 endpoint 接入 Piper；  
   - 完整 S2S 流程（ASR → NMT → TTS）可在本机模拟一条中文翻译语音；  
   - Chrome 插件 / 桌面端可以播放合成语音。

此文档可直接交给开发部门作为 **WSL2 Piper 中文 TTS 部署指南** 使用。
