# -*- coding: utf-8 -*-
"""
测试翻译服务的 Python 脚本
"""

import requests
import json

# 测试中文到英文
response = requests.post(
    "http://127.0.0.1:5008/v1/translate",
    json={
        "src_lang": "zh",
        "tgt_lang": "en",
        "text": "你好"
    }
)

print("状态码:", response.status_code)
print("响应:", json.dumps(response.json(), ensure_ascii=False, indent=2))

# 测试英文到中文
response2 = requests.post(
    "http://127.0.0.1:5008/v1/translate",
    json={
        "src_lang": "en",
        "tgt_lang": "zh",
        "text": "Hello, welcome to the test."
    }
)

print("\n状态码:", response2.status_code)
print("响应:", json.dumps(response2.json(), ensure_ascii=False, indent=2))

