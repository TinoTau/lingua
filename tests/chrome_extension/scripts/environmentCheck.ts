/**
 * 环境依赖自检脚本
 *
 * 检查 Node 版本、Rust、wasm32 target、Chrome 权限等必要条件。
 *
 * 使用方式：
 *   npm run test:env
 */

import { execSync } from "child_process";

function checkCommand(command: string, successMessage: string) {
  try {
    const output = execSync(command, { stdio: "pipe" }).toString().trim();
    console.log(`✅ ${successMessage}\n${output}\n`);
  } catch (error) {
    throw new Error(`❌ 命令失败: ${command}\n${error}`);
  }
}

function main() {
  console.log("== 环境自检 ==");
  console.log(`Node 版本: ${process.version}`);
  if (Number(process.versions.node.split(".")[0]) < 18) {
    throw new Error("Node.js 需要 >= 18 才能运行 Playwright/Puppeteer 测试");
  }

  checkCommand("npm --version", "npm 已安装");
  checkCommand("rustc --version", "Rust 已安装");
  checkCommand(
    "rustup target list --installed",
    "已安装的 Rust targets（确认包含 wasm32-unknown-unknown）"
  );
  checkCommand("chrome --version", "Chrome/Chromium 可执行可用（如无可改用浏览器路径配置）");

  console.log("== 自检完成 ==");
}

main();

