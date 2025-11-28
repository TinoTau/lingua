# Lingua 项目文档索引

**最后更新**: 2025-11-27

---

## 📚 文档分类

### 🚀 快速开始

- **[编译和启动命令参考](./operational/编译和启动命令参考.md)** ⭐ **推荐阅读**
  - 一键启动命令
  - 手动启动服务
  - 编译命令
  - 服务管理
  - 测试命令
  - GPU 加速配置

- **[GPU 启用指南](./GPU_启用指南.md)**
  - Whisper ASR GPU 加速配置
  - CUDA 安装步骤
  - 验证 GPU 是否启用

- **[PyTorch CUDA 安装指南](./operational/PyTorch_CUDA_安装指南.md)**
  - NMT 服务 GPU 加速配置
  - PyTorch CUDA 版本安装步骤
  - 验证和故障排查

- **[手动停止服务命令](./手动停止服务命令.md)**
  - 停止所有服务的命令
  - 端口占用检查

---

### 📖 项目总览

- **[PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)** ⭐ **推荐阅读**
  - 项目概述、产品功能、系统架构
  - 项目进度、当前问题、技术栈
  - 下一步计划

- **[产品完成度总结](./产品完成度总结.md)**
  - 功能完成度评估
  - 各模块状态

- **[产品验收报告_Lingua_v1.0.md](./产品验收报告_Lingua_v1.0.md)**
  - 产品验收标准
  - 测试结果

---

### 🏗️ 系统架构

#### 架构文档

- **[architecture/SYSTEM_ARCHITECTURE_OVERVIEW.md](../core/engine/docs/SYSTEM_ARCHITECTURE_OVERVIEW.md)**
  - 系统架构总览
  - 技术方案说明

- **[product/Lingua_Core_Runtime_一键启动与服务设计说明.md](./product/Lingua_Core_Runtime_一键启动与服务设计说明.md)**
  - 核心服务组成
  - API 接口说明
  - 配置文件说明

#### 架构问题分析

- **[architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md](./architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md)** 🔴 **关键问题**
  - Marian NMT 中文→英文模型运行时错误
  - 详细的问题分析和解决方案

- **[architecture/S2S_INTEGRATION_ISSUE_REPORT.md](./architecture/S2S_INTEGRATION_ISSUE_REPORT.md)**
  - S2S 集成问题报告
  - 链接器问题和 ONNX IR 版本问题

- **[architecture/LINKER_ISSUE_FIX_SUMMARY.md](./architecture/LINKER_ISSUE_FIX_SUMMARY.md)**
  - 链接器问题修复总结

- **[architecture/LINKER_ISSUE_HISTORY_AND_SOLUTIONS.md](./architecture/LINKER_ISSUE_HISTORY_AND_SOLUTIONS.md)**
  - 链接器问题历史记录

- **[architecture/ORT_UPGRADE_ANALYSIS.md](./architecture/ORT_UPGRADE_ANALYSIS.md)**
  - ONNX Runtime 升级分析

- **[architecture/ORT_UPGRADE_RECOMMENDATION.md](./architecture/ORT_UPGRADE_RECOMMENDATION.md)**
  - ONNX Runtime 升级建议

---

### 🎯 M2M100 相关文档

#### M2M100 项目进度

- **[M2M100_项目进度报告_决策版.md](./M2M100_项目进度报告_决策版.md)**
  - 项目进度总结
  - 决策建议

- **[M2M100_文档索引.md](./M2M100_文档索引.md)**
  - M2M100 相关文档索引

#### M2M100 实施报告

- **[M2M100_短期优化实施报告.md](./M2M100_短期优化实施报告.md)**
  - 短期优化实施情况

- **[M2M100_短期优化完成情况报告.md](./M2M100_短期优化完成情况报告.md)**
  - 短期优化完成度

- **[M2M100_中期优化实施报告.md](./M2M100_中期优化实施报告.md)**
  - 中期优化实施情况

- **[M2M100_工程版实时翻译改造技术说明.md](./M2M100_工程版实时翻译改造技术说明.md)**
  - 工程版改造技术细节

- **[M2M100_工程版实时翻译改造_测试说明.md](./M2M100_工程版实时翻译改造_测试说明.md)**
  - 测试说明

- **[M2M100_工程版实时翻译改造_验收报告.md](./M2M100_工程版实时翻译改造_验收报告.md)**
  - 验收报告

#### M2M100 集成测试

- **[M2M100_集成与测试报告.md](./M2M100_集成与测试报告.md)**
  - 集成测试报告

- **[M2M100_集成测试报告_20250123_完整版.md](./M2M100_集成测试报告_20250123_完整版.md)**
  - 完整版集成测试报告

