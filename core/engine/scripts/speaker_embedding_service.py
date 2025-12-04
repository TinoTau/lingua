#!/usr/bin/env python3
"""
Speaker Embedding HTTP 服务

用于从 Rust 代码调用 SpeechBrain ECAPA-TDNN 模型提取说话者特征向量。

使用方法：
    python speaker_embedding_service.py [--gpu] [--port PORT] [--host HOST]

参数：
    --gpu: 使用 GPU（如果可用）
    --port: 服务端口（默认：5003）
    --host: 服务地址（默认：127.0.0.1）

服务将在 http://127.0.0.1:5003 启动

API 端点：
    POST /extract
    Body: {"audio": [0.1, 0.2, ...]}  # 16kHz 单声道音频数据（f32）
    Response: {"embedding": [0.1, 0.2, ...], "dimension": 192}
"""

import sys
import os
import argparse
from pathlib import Path

# 添加项目路径
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

# 修复 torchaudio 兼容性问题（必须在导入 SpeechBrain 之前）
def fix_torchaudio_compatibility():
    """修复 torchaudio 2.9+ 兼容性问题"""
    try:
        import torchaudio
        # torchaudio 2.9+ 移除了 list_audio_backends 方法
        if not hasattr(torchaudio, 'list_audio_backends'):
            # 创建模拟函数
            def mock_list_audio_backends():
                return ['soundfile']  # 默认后端
            torchaudio.list_audio_backends = mock_list_audio_backends
            # 不打印，避免在导入时输出（会在 load_model 时显示）
    except ImportError:
        pass  # torchaudio 未安装，稍后会报错

# 在导入其他模块之前应用修复
fix_torchaudio_compatibility()

# 进一步修复：在 SpeechBrain 导入前修补其 backend 检查模块
def patch_speechbrain_backend_check():
    """在 SpeechBrain 导入前修补 backend 检查"""
    import types
    
    # 创建模拟的 backend 检查模块
    backend_module_name = 'speechbrain.utils.torch_audio_backend'
    
    # 如果模块还未导入，创建并注册
    if backend_module_name not in sys.modules:
        backend_module = types.ModuleType(backend_module_name)
        
        def patched_check_torchaudio_backend():
            """修补的检查函数，跳过 list_audio_backends 调用"""
            try:
                import torchaudio
                # 只检查 torchaudio 是否存在，不调用 list_audio_backends
                if not hasattr(torchaudio, '__version__'):
                    raise RuntimeError("torchaudio not properly installed")
            except ImportError:
                raise RuntimeError("torchaudio is not installed. Install it with: pip install torchaudio")

        # SpeechBrain 新版本还会从该模块导入 validate_backend / get_audio_backend / set_audio_backend
        # 这里提供简单的兼容实现，避免 ImportError，但不做复杂检查
        def patched_validate_backend():
            """兼容用的 validate_backend，内部复用检查逻辑"""
            return patched_check_torchaudio_backend()

        def get_audio_backend():
            """返回一个固定的后端名称（例如 soundfile）"""
            return "soundfile"

        def set_audio_backend(_backend: str):
            """兼容函数，占位，不执行实际切换"""
            # 在当前场景下，我们只需要避免导入错误
            return None

        backend_module.check_torchaudio_backend = patched_check_torchaudio_backend
        backend_module.validate_backend = patched_validate_backend
        backend_module.get_audio_backend = get_audio_backend
        backend_module.set_audio_backend = set_audio_backend
        sys.modules[backend_module_name] = backend_module
        print("✅ Patched SpeechBrain backend check module (check/validate/get/set)")

# 应用修补（必须在导入 SpeechBrain 之前）
patch_speechbrain_backend_check()

# 修复 huggingface_hub 兼容性问题（必须在导入 SpeechBrain 之前）
def patch_huggingface_hub():
    """修复 huggingface_hub 的 use_auth_token 参数兼容性问题"""
    try:
        import huggingface_hub
        import functools
        
        # 保存原始的 hf_hub_download 函数
        original_hf_hub_download = huggingface_hub.hf_hub_download
        
        @functools.wraps(original_hf_hub_download)
        def patched_hf_hub_download(*args, **kwargs):
            """修补的 hf_hub_download，将 use_auth_token 转换为 token"""
            # 如果提供了 use_auth_token，转换为 token
            if 'use_auth_token' in kwargs:
                token = kwargs.pop('use_auth_token')
                # 只有当 token 不为 None 时才设置
                if token is not None and 'token' not in kwargs:
                    kwargs['token'] = token
            return original_hf_hub_download(*args, **kwargs)
        
        # 替换函数
        huggingface_hub.hf_hub_download = patched_hf_hub_download
        print("✅ Patched huggingface_hub.hf_hub_download (use_auth_token -> token)")
    except ImportError:
        pass  # huggingface_hub 未安装，稍后会报错
    except Exception as e:
        print(f"⚠️  Failed to patch huggingface_hub: {e}")

