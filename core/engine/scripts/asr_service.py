"""
ASR Service using faster-whisper
Provides HTTP API for speech recognition with context support
"""
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from faster_whisper import WhisperModel
import base64
import numpy as np
import soundfile as sf
import io
import os
import logging
from typing import Optional, List

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# ---------------------
# Configuration
# ---------------------
# Model path can be:
# 1. HuggingFace model ID (e.g., "openai/whisper-large-v3", "Systran/faster-whisper-large-v3")
# 2. Local directory path (e.g., "models/whisper-large-v3")
# Default: Use faster-whisper's optimized model from HuggingFace
MODEL_PATH = os.getenv("ASR_MODEL_PATH", "Systran/faster-whisper-large-v3")
DEVICE = os.getenv("ASR_DEVICE", "cpu")  # "cpu" or "cuda"
COMPUTE_TYPE = os.getenv("ASR_COMPUTE_TYPE", "float32")  # "float32", "float16", "int8"
PORT = int(os.getenv("ASR_SERVICE_PORT", "6006"))

# ---------------------
# Load Whisper Model
# ---------------------
logger.info(f"Loading Whisper model from {MODEL_PATH}...")
logger.info(f"Device: {DEVICE}, Compute Type: {COMPUTE_TYPE}")

try:
    model = WhisperModel(
        MODEL_PATH,
        device=DEVICE,
        compute_type=COMPUTE_TYPE,
    )
    logger.info("✅ Whisper model loaded successfully")
except Exception as e:
    logger.error(f"❌ Failed to load Whisper model: {e}")
    logger.error("Please ensure the model path is correct and faster-whisper is installed")
    raise

app = FastAPI(title="ASR Service (faster-whisper)")

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ---------------------
# Request/Response Schemas
# ---------------------
class ASRRequest(BaseModel):
    audio_b64: str  # Base64 encoded audio (WAV format, 16kHz mono)
    prompt: str = ""  # Context prompt (previous sentences)
    language: Optional[str] = None  # Language code (e.g., "zh", "en"), None for auto-detect
    task: str = "transcribe"  # "transcribe" or "translate"
    beam_size: int = 5
    vad_filter: bool = False  # VAD is handled by Silero VAD in Rust, disable here to avoid double filtering
    condition_on_previous_text: bool = True  # Use context for better accuracy

class ASRResponse(BaseModel):
    text: str  # Full transcribed text
    segments: List[str]  # List of segment texts
    language: Optional[str] = None  # Detected language
    duration: float  # Audio duration in seconds

# ---------------------
# Health Check
# ---------------------
@app.get("/health")
def health_check():
    return {"status": "ok", "model_loaded": True}

# ---------------------
# ASR Endpoint
# ---------------------
@app.post("/asr", response_model=ASRResponse)
def transcribe(req: ASRRequest):
    try:
        # Decode base64 audio
        try:
            audio_bytes = base64.b64decode(req.audio_b64)
        except Exception as e:
            logger.error(f"Failed to decode base64 audio: {e}")
            raise HTTPException(status_code=400, detail=f"Invalid base64 audio: {e}")
        
        # Read audio file
        try:
            audio, sr = sf.read(io.BytesIO(audio_bytes))
        except Exception as e:
            logger.error(f"Failed to read audio file: {e}")
            raise HTTPException(status_code=400, detail=f"Invalid audio format: {e}")
        
        # Convert to float32 (required by faster-whisper)
        # soundfile may return float64, but the model expects float32
        if audio.dtype != np.float32:
            audio = audio.astype(np.float32)
        
        # Convert to mono if stereo
        if len(audio.shape) > 1:
            audio = np.mean(audio, axis=1).astype(np.float32)
        
        # Resample to 16kHz if needed
        if sr != 16000:
            logger.warning(f"Audio sample rate is {sr}Hz, expected 16kHz. Resampling...")
            from scipy import signal
            num_samples = int(len(audio) * 16000 / sr)
            audio = signal.resample(audio, num_samples).astype(np.float32)
            sr = 16000
        
        # Ensure audio is contiguous (required by faster-whisper)
        if not audio.flags['C_CONTIGUOUS']:
            audio = np.ascontiguousarray(audio)
        
        import time
        asr_start_time = time.time()
        
        logger.info(f"Processing audio: {len(audio)} samples @ {sr}Hz, duration: {len(audio)/sr:.2f}s, dtype: {audio.dtype}")
        if req.prompt:
            logger.info(f"Using context prompt ({len(req.prompt)} chars): \"{req.prompt[:100]}...\"")
        logger.info(f"VAD filter: {req.vad_filter} (disabled because Silero VAD handles boundaries)")
        
        # Run ASR
        segments, info = model.transcribe(
            audio,
            language=req.language,
            task=req.task,
            beam_size=req.beam_size,
            vad_filter=req.vad_filter,  # False: Silero VAD already handles boundaries
            initial_prompt=req.prompt if req.prompt else None,  # ★ Context support
            condition_on_previous_text=req.condition_on_previous_text,  # ★ Continuous recognition
        )
        
        asr_elapsed = time.time() - asr_start_time
        
        # Extract text and segments
        segment_texts = []
        full_text_parts = []
        
        for segment in segments:
            segment_text = segment.text.strip()
            if segment_text:
                segment_texts.append(segment_text)
                full_text_parts.append(segment_text)
        
        full_text = " ".join(full_text_parts)
        
        logger.info(f"Transcribed: {len(segment_texts)} segments, {len(full_text)} chars in {asr_elapsed:.2f}s")
        logger.info(f"Detected language: {info.language}, probability: {info.language_probability:.2f}")
        if asr_elapsed > 1.0:
            logger.warning(f"⚠️  ASR processing took {asr_elapsed:.2f}s (audio duration: {len(audio)/sr:.2f}s, ratio: {asr_elapsed/(len(audio)/sr):.2f}x)")
        
        return ASRResponse(
            text=full_text,
            segments=segment_texts,
            language=info.language,
            duration=info.duration,
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"ASR processing error: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"ASR processing failed: {str(e)}")

# ---------------------
# Main
# ---------------------
if __name__ == "__main__":
    import uvicorn
    logger.info(f"Starting ASR service on port {PORT}...")
    uvicorn.run(app, host="0.0.0.0", port=PORT)

