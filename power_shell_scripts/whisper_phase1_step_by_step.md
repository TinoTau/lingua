
# Whisper 阶段 1：在 `lingua` 仓库中集成 whisper.cpp 的一步步操作指南（Windows 11）

> 目标：在 **本地 Windows 11** 环境下完成：  
> 1. 成功编译 whisper.cpp  
> 2. 跑通官方示例（JFK 语音）  
> 3. 在 `lingua` 仓库里通过 **Rust + CLI** 调用 whisper.cpp（作为阶段 1 的最小可用原型）  

假设你的项目根目录为：

```text
D:\Programs\github\lingua
```

---

## 第 0 步：确认你已经完成的事情（你现在的状态）

你已经完成：

- 用 `setup_whisper_env.ps1` 成功：
  - 安装/修复了 LLVM（clang）
  - 安装了 Ninja
  - 在 `third_party/whisper.cpp` 成功编译了 whisper.cpp

你现在可以在 PowerShell 中看到：

```powershell
clang --version
ninja --version
```

并且目录结构中有：

```text
D:\Programs\github\lingua\third_party\whisper.cpp\build
```

下面从这里继续往下走。

---

## 第 1 步：运行 whisper.cpp 官方示例（确认“能听懂人话”）

1. 打开 PowerShell，进入 whisper.cpp 目录：

```powershell
cd D:\Programs\github\lingua
cd .\third_party\whisper.cpp
```

2. 下载一个小模型（英文 base）：

```powershell
python .\models\download-ggml-model.py base.en
```

下载完成后，确认 `models` 目录中出现：

```text
ggml-base.en.bin
```

3. 找到编译好的 `main.exe`：

```powershell
Get-ChildItem .\build -Recurse -Filter main.exe
```

记住输出中的路径，例如：

```text
D:\Programs\github\lingua\third_party\whisper.cpp\build\bin\main.exe
```

4. 使用示例音频文件 `samples/jfk.wav` 运行：

```powershell
.uildin\main.exe -m .\models\ggml-base.en.bin -f .\samples\jfk.wav
```

**预期结果：**  
终端会打印一段关于 Kennedy 的英文文本，这说明：

- C++ 编译环境 ✅  
- whisper.cpp ✅  
- 模型 & 音频 IO ✅  

---

## 第 2 步：在 `lingua` 项目中以“CLI 方式”调用 whisper.cpp

> 这一阶段我们先不写 C/C++ FFI，而是直接在 Rust 里调用 `main.exe`，拿回识别结果。  
> 这样可以快速做出一个 **“阶段 1 Whisper ASR 原型”**。  

### 2.1 在脑子里固定几个路径（后面会写死在配置里）

- whisper.cpp 根目录：  
  `D:\Programs\github\lingua\third_party\whisper.cpp`

- 模型路径：  
  `D:\Programs\github\lingua\third_party\whisper.cpp\models\ggml-base.en.bin`

- 可执行程序路径（你在第 1 步用 Get-ChildItem 查到的那个）：  
  例如：  
  `D:\Programs\github\lingua\third_party\whisper.cpp\build\bin\main.exe`

> 后续代码中我们会使用**相对路径**，所以在 Rust 中写成：  
> `third_party/whisper.cpp/models/ggml-base.en.bin`  
> `third_party/whisper.cpp/build/bin/main.exe`  

请确认这两个相对路径在你的项目根目录下是存在的。

---

### 2.2 新建 Rust 模块：`asr_whisper::cli`

1. 在 `core/engine/src` 目录下创建子目录（如果不存在）：

```powershell
cd D:\Programs\github\lingua\core\engine\src
mkdir asr_whisper -ErrorAction SilentlyContinue
```

2. 在 `asr_whisper` 目录中创建 `mod.rs`（如果已有，可以在里面再 `pub mod cli;`）：

```text
core/
  engine/
    src/
      asr_whisper/
        mod.rs
```

3. 在 `mod.rs` 里写入以下内容（如果你已经有别的内容，可以只加最后一行 `pub mod cli;`）：

```rust
// core/engine/src/asr_whisper/mod.rs

pub mod cli;
```

4. 创建 `core/engine/src/asr_whisper/cli.rs` 文件，内容如下：

```rust
// core/engine/src/asr_whisper/cli.rs

use std::process::Command;
use std::path::Path;
use anyhow::{Result, bail};

/// Whisper CLI 调用的配置
pub struct WhisperCliConfig {
    /// whisper.cpp 的 main.exe 程序路径（相对于项目根目录）
    pub exe_path: String,
    /// 模型文件路径（相对于项目根目录）
    pub model_path: String,
}

impl Default for WhisperCliConfig {
    fn default() -> Self {
        Self {
            // 注意：这里使用相对路径，假设当前工作目录是项目根目录 D:\Programs\github\lingua
            exe_path: "third_party/whisper.cpp/build/bin/main.exe".to_string(),
            model_path: "third_party/whisper.cpp/models/ggml-base.en.bin".to_string(),
        }
    }
}

/// 使用 whisper.cpp 的 main.exe 对 WAV 文件进行转写
pub fn transcribe_wav_cli(cfg: &WhisperCliConfig, wav_path: &Path) -> Result<String> {
    if !Path::new(&cfg.exe_path).exists() {
        bail!("whisper main.exe not found at: {}", cfg.exe_path);
    }
    if !Path::new(&cfg.model_path).exists() {
        bail!("whisper model not found at: {}", cfg.model_path);
    }
    if !wav_path.exists() {
        bail!("wav file not found: {}", wav_path.display());
    }

    let output = Command::new(&cfg.exe_path)
        .args([
            "-m",
            &cfg.model_path,
            "-f",
            &wav_path.to_string_lossy(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("whisper CLI failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
```

