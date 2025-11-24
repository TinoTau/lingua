# M2M100 NMT 服务测试指南

## 安装依赖

### 方式1：分别安装

```bash
# 安装基础依赖
pip install -r requirements.txt

# 安装测试依赖
pip install -r requirements-test.txt
```

### 方式2：一次性安装

```bash
pip install -r requirements.txt -r requirements-test.txt
```

## 运行测试

### 运行所有测试

```bash
# 使用 python -m pytest（推荐）
python -m pytest test_nmt_service.py -v

# 或直接使用 pytest
pytest test_nmt_service.py -v
```

### 运行特定测试

```bash
# 只运行健康检查测试
python -m pytest test_nmt_service.py::test_health_check -v

# 只运行翻译测试
python -m pytest test_nmt_service.py::test_translate_zh_to_en -v
```

### 运行测试并显示输出

```bash
python -m pytest test_nmt_service.py -v -s
```

## 测试说明

### 单元测试

所有测试都是单元测试，使用 `TestClient` 模拟 HTTP 请求，**不需要运行实际的服务器**。

### 模型加载

- 测试会在会话开始时自动加载模型（通过 `@pytest.fixture(scope="session")`）
- 首次运行可能需要下载模型（需要网络连接）
- 模型加载后会在整个测试会话中保持加载状态

### 测试覆盖

- ✅ 健康检查接口
- ✅ 中文到英文翻译
- ✅ 英文到中文翻译
- ✅ 空文本处理
- ✅ 无效语言代码处理
- ✅ 缺少必需字段验证
- ✅ 长文本翻译

## 注意事项

1. **首次运行**：首次运行测试时，模型会自动下载和加载（需要网络连接）
2. **模型下载**：如果模型未下载，首次运行会自动下载（需要网络连接）
3. **GPU/CPU**：测试会自动使用可用的设备（GPU 或 CPU）
4. **测试时间**：首次运行（包括模型下载和加载）可能需要几分钟时间

## 故障排除

### 错误：ModuleNotFoundError: No module named 'fastapi'

**解决方案**：安装依赖
```bash
pip install -r requirements.txt -r requirements-test.txt
```

### 错误：模型加载失败

**可能原因**：
- 网络连接问题（首次下载模型）
- HuggingFace 访问令牌问题（如果使用私有模型）

**解决方案**：
- 检查网络连接
- 设置 `HF_TOKEN` 环境变量（如果需要）

### 测试运行缓慢

**原因**：模型加载需要时间

**解决方案**：
- 首次运行后，模型会缓存在内存中
- 后续测试运行会更快

### 使用 python -m pytest 而不是 pytest

**原因**：确保使用正确的 Python 解释器和环境

**推荐**：始终使用 `python -m pytest` 而不是直接使用 `pytest`
