# Piper 中文模型手动下载命令

## 方法 1: 先查找可用模型（推荐）

### 步骤 1: 列出所有可用的中文模型

```powershell
python -c "from huggingface_hub import list_repo_files; files = list_repo_files('rhasspy/piper-voices', repo_type='dataset'); zh_files = [f for f in files if 'zh' in f.lower() and f.endswith('.onnx')]; print('\n'.join(sorted(zh_files)[:20]))"
```

这会列出所有包含 "zh" 的 .onnx 模型文件。

### 步骤 2: 根据列出的结果下载

假设找到了模型路径，例如：`zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx`

```powershell
# 下载模型文件
hf download rhasspy/piper-voices zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx --local-dir third_party/piper/models/zh

# 下载配置文件
hf download rhasspy/piper-voices zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx.json --local-dir third_party/piper/models/zh
```

---

## 方法 2: 使用 Python 脚本下载

创建一个文件 `download_piper_model.py`:

```python
from huggingface_hub import hf_hub_download
import os

# 创建目录
model_dir = "third_party/piper/models/zh"
os.makedirs(model_dir, exist_ok=True)

# 先列出可用文件
from huggingface_hub import list_repo_files
files = list_repo_files("rhasspy/piper-voices", repo_type="dataset")
zh_models = [f for f in files if "zh" in f.lower() and f.endswith(".onnx")]

print("Available Chinese models:")
for model in sorted(zh_models)[:10]:
    print(f"  {model}")

# 如果找到了模型，下载第一个
if zh_models:
    model_path = zh_models[0]
    print(f"\nDownloading: {model_path}")
    
    try:
        downloaded = hf_hub_download(
            repo_id="rhasspy/piper-voices",
            filename=model_path,
            local_dir=model_dir,
            local_dir_use_symlinks=False
        )
        print(f"✅ Downloaded to: {downloaded}")
        
        # 尝试下载对应的 json 文件
        json_path = model_path + ".json"
        if json_path in files:
            json_downloaded = hf_hub_download(
                repo_id="rhasspy/piper-voices",
                filename=json_path,
                local_dir=model_dir,
                local_dir_use_symlinks=False
            )
            print(f"✅ Downloaded config to: {json_downloaded}")
    except Exception as e:
        print(f"❌ Error: {e}")
else:
    print("No Chinese models found!")
```

运行：

```powershell
python download_piper_model.py
```

---

## 方法 3: 直接从 Hugging Face 网站下载

### 步骤 1: 访问仓库

打开浏览器访问：
```
https://huggingface.co/rhasspy/piper-voices/tree/main
```

### 步骤 2: 查找中文模型

在页面上查找包含 "zh" 或 "Chinese" 的文件夹，例如：
- `zh/` 文件夹
- `zh_CN/` 文件夹

### 步骤 3: 下载文件

找到模型文件后：
1. 点击 `.onnx` 文件
2. 点击 "Download" 按钮
3. 保存到 `third_party/piper/models/zh/` 目录

同样下载对应的 `.json` 配置文件。

---

## 方法 4: 尝试常见的模型路径

如果知道模型名称，可以直接尝试下载：

```powershell
# 尝试不同的可能路径
$paths = @(
    "zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx",
    "zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx",
    "zh/zh_CN/xiaoyan/zh_CN-xiaoyan-medium.onnx"
)

foreach ($path in $paths) {
    Write-Host "Trying: $path"
    try {
        hf download rhasspy/piper-voices $path --local-dir third_party/piper/models/zh
        Write-Host "✅ Success!"
        break
    } catch {
        Write-Host "❌ Failed: $_"
    }
}
```

---

## 方法 5: 使用 wget 或 curl（如果知道直接下载链接）

如果从网站找到了直接下载链接，可以使用：

```powershell
# 使用 Invoke-WebRequest (PowerShell)
Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/[模型路径]" -OutFile "third_party/piper/models/zh/[文件名]"

# 或使用 curl
curl -L "https://huggingface.co/rhasspy/piper-voices/resolve/main/[模型路径]" -o "third_party/piper/models/zh/[文件名]"
```

---

## 快速命令（一键查找并下载）

```powershell
# 创建目录
New-Item -ItemType Directory -Path "third_party\piper\models\zh" -Force

# 查找并下载第一个找到的中文模型
python -c "from huggingface_hub import list_repo_files, hf_hub_download; import os; files = list_repo_files('rhasspy/piper-voices', repo_type='dataset'); zh_models = [f for f in files if 'zh' in f.lower() and f.endswith('.onnx')]; print('Found models:', zh_models[:5]); model = zh_models[0] if zh_models else None; os.makedirs('third_party/piper/models/zh', exist_ok=True) if model else None; hf_hub_download('rhasspy/piper-voices', model, local_dir='third_party/piper/models/zh', local_dir_use_symlinks=False) if model else print('No models found')"
```

---

## 验证下载

下载完成后，验证文件：

```powershell
# 检查文件是否存在
Test-Path "third_party\piper\models\zh\*.onnx"

# 列出下载的文件
Get-ChildItem "third_party\piper\models\zh"

# 检查文件大小（模型文件应该 > 10MB）
Get-ChildItem "third_party\piper\models\zh\*.onnx" | Select-Object Name, @{Name="Size(MB)";Expression={[math]::Round($_.Length/1MB, 2)}}
```

---

## 如果所有方法都失败

1. **检查网络连接**：确保可以访问 Hugging Face
2. **使用 VPN**：如果 Hugging Face 被墙
3. **手动从网站下载**：访问 https://huggingface.co/rhasspy/piper-voices 手动查找和下载
4. **尝试其他模型源**：查看 Piper 官方文档是否有其他下载方式

