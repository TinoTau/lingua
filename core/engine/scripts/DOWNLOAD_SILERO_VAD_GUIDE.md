# Silero VAD 模型下载指南

由于网络连接问题，提供了多种下载方法。

## 方法 1：使用浏览器直接下载（最可靠）✅

1. **打开浏览器**，访问以下链接（已验证可用）：
   ```
   https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx
   ```
   
   或者主地址（可能需要登录）：
   ```
   https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx
   ```

2. **保存文件**到以下路径：
   ```
   D:\Programs\github\lingua\core\engine\models\vad\silero\silero_vad_official.onnx
   ```

3. **验证文件**：
   - 文件大小应该约为 **1.8 MB**
   - 如果浏览器自动下载到其他位置，请手动移动到上述路径

## 方法 2：使用 Python 脚本下载

```powershell
# 确保已激活 Python 环境
python core\engine\scripts\download_silero_vad_official.py
```

或者使用 Python 直接下载：

```powershell
python -c "import urllib.request; req = urllib.request.Request('https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx'); req.add_header('User-Agent', 'Mozilla/5.0'); urllib.request.urlretrieve(req, 'core\engine\models\vad\silero\silero_vad_official.onnx')"
```

## 方法 3：在 WSL 中使用 wget（如果已安装 WSL）

```bash
# 在 WSL 中运行
cd /mnt/d/Programs/github/lingua
mkdir -p core/engine/models/vad/silero
wget --user-agent="Mozilla/5.0" https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx -O core/engine/models/vad/silero/silero_vad_official.onnx
```

## 方法 4：使用 Git LFS（如果仓库支持）

```powershell
# 如果模型文件在 Git LFS 中
git lfs pull --include="models/vad/silero/silero_vad.onnx"
```

## 方法 5：使用代理或 VPN

如果网络连接不稳定，可以：

1. **使用代理**：
   ```powershell
   $proxy = "http://your-proxy:port"
   $webClient = New-Object System.Net.WebClient
   $webClient.Proxy = New-Object System.Net.WebProxy($proxy)
   $webClient = New-Object System.Net.WebClient
   $webClient.Headers.Add("User-Agent", "Mozilla/5.0")
   $webClient.DownloadFile("https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx", "core\engine\models\vad\silero\silero_vad_official.onnx")
   ```

2. **使用 VPN**：连接到稳定的网络后再下载

## 下载后验证

下载完成后，请验证文件：

```powershell
$modelPath = "core\engine\models\vad\silero\silero_vad_official.onnx"
if (Test-Path $modelPath) {
    $fileInfo = Get-Item $modelPath
    $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "文件大小: $sizeMB MB" -ForegroundColor Green
    if ($sizeMB -gt 1.5 -and $sizeMB -lt 2.5) {
        Write-Host "✓ 文件大小正常（应该在 1.8 MB 左右）" -ForegroundColor Green
    } else {
        Write-Host "⚠ 文件大小异常，可能下载不完整" -ForegroundColor Yellow
    }
} else {
    Write-Host "✗ 文件不存在" -ForegroundColor Red
}
```

## 更新配置文件

下载完成后，请更新 `lingua_core_config.toml` 中的模型路径：

```toml
[vad]
type = "silero"
model_path = "models/vad/silero/silero_vad_official.onnx"  # 更新为新的模型文件名
```

## 备用下载地址

如果 GitHub 链接无法访问，可以尝试：

1. **Hugging Face（已验证可用，推荐）**：
   ```
   https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx
   ```

2. **Hugging Face（主地址，可能需要登录）**：
   ```
   https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx
   ```

3. **GitHub Releases**：
   - 访问：https://github.com/snakers4/silero-vad/releases
   - 下载最新版本的模型文件

## 故障排除

### 问题：连接被意外关闭

**解决方案**：
- 使用浏览器直接下载（方法 1）
- 检查网络连接
- 使用代理或 VPN
- 在 WSL 中下载

### 问题：文件下载不完整

**解决方案**：
- 重新下载
- 使用支持断点续传的工具（如 aria2c）
- 验证文件大小和 MD5 哈希值

### 问题：权限错误

**解决方案**：
- 确保有写入权限
- 以管理员身份运行 PowerShell
- 检查目录是否存在

