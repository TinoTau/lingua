/**
 * 模型完整性自检脚本
 *
 * 使用方式：
 *   npm run test:models:check
 *
 * 依赖：
 *   npm install onnxruntime-node
 */

import { readdirSync, statSync } from "fs";
import { join, resolve } from "path";
import * as ort from "onnxruntime-node";

const MODEL_ROOT = resolve(__dirname, "../../../core/engine/models");

interface ModelDescriptor {
  name: string;
  path: string;
  requiredFiles: string[];
}

const descriptors: ModelDescriptor[] = [
  {
    name: "ASR / whisper-base",
    path: "asr/whisper-base",
    requiredFiles: [
      "encoder_model_int8.onnx",
      "decoder_model_int8.onnx",
      "decoder_with_past_model_int8.onnx",
      "config.json",
      "tokenizer.json",
      "tokenizer_config.json",
      "vocab.json",
      "merges.txt",
      "preprocessor_config.json",
      "special_tokens_map.json",
    ],
  },
  {
    name: "Emotion / xlm-r",
    path: "emotion/xlm-r",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "tokenizer.json",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "sentencepiece.bpe.model",
    ],
  },
];

function ensureFiles(descriptor: ModelDescriptor) {
  const base = join(MODEL_ROOT, descriptor.path);
  descriptor.requiredFiles.forEach((file) => {
    const filePath = join(base, file);
    if (!statSync(filePath, { throwIfNoEntry: false })) {
      throw new Error(`[${descriptor.name}] 缺少文件 ${file}`);
    }
  });
}

async function loadOnnxModels() {
  const onnxFiles: string[] = [];

  const collect = (dir: string) => {
    const entries = readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) {
        collect(full);
      } else if (entry.isFile() && entry.name.endsWith(".onnx")) {
        onnxFiles.push(full);
      }
    }
  };

  collect(MODEL_ROOT);

  for (const file of onnxFiles) {
    const session = await ort.InferenceSession.create(file).catch((error) => {
      throw new Error(`加载模型 ${file} 失败: ${error}`);
    });
    console.log(`✅ loaded: ${file} (inputs=${session.inputNames.join(",")})`);
  }
}

async function main() {
  console.log("== 检查模型结构 ==");
  descriptors.forEach(ensureFiles);
  console.log("✅ 核心模型文件存在");

  console.log("== 尝试加载所有 ONNX 模型 ==");
  await loadOnnxModels();
  console.log("✅ 所有模型加载成功");
}

main().catch((error) => {
  console.error("❌ 自检失败:", error);
  process.exit(1);
});

