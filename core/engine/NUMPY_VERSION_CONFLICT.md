# numpy 版本冲突说明

## 冲突详情

安装时 pip 报告了依赖冲突：

```
tts 0.22.0 requires numpy==1.22.0; python_version <= "3.10", but you have numpy 1.24.3 which is incompatible.
```

## 原因

1. **TTS 库要求**: numpy==1.22.0（针对 Python 3.10）
2. **我们安装**: numpy==1.24.3（为了与 numba 0.59.1 和 librosa 0.10.1 兼容）

## 影响分析

### 通常情况：✅ 可能工作正常

- numpy 1.24.3 与 1.22.0 **向后兼容**
- 大多数 API 没有变化
- TTS 库可能仍然可以正常工作

### 潜在问题：⚠️ 需要注意

- 某些边缘情况可能有问题
- 如果 TTS 使用了 numpy 1.22.0 特定的功能或修复

## 测试建议

### 1. 测试 TTS 库基本功能

```bash
bash core/engine/scripts/test_tts_compatibility.sh
```

### 2. 实际使用测试

启动 YourTTS 服务并发送测试请求：

```bash
bash core/engine/scripts/start_yourtts_wsl.sh
```

观察日志中是否有 numpy 相关的错误。

### 3. 检查是否有问题

常见症状：
- ImportError 或 AttributeError
- 模型加载失败
- 运行时错误

## 解决方案

### 方案 1: 保持当前版本（推荐，先测试）

如果 TTS 工作正常，保持当前配置：
- ✅ librosa 工作正常（已测试）
- ✅ numpy 1.24.3 通常向后兼容
- ⚠️ 需要验证 TTS 是否正常工作

### 方案 2: 如果 TTS 有问题，尝试降级 numpy

```bash
source venv-wsl-py310/bin/activate
pip install 'numpy==1.22.0' --force-reinstall --no-cache-dir

# 测试 librosa 是否仍然工作
python -c "import numpy, librosa; import numpy as np; test_audio = np.random.randn(1000).astype(np.float64); librosa.effects.time_stretch(test_audio, rate=1.0); print('✅ 测试通过')"
```

**注意**：降级 numpy 可能会影响 librosa 的兼容性。

### 方案 3: 使用 pip 的依赖解析忽略

如果需要，可以使用 `--no-deps` 或 `--no-dependency-check`，但这不推荐。

## 当前状态

✅ **librosa 工作正常** - 这是最重要的，因为这是之前的主要问题
⚠️ **TTS 需要验证** - 需要实际测试是否工作

## 建议

1. **先测试 TTS 是否工作**：
   - 启动服务
   - 发送测试请求
   - 观察是否有错误

2. **如果没有错误**：
   - 可以忽略版本警告
   - 保持当前配置

3. **如果有错误**：
   - 尝试方案 2（降级 numpy）
   - 或者查找 TTS 库的更新版本（可能支持更新的 numpy）

## 验证清单

- [ ] TTS 库可以正常导入
- [ ] YourTTS 服务可以正常启动
- [ ] 模型可以正常加载
- [ ] 合成请求可以正常处理
- [ ] 没有 numpy 相关的错误

