#!/bin/bash
# 检查虚拟环境的 Python 版本

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  检查 Python 版本"
echo "============================================================"
echo ""

if [ -d "venv-wsl" ]; then
    echo "1. 虚拟环境路径: venv-wsl"
    echo ""
    
    # 检查虚拟环境中的 Python 可执行文件
    if [ -f "venv-wsl/bin/python" ]; then
        echo "2. 虚拟环境中的 Python 版本："
        venv-wsl/bin/python --version
        echo ""
        
        echo "3. Python 完整路径："
        which python 2>/dev/null || echo "venv-wsl/bin/python"
        echo ""
        
        echo "4. 激活虚拟环境后检查："
        source venv-wsl/bin/activate
        python --version
        echo ""
        
        # 检查 site-packages 路径中的版本号
        if [ -d "venv-wsl/lib" ]; then
            echo "5. site-packages 目录中的 Python 版本："
            ls -d venv-wsl/lib/python* 2>/dev/null | head -1
            echo ""
        fi
        
        echo "6. 系统 Python 版本（如果可用）："
        if command -v python3 &> /dev/null; then
            python3 --version
        fi
        if command -v python3.10 &> /dev/null; then
            echo "Python 3.10 可用: $(python3.10 --version 2>&1)"
        fi
        if command -v python3.12 &> /dev/null; then
            echo "Python 3.12 可用: $(python3.12 --version 2>&1)"
        fi
    else
        echo "❌ venv-wsl/bin/python 不存在"
    fi
else
    echo "❌ venv-wsl 目录不存在"
fi

