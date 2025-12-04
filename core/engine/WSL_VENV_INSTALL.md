# WSL 虚拟环境安装指南

## 当前情况

您在虚拟环境中（`(venv)`），虚拟环境有自己的包管理，不需要 `--user` 标志。

## 正确的安装方法

### 在虚拟环境中安装（您当前的情况）

```bash
# 确保虚拟环境已激活（您已经激活了）
# 直接安装，不需要 --user
pip install torch onnx onnxruntime

# 如果需要 TTS 库
pip install TTS
```

### 完整安装命令

```bash
# 在虚拟环境中安装所有依赖
pip install torch onnx onnxruntime TTS
```

## 为什么不需要 --user？

- **虚拟环境**：有自己的 `site-packages` 目录
- **--user**：用于系统 Python，安装到用户目录
- **冲突**：虚拟环境中不能使用 `--user`

## 验证安装

```bash
# 检查是否在虚拟环境中
which python3  # 应该显示 venv 路径

# 验证安装
python3 -c "import torch; import onnx; import onnxruntime; print('✅ 所有依赖已安装')"
```

## 运行导出脚本

安装完成后，在虚拟环境中运行：

```bash
python3 core/engine/scripts/export_yourtts_to_onnx.py
```

