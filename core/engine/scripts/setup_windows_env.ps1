# Windows 环境配置脚本
# 用于创建 Python 3.10 conda 环境并安装所有依赖

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Windows 环境配置脚本" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 检查 conda 是否可用
try {
    $condaVersion = conda --version 2>&1
    Write-Host "✅ Conda 已安装: $condaVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Conda 未找到，请先安装 Anaconda 或 Miniconda" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "步骤 1: 创建 conda 环境 (Python 3.10)..." -ForegroundColor Yellow

# 检查环境是否已存在
$envExists = conda env list | Select-String "lingua-py310"
if ($envExists) {
    Write-Host "⚠️  环境 lingua-py310 已存在" -ForegroundColor Yellow
    $response = Read-Host "是否删除并重新创建? (y/n)"
    if ($response -eq "y" -or $response -eq "Y") {
        Write-Host "删除现有环境..." -ForegroundColor Yellow
        conda env remove -n lingua-py310 -y
    } else {
        Write-Host "使用现有环境" -ForegroundColor Green
    }
}

# 创建新环境
if (-not $envExists -or $response -eq "y" -or $response -eq "Y") {
    Write-Host "创建新环境..." -ForegroundColor Yellow
    conda create -n lingua-py310 python=3.10 -y
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 环境创建失败" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ 环境创建成功" -ForegroundColor Green
}

Write-Host ""
Write-Host "步骤 2: 安装 PyTorch (GPU, CUDA 12.1)..." -ForegroundColor Yellow

# 激活环境并安装 PyTorch
$installPyTorch = "conda activate lingua-py310 && conda install pytorch pytorch-cuda=12.1 -c pytorch -c nvidia -y"
Invoke-Expression $installPyTorch

if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Conda 安装失败，尝试使用 pip..." -ForegroundColor Yellow
    $installPyTorchPip = "conda activate lingua-py310 && pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121"
    Invoke-Expression $installPyTorchPip
}

Write-Host ""
Write-Host "步骤 3: 安装其他依赖..." -ForegroundColor Yellow

$installDeps = @(
    "numpy",
    "soundfile",
    "flask",
    "torchaudio",
    "speechbrain"
)

foreach ($dep in $installDeps) {
    Write-Host "  安装 $dep..." -ForegroundColor Cyan
    $cmd = "conda activate lingua-py310 && pip install $dep"
    Invoke-Expression $cmd
}

Write-Host ""
Write-Host "步骤 4: 验证安装..." -ForegroundColor Yellow

$verifyCmd = @"
conda activate lingua-py310 && python -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
"@

Invoke-Expression $verifyCmd

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "✅ Windows 环境配置完成！" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
Write-Host ""
Write-Host "使用以下命令激活环境：" -ForegroundColor Cyan
Write-Host "  conda activate lingua-py310" -ForegroundColor Yellow
Write-Host ""
Write-Host "启动服务：" -ForegroundColor Cyan
Write-Host "  python core\engine\scripts\speaker_embedding_service.py --gpu" -ForegroundColor Yellow
Write-Host ""

