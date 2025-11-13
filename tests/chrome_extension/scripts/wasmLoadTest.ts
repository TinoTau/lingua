/**
 * 确认 stub engine.wasm 可被 WebAssembly 实例化并导出预期函数。
 *
 * 用法：
 *   npx ts-node --project tsconfig.test.json tests/chrome_extension/scripts/wasmLoadTest.ts
 */

import { readFileSync } from "fs";
import { resolve } from "path";

async function main() {
  const wasmPath = resolve(__dirname, "../../../clients/chrome_extension/core/engine.wasm");

  const buffer = readFileSync(wasmPath);
  const { instance } = await WebAssembly.instantiate(buffer, {});
  const exports = instance.exports as Record<string, unknown>;

  const required = ["initialize", "dispose", "pushAudioFrame", "pollEvents"];
  const missing = required.filter((name) => typeof exports[name] !== "function");
  if (missing.length > 0) {
    throw new Error(`engine.wasm 缺少导出函数: ${missing.join(", ")}`);
  }

  console.log(`✅ wasm 导出函数: ${required.join(", ")}`);
}

main().catch((error) => {
  console.error("❌ wasm 加载测试失败:", error);
  process.exit(1);
});

