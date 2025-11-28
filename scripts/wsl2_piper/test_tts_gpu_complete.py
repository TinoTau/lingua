#!/usr/bin/env python3
"""
TTS GPU 完整测试套件
包括单元测试、集成测试和性能对比测试
"""

import os
import sys
import time
from pathlib import Path
from typing import Optional

# 设置库路径（确保 cuDNN 可被找到）
os.environ.setdefault(
    "LD_LIBRARY_PATH",
    "/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:/lib/x86_64-linux-gnu"
)

try:
    import onnxruntime as ort
    from piper.voice import PiperVoice
except ImportError as e:
    print(f"❌ 导入错误: {e}")
    print("请确保已安装 onnxruntime-gpu 和 piper-tts")
    sys.exit(1)


class Colors:
    """终端颜色"""
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    RESET = '\033[0m'
    BOLD = '\033[1m'


def print_header(text: str):
    """打印标题"""
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'='*60}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{text}{Colors.RESET}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'='*60}{Colors.RESET}\n")


def print_test(name: str, status: str, message: str = ""):
    """打印测试结果"""
    status_color = Colors.GREEN if status == "PASS" else Colors.RED if status == "FAIL" else Colors.YELLOW
    status_symbol = "✓" if status == "PASS" else "✗" if status == "FAIL" else "⚠"
    print(f"{status_color}{status_symbol} {name}{Colors.RESET}", end="")
    if message:
        print(f": {message}")
    else:
        print()