- **[M2M100_中期优化集成测试说明.md](./M2M100_中期优化集成测试说明.md)**
  - 集成测试说明

- **[product/M2M100_集成测试改造方案.md](./product/M2M100_集成测试改造方案.md)**
  - 集成测试改造方案

#### M2M100 功能评估

- **[M2M100_实时麦克风输入功能评估.md](./M2M100_实时麦克风输入功能评估.md)**
  - 麦克风输入功能评估

- **[功能完成度评估_麦克风输入到语音输出.md](./功能完成度评估_麦克风输入到语音输出.md)**
  - 功能完成度评估

#### M2M100 问题报告

- **[issues/M2M100_翻译重复问题总结.md](./issues/M2M100_翻译重复问题总结.md)**
  - 翻译重复问题分析

- **[M2M100_TTS_问题解决报告.md](./M2M100_TTS_问题解决报告.md)**
  - TTS 问题解决报告

- **[M2M100_NMT_服务依赖修复.md](./M2M100_NMT_服务依赖修复.md)**
  - NMT 服务依赖修复

#### M2M100 TTS 实现

- **[M2M100_TTS_增量播放实现方案.md](./M2M100_TTS_增量播放实现方案.md)**
  - TTS 增量播放方案

- **[M2M100_TTS_增量播放实现总结.md](./M2M100_TTS_增量播放实现总结.md)**
  - TTS 增量播放总结

#### M2M100 产品规划

- **[product/M2M100_实时翻译系统_中期优化路线图.md](./product/M2M100_实时翻译系统_中期优化路线图.md)**
  - 中期优化路线图

- **[product/M2M100_实时翻译系统·短期优化实施清单.md](./product/M2M100_实时翻译系统·短期优化实施清单.md)**
  - 短期优化实施清单

---

### 🎤 TTS 实现文档

#### Piper TTS

- **[PIPER_TTS_IMPLEMENTATION_SUMMARY.md](./PIPER_TTS_IMPLEMENTATION_SUMMARY.md)**
  - Piper TTS 实现总结

- **[PIPER_TTS_FINAL_SUMMARY.md](./PIPER_TTS_FINAL_SUMMARY.md)**
  - Piper TTS 最终总结

- **[architecture/PIPER_TTS_IMPLEMENTATION_COMPLETE.md](./architecture/PIPER_TTS_IMPLEMENTATION_COMPLETE.md)**
  - Piper TTS 完整实现文档

- **[architecture/PIPER_TTS_PLAN_PROGRESS.md](./architecture/PIPER_TTS_PLAN_PROGRESS.md)**
  - Piper TTS 计划进度

- **[architecture/PIPER_TTS_SUMMARY.md](./architecture/PIPER_TTS_SUMMARY.md)**
  - Piper TTS 总结

- **[architecture/PIPER_TTS_TESTING_GUIDE.md](./architecture/PIPER_TTS_TESTING_GUIDE.md)**
  - Piper TTS 测试指南

- **[architecture/WSL2_PIPER_IMPLEMENTATION_SUMMARY.md](./architecture/WSL2_PIPER_IMPLEMENTATION_SUMMARY.md)**
  - WSL2 Piper 部署总结

- **[architecture/WSL2_Piper_ZH_TTS_Deployment_Guide.md](./architecture/WSL2_Piper_ZH_TTS_Deployment_Guide.md)**
  - WSL2 Piper 中文 TTS 部署指南

- **[architecture/PIPER_HTTP_TTS_PLAN_ANALYSIS.md](./architecture/PIPER_HTTP_TTS_PLAN_ANALYSIS.md)**
  - Piper HTTP TTS 计划分析

---

### 🔄 NMT 实现文档

#### Marian NMT

- **[architecture/MARIAN_NMT_MODEL_VERSION_HISTORY.md](./architecture/MARIAN_NMT_MODEL_VERSION_HISTORY.md)**
  - NMT 模型版本历史

- **[architecture/MARIAN_NMT_IR_VERSION_ISSUE.md](./architecture/MARIAN_NMT_IR_VERSION_ISSUE.md)**
  - NMT IR 版本兼容性问题

- **[architecture/MARIAN_ZH_EN_IR9_EXPORT_PLAN_ANALYSIS.md](./architecture/MARIAN_ZH_EN_IR9_EXPORT_PLAN_ANALYSIS.md)**
  - Marian 中英文 IR9 导出计划分析

- **[architecture/MARIAN_ZH_EN_IR9_EXPORT_PLAN_V2_ANALYSIS.md](./architecture/MARIAN_ZH_EN_IR9_EXPORT_PLAN_V2_ANALYSIS.md)**
  - Marian 中英文 IR9 导出计划 V2 分析

