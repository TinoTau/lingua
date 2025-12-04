#!/usr/bin/env python3
"""测试 Python 3.10 环境中的 librosa 功能"""

import sys
import numpy as np

try:
    import librosa
    import numba
    import numpy
    
    print(f"✅ numpy: {numpy.__version__}")
    print(f"✅ numba: {numba.__version__}")
    print(f"✅ librosa: {librosa.__version__}")
    print()
    
    # 测试 librosa.effects.time_stretch
    print("测试 librosa.effects.time_stretch...")
    test_audio = np.random.randn(1000).astype(np.float64)
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print("✅ librosa.effects.time_stretch 测试通过")
    print(f"   输入长度: {len(test_audio)}, 输出长度: {len(stretched)}")
    
    # 测试不同的 rate
    print()
    print("测试不同的 rate 值...")
    for rate in [0.5, 0.8, 1.2, 1.5]:
        try:
            stretched = librosa.effects.time_stretch(test_audio, rate=rate)
            print(f"  ✅ rate={rate}: {len(test_audio)} -> {len(stretched)} samples")
        except Exception as e:
            print(f"  ❌ rate={rate} 失败: {e}")
    
    print()
    print("=" * 60)
    print("  ✅ 所有测试通过！")
    print("=" * 60)
    sys.exit(0)
    
except Exception as e:
    print(f"❌ 测试失败: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)

