# 端到端业务流程测试说明

## 📋 测试概述

**测试文件**: `core/engine/tests/business_flow_e2e_test.rs`  
**测试函数**: `test_full_business_flow`

## 🔄 完整业务流程

### 流程步骤

```
音频帧输入
    ↓
1. VAD 检测（语音活动检测）
    ↓
2. 累积音频帧到 ASR 缓冲区
    ↓
3. 检测到语音边界（is_boundary = true）
    ↓
4. 触发 ASR 推理（Whisper）
    ↓
5. 获取 ASR 最终结果（StableTranscript）
    ↓
6. 自动触发 NMT 翻译（Marian NMT）
    ↓
7. 发布事件到 EventBus
    - AsrFinal 事件（ASR 最终结果）
    - Translation 事件（翻译结果）
    ↓
返回 ProcessResult（包含 ASR 和 NMT 结果）
```

## 🧪 测试内容

### 1. 测试设置

- **测试音频**: 30 个音频帧，每帧 0.1 秒（总共 3 秒）
- **VAD 配置**: 每 20 帧检测一次边界（模拟自然停顿）
- **模型**:
  - ASR: Whisper Base（`models/asr/whisper-base/`）
  - NMT: Marian EN-ZH（`models/nmt/marian-en-zh/`）

### 2. 测试流程

1. **初始化 CoreEngine**
   - 加载 Whisper ASR 模型
   - 加载 Marian NMT 模型
   - 初始化所有组件

2. **处理音频帧**
   - 循环处理 30 个音频帧
   - 每帧调用 `process_audio_frame()`
   - 记录 ASR 和 NMT 结果

3. **验证事件发布**
   - 检查 EventBus 中发布的事件
   - 统计 ASR 部分结果事件数
   - 统计 ASR 最终结果事件数
   - 统计翻译事件数

### 3. 验证点

- ✅ ASR 最终结果事件应该被发布
- ✅ 如果有 ASR 最终结果，应该有对应的翻译事件
- ✅ 事件内容正确（文本、语言等）

## ⚠️ 已知问题和性能

### 问题 1: 测试可能很慢

**原因**:
- Whisper 推理很慢（每次推理需要 2-3 秒）
- 测试中每 20 帧检测一次边界，会触发完整的 Whisper 推理
- 30 帧中会有 1 次边界检测，需要等待推理完成

**解决方案**:
- 使用更少的音频帧（如 20 帧）
- 或者使用模拟的 ASR 结果（跳过真实推理）
- 或者增加超时时间

### 问题 2: 静音音频可能无法产生有效结果

**原因**:
- 测试使用的是静音音频（`vec![0.0; 1600]`）
- Whisper 可能无法从静音中提取有效文本
- 这可能导致 ASR 结果为空

**解决方案**:
- 使用真实的音频文件（如 `third_party/jfk.wav`）
- 或者接受空结果作为测试的一部分

## 📊 测试输出说明

### 预期输出

```
========== 开始端到端业务流程测试 ==========
流程：音频帧 → VAD → ASR → NMT → 事件发布

帧 20: ASR 最终结果
  文本: [转录文本]
  语言: en

帧 20: NMT 翻译结果
  翻译: [翻译文本]
  是否稳定: true

========== 事件统计 ==========
总事件数: 2
ASR 部分结果事件: 0
ASR 最终结果事件: 1
翻译事件: 1

========== 测试结果 ==========
ASR 部分结果事件: 0
ASR 最终结果事件: 1
翻译事件: 1
ASR 最终结果数: 1
翻译结果数: 1

✓ 端到端业务流程测试完成
```

## 🔍 调试建议

如果测试卡住或很慢：

1. **检查模型文件是否存在**
   ```bash
   ls models/asr/whisper-base/
   ls models/nmt/marian-en-zh/
   ```

2. **减少测试帧数**
   - 将 30 帧改为 20 帧
   - 减少边界检测频率

3. **添加超时**
   - 使用 `tokio::time::timeout()` 包装测试

4. **检查死锁**
   - 检查 Mutex 锁的使用
   - 确保没有嵌套锁

5. **使用真实音频文件**
   - 替换静音音频为真实音频
   - 使用 `third_party/jfk.wav`

## 📝 测试代码位置

- **测试文件**: `core/engine/tests/business_flow_e2e_test.rs`
- **核心方法**: `CoreEngine::process_audio_frame()`
- **实现位置**: `core/engine/src/bootstrap.rs`

## 🎯 测试目标

验证以下功能是否正常工作：

1. ✅ VAD 能够检测语音边界
2. ✅ ASR 能够从音频中提取文本
3. ✅ NMT 能够将文本翻译成目标语言
4. ✅ 事件能够正确发布到 EventBus
5. ✅ 完整流程能够端到端运行

---

**最后更新**: 2024-12-19

