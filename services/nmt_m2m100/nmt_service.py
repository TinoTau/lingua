# -*- coding: utf-8 -*-
"""
M2M100 NMT 服务（FastAPI）

提供 HTTP API 接口，使用 HuggingFace Transformers 运行 M2M100 模型进行翻译。
"""

from fastapi import FastAPI
from pydantic import BaseModel
from typing import Optional, Dict, Any
import time
import torch
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
import os

app = FastAPI(title="M2M100 NMT Service", version="1.0.0")

MODEL_NAME = "facebook/m2m100_418M"
DEVICE = torch.device("cuda" if torch.cuda.is_available() else "cpu")

# 全局模型和 tokenizer
tokenizer: Optional[M2M100Tokenizer] = None
model: Optional[M2M100ForConditionalGeneration] = None


class TranslateRequest(BaseModel):
    src_lang: str
    tgt_lang: str
    text: str


class TranslateResponse(BaseModel):
    ok: bool
    text: Optional[str] = None
    model: Optional[str] = None
    provider: str = "local-m2m100"
    extra: Optional[Dict[str, Any]] = None
    error: Optional[str] = None


@app.on_event("startup")
async def load_model():
    """启动时加载模型"""
    global tokenizer, model
    try:
        print(f"[NMT Service] Loading model: {MODEL_NAME}")
        print(f"[NMT Service] Device: {DEVICE}")
        
        # 检查是否有 HF_TOKEN 环境变量
        hf_token = os.getenv("HF_TOKEN")
        extra = {"token": hf_token} if hf_token else {}
        
        tokenizer = M2M100Tokenizer.from_pretrained(MODEL_NAME, **extra)
        model = M2M100ForConditionalGeneration.from_pretrained(MODEL_NAME, **extra)
        model = model.to(DEVICE).eval()
        
        print(f"[NMT Service] Model loaded successfully")
    except Exception as e:
        print(f"[NMT Service] Failed to load model: {e}")
        raise


@app.get("/health")
async def health():
    """健康检查"""
    return {
        "status": "ok" if model is not None else "not_ready",
        "model": MODEL_NAME if model is not None else None,
        "device": str(DEVICE)
    }


@app.post("/v1/translate", response_model=TranslateResponse)
async def translate(req: TranslateRequest) -> TranslateResponse:
    """翻译接口"""
    start = time.time()
    
    if model is None or tokenizer is None:
        return TranslateResponse(
            ok=False,
            error="Model not loaded",
            provider="local-m2m100"
        )
    
    try:
        # 设置源语言（重要：必须在编码前设置）
        tokenizer.src_lang = req.src_lang
        
        # 编码输入文本（M2M100 会在文本前自动添加源语言 token）
        encoded = tokenizer(req.text, return_tensors="pt").to(DEVICE)
        
        # 获取目标语言 token ID
        forced_bos = tokenizer.get_lang_id(req.tgt_lang)
        
        # 调试：验证 tokenizer 状态
        print(f"[DEBUG] Tokenizer src_lang: {tokenizer.src_lang}")
        print(f"[DEBUG] Tokenizer tgt_lang: {req.tgt_lang}")
        
        # 生成翻译
        with torch.no_grad():
            gen = model.generate(
                **encoded,
                forced_bos_token_id=forced_bos,
                num_beams=4,
                no_repeat_ngram_size=3,
                repetition_penalty=1.2,
                max_new_tokens=128,
            )
        
        # 调试：打印输入和生成的 token IDs（仅在开发时）
        print(f"[DEBUG] Input text: {req.text}")
        print(f"[DEBUG] Encoded input_ids: {encoded['input_ids'].cpu().numpy().tolist()}")
        print(f"[DEBUG] Forced BOS token ID: {forced_bos}")
        print(f"[DEBUG] Generated IDs: {gen[0].cpu().numpy().tolist()}")
        
        # 解码输出
        # M2M100 generate 返回的序列已经包含了完整的输入和目标序列
        # 使用 skip_special_tokens=True 应该能正确解码
        # 但为了确保正确，我们手动提取目标语言部分
        generated_ids = gen[0].cpu().numpy().tolist()
        tgt_lang_id = tokenizer.get_lang_id(req.tgt_lang)
        eos_token_id = tokenizer.eos_token_id
        
        # 找到目标语言 token 的位置
        tgt_start_idx = None
        for i, token_id in enumerate(generated_ids):
            if token_id == tgt_lang_id:
                tgt_start_idx = i + 1  # 跳过目标语言 token 本身
                break
        
        if tgt_start_idx is not None and tgt_start_idx < len(generated_ids):
            # 提取目标语言 token 之后的部分
            target_ids = generated_ids[tgt_start_idx:]
            # 移除 EOS token（如果存在）
            if len(target_ids) > 0 and target_ids[-1] == eos_token_id:
                target_ids = target_ids[:-1]
            # 解码目标语言部分
            if len(target_ids) > 0:
                out = tokenizer.decode(target_ids, skip_special_tokens=True)
            else:
                out = ""
        else:
            # 如果找不到目标语言 token，尝试直接解码（可能格式不同）
            # 先尝试跳过源语言部分
            src_lang_id = tokenizer.get_lang_id(req.src_lang)
            src_end_idx = None
            for i, token_id in enumerate(generated_ids):
                if token_id == src_lang_id:
                    src_end_idx = i
                    break
            
            if src_end_idx is not None:
                # 跳过源语言部分，解码剩余部分
                remaining_ids = generated_ids[src_end_idx+1:]
                # 移除 EOS
                if len(remaining_ids) > 0 and remaining_ids[-1] == eos_token_id:
                    remaining_ids = remaining_ids[:-1]
                out = tokenizer.decode(remaining_ids, skip_special_tokens=True)
            else:
                # 最后备用方案：直接解码
                out = tokenizer.decode(gen[0], skip_special_tokens=True)
        
        elapsed_ms = int((time.time() - start) * 1000)
        
        return TranslateResponse(
            ok=True,
            text=out,
            model=MODEL_NAME,
            provider="local-m2m100",
            extra={
                "elapsed_ms": elapsed_ms,
                "num_tokens": int(gen.shape[1])
            }
        )
    except Exception as e:
        return TranslateResponse(
            ok=False,
            error=str(e),
            provider="local-m2m100"
        )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5008)

