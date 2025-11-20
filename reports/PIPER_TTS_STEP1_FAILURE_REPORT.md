# Piper TTS Step 1 命令行验证失败报告

**日期**：2025-11-20  
**环境**：Windows 10 x64（`D:\Programs\github\lingua`）  
**任务**：按照《PIPER_TTS_IMPLEMENTATION_STEPS.md》中的“步骤 1”，在本地通过命令行使用 Piper + 中文模型生成测试音频。

---

## 1. 目标
通过以下命令验证 Piper TTS 在本地可生成中文语音：

```powershell
echo "你好，欢迎使用语音翻译系统。" |
    piper.exe --model zh_CN-huayan-medium.onnx --output_file test_piper_step1.wav
```

期望结果：生成 `test_output\test_piper_step1.wav`，可正常播放。

---

## 2. 执行情况与日志
1. 下载并放置中文模型：`third_party\piper\models\zh\zh_CN-huayan-medium.onnx`（Test-Path 返回 True）。
2. 解压 Piper 官方 Windows 包，将包含 `piper.exe` 的目录放入 `third_party\piper\`（Test-Path 返回 True）。
3. 运行测试命令：
   ```powershell
   $text = "你好，欢迎使用语音翻译系统。"
   $piperExe = "third_party\piper\piper.exe"
   $modelPath = "third_party\piper\models\zh\zh_CN-huayan-medium.onnx"
   $outputFile = "test_output\testgrain.wav"
   $text | & $piperExe --model $modelPath --output_file $outputFile
   ```
   控制台无报错，但 `Get-Item $outputFile` 提示文件不存在。
4. 查看退出码：
   ```powershell
   Write-Host "Exit code: $LASTEXITCODE"
   ```
   输出：`Exit code: -1073740791`.

此外，在安装最新 Visual C++ Redistributable 后再次尝试，结果仍为 `-1073740791`。

---

## 3. 问题描述
- Piper 可执行文件与模型均存在。
- 命令执行后没有生成任何音频文件。
- Piper 进程立即崩溃，`$LASTEXITCODE = -1073740791`，即 Windows 异常码 0xC0000409（Stack Buffer Overrun）。
- 检查 `test_output` 目录为空，确认不是路径或权限问题。

---

## 4. 已排除的因素
1. **路径/命令错误**：所有路径通过 `Test-Path` 验证，命令简单明了且在其他平台可用。
2. **模型缺失**：模型文件已存在并可读。
3. **VC++ 运行库**：已安装最新版 Visual C++ Redistributable (2015–2022)。
4. **脚本问题**：直接在 PowerShell 中手动执行命令，结果一致，排除脚本逻辑错误。

---

## 5. 高概率根因推断
1. **CPU 指令集不兼容**  
   Piper Windows 发行版常区分 `avx`、`avx2`、`noavx` 等版本。如果当前包需要 AVX，但运行环境（例如较旧 CPU 或虚拟机）不支持，启动即会产生 0xC0000409。

2. **二进制依赖问题**  
   尽管已解压所有文件，仍建议确认 `third_party\piper\` 下包含所有官方 DLL（`cpu_features.dll`、`pthreadVC2.dll`、`espeak-ng.dll` 等）。缺失关键 DLL 也会导致无提示的崩溃。

3. **Piper 本身在当前系统不可执行**  
   建议在 `third_party\piper\` 目录运行 `.\piper.exe --help` 或 `.\piper.exe --version`；如同样崩溃，则与模型无关，说明二进制与环境不兼容。

---

## 6. 建议的下一步
1. **检查 /piper 目录完整性**
   ```powershell
   Get-ChildItem third_party\piper
   ```
   确认所有 DLL 均在。

2. **验证 Piper 基本功能**
   ```powershell
   cd third_party\piper
   .\piper.exe --help
   ```
   若该命令也返回 0xC0000409，即可确定是二进制层面问题。

3. **更换二进制版本**
   - 试用 `piper_windows_amd64_noavx` 或官方提供的 “portable / no AVX” 版。
   - 或改用 Linux 环境 / WSL / 容器，利用官方的 Linux 发行包（通常兼容性更好）。

4. **收集系统信息**  
   提供 `systeminfo` 或 `wmic cpu get name` 结果，确认 CPU 是否支持 AVX 指令集并供决策时参考。

---

## 7. 结论

