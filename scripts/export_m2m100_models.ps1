# M2M100 模型导出脚本
# 使用 Python 3.10 环境导出 M2M100 模型

param(
    [string]$PythonEnv = "",  # Python 3.10 环境路径（如果为空，使用当前激活的环境）
    [string]$ModelId = "facebook/m2m100_418M",
    [string]$OutputBaseDir = "core/engine/models/nmt"
)

$ErrorActionPreference = "Stop"

Write-Host "=== M2M100 模型导出脚本 ===" -ForegroundColor Green
Write-Host ""

# 确定 Python 命令
if ($PythonEnv -ne "") {
    $PythonCmd = Join-Path $PythonEnv "python.exe"
    if (-not (Test-Path $PythonCmd)) {
        Write-Host "[ERROR] Python 环境不存在: $PythonEnv" -ForegroundColor Red
        exit 1
    }
    Write-Host "[INFO] 使用 Python 环境: $PythonEnv" -ForegroundColor Cyan
}
else {
    $PythonCmd = "python"
    Write-Host "[INFO] 使用当前激活的 Python 环境" -ForegroundColor Cyan
}

# 验证 Python 版本
Write-Host "[INFO] 检查 Python 版本..." -ForegroundColor Cyan
$PythonVersion = & $PythonCmd --version 2>&1
Write-Host "  $PythonVersion" -ForegroundColor Gray

# 验证依赖
Write-Host "[INFO] 检查依赖..." -ForegroundColor Cyan
$torchVersion = & $PythonCmd -c "import torch; print(torch.__version__)" 2>&1
$transformersVersion = & $PythonCmd -c "import transformers; print(transformers.__version__)" 2>&1
$onnxVersion = & $PythonCmd -c "import onnx; print(onnx.__version__)" 2>&1

Write-Host "  torch: $torchVersion" -ForegroundColor Gray
Write-Host "  transformers: $transformersVersion" -ForegroundColor Gray
Write-Host "  onnx: $onnxVersion" -ForegroundColor Gray
Write-Host ""

# 导出脚本路径
# 获取脚本所在目录
$ScriptPath = $MyInvocation.MyCommand.Path
if (-not $ScriptPath) {
    $ScriptPath = $PSCommandPath
}
$ScriptDir = Split-Path -Parent $ScriptPath
$ProjectRoot = Split-Path -Parent $ScriptDir

# 如果 ProjectRoot 不存在，尝试从当前目录查找
if (-not (Test-Path (Join-Path $ProjectRoot "core"))) {
    # 尝试从当前目录查找项目根目录
    $CurrentDir = Get-Location
    if (Test-Path (Join-Path $CurrentDir "core")) {
        $ProjectRoot = $CurrentDir
    }
    elseif (Test-Path (Join-Path $CurrentDir "scripts")) {
        $ProjectRoot = $CurrentDir
    }
}

$EncoderScript = Join-Path $ProjectRoot "docs/models/export_m2m100_encoder.py"
$DecoderScript = Join-Path $ProjectRoot "docs/models/export_m2m100_decoder_kv.py"

if (-not (Test-Path $EncoderScript)) {
    Write-Host "[ERROR] Encoder 导出脚本不存在: $EncoderScript" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $DecoderScript)) {
    Write-Host "[ERROR] Decoder 导出脚本不存在: $DecoderScript" -ForegroundColor Red
    exit 1
}

