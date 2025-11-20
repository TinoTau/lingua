#!/bin/bash
# WSL2 内 Piper 安装脚本
# 用途：在 WSL2 Ubuntu 中安装 Python、虚拟环境和 Piper-tts

set -e

echo "=== WSL2 内 Piper 安装 ==="
echo ""

# 检查是否为 root（不应以 root 运行）
if [ "$EUID" -eq 0 ]; then
    echo "[ERROR] 请不要以 root 用户运行此脚本。请使用普通用户。" >&2
    exit 1
fi

# 步骤 1: 更新系统并安装依赖
echo "[1/4] 更新系统并安装依赖..."
sudo apt update && sudo apt upgrade -y
sudo apt install -y python3 python3-venv python3-pip git curl wget

# 步骤 2: 创建虚拟环境目录
echo ""
echo "[2/4] 创建虚拟环境..."
mkdir -p ~/piper_env
cd ~/piper_env

if [ -d ".venv" ]; then
    echo "[INFO] 虚拟环境已存在，跳过创建"
else
    python3 -m venv .venv
    echo "[OK] 虚拟环境创建完成"
fi

# 步骤 3: 激活虚拟环境并安装 Piper
echo ""
echo "[3/4] 安装 Piper-tts (带 HTTP 支持)..."
source .venv/bin/activate
pip install --upgrade pip
pip install "piper-tts[http]"

# 步骤 4: 验证安装
echo ""
echo "[4/4] 验证安装..."
if command -v piper &> /dev/null || python -m piper_tts --help &> /dev/null; then
    echo "[OK] Piper-tts 安装成功"
else
    echo "[WARN] 无法直接验证 piper 命令，但 pip 安装已完成"
fi

echo ""
echo "=== Piper 安装完成 ==="
echo ""
echo "虚拟环境位置: ~/piper_env"
echo ""
echo "下次使用时，执行以下命令激活环境："
echo "  cd ~/piper_env"
echo "  source .venv/bin/activate"
echo ""
echo "下一步："
echo "  1. 运行 scripts/wsl2_piper/download_piper_model.sh 下载中文模型"
echo "  2. 或手动下载模型到 ~/piper_models/zh/"
echo ""

