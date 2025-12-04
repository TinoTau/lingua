#!/usr/bin/env python3
"""
YourTTS HTTP 服务（Zero-shot TTS）

用于从 Rust 代码调用 YourTTS 模型进行语音合成，支持音色克隆。

使用方法：
    python yourtts_service.py [--gpu] [--port PORT] [--host HOST]

参数：
    --gpu: 使用 GPU（如果可用）
    --port: 服务端口（默认：5004）
    --host: 服务地址（默认：127.0.0.1）

API 端点：
    POST /synthesize
    Body: {
        "text": "要合成的文本",
        "reference_audio": [0.1, 0.2, ...],  # 参考音频（可选，用于音色克隆）
        "language": "zh"  # 语言代码（可选）
    }
    Response: {
        "audio": [0.1, 0.2, ...],  # 合成的音频数据（f32）
        "sample_rate": 22050
    }
"""

import sys
import os
import argparse
from pathlib import Path

# 添加项目路径
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from flask import Flask, request, jsonify
import numpy as np
import torch
import base64
import tempfile
import soundfile as sf
from scipy import signal
import requests

app = Flask(__name__)
tts_model = None
device = None

# Speaker 缓存：存储 speaker_id -> reference_audio 的映射
# 格式：{speaker_id: {"reference_audio": np.ndarray, "sample_rate": int, "voice_embedding": np.ndarray}}
speaker_cache = {}

# 线程锁，用于保护 speaker_cache 的并发访问
import threading
speaker_cache_lock = threading.Lock()

def get_device(use_gpu=False):
    """获取计算设备"""
    if use_gpu:
        if torch.cuda.is_available():
            device = "cuda"
            print(f"✅ Using GPU: {torch.cuda.get_device_name(0)}")
            print(f"   CUDA version: {torch.version.cuda}")
            print(f"   PyTorch version: {torch.__version__}")
        else:
            device = "cpu"
            print("⚠️  GPU requested but not available, using CPU")
            print("   Check:")
            print("   1. NVIDIA drivers installed in WSL")
            print("   2. CUDA toolkit installed in WSL")
            print("   3. PyTorch with CUDA support installed")
            print("   4. Run 'nvidia-smi' in WSL to verify GPU access")
    else:
        device = "cpu"
        print("ℹ️  Using CPU (GPU not requested)")
    return device

def check_and_install_tts():
    """检查并安装 TTS 模块"""
    try:
        import TTS
        return True
    except ImportError:
        print("⚠️  TTS module not found. Attempting to install...")
        try:
            import subprocess
            import sys
            subprocess.check_call([sys.executable, "-m", "pip", "install", "TTS"])
            print("✅ TTS module installed successfully")
            return True
        except Exception as e:
            print(f"❌ Failed to install TTS module: {e}")
            print("\nPlease install manually:")
            print("  pip install TTS")
            return False

