#!/bin/bash
# 将本地 YourTTS 模型链接到 TTS 库的缓存目录

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  链接本地 YourTTS 模型到 TTS 缓存目录"
echo "============================================================"
echo ""

# 本地模型路径
LOCAL_MODEL_DIR="/mnt/d/Programs/github/lingua/core/engine/models/tts/your_tts"
CACHE_DIR="$HOME/.local/share/tts"
CACHE_MODEL_DIR="$CACHE_DIR/tts_models--multilingual--multi-dataset--your_tts"

# 检查本地模型是否存在
if [ ! -f "$LOCAL_MODEL_DIR/model.pth" ]; then
    echo "❌ 错误: 本地模型文件不存在: $LOCAL_MODEL_DIR/model.pth"
    exit 1
fi

echo "✅ 本地模型文件存在: $LOCAL_MODEL_DIR/model.pth"
echo ""

# 创建缓存目录
mkdir -p "$CACHE_DIR"

# 如果缓存目录已存在（可能正在下载中），询问是否覆盖
if [ -d "$CACHE_MODEL_DIR" ]; then
    echo "⚠️  缓存目录已存在: $CACHE_MODEL_DIR"
    echo "   这可能是正在下载中的模型"
    echo ""
    read -p "是否备份现有目录并创建链接？(y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 备份现有目录
        BACKUP_DIR="${CACHE_MODEL_DIR}_backup_$(date +%Y%m%d_%H%M%S)"
        echo "备份现有目录到: $BACKUP_DIR"
        mv "$CACHE_MODEL_DIR" "$BACKUP_DIR"
        
        # 创建符号链接
        echo "创建符号链接..."
        ln -s "$LOCAL_MODEL_DIR" "$CACHE_MODEL_DIR"
        echo "✅ 符号链接已创建"
    else
        echo "取消操作"
        exit 0
    fi
else
    # 直接创建符号链接
    echo "创建符号链接: $CACHE_MODEL_DIR -> $LOCAL_MODEL_DIR"
    ln -s "$LOCAL_MODEL_DIR" "$CACHE_MODEL_DIR"
    echo "✅ 符号链接已创建"
fi

echo ""
echo "============================================================"
echo "  ✅ 完成！"
echo "============================================================"
echo ""
echo "现在 TTS 库会直接使用本地模型，无需重新下载。"
echo ""
echo "验证链接："
ls -la "$CACHE_MODEL_DIR" | head -5