# 应用 huggingface_hub 修补（必须在导入 SpeechBrain 之前）
patch_huggingface_hub()

# 现在可以安全导入其他模块
from flask import Flask, request, jsonify
import numpy as np
import torch

# 再次确保 torchaudio 修复已应用（在导入 SpeechBrain 之前）
fix_torchaudio_compatibility()

app = Flask(__name__)
classifier = None
device = None

def get_device(use_gpu=False):
    """获取计算设备"""
    if use_gpu and torch.cuda.is_available():
        device = "cuda"
        print(f"✅ Using GPU: {torch.cuda.get_device_name(0)}")
    else:
        device = "cpu"
        if use_gpu:
            print("⚠️  GPU requested but not available, using CPU")
        else:
            print("ℹ️  Using CPU")
    return device

def load_model(model_path, device="cpu"):
    """加载 SpeechBrain ECAPA-TDNN 模型"""
    global classifier
    
    # 确保兼容性修复已应用（在导入 SpeechBrain 之前）
    fix_torchaudio_compatibility()
    patch_speechbrain_backend_check()
    patch_huggingface_hub()
    
    try:
        from speechbrain.inference.speaker import EncoderClassifier
        
        if not model_path.exists():
            raise FileNotFoundError(f"Model not found at {model_path}")
        
        print(f"📁 Loading model from: {model_path}")
        print(f"🔧 Device: {device}")
        
        classifier = EncoderClassifier.from_hparams(
            source=str(model_path),
            run_opts={"device": device}
        )
        
        print("✅ Speaker Embedding model loaded successfully")
        print(f"   Model output dimension: 192")
        print(f"   Device: {device}")
        
        return classifier
    except Exception as e:
        print(f"❌ Failed to load model: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

@app.route('/health', methods=['GET'])
def health():
    """健康检查端点"""
    return jsonify({
        "status": "ok",
        "model_loaded": classifier is not None
    })

@app.route('/extract', methods=['POST'])
def extract_embedding():
    """提取说话者特征向量"""
    try:
        # 先验证输入，再检查模型
        data = request.json
        if data is None:
            return jsonify({"error": "Invalid JSON"}), 400
        
        if 'audio' not in data:
            return jsonify({"error": "Missing 'audio' field"}), 400
        
        # 获取音频数据
        try:
            audio_data = np.array(data['audio'], dtype=np.float32)
        except (ValueError, TypeError) as e:
            return jsonify({"error": f"Invalid audio data: {str(e)}"}), 400
        
        # 验证音频数据
        if len(audio_data) == 0:
            return jsonify({"error": "Empty audio data"}), 400
        
        # 检查模型是否加载
        if classifier is None:
            return jsonify({"error": "Model not loaded"}), 500
        
        # 转换为 tensor [batch, samples]
        # ECAPA-TDNN 期望输入：16kHz 单声道音频
        # 检查音频长度，ECAPA-TDNN 需要至少 1 秒的音频（16000 样本）
        min_samples = 16000  # 1 秒 @ 16kHz
        if len(audio_data) < min_samples:
            # 音频太短，无法提取 embedding，返回标记使用默认声音
            # 尝试简单判断性别（基于音频能量和频率特征）
            # 这是一个简单的启发式方法，不保证准确性
            audio_array = np.array(audio_data, dtype=np.float32)
            # 计算音频的均方根能量
            rms = np.sqrt(np.mean(audio_array ** 2))
            # 简单的性别判断：能量较高可能是男性，能量较低可能是女性（这只是粗略估计）
            # 实际应用中可以使用更复杂的特征
            estimated_gender = "male" if rms > 0.01 else "female"
            
            return jsonify({
                "embedding": None,
                "too_short": True,
                "use_default": True,
                "estimated_gender": estimated_gender,
                "input_samples": len(audio_data),
                "sample_rate": 16000,
                "message": f"Audio too short ({len(audio_data)} samples < {min_samples} required), using default voice"
            }), 200
        
        audio_tensor = torch.from_numpy(audio_data).unsqueeze(0)
        
        # 移动到正确的设备
        # 注意：device 是全局变量，在 load_model 时设置
        current_device = device if device else "cpu"
        if current_device != "cpu":
            audio_tensor = audio_tensor.to(current_device)
        
        # 提取 embedding
        # 输出形状：[batch, 1, 192]
        embeddings = classifier.encode_batch(audio_tensor)
        
        # 转换为列表 [192]（确保移回 CPU）
        embedding = embeddings.squeeze().cpu().numpy()
        
        # 确保是 1D 数组
        if embedding.ndim > 1:
            embedding = embedding.flatten()
        
        embedding_list = embedding.tolist()
        
        # 计算音色特征统计信息（用于显示和调试）
        embedding_array = np.array(embedding_list)
        embedding_stats = {
            "mean": float(np.mean(embedding_array)),
            "std": float(np.std(embedding_array)),
            "min": float(np.min(embedding_array)),
            "max": float(np.max(embedding_array)),
            "norm": float(np.linalg.norm(embedding_array)),  # L2 范数
            "abs_mean": float(np.mean(np.abs(embedding_array))),  # 绝对值均值
        }
        
        # 显示音色信息
        print(f"[Speaker Embedding] ✅ Extracted embedding:")
        print(f"   Dimension: {len(embedding_list)}")
        print(f"   Norm (L2): {embedding_stats['norm']:.4f}")
        print(f"   Mean: {embedding_stats['mean']:.6f}, Std: {embedding_stats['std']:.6f}")
        print(f"   Range: [{embedding_stats['min']:.6f}, {embedding_stats['max']:.6f}]")
        print(f"   Abs Mean: {embedding_stats['abs_mean']:.6f}")
        print(f"   Input: {len(audio_data)} samples @ 16kHz ({len(audio_data)/16000:.2f}s)")
        
        # 显示 embedding 的前几个值（用于快速检查）
        preview_values = embedding_list[:10]
        print(f"   Preview (first 10): {[f'{v:.4f}' for v in preview_values]}")
        
        return jsonify({
            "embedding": embedding_list,
            "dimension": len(embedding_list),
            "input_samples": len(audio_data),
            "sample_rate": 16000,  # ECAPA-TDNN 期望 16kHz
            "stats": embedding_stats  # 添加统计信息
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
    parser = argparse.ArgumentParser(description="Speaker Embedding HTTP Service")
    parser.add_argument('--gpu', action='store_true', help='Use GPU if available')
    parser.add_argument('--port', type=int, default=5003, help='Server port (default: 5003)')
    parser.add_argument('--host', type=str, default='127.0.0.1', help='Server host (default: 127.0.0.1, use 0.0.0.0 for WSL)')
    parser.add_argument('--check-deps', action='store_true', help='Check dependencies and exit')
    args = parser.parse_args()
    
    # 如果只是检查依赖，运行检查后退出
    if args.check_deps:
        import check_dependencies
        sys.exit(check_dependencies.main())
    
    print("=" * 60)
    print("  Speaker Embedding HTTP Service")
    print("=" * 60)
    
    # 如果 host 是 0.0.0.0，提示可以从 Windows 访问
    if args.host == '0.0.0.0':
        print("  Running in WSL mode (accessible from Windows)")
        print(f"  Windows endpoint: http://127.0.0.1:{args.port}")
    
    # 确定模型路径
    model_path = project_root / "core" / "engine" / "models" / "speaker_embedding" / "cache"
    if not model_path.exists():
        model_path = Path("core/engine/models/speaker_embedding/cache")
    
    # 获取设备
    device = get_device(args.gpu)
    
    # 加载模型
    try:
        print("\n🔧 Applying compatibility fixes...")
        fix_torchaudio_compatibility()
        patch_speechbrain_backend_check()
        print("✅ Compatibility fixes applied")
        
        load_model(model_path, device)
    except Exception as e:
        print(f"\n❌ Failed to start service: {e}")
        print("\n💡 Troubleshooting:")
        print("   1. Check dependencies: python core/engine/scripts/check_dependencies.py")
        print("   2. Install missing packages: pip install speechbrain torch 'torchaudio<2.9' soundfile")
        print("   3. If torchaudio 2.9+, try: pip install 'torchaudio<2.9'")
        print("   4. Or the compatibility fix should be applied automatically")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    
    print(f"\n🚀 Starting server on http://{args.host}:{args.port}")
    print("   Endpoints:")
    print("     GET  /health  - Health check")
    print("     POST /extract - Extract speaker embedding")
    print(f"   Device: {device}")
    print("\n   Press Ctrl+C to stop")
    print("=" * 60)
    
    app.run(host=args.host, port=args.port, debug=False)

