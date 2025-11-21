# Lingua 项目文档索引

**最后更新**: 2025-11-21

---

## 📚 文档分类

### 1. 项目总览

- **[PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)** ⭐ **推荐阅读**
  - 项目概述、产品功能、系统架构
  - 项目进度、当前问题、技术栈
  - 下一步计划

### 2. 项目进度

- **[PROJECT_PROGRESS.md](../PROJECT_PROGRESS.md)**
  - 详细的项目进度报告
  - 各模块完成度统计

- **[SPEECH_TO_SPEECH_TRANSLATION_STATUS.md](../core/engine/docs/SPEECH_TO_SPEECH_TRANSLATION_STATUS.md)**
  - S2S 系统状态报告
  - 各组件实现状态

### 3. 系统架构

- **[SYSTEM_ARCHITECTURE_OVERVIEW.md](../core/engine/docs/SYSTEM_ARCHITECTURE_OVERVIEW.md)**
  - 系统架构总览
  - 技术方案说明

### 4. 当前问题

- **[MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md](./architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md)** 🔴 **关键问题**
  - Marian NMT 中文→英文模型运行时错误
  - 详细的问题分析和解决方案

- **[S2S_INTEGRATION_ISSUE_REPORT.md](./architecture/S2S_INTEGRATION_ISSUE_REPORT.md)**
  - S2S 集成问题报告
  - 链接器问题和 ONNX IR 版本问题

### 5. 实现文档

#### TTS 实现

- **[PIPER_TTS_IMPLEMENTATION_COMPLETE.md](./architecture/PIPER_TTS_IMPLEMENTATION_COMPLETE.md)**
  - Piper TTS 完整实现文档

- **[WSL2_PIPER_IMPLEMENTATION_SUMMARY.md](./architecture/WSL2_PIPER_IMPLEMENTATION_SUMMARY.md)**
  - WSL2 Piper 部署总结

- **[PIPER_TTS_TESTING_GUIDE.md](./architecture/PIPER_TTS_TESTING_GUIDE.md)**
  - Piper TTS 测试指南

#### NMT 实现

- **[MARIAN_ZH_EN_IR9_EXPORT_GUIDE.md](../MARIAN_ZH_EN_IR9_EXPORT_GUIDE.md)**
  - Marian NMT IR 9 导出操作指南

- **[MARIAN_NMT_IR_VERSION_ISSUE.md](./architecture/MARIAN_NMT_IR_VERSION_ISSUE.md)**
  - NMT IR 版本兼容性问题

- **[MARIAN_NMT_MODEL_VERSION_HISTORY.md](./architecture/MARIAN_NMT_MODEL_VERSION_HISTORY.md)**
  - NMT 模型版本历史

### 6. 技术分析

- **[ORT_UPGRADE_ANALYSIS.md](./architecture/ORT_UPGRADE_ANALYSIS.md)**
  - ONNX Runtime 升级分析

- **[ORT_UPGRADE_RECOMMENDATION.md](./architecture/ORT_UPGRADE_RECOMMENDATION.md)**
  - ONNX Runtime 升级建议

- **[LINKER_ISSUE_FIX_SUMMARY.md](./architecture/LINKER_ISSUE_FIX_SUMMARY.md)**
  - 链接器问题修复总结

---

## 🎯 快速导航

### 新加入项目？

1. 阅读 **[PROJECT_OVERVIEW.md](./PROJECT_OVERVIEW.md)** 了解项目全貌
2. 阅读 **[SYSTEM_ARCHITECTURE_OVERVIEW.md](../core/engine/docs/SYSTEM_ARCHITECTURE_OVERVIEW.md)** 了解系统架构
3. 查看 **[PROJECT_PROGRESS.md](../PROJECT_PROGRESS.md)** 了解当前进度

### 遇到问题？

1. 查看 **[MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md](./architecture/MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md)** 了解当前关键问题
2. 查看 **[S2S_INTEGRATION_ISSUE_REPORT.md](./architecture/S2S_INTEGRATION_ISSUE_REPORT.md)** 了解集成问题

### 需要实现功能？

1. 查看相应的实现文档（TTS、NMT 等）
2. 参考测试指南
3. 查看架构文档了解设计思路

---

## 📁 文档目录结构

```
docs/
├── PROJECT_OVERVIEW.md                    # 项目总览（推荐阅读）
├── DOCUMENTATION_INDEX.md                 # 本文档
├── architecture/                          # 架构相关文档
│   ├── MARIAN_IR7_MODEL_RUNTIME_ISSUE_REPORT.md
│   ├── S2S_INTEGRATION_ISSUE_REPORT.md
│   ├── PIPER_TTS_IMPLEMENTATION_COMPLETE.md
│   ├── WSL2_PIPER_IMPLEMENTATION_SUMMARY.md
│   └── ...
├── product/                               # 产品相关文档
└── requirements/                          # 需求文档

core/engine/docs/                          # 引擎相关文档
├── SYSTEM_ARCHITECTURE_OVERVIEW.md
├── SPEECH_TO_SPEECH_TRANSLATION_STATUS.md
└── ...
```

---

**最后更新**: 2025-11-21

