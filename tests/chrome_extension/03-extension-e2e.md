# 层级 3：Chrome 插件端到端测试

## 测试目标
- 在实际浏览器环境下验证从音频采集到 TTS 播放的完整链路。
- 覆盖多语言、情绪标签、Persona 语气切换等核心功能。
- 验证异常处理（权限拒绝、模型缺失、网络波动）是否符合预期。

## 功能点与测试内容

| 功能 | 测试要点 | 脚本 / 工具 |
| --- | --- | --- |
| 安装与启动 | 扩展安装是否无报错；背景页加载模型是否成功（查看 console） | `scripts/e2e/installAndBoot.spec.ts`（Puppeteer/Playwright） |
| 音频采集 & ASR | 通过虚拟麦克风推送预录音频，检查转写是否与预期文本匹配 | `scripts/e2e/streamAudio.spec.ts` |
| 多语言翻译 | 依次切换 en→zh、en→ja、en→es，确认 `NmtFinal` 文本正确 | `scripts/e2e/languageSwitch.spec.ts --languages en-zh,en-ja` |
| 情绪 & Persona | 调整 Persona 配置（正式 / 口语），检查输出语气变化；观察情绪标签是否变化 | `scripts/e2e/personaEmotion.spec.ts` |
| TTS 播放 | 确认 `TtsChunk` 按顺序播放、无中断；生成的音频长度与预期接近 | `scripts/e2e/ttsPlayback.spec.ts` |
| 异常路径 | 模拟：拒绝麦克风、删除某个模型文件、断网；插件是否提示并降级 | `scripts/e2e/errorHandling.spec.ts` |

## 执行步骤（示例）

1. 打包并加载扩展：  
   ```bash
   npm run build:extension
   # 或者使用开发模式直接加载 dist/ 目录
   ```
2. 启动 Playwright 测试（需要安装 Chromium）：  
   ```bash
   npx playwright install chromium
   npm run test:e2e -- --headed
   ```
3. 验证时观察：
   - 扩展背景页 console 是否有 `core-engine booted` 等日志。
   - Popup/overlay 是否实时刷新字幕、翻译、情绪。
   - TTS 播放是否自然，延迟是否在可接受范围。

## 手动测试清单
- [ ] 首次安装（模型解压 + 权限授权）
- [ ] 麦克风授权后实时翻译
- [ ] 多标签页同时运行（确认事件不串台）
- [ ] 切换目标语言、生效时间、UI 提示
- [ ] Persona 模式切换
- [ ] 情绪历史记录展示
- [ ] 异常恢复（断网后重连、模型缺失 → 提示/修复）

## 记录要求
- 自动化脚本生成的报告保存为 `tests/chrome_extension/results/e2e-report-YYYYMMDD.html`。
- 手动测试结果填写在本节末尾，包含测试人、日期、是否通过、问题回溯链接。

## 最新测试记录
- `TODO`：首次端到端测试完成后填写结论。*** End Patch