> 注意：  
> - 这段代码依赖 `anyhow` crate，请确保你的 `core/engine` 对应的 `Cargo.toml` 已经添加：  
>   ```toml
>   [dependencies]
>   anyhow = "1"
>   ```

---

### 2.3 添加一个简单测试：调用 CLI 识别 JFK 示例

1. 在你的 `core/engine` crate 里创建 `tests` 目录（如果还没有）：

```powershell
cd D:\Programs\github\lingua\core\engine
mkdir tests -ErrorAction SilentlyContinue
```

2. 创建一个测试文件：`core/engine/tests/asr_whisper_cli.rs`：

```rust
// core/engine/tests/asr_whisper_cli.rs

use std::path::Path;
use core_engine::asr_whisper::cli::{WhisperCliConfig, transcribe_wav_cli};

#[test]
fn test_whisper_cli_on_jfk_sample() {
    let cfg = WhisperCliConfig::default();
    let wav = Path::new("third_party/whisper.cpp/samples/jfk.wav");

    let text = transcribe_wav_cli(&cfg, wav).expect("failed to run whisper CLI");
    println!("ASR result:
{}", text);

    // 简单检查输出内容中是否包含关键字
    let lower = text.to_lowercase();
    assert!(lower.contains("kennedy") || lower.contains("nation"), "unexpected ASR result");
}
```

> ⚠️ 注意：  
> - `core_engine` 这里是你 engine crate 的包名，请用你实际的 crate 名称替换，比如：  
>   - 如果你的 `Cargo.toml` 写的是  
>     ```toml
>     [package]
>     name = "lingua_core_engine"
>     ```  
>     那这里就要改成：  
>     ```rust
>     use lingua_core_engine::asr_whisper::cli::{WhisperCliConfig, transcribe_wav_cli};
>     ```  
> - 如果不确定包名，可以打开 `core/engine/Cargo.toml` 看 `[package] name = "..."`。

3. 在项目根目录运行测试（假设 engine 是一个 workspace 成员）：

```powershell
cd D:\Programs\github\lingua

# （如果你的项目是 workspace，可以直接）
cargo test asr_whisper_cli -- --nocapture
```

> - `--nocapture` 可以让测试里的 `println!` 输出到终端，方便你看到 ASR 结果。  
> - 如果 workspace 比较复杂，也可以直接进入 engine crate 目录运行：  
>   ```powershell
>   cd D:\Programs\github\lingua\core\engine
>   cargo test asr_whisper_cli -- --nocapture
>   ```

**预期结果：**

- 测试通过 ✅  
- 终端打印类似 JFK 演讲的英文文本 ✅  

这说明：

> 你的 `lingua` 项目已经可以 **通过 Rust + CLI 调用 whisper.cpp**，完成一整个 wav → 文本 的 ASR 过程。  

这已经可以作为 **阶段 1：Whisper ASR 原型** 的基础版本。

---

## 第 3 步（后续可以再做）：挂接到你的引擎接口 / 服务

当上面的测试跑通之后，你后续可以：

1. 在引擎里定义一个统一的 ASR 接口（例如 trait 或 service）：
   ```rust
   pub trait AsrEngine {
       fn transcribe_wav(&self, path: &Path) -> anyhow::Result<String>;
   }
   ```

2. 实现一个基于 CLI 的版本：
   ```rust
   pub struct WhisperCliEngine {
       cfg: WhisperCliConfig,
   }

   impl AsrEngine for WhisperCliEngine {
       fn transcribe_wav(&self, path: &Path) -> anyhow::Result<String> {
           transcribe_wav_cli(&self.cfg, path)
       }
   }
   ```

3. 在 Node/TypeScript 或 HTTP 层增加一个简单 API：  
   - 提供一个 endpoint 或 function：  
     `asrWhisperTranscribe(filePath) -> string`  
   - 这样你可以从 Chrome 插件或其它地方上传 wav 文件，让引擎返回文本。

这些属于“下一阶段”的工程化集成，你可以在当前 CLI 原型稳定后慢慢做。

---

## 总结：你接下来只需按顺序做的事情

1. **在 whisper.cpp 目录下载模型并运行 JFK 示例**  
   - `python .\models\download-ggml-model.py base.en`  
   - `.uildin\main.exe -m .\models\ggml-base.en.bin -f .\samples\jfk.wav`  

2. **在 `core/engine/src/asr_whisper` 里创建 `mod.rs` + `cli.rs`**  
   - 写入 `WhisperCliConfig` 和 `transcribe_wav_cli` 函数。  

3. **在 `core/engine/tests` 中创建测试 `asr_whisper_cli.rs`**  
   - 调用 `transcribe_wav_cli` 识别 `jfk.wav`。  

4. **运行测试**  
   - `cargo test asr_whisper_cli -- --nocapture`  
   - 确认看到合理的英文转写结果。  

只要你按这个文件一步一步来，就可以把 **whisper.cpp + Rust CLI 原型** 完整地跑起来。  
后续如果你准备好继续做 C FFI 或 wasm，我可以在这个基础上帮你升级到下一版。
