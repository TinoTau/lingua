#!/bin/bash
# 启动本地 Web 服务器（Linux/macOS）

echo "启动 Lingua Web PWA 本地服务器..."
echo ""

# 检查 Python 是否可用
if command -v python3 &> /dev/null; then
    echo "使用 Python 启动服务器..."
    echo "访问地址: http://localhost:8080"
    echo "按 Ctrl+C 停止服务器"
    echo ""
    
    python3 -m http.server 8080
elif command -v python &> /dev/null; then
    echo "使用 Python 启动服务器..."
    echo "访问地址: http://localhost:8080"
    echo "按 Ctrl+C 停止服务器"
    echo ""
    
    python -m http.server 8080
else
    echo "未找到 Python，请手动启动服务器："
    echo "  python3 -m http.server 8080"
    echo ""
    echo "或使用其他方式："
    echo "  npx http-server -p 8080"
    echo "  php -S localhost:8080"
fi

