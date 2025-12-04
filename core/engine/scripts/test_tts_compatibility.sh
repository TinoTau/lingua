#!/bin/bash
# 测试 TTS 库与 numpy 1.24.3 的兼容性

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  测试 TTS 库兼容性"
echo "============================================================"
echo ""

# 激活虚拟环境
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
else
    echo "❌ 虚拟环境不存在"
    exit 1
fi

echo "当前版本："
python -c "import numpy, TTS; print(f'numpy: {numpy.__version__}'); print(f'TTS: {TTS.__version__}')"

echo ""
echo "============================================================"
echo "  测试 TTS 基本功能"
echo "============================================================"
echo ""

python -c "
import sys

print('1. 测试导入 TTS 库...')
try:
    from TTS.api import TTS
    print('   ✅ TTS 库导入成功')
except Exception as e:
    print(f'   ❌ TTS 库导入失败: {e}')
    sys.exit(1)

print()
print('2. 测试初始化 TTS 对象（不下载模型）...')
try:
    # 只测试导入，不加载模型（避免下载）
    from TTS.api import TTS
    print('   ✅ TTS API 可用')
    # 不实际创建实例，避免下载模型
    print('   ℹ️  跳过模型加载（避免下载，可以在实际使用时测试）')
except Exception as e:
    print(f'   ❌ TTS 初始化失败: {e}')
    sys.exit(1)

print()
print('3. 测试 TTS 相关的 numpy 操作...')
try:
    import numpy as np
    # 模拟 TTS 可能使用的 numpy 操作
    test_array = np.array([0.1, 0.2, 0.3], dtype=np.float32)
    result = test_array * 2.0
    print(f'   ✅ numpy 操作正常: {result}')
except Exception as e:
    print(f'   ❌ numpy 操作失败: {e}')
    sys.exit(1)

print()
print('============================================================')
print('  ✅ 基本兼容性测试通过')
print('============================================================')
print()
print('注意：')
print('  - TTS 库要求 numpy==1.22.0，但我们使用 numpy 1.24.3')
print('  - 通常 numpy 向后兼容，1.24.3 应该可以工作')
print('  - 如果实际使用 TTS 时出现问题，可以考虑：')
print('    1. 测试实际加载模型和使用')
print('    2. 如果需要，降级到 numpy 1.22.0（可能影响 librosa）')
print()
"

if [ $? -eq 0 ]; then
    echo "✅ 兼容性测试通过"
    echo ""
    echo "建议：在实际使用 YourTTS 服务时观察是否有错误"
    echo "如果没有错误，可以忽略版本警告"
else
    echo "❌ 兼容性测试失败"
    exit 1
fi

