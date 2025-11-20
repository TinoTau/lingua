# 安装 onnxruntime 的简单方法

## Windows (PowerShell)

### 方法 1: 使用脚本（推荐）

```powershell
# 在项目根目录执行
.\scripts\install_onnxruntime.ps1
```

### 方法 2: 手动执行（如果脚本有问题）

```powershell
# 激活虚拟环境（如果使用）
# conda activate emotion_ir9_py310

# 安装 onnxruntime
python -m pip install --upgrade pip
python -m pip install onnxruntime

# 验证安装
python -c "import onnxruntime; print(f'onnxruntime version: {onnxruntime.__version__}')"
```

## Linux/Mac (Bash)

### 方法 1: 使用脚本

```bash
bash scripts/install_onnxruntime.sh
```

### 方法 2: 手动执行

```bash
# 激活虚拟环境（如果使用）
# source venv/bin/activate

# 安装 onnxruntime
python3 -m pip install --upgrade pip
python3 -m pip install onnxruntime

# 验证安装
python3 -c "import onnxruntime; print(f'onnxruntime version: {onnxruntime.__version__}')"
```

## 注意事项

1. **如果使用虚拟环境**，请先激活虚拟环境再安装
2. **安装时间**：onnxruntime 比较大（约 100MB），可能需要几分钟
3. **网络问题**：如果下载慢，可以使用国内镜像：
   ```bash
   pip install onnxruntime -i https://pypi.tuna.tsinghua.edu.cn/simple
   ```

## 安装后测试

安装完成后，可以运行测试脚本：

```powershell
# Windows
python scripts/test_hifigan_model.py

# Linux/Mac
python3 scripts/test_hifigan_model.py
```

