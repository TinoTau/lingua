#!/usr/bin/env python3
"""
Piper HTTP 服务包装器
通过 HTTP API 调用 piper 命令行工具进行 TTS 合成
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional, Tuple

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import Response
    from pydantic import BaseModel
    import uvicorn
except ImportError:
    print("ERROR: FastAPI and uvicorn are required. Please install:")
    print("  pip install fastapi uvicorn")
    sys.exit(1)


class TtsRequest(BaseModel):
    text: str
    voice: str
    language: Optional[str] = None
    
    class Config:
        # 确保正确处理 UTF-8 编码
        json_encoders = {
            str: lambda v: v.encode('utf-8').decode('utf-8') if isinstance(v, str) else v
        }


app = FastAPI(title="Piper TTS HTTP Service")

# 确保正确处理 UTF-8 编码
import sys
if sys.stdout.encoding != 'utf-8':
    import codecs
    sys.stdout = codecs.getwriter('utf-8')(sys.stdout.buffer, 'strict')
    sys.stderr = codecs.getwriter('utf-8')(sys.stderr.buffer, 'strict')


def find_piper_command() -> str:
    """查找 piper 命令路径"""
    # 首先尝试在 PATH 中查找
    piper_path = os.environ.get("PIPER_CMD")
    if piper_path and os.path.exists(piper_path):
        return piper_path
    
    # 尝试在虚拟环境中查找
    venv_bin = os.environ.get("VIRTUAL_ENV")
    if venv_bin:
        venv_piper = os.path.join(venv_bin, "bin", "piper")
        if os.path.exists(venv_piper):
            return venv_piper
    
    # 使用 which 查找
    try:
        result = subprocess.run(
            ["which", "piper"],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass
    
    # 默认假设 piper 在 PATH 中
    return "piper"


def find_model_path(voice: str, model_dir: str) -> Tuple[Optional[str], Optional[str]]:
    """
    查找模型文件路径
    返回: (model_path, config_path)
    """
    model_dir_path = Path(model_dir).expanduser()
    
    # 尝试不同的可能路径
    possible_paths = [
        model_dir_path / voice / f"{voice}.onnx",
        model_dir_path / "zh" / f"{voice}.onnx",
        model_dir_path / f"{voice}.onnx",
    ]
    
    for model_path in possible_paths:
        if model_path.exists():
            config_path = model_path.with_suffix(".onnx.json")
            return str(model_path), str(config_path) if config_path.exists() else None
    
    return None, None


@app.post("/tts")
async def synthesize_tts(request: TtsRequest):
    """TTS 合成接口"""
    # 获取配置
    model_dir = os.environ.get("PIPER_MODEL_DIR", os.path.expanduser("~/piper_models"))
    piper_cmd = find_piper_command()
    
    # 查找模型文件
    model_path, config_path = find_model_path(request.voice, model_dir)
    if not model_path:
        raise HTTPException(
            status_code=404,
            detail=f"Model not found for voice: {request.voice} (searched in {model_dir})"
        )
    
    # 创建临时输入和输出文件
    import logging
    logger = logging.getLogger("uvicorn.error")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix=".txt", delete=False, encoding='utf-8') as tmp_input:
        tmp_input.write(request.text)
        input_path = tmp_input.name
    
    # 验证输入文件是否正确写入
    with open(input_path, 'r', encoding='utf-8') as f:
        written_text = f.read()
        logger.info(f"Input file written: {input_path}, content: {written_text}, length: {len(written_text)}")
        if written_text != request.text:
            logger.error(f"Text mismatch! Original: {request.text}, Written: {written_text}")
    
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp_output:
        output_path = tmp_output.name
    
    try:
        # 构建 piper 命令
        # 使用 --input-file 参数而不是 stdin，更可靠
        cmd = [
            piper_cmd,
            "--model", model_path,
            "--input_file", input_path,
            "--output_file", output_path,
        ]
        
        if config_path:
            cmd.extend(["--config", config_path])
        
        # 执行 piper 命令
        logger.info(f"Executing piper command: {' '.join(cmd)}")
        logger.info(f"Input text: {request.text} (length: {len(request.text)})")
        logger.info(f"Input file: {input_path}")
        logger.info(f"Output file: {output_path}")
        
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding='utf-8',
            errors='replace'
        )
        
        stdout, stderr = process.communicate()
        
        if process.returncode != 0:
            logger.error(f"Piper command failed with return code {process.returncode}")
            logger.error(f"stderr: {stderr}")
            if stdout:
                logger.error(f"stdout: {stdout}")
            raise HTTPException(
                status_code=500,
                detail=f"Piper command failed (return code {process.returncode}): {stderr}"
            )
        
        if stderr:
            logger.warning(f"Piper stderr output: {stderr}")
        
        logger.info(f"Piper command completed successfully")
        
        # 读取生成的 WAV 文件
        if not os.path.exists(output_path):
            logger.error(f"Output file does not exist: {output_path}")
            raise HTTPException(
                status_code=500,
                detail="Piper did not generate output file"
            )
        
        file_size = os.path.getsize(output_path)
        logger.info(f"Output file size: {file_size} bytes")
        
        with open(output_path, "rb") as f:
            audio_data = f.read()
        
        logger.info(f"Audio data read: {len(audio_data)} bytes")
        
        if not audio_data:
            logger.error("Generated audio file is empty")
            raise HTTPException(
                status_code=500,
                detail="Generated audio file is empty"
            )
        
        # 返回 WAV 数据
        return Response(
            content=audio_data,
            media_type="audio/wav",
            headers={
                "Content-Disposition": f'attachment; filename="{request.voice}.wav"'
            }
        )
    
    finally:
        # 清理临时文件
        if os.path.exists(output_path):
            try:
                os.unlink(output_path)
            except OSError:
                pass
        if os.path.exists(input_path):
            try:
                os.unlink(input_path)
            except OSError:
                pass


@app.get("/health")
async def health_check():
    """健康检查接口"""
    return {"status": "ok", "service": "piper-tts"}


@app.get("/voices")
async def list_voices():
    """列出可用的语音模型"""
    model_dir = os.environ.get("PIPER_MODEL_DIR", os.path.expanduser("~/piper_models"))
    model_dir_path = Path(model_dir).expanduser()
    
    voices = []
    if model_dir_path.exists():
        # 查找所有 .onnx 文件
        for onnx_file in model_dir_path.rglob("*.onnx"):
            voice_name = onnx_file.stem
            voices.append({
                "name": voice_name,
                "path": str(onnx_file),
            })
    
    return {"voices": voices}


def main():
    parser = argparse.ArgumentParser(description="Piper TTS HTTP Service")
    parser.add_argument(
        "--host",
        default="0.0.0.0",
        help="Host to bind to (default: 0.0.0.0)"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=5005,
        help="Port to bind to (default: 5005)"
    )
    parser.add_argument(
        "--model-dir",
        default=os.path.expanduser("~/piper_models"),
        help="Directory containing Piper models (default: ~/piper_models)"
    )
    parser.add_argument(
        "--piper-cmd",
        help="Path to piper command (default: auto-detect)"
    )
    
    args = parser.parse_args()
    
    # 设置环境变量
    os.environ["PIPER_MODEL_DIR"] = args.model_dir
    if args.piper_cmd:
        os.environ["PIPER_CMD"] = args.piper_cmd
    
    print(f"Starting Piper TTS HTTP Service...")
    print(f"  Host: {args.host}")
    print(f"  Port: {args.port}")
    print(f"  Model Directory: {args.model_dir}")
    print(f"  Piper Command: {find_piper_command()}")
    print(f"\nEndpoints:")
    print(f"  POST /tts - Synthesize speech")
    print(f"  GET /health - Health check")
    print(f"  GET /voices - List available voices")
    print()
    
    uvicorn.run(app, host=args.host, port=args.port)


if __name__ == "__main__":
    main()

