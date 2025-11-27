# Lingua Core Runtime - 编译与启动命令手册

本手册汇总当前项目在本地环境下常用的编译与启动命令，便于快速查阅与执行。所有路径均以仓库根目录 `D:\Programs\github\lingua` 为基准。

## 1. Rust CoreEngine 编译

```powershell
cd core\engine
cargo build --release --bin core_engine
```

- 生成产物：`core\engine\target\release\core_engine.exe`
- 如需清理旧产物，可先执行 `cargo clean`

## 2. 一键启动（Windows）

```powershell
cd D:\Programs\github\lingua
.\start_lingua_core.ps1
```

脚本内容包含：
1. 启动/检查 Piper TTS（WSL 5005）
2. 启动/检查 Python M2M100 NMT（5008）
3. 构建并运行 CoreEngine（9000）
4. 启动 Web PWA（8080）

所有服务日志会输出在当前 PowerShell，会自动处理端口转发与健康检查。

## 3. Web PWA 独立启动（可选）

若只需调试前端：

```powershell
cd clients\web_pwa
.\start_server.ps1 -Port 8080
```

脚本会优先尝试 `python -m http.server`，失败后退回 `npx http-server`。

## 4. NMT / TTS 服务手动启动（备用）

### NMT（Python FastAPI）

```powershell
cd services\nmt_m2m100
python -m venv .venv
.\.venv\Scripts\activate
pip install -r requirements.txt
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

### TTS（WSL Piper HTTP）

```powershell
wsl bash scripts/wsl2_piper/start_piper_service.sh
```

如需手动端口映射，可在 Windows PowerShell（管理员）中运行：

```powershell
netsh interface portproxy delete v4tov4 listenaddress=127.0.0.1 listenport=5005
netsh interface portproxy add v4tov4 listenaddress=127.0.0.1 listenport=5005 connectaddress=<WSL_IP> connectport=5005
```

## 5. CoreEngine 手动运行（跳过脚本）

```powershell
cd core\engine
cargo run --release --bin core_engine -- --config ..\..\lingua_core_config.toml
```

或在构建后直接执行：

```powershell
cd core\engine\target\release
.\core_engine.exe --config ..\..\..\lingua_core_config.toml
```

---

> 建议优先使用一键启动脚本；手动命令可用于单独调试或部署排障。完成操作后按 `Ctrl+C` 停止服务，系统会自动清理子进程与端口代理。

