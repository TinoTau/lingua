#!/bin/bash
# Silero VAD 模型下载脚本 (Linux/WSL)
# 
# 下载 IR version 9 的 Silero VAD ONNX 模型（兼容 ONNX Runtime 1.16.3）

echo "============================================================"
echo "  Silero VAD 模型下载"
echo "============================================================"
echo ""

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENGINE_DIR="$(dirname "$SCRIPT_DIR")"
MODEL_DIR="$ENGINE_DIR/models/vad/silero"

# 创建模型目录（如果不存在）
mkdir -p "$MODEL_DIR"

MODEL_PATH="$MODEL_DIR/silero_vad.onnx"

# 检查模型是否已存在
if [ -f "$MODEL_PATH" ]; then
    echo "[警告] 模型文件已存在: $MODEL_PATH"
    read -p "是否覆盖? (y/N): " overwrite
    if [ "$overwrite" != "y" ] && [ "$overwrite" != "Y" ]; then
        echo "[取消] 下载已取消"
        exit 0
    fi
fi

echo "[下载] Silero VAD ONNX 模型..."
echo "  模型路径: $MODEL_PATH"
echo ""

# Silero VAD 模型下载 URL
MODEL_URL="https://github.com/snakers4/silero-vad/raw/master/src/silero_vad/data/silero_vad.onnx"

# 备用下载地址
MODEL_URL_ALT="https://models.silero.ai/vad_models/silero_vad.onnx"

echo "[信息] 下载地址: $MODEL_URL"
echo ""

# 尝试使用 wget 下载
if command -v wget &> /dev/null; then
    echo "[下载] 使用 wget 下载模型文件..."
    if wget -O "$MODEL_PATH" "$MODEL_URL"; then
        echo "[成功] 模型下载完成!"
    else
        echo "[错误] wget 下载失败，尝试备用地址..."
        wget -O "$MODEL_PATH" "$MODEL_URL_ALT" || {
            echo "[错误] 下载失败"
            echo ""
            echo "请尝试手动下载:"
            echo "  1. 访问: https://github.com/snakers4/silero-vad"
            echo "  2. 下载 ONNX 模型文件"
            echo "  3. 保存到: $MODEL_PATH"
            exit 1
        }
    fi
# 尝试使用 curl 下载
elif command -v curl &> /dev/null; then
    echo "[下载] 使用 curl 下载模型文件..."
    if curl -L -o "$MODEL_PATH" "$MODEL_URL"; then
        echo "[成功] 模型下载完成!"
    else
        echo "[错误] curl 下载失败，尝试备用地址..."
        curl -L -o "$MODEL_PATH" "$MODEL_URL_ALT" || {
            echo "[错误] 下载失败"
            echo ""
            echo "请尝试手动下载:"
            echo "  1. 访问: https://github.com/snakers4/silero-vad"
            echo "  2. 下载 ONNX 模型文件"
            echo "  3. 保存到: $MODEL_PATH"
            exit 1
        }
    fi
else
    echo "[错误] 未找到 wget 或 curl，请先安装其中一个工具"
    exit 1
fi

# 验证文件大小
FILE_SIZE=$(stat -f%z "$MODEL_PATH" 2>/dev/null || stat -c%s "$MODEL_PATH" 2>/dev/null)
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1024 / 1024" | bc)

echo ""
echo "[验证] 文件大小: ${FILE_SIZE_MB} MB"

if (( $(echo "$FILE_SIZE_MB < 1" | bc -l) )); then
    echo "[警告] 文件大小异常小，可能下载失败"
    echo "       请检查文件内容或手动下载"
else
    echo "[成功] 模型文件验证通过"
fi

echo ""
echo "============================================================"
echo "  下载完成！"
echo "============================================================"
echo ""
echo "模型文件位置: $MODEL_PATH"
echo ""
echo "现在可以运行测试:"
echo "  cargo run --example test_silero_vad_startup"
echo ""

