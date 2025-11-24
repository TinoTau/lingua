#!/bin/bash
# M2M100 实时翻译系统 - 一键启动脚本（Linux/WSL）
# 
# 用途：启动所有必需的服务（NMT、TTS）并检查健康状态
# 
# 使用方法：
#   bash scripts/start_all.sh

set -e

echo "=== M2M100 实时翻译系统 - 一键启动 ==="
echo ""

# 配置
NMT_SERVICE_URL="http://127.0.0.1:5008"
TTS_SERVICE_URL="http://127.0.0.1:5005"
NMT_SERVICE_DIR="services/nmt_m2m100"
TTS_SERVICE_SCRIPT="scripts/wsl2_piper/start_piper_service.sh"

# 检查服务是否已运行
check_service_health() {
    local url=$1
    local service_name=$2
    
    if curl -s -f --max-time 2 "$url/health" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# 启动 NMT 服务
echo "[1/3] 检查 NMT 服务..."
if check_service_health "$NMT_SERVICE_URL" "NMT"; then
    echo "  ✅ NMT 服务已在运行"
else
    echo "  ⚠️  NMT 服务未运行，正在启动..."
    echo "  请在另一个终端运行以下命令启动 NMT 服务："
    echo "    cd $NMT_SERVICE_DIR"
    echo "    uvicorn nmt_service:app --host 127.0.0.1 --port 5008"
    echo ""
    read -p "  按 Enter 继续（稍后手动启动）..."
fi

# 启动 TTS 服务
echo ""
echo "[2/3] 检查 TTS 服务..."
if check_service_health "$TTS_SERVICE_URL" "TTS"; then
    echo "  ✅ TTS 服务已在运行"
else
    echo "  ⚠️  TTS 服务未运行，正在启动..."
    echo "  请运行以下命令启动 TTS 服务："
    echo "    bash $TTS_SERVICE_SCRIPT"
    echo ""
    read -p "  按 Enter 继续（稍后手动启动）..."
fi

# 最终健康检查
echo ""
echo "[3/3] 最终健康检查..."
NMT_OK=false
TTS_OK=false

if check_service_health "$NMT_SERVICE_URL" "NMT"; then
    NMT_OK=true
fi

if check_service_health "$TTS_SERVICE_URL" "TTS"; then
    TTS_OK=true
fi

if [ "$NMT_OK" = true ] && [ "$TTS_OK" = true ]; then
    echo "  ✅ 所有服务运行正常！"
    echo ""
    echo "服务状态："
    echo "  NMT: $NMT_SERVICE_URL - ✅ 正常"
    echo "  TTS: $TTS_SERVICE_URL - ✅ 正常"
    echo ""
    echo "现在可以运行集成测试或启动主程序了！"
else
    echo "  ⚠️  部分服务未就绪："
    if [ "$NMT_OK" = false ]; then
        echo "    NMT: ❌ 未运行"
    else
        echo "    NMT: ✅ 正常"
    fi
    if [ "$TTS_OK" = false ]; then
        echo "    TTS: ❌ 未运行"
    else
        echo "    TTS: ✅ 正常"
    fi
    echo ""
    echo "请确保所有服务都已启动后再运行测试。"
fi

echo ""

