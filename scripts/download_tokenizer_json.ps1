# 下载 M2M100 tokenizer.json 文件

param(
    [string]$ModelDir = "core/engine/models/nmt/m2m100-en-zh"
)

Write-Host "=== 下载 M2M100 tokenizer.json ===" -ForegroundColor Green
Write-Host ""

$ErrorActionPreference = "Stop"

# 检查 huggingface-cli
$hfCli = Get-Command huggingface-cli -ErrorAction SilentlyContinue
if (-not $hfCli) {
    Write-Host "[ERROR] huggingface-cli 未找到" -ForegroundColor Red
    Write-Host "请先安装: pip install huggingface_hub" -ForegroundColor Yellow
    exit 1
}

Write-Host "[INFO] 下载 tokenizer.json 到: $ModelDir" -ForegroundColor Cyan

# 下载 tokenizer.json
huggingface-cli download facebook/m2m100_418M `
    tokenizer.json `
    --local-dir $ModelDir

if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] tokenizer.json 下载成功" -ForegroundColor Green
    
    # 验证文件
    $tokenizerPath = Join-Path $ModelDir "tokenizer.json"
    if (Test-Path $tokenizerPath) {
        $size = (Get-Item $tokenizerPath).Length / 1MB
        Write-Host "[INFO] 文件大小: $([math]::Round($size, 2)) MB" -ForegroundColor Gray
    }
} else {
    Write-Host "[ERROR] 下载失败" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ 完成！" -ForegroundColor Green