- **[architecture/MARIAN_ZH_EN_IR9_EXPORT_ISSUES.md](./architecture/MARIAN_ZH_EN_IR9_EXPORT_ISSUES.md)**
  - Marian 中英文 IR9 导出问题

- **[architecture/MARIAN_ZH_EN_IR9_EXPORT_FIXED_ANALYSIS.md](./architecture/MARIAN_ZH_EN_IR9_EXPORT_FIXED_ANALYSIS.md)**
  - Marian 中英文 IR9 导出修复分析

- **[architecture/MARIAN_DECODER_MODEL_VERIFICATION_REPORT.md](./architecture/MARIAN_DECODER_MODEL_VERIFICATION_REPORT.md)**
  - Marian 解码器模型验证报告

- **[architecture/MARIAN_DECODER_MODEL_VERIFICATION_REPORT_CURRENT.md](./architecture/MARIAN_DECODER_MODEL_VERIFICATION_REPORT_CURRENT.md)**
  - Marian 解码器模型验证报告（当前版本）

- **[architecture/MARIAN_DECODER_CURRENT_MODEL_STATUS.md](./architecture/MARIAN_DECODER_CURRENT_MODEL_STATUS.md)**
  - Marian 解码器当前模型状态

- **[architecture/MARIAN_DECODER_REEXPORTED_MODEL_STATUS.md](./architecture/MARIAN_DECODER_REEXPORTED_MODEL_STATUS.md)**
  - Marian 解码器重新导出模型状态

- **[architecture/MARIAN_DECODER_EXPORT_FIX.md](./architecture/MARIAN_DECODER_EXPORT_FIX.md)**
  - Marian 解码器导出修复

- **[architecture/MARIAN_DECODER_MODEL_SIGNATURE_REPORT.md](./architecture/MARIAN_DECODER_MODEL_SIGNATURE_REPORT.md)**
  - Marian 解码器模型签名报告

- **[architecture/MARIAN_DECODER_SIGNATURE_AND_KV_CACHE_REPORT.md](./architecture/MARIAN_DECODER_SIGNATURE_AND_KV_CACHE_REPORT.md)**
  - Marian 解码器签名和 KV 缓存报告

- **[architecture/MARIAN_DECODER_INPUT_CONSTRUCTION_CODE.md](./architecture/MARIAN_DECODER_INPUT_CONSTRUCTION_CODE.md)**
  - Marian 解码器输入构造代码

---

### 🤖 模型相关文档

#### 模型导出和测试

- **[models/README.md](./models/README.md)**
  - 模型目录说明

- **[models/EXPORT_MANUAL_STEPS.md](./models/EXPORT_MANUAL_STEPS.md)**
  - 模型导出手动步骤

- **[models/M2M100_Model_Acceptance_Report.md](./models/M2M100_Model_Acceptance_Report.md)**
  - M2M100 模型验收报告

- **[models/M2M100_HF_vs_ONNX_对齐测试结果报告.md](./models/M2M100_HF_vs_ONNX_对齐测试结果报告.md)**
  - M2M100 HuggingFace vs ONNX 对齐测试

- **[models/M2M100_S2S_Integration_Test_Success_Report.md](./models/M2M100_S2S_Integration_Test_Success_Report.md)**
  - M2M100 S2S 集成测试成功报告

- **[models/NMT_Model_Upgrade_Technical_Plan.md](./models/NMT_Model_Upgrade_Technical_Plan.md)**
  - NMT 模型升级技术计划

- **[models/NMT_REPETITION_FIX_SUMMARY.md](./models/NMT_REPETITION_FIX_SUMMARY.md)**
  - NMT 重复问题修复总结

#### ASR 相关

- **[models/ASR_AUTO_LANGUAGE_DETECTION_SUMMARY.md](./models/ASR_AUTO_LANGUAGE_DETECTION_SUMMARY.md)**
  - ASR 自动语言检测总结

- **[models/ASR_LANGUAGE_DETECTION_FIX.md](./models/ASR_LANGUAGE_DETECTION_FIX.md)**
  - ASR 语言检测修复

- **[models/ASR_NMT_ISSUE_DIAGNOSIS.md](./models/ASR_NMT_ISSUE_DIAGNOSIS.md)**
  - ASR NMT 问题诊断

- **[models/ASR_NMT_TEST_SUMMARY.md](./models/ASR_NMT_TEST_SUMMARY.md)**
  - ASR NMT 测试总结

#### S2S 相关

- **[models/S2S_AUTO_LANGUAGE_DETECTION_FIX_SUMMARY.md](./models/S2S_AUTO_LANGUAGE_DETECTION_FIX_SUMMARY.md)**
  - S2S 自动语言检测修复总结

