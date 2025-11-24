# -*- coding: utf-8 -*-
"""
M2M100 NMT 服务单元测试
"""

import pytest
from fastapi.testclient import TestClient
from nmt_service import app, load_model
import asyncio

# 在模块级别加载模型（在创建 TestClient 之前）
# 这样可以确保模型在测试开始前已经加载
@pytest.fixture(scope="session", autouse=True)
def load_model_fixture():
    """在测试会话开始时加载模型"""
    try:
        # 尝试获取当前事件循环
        loop = asyncio.get_running_loop()
        # 如果已经有运行中的循环，需要特殊处理
        import nest_asyncio
        nest_asyncio.apply()
        asyncio.run(load_model())
    except RuntimeError:
        # 如果没有运行中的循环，直接运行
        asyncio.run(load_model())

# 创建测试客户端
# TestClient 会自动处理应用的生命周期事件（包括启动事件）
client = TestClient(app)


def test_health_check():
    """测试健康检查接口"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert "status" in data
    assert data["status"] in ["ok", "not_ready"]


def test_translate_zh_to_en():
    """测试中文到英文翻译"""
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "zh",
            "tgt_lang": "en",
            "text": "你好"
        }
    )
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True
    assert "text" in data
    assert data["text"] is not None
    assert len(data["text"]) > 0
    assert data["provider"] == "local-m2m100"
    assert "extra" in data
    assert "elapsed_ms" in data["extra"]
    assert "num_tokens" in data["extra"]


def test_translate_en_to_zh():
    """测试英文到中文翻译"""
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "en",
            "tgt_lang": "zh",
            "text": "Hello"
        }
    )
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True
    assert "text" in data
    assert data["text"] is not None


def test_translate_empty_text():
    """测试空文本翻译"""
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "zh",
            "tgt_lang": "en",
            "text": ""
        }
    )
    # 空文本可能返回错误或空结果，取决于实现
    assert response.status_code == 200
    data = response.json()
    # 检查响应格式是否正确
    assert "ok" in data


def test_translate_invalid_lang():
    """测试无效语言代码"""
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "invalid",
            "tgt_lang": "en",
            "text": "测试"
        }
    )
    # 可能返回错误或使用默认语言
    assert response.status_code == 200
    data = response.json()
    assert "ok" in data


def test_translate_missing_fields():
    """测试缺少必需字段"""
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "zh",
            # 缺少 tgt_lang 和 text
        }
    )
    assert response.status_code == 422  # Validation error


def test_translate_long_text():
    """测试长文本翻译"""
    long_text = "你好，" * 100  # 创建一个较长的文本
    response = client.post(
        "/v1/translate",
        json={
            "src_lang": "zh",
            "tgt_lang": "en",
            "text": long_text
        }
    )
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