# 导出函数
function Export-M2M100Model {
    param(
        [string]$Direction,  # "en-zh" 或 "zh-en"
        [string]$SourceLang,
        [string]$TargetLang
    )
    
    Write-Host "=== 导出 M2M100 $Direction 模型 ===" -ForegroundColor Yellow
    Write-Host ""
    
    $OutputDir = Join-Path $ProjectRoot $OutputBaseDir "m2m100-$Direction"
    Write-Host "[INFO] 输出目录: $OutputDir" -ForegroundColor Cyan
    
    # 1. 导出 Encoder
    Write-Host "[1/3] 导出 Encoder..." -ForegroundColor Cyan
    & $PythonCmd $EncoderScript `
        --output_dir $OutputDir `
        --model_id $ModelId
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Encoder 导出失败" -ForegroundColor Red
        return $false
    }
    Write-Host "[OK] Encoder 导出成功" -ForegroundColor Green
    Write-Host ""
    
    # 2. 导出 Decoder
    Write-Host "[2/3] 导出 Decoder..." -ForegroundColor Cyan
    & $PythonCmd $DecoderScript `
        --output_dir $OutputDir `
        --model_id $ModelId
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Decoder 导出失败" -ForegroundColor Red
        return $false
    }
    Write-Host "[OK] Decoder 导出成功" -ForegroundColor Green
    Write-Host ""
    
    # 3. 下载 Tokenizer 文件
    Write-Host "[3/3] 下载 Tokenizer 文件..." -ForegroundColor Cyan
    
    # 检查是否有 huggingface-cli
    $HfCli = & $PythonCmd -c "import shutil; print(shutil.which('huggingface-cli'))" 2>&1
    if ($HfCli -and $HfCli -ne "None") {
        Write-Host "  使用 huggingface-cli 下载..." -ForegroundColor Gray
        & huggingface-cli download $ModelId `
            tokenizer.json `
            sentencepiece.model `
            config.json `
            --local-dir $OutputDir
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[WARN] huggingface-cli 下载失败，请手动下载" -ForegroundColor Yellow
            Write-Host "  手动下载命令:" -ForegroundColor Yellow
            Write-Host "  huggingface-cli download $ModelId tokenizer.json sentencepiece.model config.json --local-dir $OutputDir" -ForegroundColor Gray
        }
        else {
            Write-Host "[OK] Tokenizer 文件下载成功" -ForegroundColor Green
        }
    }
    else {
        Write-Host "[WARN] 未找到 huggingface-cli，请手动下载 Tokenizer 文件" -ForegroundColor Yellow
        Write-Host "  下载命令:" -ForegroundColor Yellow
        Write-Host "  huggingface-cli download $ModelId tokenizer.json sentencepiece.model config.json --local-dir $OutputDir" -ForegroundColor Gray
    }
    Write-Host ""
    
    # 验证文件
    Write-Host "[验证] 检查导出文件..." -ForegroundColor Cyan
    $RequiredFiles = @(
        "encoder.onnx",
        "decoder.onnx",
        "tokenizer.json",
        "sentencepiece.model",
        "config.json"
    )
    
    $AllExists = $true
    foreach ($file in $RequiredFiles) {
        $filePath = Join-Path $OutputDir $file
        if (Test-Path $filePath) {
            $fileSize = (Get-Item $filePath).Length / 1MB
            Write-Host "  [OK] $file ($([math]::Round($fileSize, 2)) MB)" -ForegroundColor Green
        }
        else {
            Write-Host "  [MISSING] $file" -ForegroundColor Red
            $AllExists = $false
        }
    }
    
    if ($AllExists) {
        Write-Host "[OK] 所有文件已就绪" -ForegroundColor Green
        return $true
    }
    else {
        Write-Host "[WARN] 部分文件缺失，请检查" -ForegroundColor Yellow
        return $false
    }
}

# 主流程
Write-Host "开始导出 M2M100 模型..." -ForegroundColor Green
Write-Host ""

# 导出 en-zh 模型
$Success1 = Export-M2M100Model -Direction "en-zh" -SourceLang "en" -TargetLang "zh"

Write-Host ""

# 导出 zh-en 模型
$Success2 = Export-M2M100Model -Direction "zh-en" -SourceLang "zh" -TargetLang "en"

Write-Host ""
Write-Host "=== 导出完成 ===" -ForegroundColor Green
Write-Host ""

if ($Success1 -and $Success2) {
    Write-Host "[SUCCESS] 所有模型导出成功！" -ForegroundColor Green
    Write-Host ""
    Write-Host "下一步:" -ForegroundColor Cyan
    Write-Host "  1. 验证模型文件（运行验证脚本）" -ForegroundColor White
    Write-Host "  2. 开始 Phase 1: Tokenizer 实现" -ForegroundColor White
    exit 0
}
else {
    Write-Host "[WARN] 部分模型导出可能有问题，请检查上述输出" -ForegroundColor Yellow
    exit 1
}

