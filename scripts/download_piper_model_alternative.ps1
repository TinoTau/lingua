# Alternative method to download Piper Chinese model
# Try different model paths that might exist

$ErrorActionPreference = "Stop"

Write-Host "=== Alternative Piper Model Download ===" -ForegroundColor Cyan
Write-Host ""

# Create model directory
$modelDir = "third_party\piper\models\zh"
New-Item -ItemType Directory -Path $modelDir -Force | Out-Null

Write-Host "Trying alternative model paths..." -ForegroundColor Yellow
Write-Host ""

# Try different possible paths
$modelPaths = @(
    "zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx",
    "zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx.json",
    "zh_CN-xiaoyan-medium.onnx",
    "zh_CN-xiaoyan-medium.onnx.json"
)

# First, let's try to find what's actually available
Write-Host "Step 1: Listing available files in the repository..." -ForegroundColor Cyan
Write-Host ""

$listScript = @"
from huggingface_hub import list_repo_files
import sys

try:
    repo_id = "rhasspy/piper-voices"
    files = list_repo_files(repo_id, repo_type="dataset")
    
    # Find Chinese models
    chinese_files = [f for f in files if "zh" in f.lower() and ("xiaoyan" in f.lower() or "zh_cn" in f.lower())]
    
    if chinese_files:
        print("Found Chinese model files:")
        for f in sorted(chinese_files):
            print(f"  {f}")
    else:
        print("No Chinese models found. Showing all files with 'zh':")
        zh_files = [f for f in files if "zh" in f.lower()]
        for f in sorted(zh_files)[:30]:
            print(f"  {f}")
except Exception as e:
    print(f"Error listing files: {e}")
    print("\nTrying direct download of common model names...")
    sys.exit(1)
"@

try {
    $listScript | python
    Write-Host ""
} catch {
    Write-Host "Could not list files. Trying direct download..." -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "Step 2: Try downloading from common paths..." -ForegroundColor Cyan
Write-Host ""

# Try using Python hf_hub_download with different paths
$downloadScript = @"
from huggingface_hub import hf_hub_download
import os
import sys

repo_id = "rhasspy/piper-voices"
model_dir = r"third_party/piper/models/zh"
os.makedirs(model_dir, exist_ok=True)

# Try different possible paths
possible_paths = [
    ("zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx", "zh_CN-xiaoyan-medium.onnx"),
    ("zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx", "zh_CN-xiaoyan-medium.onnx"),
    ("zh_CN-xiaoyan-medium.onnx", "zh_CN-xiaoyan-medium.onnx"),
]

config_paths = [
    ("zh/zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx.json", "zh_CN-xiaoyan-medium.onnx.json"),
    ("zh_CN/xiaoyan/medium/zh_CN-xiaoyan-medium.onnx.json", "zh_CN-xiaoyan-medium.onnx.json"),
    ("zh_CN-xiaoyan-medium.onnx.json", "zh_CN-xiaoyan-medium.onnx.json"),
]

success = False

# Try downloading model
for repo_path, local_name in possible_paths:
    try:
        print(f"Trying: {repo_path}")
        downloaded = hf_hub_download(
            repo_id=repo_id,
            filename=repo_path,
            local_dir=model_dir,
            local_dir_use_symlinks=False
        )
        print(f"✅ Successfully downloaded model: {downloaded}")
        success = True
        break
    except Exception as e:
        print(f"  ❌ Failed: {e}")
        continue

if not success:
    print("\n❌ Could not download model file.")
    print("\nPlease check the repository manually:")
    print("  https://huggingface.co/rhasspy/piper-voices")
    sys.exit(1)

# Try downloading config
success_config = False
for repo_path, local_name in config_paths:
    try:
        print(f"Trying config: {repo_path}")
        downloaded = hf_hub_download(
            repo_id=repo_id,
            filename=repo_path,
            local_dir=model_dir,
            local_dir_use_symlinks=False
        )
        print(f"✅ Successfully downloaded config: {downloaded}")
        success_config = True
        break
    except Exception as e:
        print(f"  ❌ Failed: {e}")
        continue

if not success_config:
    print("\n⚠️  Could not download config file. Model might still work.")
"@

try {
    $downloadScript | python
    Write-Host ""
    
    # Check if files were downloaded
    $modelFile = Join-Path $modelDir "zh_CN-xiaoyan-medium.onnx"
    if (Test-Path $modelFile) {
        Write-Host "✅ Model file found: $modelFile" -ForegroundColor Green
        $size = (Get-Item $modelFile).Length / 1MB
        Write-Host "   Size: $([math]::Round($size, 2)) MB" -ForegroundColor Green
    } else {
        Write-Host "❌ Model file not found. Please check the repository manually." -ForegroundColor Red
        Write-Host "   https://huggingface.co/rhasspy/piper-voices" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ Download failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please try manual download from:" -ForegroundColor Yellow
    Write-Host "  https://huggingface.co/rhasspy/piper-voices" -ForegroundColor Yellow
}

