# TTS 和 YourTTS 服务依赖列表

## 核心依赖（已安装）

这些依赖已在 `setup_python310_env.sh` 中安装：

- ✅ `numpy==1.24.3` - 数值计算
- ✅ `numba==0.59.1` - JIT 编译（librosa 需要）
- ✅ `librosa==0.10.1` - 音频处理（时间拉伸）
- ✅ `llvmlite==0.42.0` - numba 的依赖
- ✅ `scipy` - 科学计算（librosa 的依赖）

## YourTTS 服务依赖

### 必需依赖

- ✅ `flask` - Web 框架
- ✅ `flask-cors` - CORS 支持（可选但推荐）
- ✅ `torch` - PyTorch 深度学习框架
- ✅ `torchaudio` - PyTorch 音频处理
- ✅ `soundfile` - 音频文件读写
- ✅ `TTS` - Coqui TTS 库（YourTTS 模型）

### 可选依赖

- `requests` - HTTP 请求（TTS 库可能需要）
- `pillow` - 图像处理（某些功能可能需要）

## Speaker Embedding 服务依赖

### 必需依赖

- ✅ `flask` - Web 框架
- ✅ `torch` - PyTorch 深度学习框架
- ✅ `torchaudio` - PyTorch 音频处理
- ✅ `speechbrain` - SpeechBrain 库（ECAPA-TDNN 模型）

### SpeechBrain 自动安装的依赖

- `huggingface_hub` - 模型下载
- `hyperpyyaml` - 配置文件解析
- `pesq` - 音频质量评估（可选）
- `sentencepiece` - 文本处理（可选）
- `librosa` - 音频处理（已有）
- 其他依赖...

## 安装顺序

1. **核心依赖**（已安装）
   - numpy, numba, librosa, scipy

2. **PyTorch 生态**
   - torch, torchaudio, torchvision（如果需要）

3. **Web 框架**
   - flask, flask-cors

4. **音频处理**
   - soundfile

5. **TTS 库**
   - TTS (Coqui TTS)

6. **Speaker Embedding**
   - speechbrain

## 版本兼容性

- **Python**: 3.10
- **numpy**: 1.24.3（与 numba 0.59.1 兼容）
- **librosa**: 0.10.1（与 numpy 1.24.3 兼容）
- **PyTorch**: 最新稳定版（支持 Python 3.10）

## 注意事项

1. **PyTorch CUDA 版本**：
   - 如果使用 GPU，需要安装 CUDA 版本的 PyTorch
   - 脚本会自动检测 CUDA 并安装相应版本

2. **TTS 库大小**：
   - TTS 库安装可能较慢，需要下载较大文件
   - 模型会在首次使用时自动下载

3. **SpeechBrain 依赖**：
   - 会自动安装所需依赖
   - 首次加载模型时会下载模型文件