- **[models/S2S_AUTO_LANGUAGE_DETECTION_TEST_RESULT.md](./models/S2S_AUTO_LANGUAGE_DETECTION_TEST_RESULT.md)**
  - S2S 自动语言检测测试结果

- **[models/M2M100 工程版实时翻译 — 下一阶段任务指引.md](./models/M2M100%20工程版实时翻译%20—%20下一阶段任务指引.md)**
  - 下一阶段任务指引

---

### 🧪 测试文档

- **[architecture/S2S_FULL_TEST_COMMANDS.md](./architecture/S2S_FULL_TEST_COMMANDS.md)**
  - S2S 完整测试命令

- **[architecture/S2S_FULL_TEST_LINKER_ISSUE.md](./architecture/S2S_FULL_TEST_LINKER_ISSUE.md)**
  - S2S 完整测试链接器问题

- **[architecture/S2S_INTEGRATION_TEST_PASSED.md](./architecture/S2S_INTEGRATION_TEST_PASSED.md)**
  - S2S 集成测试通过

- **[architecture/integration_test_issue_summary.md](./architecture/integration_test_issue_summary.md)**
  - 集成测试问题总结

---

### 💻 客户端文档

- **[客户端UI和WebSocket流式功能说明.md](./客户端UI和WebSocket流式功能说明.md)**
  - 客户端 UI 和 WebSocket 流式功能说明

---

### 🔧 运维文档

- **[operational/编译和启动命令参考.md](./operational/编译和启动命令参考.md)** ⭐ **推荐阅读**
  - 所有编译和启动命令
  - 服务管理
  - 故障排查

- **[手动停止服务命令.md](./手动停止服务命令.md)**
  - 停止服务的各种方法

---

### 📋 技术分析

- **[architecture/LIBRARY_DEPENDENCIES_EXPLANATION.md](./architecture/LIBRARY_DEPENDENCIES_EXPLANATION.md)**
  - 库依赖说明

- **[architecture/ORT_VERSION_CONSTRAINTS_ANALYSIS.md](./architecture/ORT_VERSION_CONSTRAINTS_ANALYSIS.md)**
  - ONNX Runtime 版本约束分析

---

## 🎯 快速导航

### 新加入项目？

1. 阅读 **[PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)** 了解项目全貌
2. 阅读 **[operational/编译和启动命令参考.md](./operational/编译和启动命令参考.md)** 了解如何启动项目
3. 阅读 **[product/Lingua_Core_Runtime_一键启动与服务设计说明.md](./product/Lingua_Core_Runtime_一键启动与服务设计说明.md)** 了解系统架构

### 需要启动服务？

1. 查看 **[operational/编译和启动命令参考.md](./operational/编译和启动命令参考.md)** 获取所有命令
2. 使用一键启动脚本：`.\start_lingua_core.ps1`

### 遇到问题？

1. 查看 **[issues/M2M100_翻译重复问题总结.md](./issues/M2M100_翻译重复问题总结.md)** 了解已知问题
2. 查看 **[architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md](./architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md)** 了解关键问题
3. 查看 **[operational/编译和启动命令参考.md](./operational/编译和启动命令参考.md)** 的故障排查部分

### 需要实现功能？

1. 查看相应的实现文档（TTS、NMT 等）
2. 参考测试指南
3. 查看架构文档了解设计思路

---

## 📁 文档目录结构

```
docs/
├── DOCUMENTATION_INDEX.md                 # 本文档
├── PROJECT_OVERVIEW.md                    # 项目总览（推荐阅读）
├── GPU_启用指南.md                        # GPU 加速配置
├── 手动停止服务命令.md                    # 服务管理
│
├── operational/                           # 运维文档
│   └── 编译和启动命令参考.md              # ⭐ 编译和启动命令
│
├── architecture/                          # 架构相关文档
│   ├── SYSTEM_ARCHITECTURE_OVERVIEW.md
│   ├── MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md
│   ├── S2S_INTEGRATION_ISSUE_REPORT.md
│   ├── PIPER_TTS_IMPLEMENTATION_COMPLETE.md
│   └── ...
│
├── product/                               # 产品相关文档
│   ├── Lingua_Core_Runtime_一键启动与服务设计说明.md
│   ├── M2M100_集成测试改造方案.md
│   └── ...
│
├── issues/                                # 问题报告
│   └── M2M100_翻译重复问题总结.md
│
├── models/                                # 模型相关文档
│   ├── README.md
│   ├── EXPORT_MANUAL_STEPS.md
│   └── ...
│
└── requirements/                          # 需求文档（待补充）
```

---

**最后更新**: 2025-11-27
