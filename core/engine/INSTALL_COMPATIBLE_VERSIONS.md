# 在 WSL 环境中安装兼容版本

## 快速安装命令

在 WSL 终端中运行以下命令：

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall --no-cache-dir
```

## 如果方案 1 失败，使用方案 2

```bash
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall --no-cache-dir
```

## 验证安装

```bash
python -c "import numpy; import numba; import librosa; import numpy as np; test_audio = np.random.randn(1000).astype(np.float64); librosa.effects.time_stretch(test_audio, rate=1.0); print('✅ 测试通过: numpy', numpy.__version__, 'numba', numba.__version__, 'librosa', librosa.__version__)"
```

