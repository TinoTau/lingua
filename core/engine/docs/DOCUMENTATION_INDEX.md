# 文档索引

**最后更新**: 2025-11-21

---

## 📚 核心文档

### 系统架构
- **SYSTEM_ARCHITECTURE_OVERVIEW.md** (2024-12-19) - 系统架构概览
- **SPEECH_TO_SPEECH_TRANSLATION_STATUS.md** (2024-12-19) - 语音转语音翻译系统状态

### 测试指南
- **TESTING_GUIDE.md** (2024-12-19) - 全面功能测试指南

---

## 🔧 功能模块文档

### ASR (语音识别)
- **ASR_WHISPER_PROGRESS_REPORT.md** (2024-12-19) - Whisper ASR 进度报告
- **ASR_WHISPER_REMAINING_TASKS.md** (2024-12-19) - Whisper ASR 剩余任务
- **ASR_WHISPER_TEST_SCRIPT.md** (2024-12-19) - Whisper ASR 测试脚本说明

### NMT (机器翻译)
- 参考根目录: `marian_nmt_interface_spec.md`

### TTS (文本转语音)

#### 英文 TTS ✅
- **TTS_IMPLEMENTATION_COMPLETE.md** (2024-12-19) - TTS 实现完整总结（包含英文和中文问题）

#### 中文 TTS ✅（WSL2 Piper 方案）
- **PIPER_TTS_IMPLEMENTATION_COMPLETE.md** (2025-11-21) - Piper TTS 实现完成总结（已完成）
- 参考架构文档: `../../docs/architecture/WSL2_Piper_ZH_TTS_Deployment_Guide.md`
- 参考文档: `../../docs/architecture/PIPER_TTS_IMPLEMENTATION_COMPLETE.md`

#### 中文 TTS ❌（之前尝试的方案 - 问题总结）
- **VITS_ZH_AISHELL3_ISSUE_SUMMARY.md** (2024-12-19) - vits-zh-aishell3 问题总结
- **BREEZE2_VITS_ISSUE_SUMMARY.md** (2024-12-19) - Breeze2-VITS 问题总结
- **SHERPA_ONNX_VITS_ZH_ISSUE_SUMMARY.md** (2024-12-19) - Sherpa-ONNX-VITS-ZH-LL 问题总结

### Emotion Adapter
- **EMOTION_ADAPTER_FINAL_REPORT.md** (2024-12-19) - Emotion Adapter 最终报告
- **EMOTION_SPEC_IMPLEMENTATION.md** (2024-12-19) - Emotion Adapter 规范实现

### Persona Adapter
- **PERSONA_ADAPTER_IMPLEMENTATION.md** (2024-12-19) - Persona Adapter 实现

---

## 🔍 技术问题文档

### KV Cache 优化
- **KV_CACHE_EXPLANATION.md** (2024-12-19) - KV Cache 说明
- **KV_CACHE_OPTIMIZATION_RECOMMENDED.md** (2024-12-19) - KV Cache 优化建议
- **KV_CACHE_ARCHITECTURE_IMPACT.md** (2024-12-19) - KV Cache 架构影响

### 编译和构建问题
- **MANUAL_COMPILATION_GUIDE.md** (2024-12-19) - 手动编译指南（包含编译卡住诊断）
- **WINDOWS_RUNTIME_LIBRARY_MISMATCH_FIX.md** (2024-12-19) - Windows 运行时库不匹配修复（包含链接器问题）

---

## 📋 任务和进度

### 任务优先级
- **NEXT_TASKS_PRIORITY.md** (2024-12-19) - 下一步任务优先级
- **NEXT_STEPS_RECOMMENDATION.md** (2024-12-19) - 下一步建议

### 已完成功能
- **COMPLETED_FUNCTIONALITY_SUMMARY.md** (2024-12-19) - 已完成功能总结

---

## 📁 已归档文档

### 引擎文档归档
已归档的文档位于 `core/engine/docs/archived/` 目录，包括：
- TTS 实现过程文档（已合并到 TTS_IMPLEMENTATION_COMPLETE.md）
- Emotion 实现过程文档（已合并到 EMOTION_ADAPTER_COMPLETE.md）
- 修复步骤文档（已过时）
- 其他临时文档

### 脚本文档归档
已归档的脚本说明文件位于 `scripts/archived/` 目录，包括：
- `manual_download_piper.md` - Piper 手动下载说明（已过时，使用 WSL2 方案）
- `install_onnxruntime_simple.md` - ONNX Runtime 安装说明（已过时）
- `README_export_models.md` - 模型导出说明（已过时）
- `original_vits_code/README.md` - 原始 VITS 代码说明（已过时）

**活跃的脚本文档**:
- `scripts/wsl2_piper/README.md` - WSL2 Piper 部署和使用指南（活跃）

---

## 📄 根目录文档

- **ASR_WHISPER_IMPLEMENTATION_PLAN.md** - ASR Whisper 实现计划
- **Emotion_Adapter_Spec.md** - Emotion Adapter 规范
- **KV_CACHE_OPTIMIZATION_PLANS.md** - KV Cache 优化计划
- **layer4_implementation_guide.md** - Layer 4 实现指南
- **layer4_task_plan.md** - Layer 4 任务计划
- **marian_nmt_interface_spec.md** - Marian NMT 接口规范
- **PROJECT_PROGRESS.md** - 项目进度
- **TASK_CHECKLIST.md** - 任务清单
- **TTS_VITS_TECHNICAL_PROPOSAL.md** - TTS VITS 技术提案
- **TTS_VITS2_FULL_TECHNICAL_PROPOSAL.md** - TTS VITS2 完整技术提案

---

## 📝 使用说明

1. **查找文档**: 根据功能模块或问题类型查找对应文档
2. **查看状态**: 文档标题后的日期表示最后更新时间
3. **历史参考**: 已归档的文档可在 `archived/` 目录查看
