# M2M100 TTS 问题解决报告

**日期：** 2025-01-23  
**状态：** ✅ 已解决

---

## 1. 问题概述

在集成测试过程中，发现 TTS 服务存在以下问题：
1. 英文语音模型缺失
2. 语音模型选择逻辑不正确
3. 缺少英文语音回退机制

---

## 2. 问题详情

### 2.1 英文语音模型缺失

**问题描述：**
- TTS 服务只配置了中文语音模型（`zh_CN-huayan-medium`）
- 英文语音模型（`en_US-lessac-medium`）未安装
- 导致英文文本无法正确合成语音

**错误信息：**
```
404 Not Found {"detail":"Model not found for voice: en_US-lessac-medium (searched in /home/tinot/piper_models)"}
```

**解决方案：**
1. 创建英文模型下载脚本：`scripts/wsl2_piper/download_piper_model_en.sh`
2. 修复 Piper TTS 服务的模型查找逻辑
3. 实现英文语音回退机制

---

### 2.2 语音模型选择问题

**问题描述：**
- TTS 服务在找不到英文模型时，没有回退机制
- 导致英文文本无法合成语音

**解决方案：**
- 修改 `test_s2s_full_simple_http.rs` 集成测试
- 实现动态语音选择逻辑
- 添加回退机制：如果英文模型不可用，使用中文模型并记录警告

---

### 2.3 模型查找逻辑问题

**问题描述：**
- `piper_http_server.py` 的 `find_model_path` 函数无法正确查找 `en/` 子目录下的模型

**解决方案：**
- 修改 `find_model_path` 函数，支持语言代码推断
- 支持多种模型路径结构：
  - `{model_dir}/{lang}/{voice}/{voice}.onnx`
  - `{model_dir}/{lang}/{voice}.onnx`
  - `{model_dir}/{voice}/{voice}.onnx`
  - `{model_dir}/{voice}.onnx`

---

## 3. 解决方案实施

### 3.1 英文模型下载脚本

**文件：** `scripts/wsl2_piper/download_piper_model_en.sh`

**功能：**
- 从 HuggingFace 下载英文 Piper 模型
- 自动创建模型目录
- 验证文件完整性

**使用方法：**
```bash
cd scripts/wsl2_piper
bash download_piper_model_en.sh
```

---

### 3.2 模型查找逻辑修复

**文件：** `scripts/wsl2_piper/piper_http_server.py`

**修改内容：**
```python
def find_model_path(voice: str, model_dir: str) -> Tuple[Optional[str], Optional[str]]:
    """
    查找模型文件路径
    返回: (model_path, config_path)
    """
    model_dir_path = Path(model_dir).expanduser()
    
    # 从 voice 名称推断语言代码（例如：en_US-lessac-medium -> en）
    language_code = None
    if voice.startswith("zh_"):
        language_code = "zh"
    elif voice.startswith("en_"):
        language_code = "en"
    
    possible_paths = [
        # 标准结构：{model_dir}/{lang}/{voice}/{voice}.onnx
        model_dir_path / language_code / voice / f"{voice}.onnx" if language_code else None,
        # 扁平结构：{model_dir}/{lang}/{voice}.onnx
        model_dir_path / language_code / f"{voice}.onnx" if language_code else None,
        # 旧结构：{model_dir}/{voice}/{voice}.onnx
        model_dir_path / voice / f"{voice}.onnx",
        # 旧结构：{model_dir}/zh/{voice}.onnx（向后兼容）
        model_dir_path / "zh" / f"{voice}.onnx",
        # 根目录：{model_dir}/{voice}.onnx
        model_dir_path / f"{voice}.onnx",
    ]
    
    for model_path in possible_paths:
        if model_path and model_path.exists():
            config_path = model_path.with_suffix(".onnx.json")
            return str(model_path), str(config_path) if config_path.exists() else None
    
    return None, None
```

---

### 3.3 英文语音回退机制

**文件：** `core/engine/examples/test_s2s_full_simple_http.rs`

**实现：**
- 根据目标语言动态选择 TTS 语音
- 如果英文模型不可用，回退到中文模型
- 记录警告信息，保存文件时添加 `_fallback` 后缀

---

## 4. 测试结果

### 4.1 英文模型下载测试

✅ **成功下载英文模型**
- 模型文件：`en_US-lessac-medium.onnx`
- 配置文件：`en_US-lessac-medium.onnx.json`
- 文件完整性验证通过

### 4.2 模型查找测试

✅ **模型查找逻辑修复成功**
- 可以正确找到 `en/` 子目录下的模型
- 向后兼容旧路径结构
- 支持多种路径格式

### 4.3 集成测试

✅ **集成测试通过**
- 英文音频可以正确翻译并合成中文语音
- 中文音频可以正确翻译并合成英文语音
- 回退机制正常工作

---

## 5. 相关文档

- **英文模型下载指南：** `docs/M2M100_TTS_英文模型下载指南.md`
- **英文模型缺失问题：** `docs/M2M100_TTS_英文模型缺失问题.md`
- **语音模型选择问题修复：** `docs/M2M100_TTS_语音模型选择问题修复.md`
- **英文语音回退机制：** `docs/M2M100_TTS_英文语音回退机制.md`

---

## 6. 总结

✅ **所有 TTS 问题已解决**

- ✅ 英文模型已下载并配置
- ✅ 模型查找逻辑已修复
- ✅ 英文语音回退机制已实现
- ✅ 集成测试通过

**状态：** TTS 服务已正常工作，支持中英文语音合成

---

**报告生成时间：** 2025-01-23

