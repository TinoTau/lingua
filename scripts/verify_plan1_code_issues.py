#!/usr/bin/env python3
"""
验证方案 1 的代码问题

这个脚本分析当前代码实现，检查可能的问题：
1. build_initial_kv_values 的 dec_len 是否正确
2. input_ids 的形状是否一致
3. KV cache 的提取逻辑是否正确
"""

import re
from pathlib import Path

def analyze_code():
    """分析代码实现"""
    code_path = Path("core/engine/src/nmt_incremental/mod.rs")
    
    if not code_path.exists():
        print(f"Error: Code file not found: {code_path}")
        return
    
    print("="*60)
    print("Analyzing Code Implementation (方案 1)")
    print("="*60)
    
    with open(code_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 1. 检查 build_initial_kv_values 的 dec_len
    print("\n1. Checking build_initial_kv_values() dec_len:")
    match = re.search(r'fn build_initial_kv_values.*?let dec_len = (\d+)', content, re.DOTALL)
    if match:
        dec_len = int(match.group(1))
        print(f"   Found: dec_len = {dec_len}")
        if dec_len == 0:
            print("   ⚠️  ISSUE: dec_len is 0, should be 1 (first step has BOS token)")
            print("   → This is a potential bug!")
        elif dec_len == 1:
            print("   ✓ dec_len is correct (1)")
        else:
            print(f"   ⚠️  WARNING: dec_len is {dec_len}, expected 0 or 1")
    else:
        print("   ⚠️  Could not find dec_len in build_initial_kv_values")
    
    # 2. 检查第一步是否提取 KV cache
    print("\n2. Checking if Step 0 extracts KV cache:")
    if "use_cache_branch = false" in content or "use_cache_branch=false" in content:
        # 查找 else 分支（第一步的处理）
        if re.search(r'else\s*\{[^}]*//.*workaround.*跳过', content, re.IGNORECASE | re.DOTALL):
            print("   ⚠️  ISSUE: Step 0 (use_cache_branch=false) skips KV cache extraction")
            print("   → This is a potential bug!")
            print("   → Should extract present.* outputs for next step")
        elif re.search(r'else\s*\{[^}]*next_kv\.push', content, re.DOTALL):
            print("   ✓ Step 0 extracts KV cache")
        else:
            print("   ⚠️  Could not determine if Step 0 extracts KV cache")
    else:
        print("   ⚠️  Could not find use_cache_branch=false handling")
    
    # 3. 检查 input_ids 形状一致性
    print("\n3. Checking input_ids shape consistency:")
    # 查找 translate() 中的 input_ids 处理
    if re.search(r'input_ids.*vec!\[.*last_token', content):
        print("   ✓ Uses last_token for KV cache mode (length 1)")
    else:
        print("   ⚠️  Could not verify input_ids shape handling")
    
    if re.search(r'input_ids.*current_generated_ids', content):
        print("   ✓ Uses full sequence for workaround mode (length > 1)")
    else:
        print("   ⚠️  Could not verify workaround mode input_ids")
    
    # 4. 检查是否有 Reshape 相关的注释
    print("\n4. Checking for Reshape error mentions:")
    if "Reshape" in content or "reshape" in content:
        print("   ⚠️  Found Reshape error mentions in code")
        print("   → This confirms the issue exists")
    else:
        print("   No Reshape error mentions found")
    
    # 5. 检查 workaround 模式
    print("\n5. Checking workaround mode:")
    if "workaround" in content.lower():
        print("   ⚠️  Workaround mode is active")
        print("   → This means KV cache is currently disabled")
    else:
        print("   No workaround mode found")
    
    print("\n" + "="*60)
    print("Summary:")
    print("  If issues found above, they can be fixed in 方案 1")
    print("  If no issues found, the problem is likely in model export (方案 2)")
    print("="*60)


if __name__ == "__main__":
    analyze_code()

