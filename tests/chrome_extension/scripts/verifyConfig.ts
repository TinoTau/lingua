import { existsSync, readdirSync } from "fs";
import { join, resolve } from "path";

interface EngineConfig {
    mode: string;
    sourceLanguage: string;
    targetLanguage: string;
    featureFlags: Record<string, boolean>;
}

const CONFIG_PATH = resolve(__dirname, "../../../configs/env/templates/chrome.dev.json");
const MODEL_ROOT = resolve(__dirname, "../../../core/engine/models/nmt");

function loadConfig(): EngineConfig {
    const json = require(CONFIG_PATH);
    return json as EngineConfig;
}

function resolveNmtDir(source: string, target: string): string {
    return `marian-${source}-${target}`;
}

function ensureDirectoryExists(path: string) {
    if (!existsSync(path)) {
        throw new Error(`缺少目录：${path}`);
    }
}

function ensureHasModelFiles(dir: string) {
    const files = readdirSync(dir);
    const required = ["model.onnx", "model.onnx_data", "tokenizer_config.json", "source.spm", "target.spm"];
    for (const file of required) {
        if (!files.includes(file)) {
            throw new Error(`目录 ${dir} 缺少文件 ${file}`);
        }
    }
}

function main() {
    console.log("== 校验 chrome.dev 配置与模型 ==");
    const cfg = loadConfig();
    console.log(`当前模式：${cfg.mode}，默认语言：${cfg.sourceLanguage} -> ${cfg.targetLanguage}`);

    const dir = resolveNmtDir(cfg.sourceLanguage, cfg.targetLanguage);
    const targetPath = join(MODEL_ROOT, dir);
    ensureDirectoryExists(targetPath);
    ensureHasModelFiles(targetPath);
    console.log(`✅ 找到 NMT 模型目录 ${dir}`);

    const requiredFlags = ["enableEmotionAdapter", "enablePersonaAdapter"];
    requiredFlags.forEach((flag) => {
        if (!cfg.featureFlags?.[flag]) {
            throw new Error(`featureFlags.${flag} 未开启，可能导致相关功能不可用`);
        }
    });
    console.log("✅ featureFlags 配置合理");
    console.log("== 配置校验通过 ==");
}

main();

