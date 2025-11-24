#!/bin/bash
# Lingua Core Runtime - 一键启动脚本（Linux/macOS）
# 
# 用途：启动所有必需的服务（NMT、TTS、CoreEngine）
# 
# 使用方法：
#   bash start_lingua_core.sh

set -e

echo "=== Lingua Core Runtime - 一键启动 ==="
echo ""

# 配置
NMT_SERVICE_URL="http://127.0.0.1:9001"
TTS_SERVICE_URL="http://127.0.0.1:9002"
CORE_ENGINE_PORT=9000
CONFIG_FILE="lingua_core_config.toml"

# 检查配置文件
if [ ! -f "$CONFIG_FILE" ]; then
    echo "⚠️  配置文件不存在: $CONFIG_FILE"
    echo "请确保配置文件存在后再运行此脚本。"
    exit 1
fi

# 启动 Piper TTS 服务
echo "[1/3] 启动 Piper TTS 服务..."
if command -v piper &> /dev/null; then
    nohup piper --server --port 9002 > /dev/null 2>&1 &
    echo "  ✅ Piper TTS 服务已启动（端口 9002）"
else
    echo "  ⚠️  piper 命令未找到，请手动启动 TTS 服务"
fi

# 启动 Python NMT 服务
echo ""
echo "[2/3] 启动 Python NMT 服务..."
if command -v python3 &> /dev/null; then
    # 检查虚拟环境
    if [ ! -d "venv" ]; then
        echo "  创建虚拟环境..."
        python3 -m venv venv
    fi
    
    # 激活虚拟环境并启动服务
    source venv/bin/activate
    nohup uvicorn nmt_service:app --host 127.0.0.1 --port 9001 > /dev/null 2>&1 &
    echo "  ✅ Python NMT 服务已启动（端口 9001）"
else
    echo "  ⚠️  python3 未找到，请手动启动 NMT 服务"
fi

# 等待服务启动
echo ""
echo "等待服务启动..."
sleep 3

# 启动 CoreEngine
echo ""
echo "[3/3] 启动 CoreEngine..."
if [ ! -f "target/release/core_engine" ]; then
    echo "  构建 CoreEngine..."
    cargo build --release --bin core_engine
fi

nohup ./target/release/core_engine --config "$CONFIG_FILE" > /dev/null 2>&1 &
echo "  ✅ CoreEngine 已启动（端口 $CORE_ENGINE_PORT）"

echo ""
echo "=== 启动完成 ==="
echo ""
echo "服务状态："
echo "  NMT: $NMT_SERVICE_URL - 检查 /health"
echo "  TTS: $TTS_SERVICE_URL - 检查 /health"
echo "  CoreEngine: http://127.0.0.1:$CORE_ENGINE_PORT - 检查 /health"
echo ""
echo "API 端点："
echo "  POST http://127.0.0.1:$CORE_ENGINE_PORT/s2s - 整句翻译"
echo "  WS   ws://127.0.0.1:$CORE_ENGINE_PORT/stream - 流式翻译"
echo "  GET  http://127.0.0.1:$CORE_ENGINE_PORT/health - 健康检查"
echo ""

