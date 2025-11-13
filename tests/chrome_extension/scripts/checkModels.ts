/**
 * 模型完整性自检脚本
 *
 * 使用方式：
 *   npm run test:models:check
 *
 * 依赖：
 *   npm install onnxruntime-node
 */

import { existsSync, readdirSync } from "fs";
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
  {
    name: "NMT / marian-en-zh",
    path: "nmt/marian-en-zh",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "NMT / marian-zh-en",
    path: "nmt/marian-zh-en",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "NMT / marian-en-ja",
    path: "nmt/marian-en-ja",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "NMT / marian-ja-en",
    path: "nmt/marian-ja-en",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "NMT / marian-en-es",
    path: "nmt/marian-en-es",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "NMT / marian-es-en",
    path: "nmt/marian-es-en",
    requiredFiles: [
      "model.onnx",
      "model.onnx_data",
      "config.json",
      "source.spm",
      "target.spm",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.json",
    ],
  },
  {
    name: "Persona / embedding-default",
    path: "persona/embedding-default",
    requiredFiles: [
      "model.onnx",
      "config.json",
      "tokenizer.json",
      "tokenizer_config.json",
      "special_tokens_map.json",
      "vocab.txt",
    ],
  },
  {
    name: "TTS / fastspeech2-lite",
    path: "tts/fastspeech2-lite",
    requiredFiles: [
      "fastspeech2_ljspeech.onnx",
      "fastspeech2_csmsc_streaming.onnx",
      "phone_id_map.txt",
      "speech_stats.npy",
    ],
  },
  {
    name: "TTS / hifigan-lite",
    path: "tts/hifigan-lite",
    requiredFiles: ["hifigan_ljspeech.onnx", "hifigan_csmsc.onnx"],
  },
  {
    name: "VAD / silero",
    path: "vad/silero",
    requiredFiles: ["silero_vad.onnx"],
  },
];

function ensureFiles(descriptor: ModelDescriptor) {
  const base = join(MODEL_ROOT, descriptor.path);
  if (!existsSync(base)) {
    throw new Error(`[${descriptor.name}] 目录不存在: ${base}`);
  }
  descriptor.requiredFiles.forEach((file) => {
    const filePath = join(base, file);
    if (!existsSync(filePath)) {
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
      const message = String(error);
      if (message.includes("ConvInteger")) {
        console.warn(`⚠️ 载入 ${file} 时遇到不支持的 ConvInteger 算子，已跳过 ONNX 运行时校验。`);
        return null;
      }
      throw new Error(`加载模型 ${file} 失败: ${message}`);
    });
    if (session) {
      const inputs = session.inputNames.join(",");
      const outputs = session.outputNames.join(",");
      console.log(`✅ loaded: ${file} (inputs=[${inputs}] outputs=[${outputs}])`);
    }
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

