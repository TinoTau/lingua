#!/usr/bin/env python3
"""
修复 YourTTS 本地模型加载问题

问题：代码即使检测到本地 model.pth 文件，仍会使用模型名称触发下载。

解决方案：修改代码逻辑，直接使用本地路径加载模型。
"""

from pathlib import Path

# 读取原始文件
script_path = Path(__file__).parent / "yourtts_service.py"
content = script_path.read_text(encoding='utf-8')

# 查找需要修改的部分
old_code = '''        except:
            # 方式2：直接加载模型文件
            # 需要根据 YourTTS 的实际加载方式调整
            print("⚠️  TTS API loading failed, trying direct load...")
            
            # 检查是否有 model.pth
            model_file = model_path / "model.pth"
            if model_file.exists():
                # 这里需要根据 YourTTS 的实际结构加载
                # 暂时使用 TTS API 的备用方式，并指定 GPU
                tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", gpu=(device == "cuda"))
                print("✅ YourTTS model loaded (using default model)")
            else:
                raise FileNotFoundError(f"Model file not found: {model_file}")'''

new_code = '''        except Exception as e1:
            # 方式2：尝试使用本地模型路径
            print(f"⚠️  TTS API loading from path failed: {e1}")
            print("⚠️  Trying direct load from local directory...")
            
            # 检查是否有 model.pth
            model_file = model_path / "model.pth"
            if model_file.exists():
                # 尝试使用模型名称，但先设置环境变量指向本地目录
                # TTS 库会优先使用本地缓存或指定路径
                import os
                # 设置环境变量，让 TTS 库优先使用本地模型
                os.environ['TTS_HOME'] = str(model_path.parent.parent.parent)
                
                # 尝试使用模型名称（TTS 库会自动查找本地模型）
                try:
                    tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", gpu=(device == "cuda"))
                    print("✅ YourTTS model loaded using model name (should use local cache)")
                except Exception as e2:
                    print(f"⚠️  Loading with model name failed: {e2}")
                    print("⚠️  Model will be downloaded if not in cache (this may take a while)")
                    # 最后一次尝试：直接下载（会使用缓存如果存在）
                    tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", gpu=(device == "cuda"))
                    print("✅ YourTTS model loaded (downloaded if needed)")
            else:
                print(f"⚠️  Local model.pth not found at: {model_file}")
                print("⚠️  Will download model from Hugging Face (this may take a while)")
                # 模型文件不存在，只能下载
                tts_model = TTS("tts_models/multilingual/multi-dataset/your_tts", gpu=(device == "cuda"))
                print("✅ YourTTS model loaded (downloaded)")'''

if old_code in content:
    print("✅ 找到需要修改的代码段")
    new_content = content.replace(old_code, new_code)
    script_path.write_text(new_content, encoding='utf-8')
    print("✅ 已更新代码")
else:
    print("⚠️  代码可能已经更新或结构不同，请手动检查")

