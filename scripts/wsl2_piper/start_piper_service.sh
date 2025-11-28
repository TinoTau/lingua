#!/bin/bash
# WSL2 内 Piper HTTP 服务启动脚本
# 用途：启动 Piper HTTP 服务，监听 0.0.0.0:5005

set -e

echo "=== 启动 Piper HTTP 服务 ==="
echo ""

# 配置
PIPER_ENV_DIR="$HOME/piper_env"
PIPER_MODEL_DIR="$HOME/piper_models"
DEFAULT_VOICE="zh_CN-huayan-medium"
HOST="0.0.0.0"
PORT="5005"

# 检查虚拟环境
if [ ! -d "$PIPER_ENV_DIR/.venv" ]; then
    echo "[ERROR] 虚拟环境不存在: $PIPER_ENV_DIR/.venv" >&2
    echo "[INFO] 请先运行 install_piper_in_wsl.sh" >&2
    exit 1
fi

# 检查模型
if [ ! -f "$PIPER_MODEL_DIR/zh/${DEFAULT_VOICE}.onnx" ]; then
    echo "[ERROR] 模型文件不存在: $PIPER_MODEL_DIR/zh/${DEFAULT_VOICE}.onnx" >&2
    echo "[INFO] 请先运行 download_piper_model.sh" >&2
    exit 1
fi

# 获取脚本所在目录（在切换目录之前）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HTTP_SERVER_SCRIPT="$SCRIPT_DIR/piper_http_server.py"

# 激活虚拟环境
cd "$PIPER_ENV_DIR"
source .venv/bin/activate

# 设置环境变量
export PIPER_MODEL_DIR="$PIPER_MODEL_DIR"
export PIPER_DEFAULT_VOICE="$DEFAULT_VOICE"

# 设置 CUDA 库路径（确保 cuDNN 和 CUDA 库可被找到）
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

# 启用 GPU（默认）
export PIPER_USE_GPU=true

echo "[INFO] GPU 加速: 已启用"
echo "[INFO] CUDA 库路径: $LD_LIBRARY_PATH"

# 检查 ONNX Runtime GPU 支持
echo "[INFO] 检查 GPU 支持..."
if python -c "import onnxruntime as ort; providers = ort.get_available_providers(); print('CUDA 可用' if 'CUDAExecutionProvider' in providers else 'CUDA 不可用')" 2>/dev/null; then
    echo "[INFO] GPU 检查完成"
else
    echo "[WARN] 无法检查 GPU 状态"
fi
echo ""

echo "[INFO] 模型目录: $PIPER_MODEL_DIR"
echo "[INFO] 默认语音: $DEFAULT_VOICE"
echo "[INFO] 监听地址: $HOST:$PORT"
echo ""

# 检查并安装 HTTP 服务依赖
echo "[INFO] 检查 HTTP 服务依赖..."
if ! python -c "import fastapi, uvicorn" &> /dev/null; then
    echo "[INFO] 安装 FastAPI 和 Uvicorn..."
    pip install fastapi uvicorn pydantic
fi

# 检查 HTTP 服务脚本是否存在
if [ ! -f "$HTTP_SERVER_SCRIPT" ]; then
    echo "[ERROR] HTTP 服务脚本不存在: $HTTP_SERVER_SCRIPT" >&2
    echo "[INFO] 请确保脚本在正确的位置" >&2
    exit 1
fi

# 启动 HTTP 服务
echo "[INFO] 启动 Piper HTTP 服务..."
echo ""
python "$HTTP_SERVER_SCRIPT" \
    --host "$HOST" \
    --port "$PORT" \
    --model-dir "$PIPER_MODEL_DIR"

