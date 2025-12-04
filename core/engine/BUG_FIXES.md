# Bug 修复记录

## 已修复的 Bug

### 1. speaker_embedding_service.py - 错误检查顺序问题

**问题**：
- 测试发现：当模型未加载时，即使输入验证失败，也返回 500（模型未加载）而不是 400（输入错误）
- 错误检查顺序：先检查模型，再验证输入

**修复**：
```python
# 修复前
if classifier is None:
    return jsonify({"error": "Model not loaded"}), 500
# 然后验证输入...

# 修复后
# 先验证输入
if 'audio' not in data:
    return jsonify({"error": "Missing 'audio' field"}), 400
# 再检查模型
if classifier is None:
    return jsonify({"error": "Model not loaded"}), 500
```

**影响**：中（错误消息更准确，便于调试）

### 2. yourtts_service.py - 错误检查顺序问题

**问题**：
- 同 speaker_embedding_service.py，错误检查顺序不当

**修复**：
- 先验证输入（text、reference_audio），再检查模型

**影响**：中（错误消息更准确）

### 3. yourtts_service.py - Windows 文件锁定问题

**问题**：
- 测试发现：在 Windows 上，使用 `NamedTemporaryFile` 时，文件可能被锁定，无法立即删除
- 错误：`PermissionError: [WinError 32] 另一个程序正在使用此文件`

**修复**：
```python
# 修复前
with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp_file:
    sf.write(tmp_file.name, ref_audio_array, 22050)
    speaker_wav = tmp_file.name

# 修复后
tmp_file = tempfile.NamedTemporaryFile(suffix='.wav', delete=False)
tmp_file.close()  # 关闭文件句柄，避免 Windows 锁定
try:
    sf.write(tmp_file.name, ref_audio_array, 22050)
    speaker_wav = tmp_file.name
except Exception as e:
    if os.path.exists(tmp_file.name):
        os.unlink(tmp_file.name)
    raise
```

**影响**：高（Windows 兼容性）

### 4. test_yourtts_service.py - 浮点数精度问题

**问题**：
- 测试发现：`np.float32` 转换为列表时，精度损失导致测试失败
- 错误：`AssertionError: [0.10000000149011612, ...] != [0.1, 0.2, 0.3]`

**修复**：
```python
# 修复前
self.assertEqual(convert_to_list(wav_numpy), [0.1, 0.2, 0.3])

# 修复后
for i in range(3):
    self.assertAlmostEqual(result_numpy[i], [0.1, 0.2, 0.3][i], places=5)
```

**影响**：低（测试问题，不影响功能）

### 5. speaker_embedding_service.py - device 全局变量问题

**问题**：
- Line 112: `if device and device != "cpu"` 中，`device` 是全局变量，但在 `extract_embedding` 函数中可能未正确设置
- 如果 `device` 是 `None`，会导致逻辑错误

**修复**：
```python
# 修复前
if device and device != "cpu":
    audio_tensor = audio_tensor.to(device)

# 修复后
current_device = device if device else "cpu"
if current_device != "cpu":
    audio_tensor = audio_tensor.to(current_device)
```

**影响**：低（只有在设备未正确初始化时才会出现问题）

### 6. yourtts_service.py - wav 类型转换问题

**问题**：
- Line 174: `wav.tolist() if isinstance(wav, np.ndarray) else list(wav)`
- 如果 `wav` 是 `torch.Tensor`，直接调用 `list(wav)` 会失败

**修复**：
```python
# 修复前
audio_list = wav.tolist() if isinstance(wav, np.ndarray) else list(wav)

# 修复后
if isinstance(wav, np.ndarray):
    audio_list = wav.tolist()
elif isinstance(wav, torch.Tensor):
    audio_list = wav.cpu().numpy().tolist()
else:
    audio_list = list(wav)
```

**影响**：高（如果 YourTTS 返回 torch.Tensor，会导致服务崩溃）

### 7. yourtts_service.py - 参考音频采样率假设

**问题**：
- Line 150: 代码假设参考音频是 22050 Hz，但实际可能不是
- 如果输入音频是其他采样率（如 16kHz），会导致播放速度错误

**修复**：
- 添加了注释说明当前假设
- 添加了 TODO 标记，需要实现重采样功能

**影响**：中（如果参考音频不是 22050 Hz，音色克隆效果会受影响）

## 潜在问题（未修复）

### 8. 临时文件清理异常处理（已部分修复）

**问题**：
- `yourtts_service.py` 中，如果合成过程中发生异常，临时文件可能不会被清理

**建议修复**：
```python
try:
    # 合成语音
    wav = tts_model.tts(...)
finally:
    # 确保临时文件被清理
    if speaker_wav and os.path.exists(speaker_wav):
        os.unlink(speaker_wav)
```

### 2. 音频数据验证不足

**问题**：
- `speaker_embedding_service.py` 中，没有验证音频数据的范围（应该在 -1.0 到 1.0 之间）
- 没有验证音频长度是否足够（ECAPA-TDNN 可能需要最小长度）

**建议修复**：
```python
# 验证音频数据范围
if np.any(np.abs(audio_data) > 1.0):
    return jsonify({"error": "Audio data out of range [-1.0, 1.0]"}), 400

# 验证最小长度（例如：至少 1 秒）
min_samples = 16000  # 1 秒 @ 16kHz
if len(audio_data) < min_samples:
    return jsonify({"error": f"Audio too short, minimum {min_samples} samples required"}), 400
```

## 测试覆盖

### Python 服务测试
- ✅ `test_speaker_embedding_service.py` - Speaker Embedding 服务测试
- ✅ `test_yourtts_service.py` - YourTTS 服务测试

### Rust 客户端测试
- ✅ `speaker_embedding_client_test.rs` - Speaker Embedding 客户端测试
- ✅ `yourtts_http_test.rs` - YourTTS 客户端测试

## 运行测试

### Python 测试
```bash
# Speaker Embedding 服务测试
python core/engine/scripts/test_speaker_embedding_service.py

# YourTTS 服务测试
python core/engine/scripts/test_yourtts_service.py
```

### Rust 测试
```bash
# 需要服务运行
cargo test --test speaker_embedding_client_test -- --ignored
cargo test --test yourtts_http_test -- --ignored
```

