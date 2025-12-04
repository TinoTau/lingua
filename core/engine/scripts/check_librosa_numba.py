#!/usr/bin/env python3
"""检查 librosa 和 numba 版本"""

try:
    import librosa
    print(f"✅ librosa: {librosa.__version__}")
except ImportError as e:
    print(f"❌ librosa not installed: {e}")

try:
    import numba
    print(f"✅ numba: {numba.__version__}")
except ImportError as e:
    print(f"❌ numba not installed: {e}")

try:
    import numpy as np
    print(f"✅ numpy: {np.__version__}")
except ImportError as e:
    print(f"❌ numpy not installed: {e}")

# 测试 float64 转换
try:
    import numpy as np
    test_array = np.array([0.1, 0.2, 0.3], dtype=np.float32)
    converted = test_array.astype(np.float64)
    print(f"✅ Float32 to float64 conversion works: {test_array.dtype} -> {converted.dtype}")
except Exception as e:
    print(f"❌ Float conversion test failed: {e}")

# 测试 librosa time_stretch（如果可用）
try:
    import librosa
    import numpy as np
    test_audio = np.array([0.1, 0.2, 0.3, 0.4, 0.5], dtype=np.float64)
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print(f"✅ librosa.effects.time_stretch works with float64")
except Exception as e:
    print(f"❌ librosa.effects.time_stretch test failed: {e}")

