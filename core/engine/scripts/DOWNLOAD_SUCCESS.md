# ✅ Silero VAD 模型下载成功

模型文件已成功下载到：
```
D:\Programs\github\lingua\core\engine\models\vad\silero\silero_vad_official.onnx
```

**文件信息**：
- 文件大小：2.22 MB
- 下载地址：https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx

## 下一步操作

### 1. 更新配置文件

请更新 `lingua_core_config.toml` 中的模型路径：

```toml
[vad]
type = "silero"
model_path = "models/vad/silero/silero_vad_official.onnx"  # 更新为新的模型文件名
```

### 2. 测试模型

运行 SileroVad 测试：

```powershell
cd core\engine
cargo run --example test_silero_vad_startup
```

### 3. 如果遇到问题

如果模型文件无法加载，请检查：
- 文件路径是否正确
- 文件是否完整（2.22 MB）
- ONNX Runtime 版本是否兼容（需要 1.16.3 或更高版本）

## 下载脚本

以后如果需要重新下载，可以使用：

```powershell
# PowerShell
.\core\engine\scripts\download_silero_vad_official.ps1

# Python
python core\engine\scripts\download_silero_vad_official.py

# WSL (Bash)
bash core/engine/scripts/download_silero_vad_official.sh
```

