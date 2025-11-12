# 层级 4：UI / 体验测试

## 测试目标
- 确保 Popup、Overlay、Options 等界面显示的数据与后台事件保持一致。
- 验证状态管理（`UiStateStore`）在各种事件下的正确性。
- 检查文案、布局、可访问性和国际化支持。

## 功能点与测试内容

| 功能 | 测试要点 | 脚本 / 操作 |
| --- | --- | --- |
| 状态管理 | `UiStateStore` 对 `EngineEvent` 的 reducer 是否正确更新 transcript/emotion/audio/errors | `npm run test:ui:store` → `scripts/ui/uiStateStore.spec.ts` |
| 事件驱动 UI | Popup 中的字幕、翻译、情绪历史是否与事件一致；TTS 播放指示是否实时 | Playwright 测试 `scripts/ui/popupRender.spec.ts` |
| 多语言界面 | 检查 `shared/localization` 文案在 en-US / zh-CN 下是否正确显示 | `scripts/ui/i18nSmoke.spec.ts` |
| 可访问性 | 键盘导航、ARIA 标签、对比度、字体大小 | 使用 Chrome Lighthouse / axe DevTools 或自写 `scripts/ui/accessibility.spec.ts` |
| 设置页面 | 模式切换（快速 / 平衡 / 高精度）、Persona 下拉是否持久化、同步到背景逻辑 | 手动 + `scripts/ui/settingsPersistence.spec.ts` |
| 错误提示 | 当引擎抛出 `EngineError`、TTS 播放失败时，UI 是否弹出 toast/提示，用户能否重试 | `scripts/ui/errorToast.spec.ts` |

## 自动化测试指引

1. 安装依赖：
   ```bash
   npm install
   npm install @playwright/test axe-playwright
   ```
2. 运行 UI 单元测试：
   ```bash
   npm run test:ui:store
   ```
3. 运行 Playwright UI 脚本：
   ```bash
   npm run test:ui -- --headed
   ```
4. 可访问性扫描：
   ```bash
   npm run test:ui:a11y
   ```

## 手动检查清单
- [ ] Popup 打开后延迟 < 1s，状态即时刷新。
- [ ] 字幕、翻译区域对长文本自动换行。
- [ ] 情绪历史列表不会无限增长（只保留最近 N 条）。
- [ ] TTS 播放可暂停/继续（如有该功能）。
- [ ] Options 中保存的配置刷新页面后仍生效。
- [ ] 文案/按钮遵循设计（字体、颜色、间距）。

## 记录要求
- Playwright 生成的截图/视频保存至 `tests/chrome_extension/results/ui/`。
- 手动体验结果写入本文件末尾，注明版本、浏览器、操作系统。

## 最新测试记录
- `TODO`：待首次 UI 验收后填写。*** End Patch

