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
        
        # GPU 检查
        if torch.cuda.is_available():
            print(f"[NMT Service] ✓ CUDA available: {torch.cuda.is_available()}")
            print(f"[NMT Service] ✓ CUDA version: {torch.version.cuda}")
            print(f"[NMT Service] ✓ GPU count: {torch.cuda.device_count()}")
            print(f"[NMT Service] ✓ GPU name: {torch.cuda.get_device_name(0)}")
            print(f"[NMT Service] ✓ GPU memory: {torch.cuda.get_device_properties(0).total_memory / 1024**3:.2f} GB")
        else:
            print(f"[NMT Service] ⚠ WARNING: CUDA not available, using CPU")
        
        # 尝试完全禁用网络验证（使用本地文件）
        # 如果模型已完全下载到本地，可以使用 local_files_only=True
        # 这样可以完全避免网络请求和 token 验证
        try_local_only = os.getenv("HF_LOCAL_FILES_ONLY", "false").lower() == "true"
        
        # 检查是否有 HF_TOKEN 环境变量或配置文件
        hf_token = os.getenv("HF_TOKEN")
        
        # 如果没有环境变量，尝试从配置文件读取
        if not hf_token:
            config_file = os.path.join(os.path.dirname(__file__), "hf_token.txt")
            if os.path.exists(config_file):
                try:
                    with open(config_file, "r", encoding="utf-8") as f:
                        hf_token = f.read().strip()
                except Exception as e:
                    print(f"[NMT Service] Warning: Failed to read token from config file: {e}")
        
        # 配置加载选项
        extra = {}
        
        # 如果设置了 local_files_only，完全禁用网络验证
        if try_local_only:
            extra["local_files_only"] = True
            # 禁用 safetensors 自动转换（避免网络请求）
            extra["use_safetensors"] = False
            print("[NMT Service] Using local files only (no network requests, no token needed)")
            print("[NMT Service] Note: Disabled safetensors to avoid network requests")
        else:
            # 只有当 HF_TOKEN 非空且不是空字符串时才使用
            if hf_token and hf_token.strip():
                extra["token"] = hf_token
                extra["use_safetensors"] = True
                print("[NMT Service] Using HF_TOKEN from environment or config file")
            else:
                # 禁用隐式 token 使用（避免使用过期的缓存 token）
                os.environ["HF_HUB_DISABLE_IMPLICIT_TOKEN"] = "1"
                # 尝试不使用 safetensors，避免自动转换的网络请求
                extra["use_safetensors"] = False
                print("[NMT Service] No token provided, trying without safetensors")
        
        # 清除过期的 token 缓存（通过环境变量）
        # 这可以防止使用缓存的过期 token
        os.environ["HF_HUB_DISABLE_IMPLICIT_TOKEN"] = "1"
        
        # 如果 local_files_only 失败，尝试使用配置文件中的 token
        try:
            tokenizer = M2M100Tokenizer.from_pretrained(MODEL_NAME, **extra)
        except Exception as e:
            if try_local_only and "401" in str(e) or "token" in str(e).lower():
                print(f"[NMT Service] local_files_only failed, trying with token from config file...")
                # 回退到使用配置文件中的 token
                if hf_token and hf_token.strip():
                    extra = {
                        "token": hf_token,
                        "use_safetensors": True,
                    }
                    os.environ["HF_HUB_DISABLE_IMPLICIT_TOKEN"] = "0"  # 允许使用 token
                    tokenizer = M2M100Tokenizer.from_pretrained(MODEL_NAME, **extra)
                    print("[NMT Service] Successfully loaded with token from config file")
                else:
                    raise
            else:
                raise
        
        # 禁用所有可能导致 meta tensor 的优化选项
        # 关键：low_cpu_mem_usage=False 必须设置，否则会使用 meta device
        os.environ["TRANSFORMERS_NO_ADVISORY_WARNINGS"] = "1"
        
        # 加载模型，禁用所有优化选项
        model = M2M100ForConditionalGeneration.from_pretrained(
            MODEL_NAME, 
            **extra,
            low_cpu_mem_usage=False,  # 禁用低内存模式（这是关键！必须为 False）
            torch_dtype=torch.float32,  # 明确指定数据类型
        )
        
        # 检查模型是否在 meta 设备上
        try:
            first_param = next(model.parameters(), None)
            if first_param is not None:
                param_device = str(first_param.device)
                if param_device == "meta":
                    raise RuntimeError(f"Model loaded to meta device: {param_device}. This should not happen with low_cpu_mem_usage=False")
        except StopIteration:
            pass  # 模型没有参数（不应该发生）
        
        # 移动到目标设备
        model = model.to(DEVICE).eval()
        
        print(f"[NMT Service] Model loaded successfully on {DEVICE}")
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
    import datetime
    request_start = time.time()
    request_timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
    
    print(f"[NMT Service] [{request_timestamp}] ===== Translation Request Started =====")
    print(f"[NMT Service] Input: '{req.text[:50]}{'...' if len(req.text) > 50 else ''}' (src={req.src_lang}, tgt={req.tgt_lang})")
    
    if model is None or tokenizer is None:
        return TranslateResponse(
            ok=False,
            error="Model not loaded",
            provider="local-m2m100"
        )
    
    try:
        # 设置源语言（重要：必须在编码前设置）
        tokenizer_start = time.time()
        tokenizer.src_lang = req.src_lang
        
        # 编码输入文本（M2M100 会在文本前自动添加源语言 token）
        encoded = tokenizer(req.text, return_tensors="pt").to(DEVICE)
        tokenizer_elapsed = (time.time() - tokenizer_start) * 1000
        print(f"[NMT Service] [Tokenization] Completed in {tokenizer_elapsed:.2f}ms")
        
        # 获取目标语言 token ID
        forced_bos = tokenizer.get_lang_id(req.tgt_lang)
        
        # 调试：验证 tokenizer 状态
        print(f"[DEBUG] Tokenizer src_lang: {tokenizer.src_lang}")
        print(f"[DEBUG] Tokenizer tgt_lang: {req.tgt_lang}")
        
        # 生成翻译
        generation_start = time.time()
        with torch.no_grad():
            gen = model.generate(
                **encoded,
                forced_bos_token_id=forced_bos,
                num_beams=4,
                no_repeat_ngram_size=3,
                repetition_penalty=1.2,
                max_new_tokens=256,  # 增加最大 token 数，避免截断
                early_stopping=False,  # 禁用早停，确保完整翻译
            )
        generation_elapsed = (time.time() - generation_start) * 1000
        print(f"[NMT Service] [Generation] Completed in {generation_elapsed:.2f}ms")
        
        # 调试：打印输入和生成的 token IDs（仅在开发时）
        print(f"[DEBUG] Input text: {req.text}")
        print(f"[DEBUG] Input length: {len(req.text)} chars")
        encoded_input_ids = encoded['input_ids'].cpu().numpy().tolist()[0]
        print(f"[DEBUG] Encoded input_ids: {encoded_input_ids}")
        print(f"[DEBUG] Encoded input length: {len(encoded_input_ids)} tokens")
        print(f"[DEBUG] Forced BOS token ID: {forced_bos}")
        generated_ids_list = gen[0].cpu().numpy().tolist()
        print(f"[DEBUG] Generated IDs (full): {generated_ids_list}")
        print(f"[DEBUG] Generated length: {len(generated_ids_list)} tokens")
        print(f"[DEBUG] EOS token ID: {tokenizer.eos_token_id}")
        
        # 检查生成的序列中是否包含输入序列
        # M2M100 的 generate 可能返回 [input_ids + generated_ids] 或只返回 generated_ids
        input_length = len(encoded_input_ids)
        if len(generated_ids_list) > input_length and generated_ids_list[:input_length] == encoded_input_ids:
            print(f"[DEBUG] Generated sequence includes input (first {input_length} tokens match)")
            generated_only = generated_ids_list[input_length:]
            print(f"[DEBUG] Generated-only IDs: {generated_only}")
        else:
            print(f"[DEBUG] Generated sequence does not include input (or format differs)")
            generated_only = generated_ids_list
        
        # 解码输出
        decode_start = time.time()
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
        
        print(f"[DEBUG] Target lang token ID: {tgt_lang_id}, found at index: {tgt_start_idx}")
        
        if tgt_start_idx is not None and tgt_start_idx < len(generated_ids):
            # 提取目标语言 token 之后的部分
            target_ids = generated_ids[tgt_start_idx:]
            print(f"[DEBUG] Target IDs before EOS removal: {target_ids} (length: {len(target_ids)})")
            
            # 移除 EOS token（如果存在）
            # 注意：可能有多处 EOS token，需要找到第一个有效的 EOS
            eos_positions = [i for i, tid in enumerate(target_ids) if tid == eos_token_id]
            print(f"[DEBUG] EOS token positions in target: {eos_positions}")
            
            if len(eos_positions) > 0:
                # 使用第一个 EOS token 之前的内容
                target_ids = target_ids[:eos_positions[0]]
                print(f"[DEBUG] Truncated at first EOS token (position {eos_positions[0]})")
            else:
                print(f"[DEBUG] No EOS token found in target sequence")
            
            print(f"[DEBUG] Target IDs after EOS removal: {target_ids} (length: {len(target_ids)})")
            
            # 解码目标语言部分
            if len(target_ids) > 0:
                out = tokenizer.decode(target_ids, skip_special_tokens=True)
                print(f"[DEBUG] Decoded output: '{out}' (length: {len(out)} chars)")
                
                # 尝试逐个 token 解码，看看哪里出了问题
                if len(out) < len(req.text) * 0.5:  # 如果输出明显短于输入
                    print(f"[DEBUG] WARNING: Output seems truncated. Decoding tokens individually:")
                    for i, tid in enumerate(target_ids[:min(30, len(target_ids))]):
                        token_text = tokenizer.decode([tid], skip_special_tokens=False)
                        print(f"[DEBUG]   Token {i} (ID {tid}): '{token_text}'")
            else:
                out = ""
                print(f"[DEBUG] Target IDs empty after processing")
        else:
            # 如果找不到目标语言 token，尝试直接解码（可能格式不同）
            # 先尝试跳过源语言部分
            src_lang_id = tokenizer.get_lang_id(req.src_lang)
            src_end_idx = None
            for i, token_id in enumerate(generated_ids):
                if token_id == src_lang_id:
                    src_end_idx = i
                    break
            
            print(f"[DEBUG] Fallback: src_lang_id={src_lang_id}, src_end_idx={src_end_idx}")
            
            if src_end_idx is not None:
                # 跳过源语言部分，解码剩余部分
                remaining_ids = generated_ids[src_end_idx+1:]
                # 移除 EOS
                eos_positions = [i for i, tid in enumerate(remaining_ids) if tid == eos_token_id]
                if len(eos_positions) > 0:
                    remaining_ids = remaining_ids[:eos_positions[0]]
                out = tokenizer.decode(remaining_ids, skip_special_tokens=True)
            else:
                # 最后备用方案：直接解码，但需要跳过输入部分
                # M2M100 的 generate 返回的是完整序列，需要提取生成部分
                input_length = encoded['input_ids'].shape[1]
                if len(generated_ids) > input_length:
                    generated_only = generated_ids[input_length:]
                    # 移除 EOS
                    eos_positions = [i for i, tid in enumerate(generated_only) if tid == eos_token_id]
                    if len(eos_positions) > 0:
                        generated_only = generated_only[:eos_positions[0]]
                    out = tokenizer.decode(generated_only, skip_special_tokens=True)
                else:
                    out = tokenizer.decode(gen[0], skip_special_tokens=True)
            
            print(f"[DEBUG] Fallback decoded output: '{out}'")
        
        # 解码步骤完成
        decode_elapsed = (time.time() - decode_start) * 1000
        
        total_elapsed = (time.time() - request_start) * 1000
        print(f"[NMT Service] [Decoding] Completed in {decode_elapsed:.2f}ms")
        print(f"[NMT Service] Output: '{out[:50]}{'...' if len(out) > 50 else ''}'")
        print(f"[NMT Service] ===== Translation Request Completed in {total_elapsed:.2f}ms =====")
        print(f"[NMT Service]   - Tokenization: {tokenizer_elapsed:.2f}ms")
        print(f"[NMT Service]   - Generation: {generation_elapsed:.2f}ms")
        print(f"[NMT Service]   - Decoding: {decode_elapsed:.2f}ms")
        print(f"[NMT Service]   - Total: {total_elapsed:.2f}ms")
        
        return TranslateResponse(
            ok=True,
            text=out,
            model=MODEL_NAME,
            provider="local-m2m100",
            extra={
                "elapsed_ms": int(total_elapsed),
                "num_tokens": int(gen.shape[1]),
                "tokenization_ms": int(tokenizer_elapsed),
                "generation_ms": int(generation_elapsed),
                "decoding_ms": int(decode_elapsed)
            }
        )
    except Exception as e:
        total_elapsed = (time.time() - request_start) * 1000
        print(f"[NMT Service] Error: {e}")
        print(f"[NMT Service] ===== Translation Request Failed in {total_elapsed:.2f}ms =====")
        import traceback
        traceback.print_exc()
        return TranslateResponse(
            ok=False,
            error=str(e),
            provider="local-m2m100",
            extra={"elapsed_ms": int(total_elapsed)}
        )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5008)

