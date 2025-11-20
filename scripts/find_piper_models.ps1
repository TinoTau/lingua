# Find available Piper Chinese models
# This script helps find the correct model paths

$ErrorActionPreference = "Stop"

Write-Host "=== Finding Available Piper Chinese Models ===" -ForegroundColor Cyan
Write-Host ""

Write-Host "Checking Hugging Face repository structure..." -ForegroundColor Yellow
Write-Host ""

# Try to list available models using Python
$pythonScript = @"
from huggingface_hub import list_repo_files
import sys

try:
    repo_id = "rhasspy/piper-voices"
    files = list_repo_files(repo_id, repo_type="dataset")
    
    # Filter Chinese models
    chinese_models = [f for f in files if f.startswith("zh/") and f.endswith(".onnx")]
    
    if chinese_models:
        print("Available Chinese models:")
        for model in sorted(chinese_models):
            print(f"  {model}")
    else:
        print("No Chinese models found in expected location.")
        print("\nAll files starting with 'zh':")
        zh_files = [f for f in files if f.startswith("zh/")]
        for f in sorted(zh_files)[:20]:  # Show first 20
            print(f"  {f}")
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
"@

$pythonScript | python

Write-Host ""
Write-Host "=== Alternative: Check Piper Official Documentation ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "Visit: https://github.com/rhasspy/piper"
Write-Host "Or: https://huggingface.co/rhasspy/piper-voices"
Write-Host ""