def extract_voice_info(audio_array, label="Audio"):
    """提取音频的音色信息（通过 Speaker Embedding 服务）"""
    try:
        # 尝试调用 Speaker Embedding 服务
        speaker_embedding_url = "http://127.0.0.1:5003/extract"
        
        # 准备请求数据
        request_data = {
            "audio": audio_array.tolist(),
            "sample_rate": 16000
        }
        
        try:
            response = requests.post(
                speaker_embedding_url,
                json=request_data,
                timeout=5.0
            )
            
            if response.status_code == 200:
                result = response.json()
                embedding = np.array(result.get("embedding", []))
                stats = result.get("stats", {})
                
                if len(embedding) > 0:
                    print(f"[YourTTS Service] 🎤 {label} Voice Info:")
                    print(f"   Embedding dimension: {len(embedding)}")
                    print(f"   Norm (L2): {stats.get('norm', np.linalg.norm(embedding)):.4f}")
                    print(f"   Mean: {stats.get('mean', np.mean(embedding)):.6f}, Std: {stats.get('std', np.std(embedding)):.6f}")
                    print(f"   Range: [{stats.get('min', np.min(embedding)):.6f}, {stats.get('max', np.max(embedding)):.6f}]")
                    print(f"   Abs Mean: {stats.get('abs_mean', np.mean(np.abs(embedding))):.6f}")
                    
                    return {
                        "embedding": embedding,
                        "stats": stats,
                        "available": True
                    }
        except (requests.exceptions.RequestException, Exception) as e:
            print(f"[YourTTS Service] ⚠️  Could not extract voice info from Speaker Embedding service: {e}")
            
    except Exception as e:
        print(f"[YourTTS Service] ⚠️  Error extracting voice info: {e}")
    
    # 如果无法获取 embedding，至少显示基本统计
    audio_array_np = np.array(audio_array, dtype=np.float32)
    basic_stats = {
        "mean": float(np.mean(audio_array_np)),
        "std": float(np.std(audio_array_np)),
        "min": float(np.min(audio_array_np)),
        "max": float(np.max(audio_array_np)),
        "rms": float(np.sqrt(np.mean(audio_array_np ** 2)))
    }
    
    print(f"[YourTTS Service] 🎤 {label} Basic Audio Stats:")
    print(f"   RMS: {basic_stats['rms']:.6f}")
    print(f"   Mean: {basic_stats['mean']:.6f}, Std: {basic_stats['std']:.6f}")
    print(f"   Range: [{basic_stats['min']:.6f}, {basic_stats['max']:.6f}]")
    
    return {
        "embedding": None,
        "stats": basic_stats,
        "available": False
    }

def _get_default_speaker(tts_model):
    """获取默认说话者"""
    try:
        # 方法1：检查 tts_model.speakers 属性
        if hasattr(tts_model, 'speakers') and tts_model.speakers:
            if isinstance(tts_model.speakers, list) and len(tts_model.speakers) > 0:
                return tts_model.speakers[0]
            elif isinstance(tts_model.speakers, dict) and len(tts_model.speakers) > 0:
                return list(tts_model.speakers.keys())[0]
        # 方法2：检查 speaker_manager
        if hasattr(tts_model, 'speaker_manager'):
            if hasattr(tts_model.speaker_manager, 'speaker_names') and tts_model.speaker_manager.speaker_names:
                return tts_model.speaker_manager.speaker_names[0]
            elif hasattr(tts_model.speaker_manager, 'speakers') and tts_model.speaker_manager.speakers:
                if isinstance(tts_model.speaker_manager.speakers, list) and len(tts_model.speaker_manager.speakers) > 0:
                    return tts_model.speaker_manager.speakers[0]
                elif isinstance(tts_model.speaker_manager.speakers, dict) and len(tts_model.speaker_manager.speakers) > 0:
                    return list(tts_model.speaker_manager.speakers.keys())[0]
    except Exception as e:
        print(f"Warning: Could not get default speaker: {e}")
    return None

