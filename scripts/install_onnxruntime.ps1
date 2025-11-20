# 安装 onnxruntime 的 PowerShell 脚本
# 
# 使用方法:
#   .\scripts\install_onnxruntime.ps1
#   或者指定 Python 解释器:
#   .\scripts\install_onnxruntime.ps1 -PythonExe "python3.10"

param(
    [string]$PythonExe = "python",
    [string]$Package = "onnxruntime"
)

Write-Host "=== 安装 $Package ===" -ForegroundColor Cyan
Write-Host "Python 解释器: $PythonExe" -ForegroundColor Yellow
Write-Host ""

# 检查 Python 是否可用
Write-Host "检查 Python 版本..." -ForegroundColor Yellow
$versionOutput = & $PythonExe --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 错误: 无法运行 $PythonExe" -ForegroundColor Red
    Write-Host "请确保 Python 已安装并在 PATH 中" -ForegroundColor Yellow
    exit 1
}
Write-Host "✅ $versionOutput" -ForegroundColor Green
Write-Host ""

# 检查 pip 是否可用
Write-Host "检查 pip 是否可用..." -ForegroundColor Yellow
$pipVersion = & $PythonExe -m pip --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 错误: pip 不可用" -ForegroundColor Red
    Write-Host "请先安装 pip" -ForegroundColor Yellow
    exit 1
}
Write-Host "✅ $pipVersion" -ForegroundColor Green
Write-Host ""

# 升级 pip
Write-Host "升级 pip..." -ForegroundColor Yellow
& $PythonExe -m pip install --upgrade pip --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  警告: pip 升级失败，继续安装..." -ForegroundColor Yellow
}
Write-Host ""

# 安装 onnxruntime
Write-Host "安装 $Package..." -ForegroundColor Yellow
Write-Host "这可能需要几分钟时间，请耐心等待..." -ForegroundColor Cyan
Write-Host ""

& $PythonExe -m pip install $Package

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ $Package 安装成功！" -ForegroundColor Green
    Write-Host ""
    
    # 验证安装
    Write-Host "验证安装..." -ForegroundColor Yellow
    $testOutput = & $PythonExe -c "import onnxruntime; print(f'onnxruntime version: {onnxruntime.__version__}')" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ $testOutput" -ForegroundColor Green
    } else {
        Write-Host "⚠️  警告: 无法导入 onnxruntime" -ForegroundColor Yellow
    }
} else {
    Write-Host ""
    Write-Host "❌ 安装失败" -ForegroundColor Red
    Write-Host "请检查错误信息并重试" -ForegroundColor Yellow
    exit 1
}

