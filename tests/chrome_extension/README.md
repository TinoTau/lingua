# Chrome 插件测试总览

本目录按“模型➡核心引擎➡插件端到端➡UI/体验”四个层次规划测试。  
每个子文档说明了测试目标、覆盖功能、操作步骤以及配套脚本。运行前请确认：

- `core/engine/models` 中的模型文件已经准备完毕。
- Chrome 插件项目已执行依赖安装（`npm install` / `pnpm install`）。
- 若需驱动 WASM 引擎，先执行 `cargo build --target wasm32-unknown-unknown` 并生成对应的 `engine.wasm`。

测试层次对应关系：

| 文档 | 目标 | 主要职责 |
| --- | --- | --- |
| `01-model-validation.md` | 模型与环境自检 | 确认 ONNX 模型、配置文件和依赖加载正常 |
| `02-engine-integration.md` | 核心引擎集成 | 验证 `CoreEngine` 模块事件流和性能指标 |
| `03-extension-e2e.md` | 插件端到端 | 浏览器环境下从音频采集到 TTS 播放的完整流程 |
| `04-ui-ux.md` | UI / 体验 | Popup / Overlay 交互、状态刷新、可访问性 |

现有的脚本样例存放在 `tests/chrome_extension/scripts/` 下，可按需扩展。运行结果请记录到对应的测试文档中，确保回溯。*** End Patch

