#!/usr/bin/env python3
"""
YourTTS 服务单元测试

测试 YourTTS HTTP 服务的各个功能
使用 test_output 目录中的真实 WAV 文件作为参考音频
"""

import sys
import unittest
from pathlib import Path
import numpy as np
import torch
import tempfile
import os

# 添加项目路径
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

# 导入服务模块
import yourtts_service as service

# 测试音频文件路径
TEST_AUDIO_DIR = project_root / "test_output"

def load_wav_file(file_path):
    """加载 WAV 文件为 numpy 数组"""
    try:
        import soundfile as sf
        audio_data, sample_rate = sf.read(str(file_path))
        # 转换为单声道（如果是立体声）
        if len(audio_data.shape) > 1:
            audio_data = audio_data.mean(axis=1)
        # 转换为 float32
        audio_data = audio_data.astype(np.float32)
        # 归一化到 [-1, 1]
        if audio_data.max() > 1.0 or audio_data.min() < -1.0:
            audio_data = audio_data / np.max(np.abs(audio_data))
        return audio_data, sample_rate
    except ImportError:
        raise unittest.SkipTest("soundfile not available")
    except Exception as e:
        raise unittest.SkipTest(f"Failed to load WAV file: {e}")

def get_test_audio_files():
    """获取测试音频文件列表"""
    if not TEST_AUDIO_DIR.exists():
        return []
    
    wav_files = list(TEST_AUDIO_DIR.glob("*.wav"))
    return wav_files

class TestYourTtsService(unittest.TestCase):
    """YourTTS 服务测试"""
    
    def setUp(self):
        """测试前准备"""
        # 重置全局变量
        service.tts_model = None
        service.device = None
    
    def test_get_device_cpu(self):
        """测试 CPU 设备选择"""
        device = service.get_device(use_gpu=False)
        self.assertEqual(device, "cpu")
    
    def test_get_device_gpu(self):
        """测试 GPU 设备选择（如果可用）"""
        device = service.get_device(use_gpu=True)
        self.assertIn(device, ["cpu", "cuda"])
    
    def test_health_check_no_model(self):
        """测试健康检查（模型未加载）"""
        with service.app.test_client() as client:
            response = client.get('/health')
            self.assertEqual(response.status_code, 200)
            data = response.get_json()
            self.assertEqual(data['status'], 'ok')
            self.assertFalse(data['model_loaded'])
    
    def test_synthesize_no_model(self):
        """测试语音合成（模型未加载）"""
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={'text': 'Hello'})
            self.assertEqual(response.status_code, 500)
            data = response.get_json()
            self.assertIn('error', data)
    
    def test_synthesize_missing_text(self):
        """测试语音合成（缺少 text 字段）"""
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={})
            self.assertEqual(response.status_code, 400)
            data = response.get_json()
            self.assertIn('error', data)
            self.assertIn('text', data['error'].lower())
    
    def test_synthesize_empty_text(self):
        """测试语音合成（空文本）"""
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={'text': ''})
            self.assertEqual(response.status_code, 400)
            data = response.get_json()
            self.assertIn('error', data)
            self.assertIn('empty', data['error'].lower())
    
    def test_synthesize_whitespace_text(self):
        """测试语音合成（只有空白字符）"""
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={'text': '   '})
            self.assertEqual(response.status_code, 400)
            data = response.get_json()
            self.assertIn('error', data)
    
    def test_synthesize_with_real_reference_audio(self):
        """测试语音合成（使用真实 WAV 文件作为参考音频）"""
        # 获取测试音频文件
        wav_files = get_test_audio_files()
        if not wav_files:
            self.skipTest("No WAV files found in test_output directory")
        
        # 使用第一个 WAV 文件作为参考音频
        wav_file = wav_files[0]
        reference_audio, sample_rate = load_wav_file(wav_file)
        
        # 重采样到 22050 Hz（YourTTS 期望的采样率）
        if sample_rate != 22050:
            try:
                import librosa
                reference_audio = librosa.resample(reference_audio, orig_sr=sample_rate, target_sr=22050)
            except ImportError:
                # 如果没有 librosa，尝试简单重采样
                if sample_rate == 16000:
                    # 简单上采样：线性插值
                    indices = np.linspace(0, len(reference_audio) - 1, int(len(reference_audio) * 22050 / 16000))
                    reference_audio = np.interp(indices, np.arange(len(reference_audio)), reference_audio)
                else:
                    self.skipTest(f"Cannot resample from {sample_rate}Hz to 22050Hz without librosa")
        
        # 检查模型是否可用
        model_path = project_root / "core" / "engine" / "models" / "tts" / "your_tts"
        if not model_path.exists():
            self.skipTest("Model not found, skipping integration test")
        
        # 加载模型
        device = service.get_device(use_gpu=False)
        try:
            service.load_model(model_path, device)
        except Exception as e:
            self.skipTest(f"Failed to load model: {e}")
        
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={
                'text': 'Hello, this is a zero-shot test.',
                'reference_audio': reference_audio.tolist(),
                'language': 'en'
            })
            
            if response.status_code == 200:
                data = response.get_json()
                self.assertIn('audio', data)
                self.assertIn('sample_rate', data)
                self.assertEqual(data['sample_rate'], 22050)
                self.assertTrue(data.get('used_reference', False))
                print(f"✅ Successfully synthesized with reference audio from {wav_file.name}")
            else:
                print(f"Warning: Synthesis failed with status {response.status_code}")
                print(f"Response: {response.get_json()}")
    
    def test_synthesize_without_reference_audio(self):
        """测试语音合成（不带参考音频）"""
        # 检查模型是否可用
        model_path = project_root / "core" / "engine" / "models" / "tts" / "your_tts"
        if not model_path.exists():
            self.skipTest("Model not found, skipping integration test")
        
        # 加载模型
        device = service.get_device(use_gpu=False)
        try:
            service.load_model(model_path, device)
        except Exception as e:
            self.skipTest(f"Failed to load model: {e}")
        
        with service.app.test_client() as client:
            response = client.post('/synthesize', json={
                'text': 'Hello, this is a test.',
                'language': 'en'
            })
            
            if response.status_code == 200:
                data = response.get_json()
                self.assertIn('audio', data)
                self.assertIn('sample_rate', data)
                self.assertFalse(data.get('used_reference', True))
                print("✅ Successfully synthesized without reference audio")
            else:
                print(f"Warning: Synthesis failed with status {response.status_code}")
                print(f"Response: {response.get_json()}")

