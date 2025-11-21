# MARIAN_ZH_EN_IR9_EXPORT_PLAN.md

本方案用于在不升级 ONNX Runtime（固定 1.16.3） 的前提下，将 marian-zh-en 模型重新导出为  
**IR ≤ 9、opset = 12** 的 ONNX 模型，使其能够在当前 NMT 框架正常加载。

---

## 1. 环境准备（必须使用 Python 3.10）

### 1.1 创建虚拟环境

```bash
python3.10 -m venv .marian_zh_en_ir9
source .marian_zh_en_ir9/bin/activate  # Linux / WSL
```

Windows:

```powershell
py -3.10 -m venv .marian_zh_en_ir9
.\.marian_zh_en_ir9\Scriptsctivate
```

---

## 2. 安装固定版本依赖

```bash
pip install torch==1.13.1+cpu -f https://download.pytorch.org/whl/torch_stable.html
pip install transformers==4.40.0 onnx==1.14.0 onnxruntime==1.16.3
```

---

## 3. 运行导出脚本

```bash
python scripts/export_marian_ir9.py --output_dir core/engine/models/nmt/marian-zh-en
```

---

## 4. 导出后文件结构

```
core/engine/models/nmt/marian-zh-en/
    model_ir9.onnx
    tokenizer.json
    config.json
```

---

## 5. 验证 ONNX

```bash
python - << 'EOF'
import onnx
import onnxruntime as ort

m = onnx.load("core/engine/models/nmt/marian-zh-en/model_ir9.onnx")
onnx.checker.check_model(m)

print("IR:", m.ir_version)
print("Opset:", [(o.domain, o.version) for o in m.opset_import])

sess = ort.InferenceSession(
    "core/engine/models/nmt/marian-zh-en/model_ir9.onnx",
    providers=["CPUExecutionProvider"]
)

print("Inputs:", [i.name for i in sess.get_inputs()])
print("Outputs:", [o.name for o in sess.get_outputs()])
EOF
```

---

## 6. 注意事项

- 必须使用 Python 3.10  
- 必须使用 torch 1.13.1  
- 必须使用 onnxruntime 1.16.3  

否则会出现 dynamo export 错误、IR 10 导出失败、ORT 无法加载 IR ≥ 10 等问题。

---

