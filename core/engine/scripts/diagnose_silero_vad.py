#!/usr/bin/env python3
"""
Silero VAD 诊断脚本
用于诊断为什么单元测试通过但实际语音不对
"""

import sys
import numpy as np
from pathlib import Path

try:
    import onnxruntime as ort
except ImportError:
    print("❌ 缺少依赖: pip install onnxruntime numpy")
    sys.exit(1)

def analyze_model_output(model_path, audio_data, frame_size=512):
    """分析模型输出"""
    print("=" * 60)
    print("模型输出分析")
    print("=" * 60)
    
    # 加载模型
    session = ort.InferenceSession(str(model_path), providers=['CPUExecutionProvider'])
    
    # 获取输入输出信息
    input_names = [inp.name for inp in session.get_inputs()]
    output_names = [out.name for out in session.get_outputs()]
    
    print(f"\n模型输入: {input_names}")
    print(f"模型输出: {output_names}")
    
    # 初始化状态
    hidden_state = np.zeros((2, 1, 128), dtype=np.float32)
    sample_rate = np.array([16000], dtype=np.int64)
    
    # 处理音频帧
    results = []
    num_frames = len(audio_data) // frame_size
    
    print(f"\n音频信息:")
    print(f"  总长度: {len(audio_data)} 样本 ({len(audio_data)/16000:.2f} 秒)")
    print(f"  帧数: {num_frames}")
    print(f"  每帧: {frame_size} 样本 ({frame_size/16000*1000:.1f} ms)")
    print(f"  音频统计: min={audio_data.min():.4f}, max={audio_data.max():.4f}, rms={np.sqrt(np.mean(audio_data**2)):.4f}")
    
    print(f"\n处理帧:")
    for i in range(min(num_frames, 10)):  # 只处理前 10 帧
        start_idx = i * frame_size
        end_idx = start_idx + frame_size
        frame = audio_data[start_idx:end_idx]
        
        # 归一化
        frame = np.clip(frame, -1.0, 1.0)
        
        # 准备输入
        audio_input = frame.reshape(1, -1).astype(np.float32)
        
        inputs = {
            input_names[0]: audio_input,
            input_names[1]: hidden_state,
            input_names[2]: sample_rate,
        }
        
        # 运行推理
        outputs = session.run(output_names, inputs)
        output = outputs[0]
        new_state = outputs[1] if len(outputs) > 1 else None
        
        # 更新状态
        if new_state is not None:
            hidden_state = new_state
        
        # 解析输出
        raw_output = float(output.flatten()[0])
        
        # 尝试不同的解析方式
        prob_1000 = 1.0 / (1.0 + np.exp(-raw_output * 1000.0))
        prob_100 = 1.0 / (1.0 + np.exp(-raw_output * 100.0))
        prob_direct = raw_output
        prob_inverted = 1.0 - raw_output
        
        results.append({
            'frame': i,
            'raw_output': raw_output,
            'prob_1000': prob_1000,
            'prob_100': prob_100,
            'prob_direct': prob_direct,
            'prob_inverted': prob_inverted,
            'audio_max': np.abs(frame).max(),
            'audio_rms': np.sqrt(np.mean(frame**2)),
        })
        
        print(f"  帧 {i+1}: raw={raw_output:.6f}, scale_1000={prob_1000:.4f}, scale_100={prob_100:.4f}, direct={prob_direct:.4f}, inverted={prob_inverted:.4f}, audio_rms={results[-1]['audio_rms']:.4f}")
    
    # 统计分析
    print(f"\n统计分析:")
    raw_outputs = [r['raw_output'] for r in results]
    print(f"  原始输出范围: [{min(raw_outputs):.6f}, {max(raw_outputs):.6f}]")
    print(f"  原始输出均值: {np.mean(raw_outputs):.6f}")
    print(f"  原始输出标准差: {np.std(raw_outputs):.6f}")
    
    prob_1000s = [r['prob_1000'] for r in results]
    print(f"  系数 1000 后范围: [{min(prob_1000s):.4f}, {max(prob_1000s):.4f}]")
    print(f"  系数 1000 后均值: {np.mean(prob_1000s):.4f}")
    print(f"  系数 1000 后标准差: {np.std(prob_1000s):.4f}")
    
    # 判断建议
    print(f"\n判断建议:")
    if np.std(raw_outputs) < 0.0001:
        print("  ⚠️  警告: 输出值差异很小，可能无法区分语音和静音")
        print("     建议: 检查模型文件或输入格式")
    else:
        print(f"  ✅ 输出值有差异（标准差: {np.std(raw_outputs):.6f}）")
    
    if np.std(prob_1000s) < 0.1:
        print("  ⚠️  警告: 系数 1000 后的概率差异很小")
        print("     建议: 尝试不同的系数（500, 2000, 5000）")
    else:
        print(f"  ✅ 系数 1000 后的概率有差异（标准差: {np.std(prob_1000s):.4f}）")
    
    return results

def create_test_audio():
    """创建测试音频"""
    sample_rate = 16000
    duration = 1.0  # 1 秒
    
    # 测试 1: 静音
    silence = np.zeros(int(sample_rate * duration), dtype=np.float32)
    
    # 测试 2: 语音（正弦波）
    t = np.arange(int(sample_rate * duration)) / sample_rate
    speech = np.sin(2 * np.pi * 440.0 * t).astype(np.float32) * 0.5
    
    # 测试 3: 混合（前半段语音，后半段静音）
    mixed = np.concatenate([speech[:len(speech)//2], silence[:len(silence)//2]])
    
    return {
        '静音': silence,
        '语音（正弦波）': speech,
        '混合': mixed,
    }

def main():
    # 确定模型路径
    script_dir = Path(__file__).parent
    core_engine_dir = script_dir.parent
    
    # 尝试多个可能的模型路径
    possible_paths = [
        core_engine_dir / "models" / "vad" / "silero" / "silero_vad_github.onnx",
        core_engine_dir / "models" / "vad" / "silero" / "silero_vad_official.onnx",
        core_engine_dir / "models" / "vad" / "silero" / "silero_vad.onnx",
    ]
    
    model_path = None
    for path in possible_paths:
        if path.exists():
            model_path = path
            break
    
    if model_path is None:
        print("❌ 未找到模型文件")
        print("请先运行下载脚本下载模型")
        print("  或手动下载到以下位置之一:")
        for path in possible_paths:
            print(f"    - {path}")
        sys.exit(1)
    
    print(f"使用模型: {model_path}")
    print()
    
    # 创建测试音频
    test_audios = create_test_audio()
    
    # 分析每种测试音频
    for name, audio in test_audios.items():
        print(f"\n{'='*60}")
        print(f"测试: {name}")
        print(f"{'='*60}")
        analyze_model_output(model_path, audio)
    
    print(f"\n{'='*60}")
    print("诊断完成")
    print(f"{'='*60}")
    print("\n建议:")
    print("1. 检查输出值范围是否合理")
    print("2. 检查不同音频的输出值差异是否足够大")
    print("3. 根据实际输出值调整系数（1000, 500, 2000 等）")
    print("4. 如果输出值差异很小，可能需要检查模型文件或输入格式")

if __name__ == "__main__":
    main()