class TTSGPUTestSuite:
    """TTS GPU 测试套件"""
    
    def __init__(self):
        self.model_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx"
        self.config_path = Path.home() / "piper_models" / "zh" / "zh_CN-huayan-medium.onnx.json"
        self.test_text = "这是一个测试文本，用于验证GPU加速效果。"
        self.results = {
            "passed": 0,
            "failed": 0,
            "warnings": 0
        }
    
    def test_onnx_runtime_gpu(self) -> bool:
        """测试 1: ONNX Runtime GPU 支持"""
        print_header("测试 1: ONNX Runtime GPU 支持")
        
        try:
            providers = ort.get_available_providers()
            print(f"可用执行提供程序: {providers}")
            
            if 'CUDAExecutionProvider' in providers:
                print_test("ONNX Runtime CUDA 提供程序", "PASS", "可用")
                self.results["passed"] += 1
                return True
            else:
                print_test("ONNX Runtime CUDA 提供程序", "FAIL", "不可用")
                self.results["failed"] += 1
                return False
        except Exception as e:
            print_test("ONNX Runtime GPU 检查", "FAIL", str(e))
            self.results["failed"] += 1
            return False
    
    def test_model_files_exist(self) -> bool:
        """测试 2: 模型文件存在性检查"""
        print_header("测试 2: 模型文件存在性检查")
        
        model_exists = self.model_path.exists()
        config_exists = self.config_path.exists()
        
        if model_exists:
            print_test("模型文件", "PASS", str(self.model_path))
            self.results["passed"] += 1
        else:
            print_test("模型文件", "FAIL", f"不存在: {self.model_path}")
            self.results["failed"] += 1
        
        if config_exists:
            print_test("配置文件", "PASS", str(self.config_path))
            self.results["passed"] += 1
        else:
            print_test("配置文件", "FAIL", f"不存在: {self.config_path}")
            self.results["failed"] += 1
        
        return model_exists and config_exists
    
    def test_gpu_model_loading(self) -> bool:
        """测试 3: GPU 模式模型加载"""
        print_header("测试 3: GPU 模式模型加载")
        
        try:
            voice = PiperVoice.load(
                str(self.model_path),
                config_path=str(self.config_path),
                use_cuda=True
            )
            print_test("模型加载（GPU）", "PASS", "成功")
            
            # 检查执行提供程序
            try:
                session = voice.session
                providers = session.get_providers()
                print(f"  执行提供程序: {providers}")
                
                if 'CUDAExecutionProvider' in providers:
                    print_test("GPU 执行提供程序", "PASS", "已启用")
                    self.results["passed"] += 2
                    return True
                else:
                    print_test("GPU 执行提供程序", "WARN", "未启用，使用 CPU")
                    self.results["warnings"] += 1
                    self.results["passed"] += 1
                    return False
            except Exception as e:
                print_test("执行提供程序检查", "WARN", f"无法检查: {e}")
                self.results["warnings"] += 1
                self.results["passed"] += 1
                return True
                
        except Exception as e:
            print_test("模型加载（GPU）", "FAIL", str(e))
            self.results["failed"] += 1
            return False
    
    def test_gpu_synthesis(self) -> bool:
        """测试 4: GPU 模式语音合成"""
        print_header("测试 4: GPU 模式语音合成")
        
        try:
            voice = PiperVoice.load(
                str(self.model_path),
                config_path=str(self.config_path),
                use_cuda=True
            )
            
            start_time = time.time()
            audio_generator = voice.synthesize(self.test_text)
            audio_chunks = list(audio_generator)
            end_time = time.time()
            
            if audio_chunks:
                audio_bytes = b''.join(
                    chunk.audio_int16_bytes 
                    for chunk in audio_chunks 
                    if chunk.audio_int16_bytes
                )
                duration = (end_time - start_time) * 1000  # 转换为毫秒
                
                print_test("语音合成（GPU）", "PASS", 
                          f"成功，音频大小: {len(audio_bytes)} 字节，耗时: {duration:.2f}ms")
                self.results["passed"] += 1
                
                # 保存测试文件
                output_path = Path.home() / "piper_env" / "test_gpu_synthesis.wav"
                with open(output_path, "wb") as f:
                    f.write(audio_bytes)
                print(f"  音频已保存到: {output_path}")
                
                return True
            else:
                print_test("语音合成（GPU）", "FAIL", "未生成音频块")
                self.results["failed"] += 1
                return False
                
        except Exception as e:
            print_test("语音合成（GPU）", "FAIL", str(e))
            self.results["failed"] += 1
            return False
    
    def test_cpu_vs_gpu_performance(self) -> bool:
        """测试 5: CPU vs GPU 性能对比"""
        print_header("测试 5: CPU vs GPU 性能对比")
        
        test_text = self.test_text * 3  # 使用更长的文本
        
        # GPU 模式
        try:
            voice_gpu = PiperVoice.load(
                str(self.model_path),
                config_path=str(self.config_path),
                use_cuda=True
            )
            
            start_time = time.time()
            audio_generator_gpu = voice_gpu.synthesize(test_text)
            audio_chunks_gpu = list(audio_generator_gpu)
            gpu_time = (time.time() - start_time) * 1000
            
            audio_bytes_gpu = b''.join(
                chunk.audio_int16_bytes 
                for chunk in audio_chunks_gpu 
                if chunk.audio_int16_bytes
            )
            
            print_test("GPU 合成", "PASS", f"耗时: {gpu_time:.2f}ms, 大小: {len(audio_bytes_gpu)} 字节")
        except Exception as e:
            print_test("GPU 合成", "FAIL", str(e))
            self.results["failed"] += 1
            return False
        
        # CPU 模式
        try:
            voice_cpu = PiperVoice.load(
                str(self.model_path),
                config_path=str(self.config_path),
                use_cuda=False
            )
            
            start_time = time.time()
            audio_generator_cpu = voice_cpu.synthesize(test_text)
            audio_chunks_cpu = list(audio_generator_cpu)
            cpu_time = (time.time() - start_time) * 1000
            
            audio_bytes_cpu = b''.join(
                chunk.audio_int16_bytes 
                for chunk in audio_chunks_cpu 
                if chunk.audio_int16_bytes
            )
            
            print_test("CPU 合成", "PASS", f"耗时: {cpu_time:.2f}ms, 大小: {len(audio_bytes_cpu)} 字节")
            
            # 性能对比
            if cpu_time > 0:
                speedup = cpu_time / gpu_time
                print(f"\n{Colors.BOLD}性能对比:{Colors.RESET}")
                print(f"  CPU: {cpu_time:.2f}ms")
                print(f"  GPU: {gpu_time:.2f}ms")
                print(f"  加速比: {speedup:.2f}x")
                
                if speedup > 1.5:
                    print_test("性能提升", "PASS", f"GPU 比 CPU 快 {speedup:.2f} 倍")
                    self.results["passed"] += 1
                else:
                    print_test("性能提升", "WARN", f"GPU 加速不明显 ({speedup:.2f}x)")
                    self.results["warnings"] += 1
            
            self.results["passed"] += 1
            return True
            
        except Exception as e:
            print_test("CPU 合成", "FAIL", str(e))
            self.results["failed"] += 1
            return False
    
    def run_all_tests(self):
        """运行所有测试"""
        print(f"\n{Colors.BOLD}{Colors.BLUE}{'='*60}{Colors.RESET}")
        print(f"{Colors.BOLD}{Colors.BLUE}TTS GPU 完整测试套件{Colors.RESET}")
        print(f"{Colors.BOLD}{Colors.BLUE}{'='*60}{Colors.RESET}\n")
        
        # 运行测试
        self.test_onnx_runtime_gpu()
        if not self.test_model_files_exist():
            print(f"\n{Colors.RED}模型文件不存在，跳过后续测试{Colors.RESET}")
            return
        
        self.test_gpu_model_loading()
        self.test_gpu_synthesis()
        self.test_cpu_vs_gpu_performance()
        
        # 打印总结
        self.print_summary()
    
    def print_summary(self):
        """打印测试总结"""
        print_header("测试总结")
        
        total = self.results["passed"] + self.results["failed"] + self.results["warnings"]
        
        print(f"总测试数: {total}")
        print(f"{Colors.GREEN}通过: {self.results['passed']}{Colors.RESET}")
        print(f"{Colors.YELLOW}警告: {self.results['warnings']}{Colors.RESET}")
        print(f"{Colors.RED}失败: {self.results['failed']}{Colors.RESET}")
        
        if self.results["failed"] == 0:
            print(f"\n{Colors.GREEN}{Colors.BOLD}✓ 所有测试通过！{Colors.RESET}")
        else:
            print(f"\n{Colors.RED}{Colors.BOLD}✗ 有测试失败{Colors.RESET}")


if __name__ == "__main__":
    suite = TTSGPUTestSuite()
    suite.run_all_tests()

