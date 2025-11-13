Write-Host "=== Step 1: 安装 LLVM (clang) 和 Ninja（如果可用） ==="

# 安装 LLVM
try {
    winget install -e --id LLVM.LLVM -h 0
} catch {
    Write-Warning "安装 LLVM 可能失败，请在 Microsoft Store 或官网手动安装 LLVM。"
}

# 安装 Ninja
try {
    winget install -e --id Ninja-build.Ninja -h 0
} catch {
    Write-Warning "安装 Ninja 可能失败，请在官网手动安装 Ninja。"
}

# 刷新当前会话 PATH（简单粗暴一点：重新开终端效果更好）
$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" +
            [System.Environment]::GetEnvironmentVariable("PATH","User")

Write-Host "`n=== Step 2: 检查 clang / ninja 是否可用 ==="
try {
    clang --version
} catch {
    Write-Warning "clang 不可用，请确认 LLVM 安装成功并且在 PATH 中。"
}
try {
    ninja --version
} catch {
    Write-Warning "ninja 不可用，请确认 Ninja 安装成功并且在 PATH 中。"
}

Write-Host "`n=== Step 3: 获取 whisper.cpp 源码 ==="
$thirdPartyDir = Join-Path (Get-Location) "third_party"
if (-not (Test-Path $thirdPartyDir)) {
    New-Item -ItemType Directory -Path $thirdPartyDir | Out-Null
}
Set-Location $thirdPartyDir

if (-not (Test-Path (Join-Path $thirdPartyDir "whisper.cpp"))) {
    git clone https://github.com/ggerganov/whisper.cpp.git
} else {
    Write-Host "whisper.cpp 已存在，执行 git pull 更新..."
    Set-Location (Join-Path $thirdPartyDir "whisper.cpp")
    git pull
    Set-Location $thirdPartyDir
}

Write-Host "`n=== Step 4: 用 CMake + Ninja 编译 whisper.cpp ==="
$whisperSrc = Join-Path $thirdPartyDir "whisper.cpp"
$whisperBuild = Join-Path $whisperSrc "build"

cmake -S $whisperSrc -B $whisperBuild `
      -G Ninja `
      -DWHISPER_BUILD_EXAMPLES=ON

cmake --build $whisperBuild --config Release

Write-Host "`n=== 完成：C++ 环境 + whisper.cpp 已就绪 ==="
Write-Host "构建目录在: $whisperBuild"
