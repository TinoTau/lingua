# 检查虚拟环境 Python 版本

## 快速检查命令

在 WSL 中运行：

```bash
cd /mnt/d/Programs/github/lingua

# 方法 1: 直接检查虚拟环境中的 Python
venv-wsl/bin/python --version

# 方法 2: 激活后检查
source venv-wsl/bin/activate
python --version

# 方法 3: 使用检查脚本
bash core/engine/scripts/check_python_version.sh
```

## 说明

从错误信息中看到的路径：
```
/mnt/d/Programs/github/lingua/venv-wsl/lib/python3.12/site-packages
```

这表明虚拟环境可能是用 Python 3.12 创建的，但需要确认：
1. 虚拟环境实际使用的 Python 版本
2. 系统中有哪些 Python 版本可用

## 如果是 Python 3.12

有两个选择：

### 选择 1: 使用兼容 Python 3.12 的版本
- numpy 1.26.4（支持 Python 3.12）
- 可能仍有 numba 兼容性问题，但代码已有 fallback

### 选择 2: 重新创建 Python 3.10 虚拟环境
```bash
cd /mnt/d/Programs/github/lingua
rm -rf venv-wsl  # 备份重要数据后删除
python3.10 -m venv venv-wsl
source venv-wsl/bin/activate
pip install --upgrade pip
# 然后安装依赖
```

