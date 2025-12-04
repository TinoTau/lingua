# 测试指南

## 单元测试

### Python 服务测试

#### Speaker Embedding 服务测试

```bash
# 运行所有测试
python core/engine/scripts/test_speaker_embedding_service.py

# 运行特定测试
python core/engine/scripts/test_speaker_embedding_service.py TestSpeakerEmbeddingService.test_health_check_no_model

# 详细输出
python core/engine/scripts/test_speaker_embedding_service.py -v
```

**测试覆盖**：
- ✅ 设备选择（CPU/GPU）
- ✅ 健康检查（模型未加载）
- ✅ 提取 embedding（各种错误情况）
- ✅ 提取 embedding（有效数据，需要模型）
- ✅ 潜在 bug 测试（device 全局变量、embedding squeeze）

#### YourTTS 服务测试

```bash
# 运行所有测试
python core/engine/scripts/test_yourtts_service.py

# 运行特定测试
python core/engine/scripts/test_yourtts_service.py TestYourTtsService.test_health_check_no_model

# 详细输出
python core/engine/scripts/test_yourtts_service.py -v
```

**测试覆盖**：
- ✅ 设备选择（CPU/GPU）
- ✅ 健康检查（模型未加载）
- ✅ 语音合成（各种错误情况）
- ✅ 语音合成（带/不带参考音频）
- ✅ 潜在 bug 测试（wav 类型转换、采样率假设、临时文件清理）

### Rust 客户端测试

#### Speaker Embedding 客户端测试

```bash
# 运行所有测试（不需要服务）
cargo test --test speaker_embedding_client_test

# 运行需要服务的测试
cargo test --test speaker_embedding_client_test -- --ignored

# 运行特定测试
cargo test --test speaker_embedding_client_test test_speaker_embedding_client_config
```

**测试覆盖**：
- ✅ 配置创建
- ✅ 健康检查（需要服务）
- ✅ 提取 embedding（需要服务）
- ✅ 错误处理（空音频、短音频）

#### YourTTS 客户端测试

```bash
# 运行所有测试（不需要服务）
cargo test --test yourtts_http_test

# 运行需要服务的测试
cargo test --test yourtts_http_test -- --ignored

# 运行特定测试
cargo test --test yourtts_http_test test_yourtts_http_config
```

**测试覆盖**：
- ✅ 配置创建
- ✅ 语音合成（需要服务）
- ✅ Zero-shot TTS（需要服务）
- ✅ 错误处理（空文本）
- ✅ 多语言支持（中文）

## 集成测试

### 端到端测试流程

1. **启动服务**：
   ```bash
   # 终端 1
   python core/engine/scripts/speaker_embedding_service.py --gpu
   
   # 终端 2
   python core/engine/scripts/yourtts_service.py --gpu
   ```

2. **运行 Rust 集成测试**：
   ```bash
   cargo test --test speaker_embedding_client_test -- --ignored
   cargo test --test yourtts_http_test -- --ignored
   ```

3. **验证功能**：
   - Speaker Embedding 服务能正确提取 192 维特征向量
   - YourTTS 服务能正确合成语音
   - Zero-shot TTS 能使用参考音频

## 发现的 Bug 和修复

### 已修复

1. **speaker_embedding_service.py - device 全局变量问题**
   - 问题：`device` 可能未正确初始化
   - 修复：添加了默认值处理

2. **yourtts_service.py - wav 类型转换问题**
   - 问题：未处理 `torch.Tensor` 类型
   - 修复：添加了 `torch.Tensor` 转换逻辑

3. **yourtts_service.py - 临时文件清理**
   - 问题：异常时临时文件可能未清理
   - 修复：使用 `try-finally` 确保清理

### 已知问题

1. **参考音频采样率假设**
   - 问题：代码假设参考音频是 22050 Hz
   - 影响：如果输入不是 22050 Hz，音色克隆效果会受影响
   - 状态：已添加 TODO 标记，需要实现重采样

2. **音频数据验证不足**
   - 问题：未验证音频数据范围和最小长度
   - 影响：可能导致模型处理失败
   - 状态：建议添加验证（见 BUG_FIXES.md）

## 测试最佳实践

1. **单元测试**：测试单个函数/方法，不依赖外部服务
2. **集成测试**：测试完整流程，需要服务运行
3. **错误处理**：测试各种错误情况
4. **边界条件**：测试空数据、极值等

## 持续集成

建议在 CI/CD 中：
1. 运行所有单元测试（不需要服务）
2. 可选：运行集成测试（需要服务环境）
3. 检查代码覆盖率