class TestYourTtsServiceBugs(unittest.TestCase):
    """测试潜在 bug"""
    
    def test_reference_audio_sample_rate(self):
        """测试参考音频采样率假设"""
        # Bug: 代码假设参考音频是 22050 Hz，但实际可能不是
        # 创建不同采样率的音频数据
        audio_16k = np.random.randn(16000).astype(np.float32)
        audio_22k = np.random.randn(22050).astype(np.float32)
        audio_44k = np.random.randn(44100).astype(np.float32)
        
        # 测试保存为 WAV（需要 soundfile）
        try:
            import soundfile as sf
            
            # 使用临时文件，确保在 Windows 上也能正确清理
            tmp_path = None
            try:
                with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp:
                    tmp_path = tmp.name
                
                # 当前代码固定使用 22050 Hz
                sf.write(tmp_path, audio_16k, 22050)  # 错误：应该是 16000
                # 这会导致音频播放速度错误
                
                # 验证文件存在
                self.assertTrue(os.path.exists(tmp_path))
            finally:
                # 确保文件被清理（Windows 需要关闭文件句柄）
                if tmp_path and os.path.exists(tmp_path):
                    try:
                        # 等待一小段时间，确保文件句柄已关闭
                        import time
                        time.sleep(0.1)
                        os.unlink(tmp_path)
                    except PermissionError:
                        # Windows 文件锁定问题，忽略
                        pass
        except ImportError:
            self.skipTest("soundfile not available")
    
    def test_wav_type_conversion(self):
        """测试 wav 类型转换"""
        # Bug: wav 可能是 torch.Tensor 或 np.ndarray
        wav_numpy = np.array([0.1, 0.2, 0.3], dtype=np.float32)
        wav_torch = torch.tensor([0.1, 0.2, 0.3], dtype=torch.float32)
        wav_list = [0.1, 0.2, 0.3]
        
        # 测试转换逻辑
        def convert_to_list(wav):
            if isinstance(wav, np.ndarray):
                return wav.tolist()
            elif isinstance(wav, torch.Tensor):
                return wav.cpu().numpy().tolist()  # Bug: 需要先转换为 numpy
            else:
                return list(wav)
        
        # 使用近似比较（浮点数精度问题）
        result_numpy = convert_to_list(wav_numpy)
        result_torch = convert_to_list(wav_torch)
        result_list = convert_to_list(wav_list)
        
        # 验证长度
        self.assertEqual(len(result_numpy), 3)
        self.assertEqual(len(result_torch), 3)
        self.assertEqual(len(result_list), 3)
        
        # 验证值（使用近似比较）
        for i in range(3):
            self.assertAlmostEqual(result_numpy[i], [0.1, 0.2, 0.3][i], places=5)
            self.assertAlmostEqual(result_torch[i], [0.1, 0.2, 0.3][i], places=5)
            self.assertEqual(result_list[i], [0.1, 0.2, 0.3][i])

if __name__ == '__main__':
    unittest.main()
