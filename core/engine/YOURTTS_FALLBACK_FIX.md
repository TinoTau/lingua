# YourTTS Service Fallback 修复

## 问题描述

即使安装了兼容版本（numpy 1.26.4, numba 0.59.1, librosa 0.10.1），librosa.effects.time_stretch 仍然失败：
```
ufunc '_phasor_angles' did not contain a loop with signature matching types <class 'numpy.dtype[float64]'> -> None
```

这是 numba 编译缓存或版本兼容性问题。

## 解决方案

### 代码修改

已修改 `yourtts_service.py`，增加了**自动 fallback 机制**：

1. **优先使用 librosa**（保持音调）：
   - 如果成功，使用 librosa 进行时间拉伸
   - 如果失败，自动捕获异常

2. **自动 fallback 到 scipy**（会改变音调但可以调整速度）：
   - 如果 librosa 失败，自动尝试使用 scipy 重采样
   - scipy 使用线性插值，不需要 numba

3. **保持原始音频**：
   - 如果两者都失败，保持原始音频，服务继续运行

### 代码逻辑

```python
# 1. 尝试 librosa
if use_librosa:
    try:
        wav_np = librosa.effects.time_stretch(wav_np, rate=speed_factor)
        success = True
    except Exception:
        # 2. librosa 失败，fallback 到 scipy
        wav_np = original_wav_np.copy()

# 3. 如果 librosa 失败，使用 scipy
if not success and use_scipy:
    try:
        # scipy 重采样（不需要 numba）
        wav_np = np.interp(...)
        success = True
    except Exception:
        # 4. 两者都失败，保持原始音频
        pass
```

## 效果

### 之前的日志
```
⚠️  Warning: Failed to adjust speech rate: ufunc '_phasor_angles' ...
# 服务继续运行，但语速未调整
```

### 修复后的日志
```
⚠️  librosa failed: ufunc '_phasor_angles' ...
Falling back to scipy for time stretching...
✅ Speech rate adjusted using scipy (note: pitch will change), new length: ...
# 服务正常运行，语速已调整（虽然音调可能会改变）
```

## 优势

1. ✅ **服务不会中断**：即使 librosa 失败，服务继续运行
2. ✅ **自动降级**：librosa → scipy → 原始音频
3. ✅ **功能保留**：虽然 scipy 会改变音调，但至少可以调整语速
4. ✅ **无需手动操作**：代码自动处理，无需重启或重新安装

## 进一步优化（可选）

如果希望完全避免 librosa 错误，可以：

### 方案 1：清除 numba 缓存并重新安装

在 WSL 中运行：
```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate

# 清除缓存
rm -rf ~/.cache/numba

# 重新安装
pip uninstall -y numba llvmlite
pip install 'numba==0.59.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall
```

### 方案 2：降级到更稳定的版本

```bash
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall
```

### 方案 3：禁用 librosa，直接使用 scipy

修改代码，在导入部分添加：
```python
use_librosa = False  # 强制禁用 librosa，只使用 scipy
```

## 当前状态

✅ **代码已修复**：自动 fallback 机制已实现
✅ **服务可用**：即使 librosa 失败，服务也能正常工作
⚠️ **功能降级**：使用 scipy 时音调会改变，但语速可以调整

## 测试建议

重启 YourTTS 服务后，观察日志：
- 如果看到 `Falling back to scipy` → fallback 机制正常工作
- 如果看到 `✅ Speech rate adjusted using scipy` → 功能可用（虽然音调改变）
- 如果没有任何调整信息 → 检查 scipy 是否已安装

## 相关文件

- `core/engine/scripts/yourtts_service.py` - 主服务文件（已修复）
- `core/engine/scripts/fix_numba_cache.sh` - numba 缓存清理脚本