def load_model(model_path, device="cpu"):
    """加载 YourTTS 模型"""
    global tts_model
    
    # 检查并安装 TTS 模块
    if not check_and_install_tts():
        raise ImportError("TTS module is required but not available")
    
    try:
        from TTS.api import TTS
        
        if not model_path.exists():
            raise FileNotFoundError(f"Model not found at {model_path}")
        
        print(f"📁 Loading YourTTS model from: {model_path}")
        print(f"🔧 Device: {device}")
        
        # YourTTS 模型路径
        # 注意：TTS API 可能需要模型名称而不是路径
        # 如果直接使用路径，可能需要自定义加载
        
        # 方式1：使用 TTS API（如果模型已注册）
        try:
            # 尝试使用模型路径
            tts_model = TTS(model_path=str(model_path), progress_bar=False, gpu=(device == "cuda"))
            print("✅ YourTTS model loaded via TTS API")
        except:
            # 方式2：直接加载模型文件
            # 需要根据 YourTTS 的实际加载方式调整
            print("⚠️  TTS API loading failed, trying direct load...")
            
            # 检查是否有 model.pth
            model_file = model_path / "model.pth"
            if model_file.exists():
                # 这里需要根据 YourTTS 的实际结构加载
                # 暂时使用 TTS API 的备用方式，并指定 GPU
                tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", gpu=(device == "cuda"))
                print("✅ YourTTS model loaded (using default model)")
            else:
                raise FileNotFoundError(f"Model file not found: {model_file}")
        
        # 移动到指定设备（如果 TTS API 没有自动处理）
        if hasattr(tts_model, 'to') and device == "cuda":
            try:
                tts_model = tts_model.to(device)
                print(f"✅ Model moved to {device}")
            except Exception as e:
                print(f"⚠️  Warning: Failed to move model to {device}: {e}")
                print("   Model may still work on CPU")
        
        print(f"✅ YourTTS model loaded successfully")
        print(f"   Device: {device}")
        print(f"   Supports zero-shot: Yes")
        
        return tts_model
    except Exception as e:
        print(f"❌ Failed to load model: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

@app.route('/health', methods=['GET'])
def health():
    """健康检查端点"""
    with speaker_cache_lock:
        cache_size = len(speaker_cache)
    return jsonify({
        "status": "ok",
        "model_loaded": tts_model is not None,
        "device": device,
        "cached_speakers": cache_size
    })

@app.route('/register_speaker', methods=['POST'])
def register_speaker():
    """注册说话者（异步接收 reference_audio）
    
    当识别到新说话者时，异步调用此端点注册其 reference_audio。
    后续合成请求只需传递 speaker_id 即可使用缓存的 reference_audio。
    
    Request Body:
        {
            "speaker_id": "speaker_123",
            "reference_audio": [0.1, 0.2, ...],  # 参考音频（f32 数组）
            "reference_sample_rate": 16000,  # 参考音频采样率（默认 16000 Hz）
            "voice_embedding": [0.1, 0.2, ...]  # 可选，音色embedding（用于验证）
        }
    
    Response:
        {
            "status": "ok",
            "speaker_id": "speaker_123",
            "message": "Speaker registered successfully"
        }
    """
    try:
        data = request.json
        if data is None:
            return jsonify({"error": "Invalid JSON"}), 400
        
        speaker_id = data.get('speaker_id')
        if not speaker_id:
            return jsonify({"error": "Missing 'speaker_id' field"}), 400
        
        reference_audio = data.get('reference_audio')
        if not reference_audio:
            return jsonify({"error": "Missing 'reference_audio' field"}), 400
        
        reference_sample_rate = data.get('reference_sample_rate', 16000)
        voice_embedding = data.get('voice_embedding')  # 可选
        
        # 将参考音频转换为 numpy 数组
        ref_audio_array = np.array(reference_audio, dtype=np.float32)
        
        # YourTTS 需要 22050 Hz 的参考音频，预先重采样
        target_sample_rate = 22050
        if reference_sample_rate != target_sample_rate:
            num_samples = int(len(ref_audio_array) * target_sample_rate / reference_sample_rate)
            ref_audio_array = signal.resample(ref_audio_array, num_samples)
            print(f"[YourTTS Service] Resampled reference audio from {reference_sample_rate} Hz to {target_sample_rate} Hz for speaker {speaker_id}")
        
        # 保存 voice_embedding（如果提供）
        embedding_array = None
        if voice_embedding:
            embedding_array = np.array(voice_embedding, dtype=np.float32)
        
        # 保存到缓存
        with speaker_cache_lock:
            speaker_cache[speaker_id] = {
                "reference_audio": ref_audio_array,
                "sample_rate": target_sample_rate,
                "voice_embedding": embedding_array
            }
            cache_size = len(speaker_cache)
        
        print(f"[YourTTS Service] ✅ Registered speaker '{speaker_id}' (reference_audio: {len(ref_audio_array)} samples @ {target_sample_rate} Hz, cache size: {cache_size})")
        
        return jsonify({
            "status": "ok",
            "speaker_id": speaker_id,
            "message": "Speaker registered successfully",
            "cache_size": cache_size
        })
    
    except Exception as e:
        print(f"[YourTTS Service] ❌ Failed to register speaker: {e}")
        import traceback
        traceback.print_exc()
        return jsonify({"error": str(e)}), 500

@app.route('/synthesize', methods=['POST'])
def synthesize():
    """语音合成（支持 zero-shot）"""
    try:
        # 先验证输入，再检查模型
        data = request.json
        if data is None:
            return jsonify({"error": "Invalid JSON"}), 400
        
        if 'text' not in data:
            return jsonify({"error": "Missing 'text' field"}), 400
        
        text = data['text']
        speaker_id = data.get('speaker_id')  # 可选，说话者ID（用于查找缓存的 reference_audio）
        reference_audio = data.get('reference_audio')  # 可选（如果没有提供 speaker_id）
        reference_sample_rate = data.get('reference_sample_rate', 16000)  # 参考音频采样率（默认 16000 Hz）
        voice_embedding = data.get('voice_embedding')  # 可选，说话者音色embedding（优先使用，避免查询服务）
        speaker = data.get('speaker')  # 可选，说话者名称（当没有 reference_audio 时使用）
        language = data.get('language', 'zh')  # 默认中文
        speech_rate = data.get('speech_rate')  # 可选，语速（字符/秒），用于调整合成速度
        
        # 记录语速参数（用于调试）
        if speech_rate is not None:
            print(f"[YourTTS Service] 📊 Received speech_rate parameter: {speech_rate:.2} chars/s")
        else:
            print(f"[YourTTS Service] 📊 No speech_rate parameter provided (will use default/normal rate)")
        
        # 验证文本
        if not text or len(text.strip()) == 0:
            return jsonify({"error": "Empty text"}), 400
        
        # 检查模型是否加载
        if tts_model is None:
            return jsonify({"error": "Model not loaded"}), 500
        
        # 准备参考音频（优先使用缓存的，如果没有则使用提供的）
        speaker_wav = None
        cached_ref_audio = None
        cached_sample_rate = None
        
        # 如果提供了 speaker_id，尝试从缓存中获取 reference_audio
        if speaker_id:
            with speaker_cache_lock:
                cached_entry = speaker_cache.get(speaker_id)
                if cached_entry:
                    cached_ref_audio = cached_entry["reference_audio"]
                    cached_sample_rate = cached_entry["sample_rate"]
                    print(f"[YourTTS Service] ✅ Using cached reference_audio for speaker_id '{speaker_id}' ({len(cached_ref_audio)} samples @ {cached_sample_rate} Hz)")
                else:
                    print(f"[YourTTS Service] ⚠️  Speaker_id '{speaker_id}' not found in cache yet (async registration may be in progress)")
                    print(f"[YourTTS Service]    Will use default voice for now (synthesis won't wait for async registration)")
        
        # 确定使用哪个 reference_audio
        use_cached = cached_ref_audio is not None
        ref_audio_to_use = cached_ref_audio if use_cached else reference_audio
        ref_sample_rate_to_use = cached_sample_rate if use_cached else reference_sample_rate
        
        try:
            if ref_audio_to_use is not None:
                # 将参考音频转换为 numpy 数组
                if use_cached:
                    # 使用缓存的参考音频（已经重采样到 22050 Hz）
                    ref_audio_array = ref_audio_to_use
                else:
                    # 使用提供的参考音频（需要重采样）
                    ref_audio_array = np.array(ref_audio_to_use, dtype=np.float32)
                
                # YourTTS 需要 22050 Hz 的参考音频
                target_sample_rate = 22050
                
                # 如果使用缓存的参考音频，已经重采样过了
                # 如果使用提供的参考音频，需要重采样
                if not use_cached:
                    if ref_sample_rate_to_use != target_sample_rate:
                        print(f"[YourTTS Service] Resampling reference audio from {ref_sample_rate_to_use} Hz to {target_sample_rate} Hz")
                        num_samples = int(len(ref_audio_array) * target_sample_rate / ref_sample_rate_to_use)
                        ref_audio_array = signal.resample(ref_audio_array, num_samples)
                        print(f"[YourTTS Service] Resampled: {len(ref_audio_to_use)} samples -> {len(ref_audio_array)} samples")
                    else:
                        print(f"[YourTTS Service] Reference audio sample rate matches target ({target_sample_rate} Hz)")
                else:
                    # 使用缓存的音频，无需重复输出信息（已在上面输出）
                    pass
                
                # 保存为临时文件（YourTTS 需要文件路径）
                # 使用临时文件，确保在 Windows 上也能正确清理
                tmp_file = tempfile.NamedTemporaryFile(suffix='.wav', delete=False)
                tmp_file.close()  # 关闭文件句柄，避免 Windows 锁定问题
                try:
                    sf.write(tmp_file.name, ref_audio_array, target_sample_rate)
                    speaker_wav = tmp_file.name
                    print(f"[YourTTS Service] ✅ Reference audio saved to temp file: {speaker_wav} ({len(ref_audio_array)} samples @ {target_sample_rate} Hz)")
                except Exception as e:
                    # 如果写入失败，清理文件
                    if os.path.exists(tmp_file.name):
                        os.unlink(tmp_file.name)
                    raise
            
            # 合成语音
            # YourTTS API 使用方式
            if speaker_wav:
                # Zero-shot 模式：使用参考音频
                # 记录使用的 reference_audio 来源（简化日志，避免重复）
                if use_cached:
                    print(f"[YourTTS Service] 🎤 Synthesizing with cached reference_audio (speaker_id: '{speaker_id}', {len(ref_audio_array)} samples)")
                else:
                    print(f"[YourTTS Service] 🎤 Synthesizing with provided reference_audio ({len(ref_audio_array)} samples @ {target_sample_rate} Hz)")
                
                wav = tts_model.tts(
                    text=text,
                    speaker_wav=speaker_wav,  # 使用参考音频文件（模型内部会提取 embedding 用于合成）
                    language=language
                )
                print(f"[YourTTS Service] ✅ Synthesis completed, output: {len(wav)} samples")
            elif speaker:
                # 使用指定的说话者（从 voice 字段传递过来）
                # 注意：speaker 参数应该是 YourTTS 模型支持的说话者名称
                # 如果传递的是 voice ID（如 "zh_CN-huayan-medium"），需要映射到 YourTTS 的 speaker
                # 这里简化处理：先尝试使用传递的 speaker 值，如果失败则使用默认说话者
                print(f"[YourTTS Service] ⚠️  Using predefined speaker '{speaker}' (NOT using reference_audio for voice cloning)")
                default_speaker = _get_default_speaker(tts_model)
                try:
                    wav = tts_model.tts(
                        text=text,
                        speaker=speaker,
                        language=language
                    )
                    print(f"[YourTTS Service] ✅ Synthesis completed with predefined speaker '{speaker}' (default voice, no voice cloning)")
                except Exception as e:
                    # 如果指定的 speaker 不存在，使用默认说话者
                    print(f"[YourTTS Service] ⚠️  Warning: Speaker '{speaker}' not found, using default speaker: {e}")
                    if default_speaker:
                        wav = tts_model.tts(
                            text=text,
                            speaker=default_speaker,
                            language=language
                        )
                        print(f"[YourTTS Service] ✅ Synthesis completed with default speaker '{default_speaker}' (default voice, no voice cloning)")
                    else:
                        raise ValueError(f"Speaker '{speaker}' not found and no default speaker available. Error: {e}")
            else:
                # 默认模式：没有 reference_audio，也没有 speaker_id，也没有 speaker 参数
                # 如果提供了 speaker_id 但缓存中没有，也走这里（使用默认音色）
                default_speaker = _get_default_speaker(tts_model)
                if speaker_id:
                    print(f"[YourTTS Service] ⚠️  Speaker_id '{speaker_id}' not yet registered in cache, using default voice")
                    print(f"[YourTTS Service] ⚠️  NOT using reference_audio - voice cloning NOT applied (fallback to default voice)")
                else:
                    print(f"[YourTTS Service] ⚠️  WARNING: No reference audio and no speaker specified, using default speaker")
                    print(f"[YourTTS Service] ⚠️  NOT using reference_audio - voice cloning NOT applied (using default voice)")
                if default_speaker:
                    # 使用默认说话者
                    wav = tts_model.tts(
                        text=text,
                        speaker=default_speaker,
                        language=language
                    )
                    print(f"[YourTTS Service] ✅ Synthesis completed with default speaker '{default_speaker}' (default voice, no voice cloning)")
                else:
                    # 如果没有可用的说话者，返回错误
                    raise ValueError(
                        "YourTTS is a multi-speaker model. Please provide either:\n"
                        "1. A reference audio (reference_audio parameter) for zero-shot voice cloning, or\n"
                        "2. A speaker name (speaker parameter), or\n"
                        "3. Ensure the model has speaker configurations available."
                    )
            
            # 如果提供了语速参数，调整音频速度（在所有合成路径之后统一处理）
            if speech_rate is not None:
                print(f"[YourTTS Service] 🎯 Processing speech_rate adjustment: {speech_rate:.2} chars/s")
                # 计算目标语速（字符/秒）
                # 正常语速大约是 4-6 字符/秒（中文）或 10-15 字符/秒（英文）
                # 如果 speech_rate 与正常语速不同，需要调整音频速度
                # 优先使用 librosa 进行时间拉伸（保持音调），如果不可用则使用 scipy 重采样
                use_librosa = False
                use_scipy = False
                
                try:
                    import librosa
                    use_librosa = True
                except ImportError:
                    # librosa 不可用，尝试使用 scipy
                    try:
                        # scipy 已经在文件顶部导入
                        use_scipy = True
                    except ImportError:
                        print(f"[YourTTS Service] ⚠️  Warning: Neither librosa nor scipy available, cannot adjust speech rate. Install with: pip install librosa")
                
                if use_librosa or use_scipy:
                    try:
                        # 确保 wav 是 numpy.ndarray 类型，并转换为 float64
                        # librosa.effects.time_stretch 需要 float64 类型（numba 编译的函数不支持 float32）
                        if isinstance(wav, torch.Tensor):
                            # 从 Tensor 转换为 numpy，直接转换为 float64
                            wav_np = wav.cpu().numpy().astype(np.float64)
                        elif isinstance(wav, np.ndarray):
                            # 确保是 float64 类型
                            wav_np = wav.astype(np.float64)
                        elif isinstance(wav, list):
                            # 从列表创建，直接使用 float64
                            wav_np = np.array(wav, dtype=np.float64)
                        else:
                            # 尝试转换为 numpy 数组，使用 float64
                            wav_np = np.array(wav, dtype=np.float64)
                        
                        # 确保是一维数组
                        if wav_np.ndim > 1:
                            wav_np = wav_np.flatten()
                        
                        # 再次确保是 float64（防止之前的转换失败）
                        if wav_np.dtype != np.float64:
                            print(f"[YourTTS Service] ⚠️  Warning: wav_np dtype is {wav_np.dtype}, converting to float64")
                            wav_np = wav_np.astype(np.float64)
                        
                        # 验证类型
                        if wav_np.dtype != np.float64:
                            raise ValueError(f"Failed to convert audio to float64, current dtype: {wav_np.dtype}")
                        
                        # 计算速度因子
                        # 重要：speech_rate 是基于参考音频计算的，可能包含停顿时间
                        # 需要根据目标语言和目标文本长度调整
                        # 正常语速：中文约 4-5 字符/秒，英文约 12-15 字符/秒
                        # 比例：英文正常语速约为中文的 2.4-3 倍
                        
                        # 计算当前文本的目标语速（基于文本长度）
                        text_length = len(text)
                        if language.startswith('zh'):
                            # 中文：正常语速约 5 字符/秒
                            normal_rate = 5.0
                            target_rate = speech_rate  # 中文直接使用
                            speed_factor = speech_rate / normal_rate
                        else:
                            # 英文或其他语言：正常语速约 12 字符/秒
                            normal_rate = 12.0
                            
                            # 如果 speech_rate 很小（< 6），可能是中文语速，需要转换
                            # 假设中文和英文的语速比例约为 2.4:1（英文是中文的 2.4 倍）
                            if speech_rate < 6.0:
                                # 可能是中文语速，转换为英文等效语速
                                # 例如：中文 3 字符/秒 -> 英文约 7.2 字符/秒（3 * 2.4）
                                converted_rate = speech_rate * 2.4
                                target_rate = converted_rate
                                speed_factor = converted_rate / normal_rate
                                print(f"[YourTTS Service] ⚠️  Detected Chinese-like speech rate ({speech_rate:.2} chars/s), converted to English equivalent ({converted_rate:.2} chars/s)")
                            else:
                                # 已经是英文语速范围，直接使用
                                target_rate = speech_rate
                                speed_factor = speech_rate / normal_rate
                        
                        # 限制速度因子范围（0.5x - 2.0x），但允许更宽的范围以跟随用户语速
                        # 如果用户说得很快或很慢，应该反映出来
                        speed_factor = max(0.4, min(2.5, speed_factor))
                        
                        if abs(speed_factor - 1.0) > 0.05:  # 只有当差异超过 5% 时才调整
                            print(f"[YourTTS Service] Adjusting speech rate: {target_rate:.2} chars/s (normal: {normal_rate:.2} chars/s, factor: {speed_factor:.2}x)")
                            print(f"[YourTTS Service] Audio dtype before stretch: {wav_np.dtype}, shape: {wav_np.shape}")
                            
                            # 使用 librosa 进行时间拉伸（保持音调）
                            if use_librosa:
                                try:
                                    # 注意：确保输入是 float64，并且是连续的数组（C-contiguous）
                                    if not wav_np.flags['C_CONTIGUOUS']:
                                        wav_np = np.ascontiguousarray(wav_np, dtype=np.float64)
                                    
                                    wav_np = librosa.effects.time_stretch(wav_np, rate=speed_factor)
                                    print(f"[YourTTS Service] ✅ Speech rate adjusted using librosa, new length: {len(wav_np)} samples, dtype: {wav_np.dtype}")
                                    
                                    # 更新 wav 变量（保持原始类型，但使用调整后的数据）
                                    wav = wav_np
                                except Exception as librosa_error:
                                    print(f"[YourTTS Service] ❌ Error: librosa.effects.time_stretch failed: {librosa_error}")
                                    import traceback
                                    traceback.print_exc()
                                    # 保持原始音频，不进行调整
                            else:
                                print(f"[YourTTS Service] ⚠️  Warning: librosa not available, cannot adjust speech rate")
                        else:
                            print(f"[YourTTS Service] Speech rate ({speech_rate:.2} chars/s) is close to normal ({normal_rate:.2} chars/s), no adjustment needed")
                    except Exception as e:
                        print(f"[YourTTS Service] ⚠️  Warning: Failed to adjust speech rate: {e}")
                        import traceback
                        traceback.print_exc()
                        # 即使失败也继续，使用原始音频
        finally:
            # 确保临时文件被清理（即使发生异常）
            if speaker_wav and os.path.exists(speaker_wav):
                try:
                    os.unlink(speaker_wav)
                except Exception as e:
                    print(f"Warning: Failed to delete temp file {speaker_wav}: {e}")
        
        # 转换为列表
        # 处理不同的返回类型：np.ndarray, torch.Tensor, 或 list
        # 注意：需要将 numpy float32 转换为 Python float，以便 JSON 序列化
        if isinstance(wav, np.ndarray):
            # 确保转换为 Python float 类型
            audio_list = [float(x) for x in wav.flatten()]
        elif isinstance(wav, torch.Tensor):
            # 从 Tensor 转换为 numpy，再转换为 Python float
            audio_array = wav.cpu().numpy()
            audio_list = [float(x) for x in audio_array.flatten()]
        else:
            # 如果是 list，也需要确保是 Python float
            audio_list = [float(x) for x in wav]
        
        # 确定是否使用了 reference_audio
        used_reference = speaker_wav is not None
        
        # 输出最终状态日志
        print(f"[YourTTS Service] " + "=" * 70)
        if used_reference:
            if use_cached and speaker_id:
                print(f"[YourTTS Service] 🎯 FINAL STATUS: ✅ Voice cloning APPLIED")
                print(f"[YourTTS Service]    ✓ Used cached reference_audio (speaker_id: '{speaker_id}')")
                print(f"[YourTTS Service]    ✓ Reference audio was successfully used for zero-shot voice cloning")
            else:
                print(f"[YourTTS Service] 🎯 FINAL STATUS: ✅ Voice cloning APPLIED")
                print(f"[YourTTS Service]    ✓ Used provided reference_audio")
                print(f"[YourTTS Service]    ✓ Reference audio was successfully used for zero-shot voice cloning")
        else:
            if speaker_id:
                print(f"[YourTTS Service] 🎯 FINAL STATUS: ⚠️  Voice cloning NOT applied")
                print(f"[YourTTS Service]    ✗ Speaker_id '{speaker_id}' not found in cache")
                print(f"[YourTTS Service]    ✗ Used default voice instead (no voice cloning)")
            else:
                print(f"[YourTTS Service] 🎯 FINAL STATUS: ⚠️  Voice cloning NOT applied")
                print(f"[YourTTS Service]    ✗ No reference_audio available")
                print(f"[YourTTS Service]    ✗ Used default voice instead (no voice cloning)")
        print(f"[YourTTS Service] " + "=" * 70)
        
        return jsonify({
            "audio": audio_list,
            "sample_rate": 22050,  # YourTTS 默认采样率
            "text": text,
            "used_reference": used_reference,  # 指示是否使用了参考音频
            "speaker_applied": used_reference  # 指示音色是否被应用（zero-shot）
        })
        
    except Exception as e:
        import traceback
        error_msg = str(e)
        traceback.print_exc()
        return jsonify({
            "error": error_msg,
            "type": type(e).__name__
        }), 500

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="YourTTS HTTP Service")
    parser.add_argument('--gpu', action='store_true', help='Use GPU if available')
    parser.add_argument('--port', type=int, default=5004, help='Server port (default: 5004)')
    parser.add_argument('--host', type=str, default='127.0.0.1', help='Server host (default: 127.0.0.1, use 0.0.0.0 for WSL)')
    parser.add_argument('--check-deps', action='store_true', help='Check dependencies and exit')
    args = parser.parse_args()
    
    # 如果只是检查依赖，运行检查后退出
    if args.check_deps:
        import check_dependencies
        sys.exit(check_dependencies.main())
    
    print("=" * 60)
    print("  YourTTS HTTP Service (Zero-shot TTS)")
    print("=" * 60)
    
    # 如果 host 是 0.0.0.0，提示可以从 Windows 访问
    if args.host == '0.0.0.0':
        print("  Running in WSL mode (accessible from Windows)")
        print(f"  Windows endpoint: http://127.0.0.1:{args.port}")
    
    # 确定模型路径
    model_path = project_root / "core" / "engine" / "models" / "tts" / "your_tts"
    if not model_path.exists():
        model_path = Path("core/engine/models/tts/your_tts")
    
    # 获取设备
    device = get_device(args.gpu)
    
    # 如果请求使用 GPU 但检测到 CPU，输出警告
    if args.gpu and device == "cpu":
        print("⚠️  WARNING: GPU was requested but not available!")
        print("   Make sure:")
        print("   1. NVIDIA drivers are installed in WSL")
        print("   2. CUDA toolkit is installed in WSL")
        print("   3. PyTorch with CUDA support is installed")
        print("   4. Run: nvidia-smi in WSL to verify GPU access")
        print("")
    
    # 加载模型
    try:
        load_model(model_path, device)
    except Exception as e:
        print(f"\n❌ Failed to start service: {e}")
        print("\n💡 Troubleshooting:")
        print("   1. Check dependencies: python core/engine/scripts/check_dependencies.py")
        print("   2. Install TTS: pip install TTS")
        print("   3. Install other dependencies: pip install torch torchaudio soundfile")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    
    print(f"\n🚀 Starting server on http://{args.host}:{args.port}")
    print("   Endpoints:")
    print("     GET  /health     - Health check")
    print("     POST /synthesize - Synthesize speech (zero-shot supported)")
    print(f"   Device: {device}")
    print("\n   Press Ctrl+C to stop")
    print("=" * 60)
    
    app.run(host=args.host, port=args.port, debug=False)

