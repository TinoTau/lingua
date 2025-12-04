#!/bin/bash
# Silero VAD 模型下载脚本（从 GitHub）
# 
# 从 GitHub 仓库下载最新的 Silero VAD ONNX 模型

echo "============================================================"
echo "  Silero VAD 模型下载（GitHub）"
echo "============================================================"
echo ""

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENGINE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MODEL_DIR="$ENGINE_DIR/models/vad/silero"

# 创建模型目录（如果不存在）
mkdir -p "$MODEL_DIR"

MODEL_PATH="$MODEL_DIR/silero_vad_github.onnx"
BACKUP_PATH="$MODEL_DIR/silero_vad.onnx.backup"

# 备份现有模型（如果存在）
if [ -f "$MODEL_DIR/silero_vad.onnx" ]; then
    echo "[备份] 备份现有模型..."
    cp "$MODEL_DIR/silero_vad.onnx" "$BACKUP_PATH"
    echo "[备份] 已备份到: $BACKUP_PATH"
fi

# 检查模型是否已存在
if [ -f "$MODEL_PATH" ]; then
    echo "[警告] 模型文件已存在: $MODEL_PATH"
    read -p "是否覆盖? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "[取消] 下载已取消"
        exit 0
    fi
fi

echo "[下载] Silero VAD ONNX 模型（从 GitHub）..."
echo "  模型路径: $MODEL_PATH"
echo ""

# GitHub 原始文件链接（多个备用地址）
DOWNLOAD_URLS=(
    "https://github.com/snakers4/silero-vad/raw/master/src/silero_vad/data/silero_vad.onnx"
    "https://raw.githubusercontent.com/snakers4/silero-vad/master/src/silero_vad/data/silero_vad.onnx"
    "https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx"
    "https://models.silero.ai/vad_models/silero_vad.onnx"
)

DOWNLOAD_SUCCESS=false

for url in "${DOWNLOAD_URLS[@]}"; do
    echo "[尝试] 从以下地址下载:"
    echo "  $url"
    echo ""
    
    if command -v wget &> /dev/null; then
        if wget --user-agent="Mozilla/5.0" -O "$MODEL_PATH" "$url" 2>/dev/null; then
            DOWNLOAD_SUCCESS=true
        fi
    elif command -v curl &> /dev/null; then
        if curl -L -A "Mozilla/5.0" -o "$MODEL_PATH" "$url" 2>/dev/null; then
            DOWNLOAD_SUCCESS=true
        fi
    else
        echo "[错误] 未找到 wget 或 curl，请先安装其中一个"
        exit 1
    fi
    
    if [ "$DOWNLOAD_SUCCESS" = true ]; then
        # 验证文件大小
        FILE_SIZE=$(stat -f%z "$MODEL_PATH" 2>/dev/null || stat -c%s "$MODEL_PATH" 2>/dev/null)
        FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1024 / 1024" | bc)
        
        echo "[成功] 模型下载完成!"
        echo "[验证] 文件大小: ${FILE_SIZE_MB} MB"
        
        # 检查文件大小是否合理（应该在 1-10 MB 之间）
        if (( $(echo "$FILE_SIZE_MB < 1" | bc -l) )); then
            echo "[警告] 文件大小异常小，可能下载失败"
            rm -f "$MODEL_PATH"
            DOWNLOAD_SUCCESS=false
            continue
        fi
        
        if (( $(echo "$FILE_SIZE_MB > 10" | bc -l) )); then
            echo "[警告] 文件大小异常大，可能下载了错误文件"
            rm -f "$MODEL_PATH"
            DOWNLOAD_SUCCESS=false
            continue
        fi
        
        break
    else
        echo "[失败] 下载失败"
        echo ""
        rm -f "$MODEL_PATH"
    fi
done

if [ "$DOWNLOAD_SUCCESS" = false ]; then
    echo ""
    echo "[错误] 所有下载源都失败了"
    echo ""
    echo "请尝试手动下载:"
    echo "  1. 访问: https://github.com/snakers4/silero-vad"
    echo "  2. 进入: src/silero_vad/data/"
    echo "  3. 下载: silero_vad.onnx"
    echo "  4. 保存到: $MODEL_PATH"
    echo ""
    echo "或者使用 Git LFS:"
    echo "  git clone https://github.com/snakers4/silero-vad.git"
    echo "  cd silero-vad"
    echo "  git lfs pull"
    echo "  cp src/silero_vad/data/silero_vad.onnx $MODEL_PATH"
    echo ""
    exit 1
fi

echo ""
echo "============================================================"
echo "  下载完成！"
echo "============================================================"
echo ""
echo "模型文件位置: $MODEL_PATH"
echo ""
echo "提示: 如果这是新下载的模型，请更新配置文件中的模型路径"
echo "  或者将文件重命名为: silero_vad.onnx"
echo ""

