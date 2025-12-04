# ASR 专用测试快速启动指南

## 快速启动命令

### 方式 1：从项目根目录运行（推荐）✅

```powershell
# 1. 切换到项目根目录
cd D:\Programs\github\lingua

# 2. 运行 CoreEngine（会自动查找配置文件）
cargo run --bin core_engine --manifest-path "core\engine\Cargo.toml"
```

**完整命令说明**：
- `cargo run` - 编译并运行
- `--bin core_engine` - 指定要运行的二进制文件
- `--manifest-path core\engine\Cargo.toml` - 指定 Cargo.toml 的路径（相对于项目根目录）
- 配置文件会自动从项目根目录查找（`lingua_core_config.toml`）

**如果配置文件不在项目根目录，可以显式指定**：
```powershell
cargo run --bin core_engine --manifest-path "core\engine\Cargo.toml" -- --config lingua_core_config.toml
```

### 方式 2：从 core/engine 目录运行

```powershell
# 切换到 core/engine 目录
cd D:\Programs\github\lingua\core\engine

# 使用相对路径指向项目根目录的配置文件
cargo run --bin core_engine -- --config ..\..\lingua_core_config.toml
```

### 方式 3：使用启动脚本（最简单）

```powershell
# 在项目根目录下
.\core\engine\scripts\start_core_engine_asr_only.ps1
```

## 启动 Web 前端

```powershell
# 在项目根目录下
cd clients\web_pwa
.\start_web_server.ps1
```

## 访问测试页面

在浏览器中打开：
```
http://localhost:8080/index_asr_only.html
```

## 测试功能

- ✅ **停顿识别**：VAD 实时检测语音边界
- ✅ **文本识别**：ASR 实时识别语音文本
- ❌ **不包含翻译**：只测试 ASR 功能
- ❌ **不包含 TTS**：只测试 ASR 功能

## 注意事项

1. **只需要 CoreEngine 服务**：不需要启动 NMT 和 TTS 服务
2. **配置文件位置**：`lingua_core_config.toml` 应该在项目根目录
3. **端口**：CoreEngine 默认运行在端口 9000

