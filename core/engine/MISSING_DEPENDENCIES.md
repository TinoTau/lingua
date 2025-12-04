# 缺失依赖补充说明

## 已安装的依赖

### 核心依赖
- numpy
- soundfile
- flask
- torch (2.5.1+cu121)
- torchaudio
- speechbrain

### 额外依赖（自动发现）
- requests（SpeechBrain 的依赖）

## 如果遇到其他缺失依赖

如果启动服务时遇到 `ModuleNotFoundError`，可以：

### 方法 1：安装缺失的模块

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install <模块名>
```

### 方法 2：安装 SpeechBrain 的所有依赖

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install speechbrain[all]
```

### 方法 3：查看 SpeechBrain 的依赖

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip show speechbrain
```

## 常见缺失依赖

- `requests` - HTTP 请求库（已安装）
- `huggingface_hub` - Hugging Face 模型下载
- `scipy` - 科学计算库
- `librosa` - 音频处理库（可选）

如果遇到其他缺失依赖，按提示安装即可。

