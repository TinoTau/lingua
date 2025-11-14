#!/usr/bin/env python3
"""
批量导出所有 Marian NMT 语言对的 Encoder 模型

使用方法:
    python scripts/export_all_marian_encoders.py
"""

import subprocess
import sys
from pathlib import Path

# 定义所有需要导出的语言对
# 注意：marian-en-ja 的模型名称是 opus-mt-en-jap
LANGUAGE_PAIRS = [
    ("Helsinki-NLP/opus-mt-zh-en", "core/engine/models/nmt/marian-zh-en"),
    ("Helsinki-NLP/opus-mt-en-es", "core/engine/models/nmt/marian-en-es"),
    ("Helsinki-NLP/opus-mt-es-en", "core/engine/models/nmt/marian-es-en"),
    ("Helsinki-NLP/opus-mt-en-jap", "core/engine/models/nmt/marian-en-ja"),  # 注意：模型名是 en-jap
    ("Helsinki-NLP/opus-mt-ja-en", "core/engine/models/nmt/marian-ja-en"),
]

def export_encoder(model_name: str, output_dir: str) -> bool:
    """导出单个 encoder 模型"""
    print(f"\n{'='*60}")
    print(f"Exporting: {model_name}")
    print(f"Output: {output_dir}")
    print(f"{'='*60}\n")
    
    script_path = Path(__file__).parent / "export_marian_encoder.py"
    
    cmd = [
        sys.executable,
        str(script_path),
        "--model_name", model_name,
        "--output_dir", output_dir,
        "--verify"
    ]
    
    try:
        result = subprocess.run(
            cmd,
            check=True,
            capture_output=False,
            text=True
        )
        print(f"\n[OK] Successfully exported: {model_name}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n[ERROR] Failed to export: {model_name}")
        print(f"Exit code: {e.returncode}")
        return False
    except Exception as e:
        print(f"\n[ERROR] Unexpected error exporting {model_name}: {e}")
        return False


def main():
    print("="*60)
    print("Batch Exporting Marian NMT Encoder Models")
    print("="*60)
    print(f"\nTotal language pairs: {len(LANGUAGE_PAIRS)}")
    
    success_count = 0
    failed_pairs = []
    
    for model_name, output_dir in LANGUAGE_PAIRS:
        if export_encoder(model_name, output_dir):
            success_count += 1
        else:
            failed_pairs.append((model_name, output_dir))
    
    # 总结
    print("\n" + "="*60)
    print("Export Summary")
    print("="*60)
    print(f"Successfully exported: {success_count}/{len(LANGUAGE_PAIRS)}")
    
    if failed_pairs:
        print(f"\nFailed pairs ({len(failed_pairs)}):")
        for model_name, output_dir in failed_pairs:
            print(f"  - {model_name} -> {output_dir}")
        sys.exit(1)
    else:
        print("\n[OK] All encoder models exported successfully!")
        sys.exit(0)


if __name__ == "__main__":
    main()

