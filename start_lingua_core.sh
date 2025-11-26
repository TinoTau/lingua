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
NMT_SERVICE_URL="http://127.0.0.1:5008"
TTS_SERVICE_URL="http://127.0.0.1:5005"
CORE_ENGINE_PORT=9000
CONFIG_FILE="lingua_core_config.toml"

# 进程跟踪
PIDS=()

# 清理函数
cleanup() {
    echo ""
    echo "=== 正在停止所有服务 ==="
    
    # 停止所有启动的进程
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "  停止进程 PID: $pid"
            kill "$pid" 2>/dev/null || true
        fi
    done
    
    # 等待进程退出
    sleep 2
    
    # 强制杀死仍在运行的进程
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "  强制停止进程 PID: $pid"
            kill -9 "$pid" 2>/dev/null || true
        fi
    done
    
    # 按端口清理
    echo "  清理端口上的进程..."
    for port in 5005 5008 9000; do
        lsof -ti:$port | xargs kill -9 2>/dev/null || true
    done
    
    echo "  所有服务已停止"
    exit 0
}

# 注册信号处理
trap cleanup SIGINT SIGTERM EXIT

# 检查配置文件
if [ ! -f "$CONFIG_FILE" ]; then
    echo "⚠️  配置文件不存在: $CONFIG_FILE"
    echo "请确保配置文件存在后再运行此脚本。"
    exit 1
fi

# 启动 Piper TTS 服务
echo "[1/3] 启动 Piper TTS 服务..."
if command -v piper &> /dev/null; then
    nohup piper --server --port 5005 > /dev/null 2>&1 &
    TTS_PID=$!
    PIDS+=($TTS_PID)
    echo "  ✅ Piper TTS 服务已启动（端口 5005, PID: $TTS_PID）"
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
    nohup uvicorn nmt_service:app --host 127.0.0.1 --port 5008 > /dev/null 2>&1 &
    NMT_PID=$!
    PIDS+=($NMT_PID)
    echo "  ✅ Python NMT 服务已启动（端口 5008, PID: $NMT_PID）"
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
CORE_PID=$!
PIDS+=($CORE_PID)
echo "  ✅ CoreEngine 已启动（端口 $CORE_ENGINE_PORT, PID: $CORE_PID）"

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
echo "按 Ctrl+C 停止所有服务..."
echo ""

# 等待主进程（CoreEngine）退出
wait $CORE_PID 2>/dev/null || true

