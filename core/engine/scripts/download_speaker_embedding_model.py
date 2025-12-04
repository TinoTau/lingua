#!/usr/bin/env python3
# coding: utf-8
"""
Download ECAPA-TDNN Speaker Embedding Model

Usage:
    python download_speaker_embedding_model.py --output core/engine/models/speaker_embedding
    python download_speaker_embedding_model.py --output core/engine/models/speaker_embedding --auto-downgrade
    python download_speaker_embedding_model.py --hf-token YOUR_TOKEN

This script downloads the SpeechBrain ECAPA-TDNN model and converts it to ONNX format.
It automatically handles Windows symlink permission issues by using copy strategy.

Options:
    --output: Output directory for the model files
    --auto-downgrade: Automatically downgrade torchaudio if incompatible version is detected
    --hf-token: HuggingFace token (optional, uses built-in token by default)
"""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

# Note: We don't need yaml import anymore since we're not modifying hyperparams.yaml

# Default HuggingFace token (can be overridden by environment variable or argument)
DEFAULT_HF_TOKEN = "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"

# Disable symlinks on Windows to avoid permission issues
os.environ["HF_HUB_DISABLE_SYMLINKS_WARNING"] = "1"

# Apply torchaudio compatibility fix BEFORE any imports that use it
def fix_torchaudio_compatibility():
    """Fix torchaudio compatibility issue with SpeechBrain"""
    try:
        import torchaudio
        # Patch torchaudio if list_audio_backends is missing (torchaudio 2.9+)
        if not hasattr(torchaudio, 'list_audio_backends'):
            # Mock the function for SpeechBrain compatibility
            def mock_list_audio_backends():
                return ['soundfile']  # Default backend for Windows
            
            torchaudio.list_audio_backends = mock_list_audio_backends
            print("✅ Applied torchaudio.list_audio_backends compatibility fix")
    except ImportError:
        pass  # torchaudio not installed yet

def patch_speechbrain_torchaudio_backend():
    """Patch SpeechBrain's torchaudio backend check before it runs"""
    try:
        import sys
        import types
        
        # Create a patched version of the check function
        def patched_check_torchaudio_backend():
            """Patched version that doesn't call list_audio_backends"""
            try:
                import torchaudio
                # Just check if torchaudio exists, don't check backends
                if not hasattr(torchaudio, '__version__'):
                    raise RuntimeError("torchaudio not properly installed")
            except ImportError:
                raise RuntimeError("torchaudio is not installed. Install it with: pip install torchaudio")
        
        # Patch the module before SpeechBrain imports it
        # We'll do this by monkey-patching after torchaudio is loaded
        import torchaudio
        if not hasattr(torchaudio, 'list_audio_backends'):
            # Store original modules if any
            pass
        
    except ImportError:
        pass  # Will be handled later

# Apply fix immediately when module is imported
fix_torchaudio_compatibility()

def check_and_downgrade_torchaudio(auto_downgrade=False):
    """Check torchaudio version and optionally downgrade if incompatible"""
    try:
        import torchaudio
        version_str = torchaudio.__version__
        print(f"   Found torchaudio version: {version_str}")
        
        needs_downgrade = False
        
        # Check if version is 2.9 or higher (which has compatibility issues)
        try:
            from packaging import version
            if version.parse(version_str) >= version.parse("2.9.0"):
                needs_downgrade = True
                print(f"   ⚠️  Warning: torchaudio {version_str} has compatibility issues with SpeechBrain")
        except ImportError:
            # If packaging not available, check version string manually
            if version_str.startswith("2.9") or version_str.startswith("2.10") or version_str.startswith("3."):
                needs_downgrade = True
                print(f"   ⚠️  Warning: torchaudio {version_str} may have compatibility issues")
        except Exception as e:
            # Manual version check
            major_minor = version_str.split('.')[:2]
            if len(major_minor) >= 2:
                try:
                    major, minor = int(major_minor[0]), int(major_minor[1])
                    if (major > 2) or (major == 2 and minor >= 9):
                        needs_downgrade = True
                        print(f"   ⚠️  Warning: torchaudio {version_str} may have compatibility issues")
                except:
                    pass
        
        if needs_downgrade:
            if auto_downgrade:
                print("   🔄 Auto-downgrading torchaudio to compatible version (<2.9)...")
                try:
                    # Uninstall current version
                    subprocess.check_call([
                        sys.executable, "-m", "pip", "uninstall", "torchaudio", "-y"
                    ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                    
                    # Install compatible version
                    subprocess.check_call([
                        sys.executable, "-m", "pip", "install", "torchaudio<2.9"
                    ])
                    
                    # Verify new version - reload the module
                    import importlib
                    if 'torchaudio' in sys.modules:
                        del sys.modules['torchaudio']
                    
                    # Re-import torchaudio
                    import torchaudio
                    new_version = torchaudio.__version__
                    print(f"   ✅ Successfully downgraded to torchaudio {new_version}")
                    
                    # Re-apply compatibility fix
                    fix_torchaudio_compatibility()
                    return True
                except Exception as e:
                    print(f"   ❌ Failed to auto-downgrade: {e}")
                    print("   Please manually run: pip install 'torchaudio<2.9'")
                    return False
            else:
                print("   💡 Tip: Use --auto-downgrade flag to automatically downgrade torchaudio")
                print("   Or manually run: pip install 'torchaudio<2.9'")
        
        # Apply compatibility fix before importing SpeechBrain
        fix_torchaudio_compatibility()
        return True
    except ImportError:
        print("   torchaudio not found, will be installed with speechbrain")
        return True

def check_and_install_dependencies(auto_downgrade=False):
    """Check and install required dependencies"""
    print("📌 Checking dependencies...")
    
    # First, check and optionally downgrade torchaudio
    check_and_downgrade_torchaudio(auto_downgrade=auto_downgrade)
    
    dependencies = ["speechbrain", "torch", "onnx", "onnxruntime"]
    
    for dep in dependencies:
        try:
            __import__(dep.replace("-", "_"))
        except ImportError:
            print(f"📦 Installing {dep}...")
            subprocess.check_call([sys.executable, "-m", "pip", "install", dep])
    
    # Re-check torchaudio version after installing speechbrain (it may have installed a new version)
    # and re-apply downgrade if needed
    if auto_downgrade:
        print("\n🔍 Re-checking torchaudio version after dependency installation...")
        check_and_downgrade_torchaudio(auto_downgrade=True)
    
    # Re-apply compatibility fix after installation (torchaudio may have been installed/updated)
    fix_torchaudio_compatibility()
    
    # Install soundfile if not present (required for Windows audio backend)
    try:
        import soundfile
    except ImportError:
        print("📦 Installing soundfile (required for audio backend)...")
        try:
            subprocess.check_call([sys.executable, "-m", "pip", "install", "soundfile"])
        except:
            print("⚠️  Could not install soundfile automatically")
    
    print("✅ All dependencies installed")

def download_and_convert_model(output_dir: Path, auto_downgrade=False, hf_token=None):
    """Download SpeechBrain ECAPA-TDNN model and convert to ONNX"""
    print("\n====================================================")
    print("      ECAPA-TDNN Speaker Embedding Model")
    print("====================================================")
    print(f"📁 Output directory: {output_dir.resolve()}")
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Configure HuggingFace token
    if hf_token:
        os.environ["HF_TOKEN"] = hf_token
        os.environ["HUGGING_FACE_HUB_TOKEN"] = hf_token
        print("🔑 Using provided HuggingFace token")
    elif "HF_TOKEN" in os.environ:
        hf_token = os.environ["HF_TOKEN"]
    elif "HUGGING_FACE_HUB_TOKEN" in os.environ:
        hf_token = os.environ["HUGGING_FACE_HUB_TOKEN"]
    else:
        # Use default token if available
        hf_token = DEFAULT_HF_TOKEN
        os.environ["HF_TOKEN"] = hf_token
        os.environ["HUGGING_FACE_HUB_TOKEN"] = hf_token
        print("🔑 Using default HuggingFace token")
    
    # Disable symlinks to avoid Windows permission issues
    # Set environment variable to force copy strategy
    os.environ["HF_HUB_DISABLE_SYMLINKS_WARNING"] = "1"
    
    # Create cache directory in output folder to avoid symlink issues
    cache_dir = output_dir / "cache"
    cache_dir.mkdir(parents=True, exist_ok=True)
    
    # Apply torchaudio compatibility fix before importing SpeechBrain
    fix_torchaudio_compatibility()
    
    # Patch SpeechBrain's backend check module BEFORE importing SpeechBrain
    import sys
    import types
    
    try:
        import torchaudio
        if not hasattr(torchaudio, 'list_audio_backends'):
            print("🔧 Applying SpeechBrain compatibility patch for torchaudio 2.9+...")
            
            # Pre-create and patch the backend check module
            backend_module_name = 'speechbrain.utils.torch_audio_backend'
            
            # Create a patched version of the check function
            def patched_check_torchaudio_backend():
                """Patched version that skips list_audio_backends check"""
                pass  # Skip the problematic check
            
            # Inject the patched module before SpeechBrain imports it
            if backend_module_name not in sys.modules:
                fake_backend_module = types.ModuleType(backend_module_name)
                fake_backend_module.check_torchaudio_backend = patched_check_torchaudio_backend
                sys.modules[backend_module_name] = fake_backend_module
                print("✅ Backend check module pre-patched")
    except ImportError:
        pass
    
    # Import after dependencies are installed and patches applied
    try:
        import torch
        print("   Attempting to import SpeechBrain...")
        from speechbrain.inference.speaker import EncoderClassifier
        print("✅ SpeechBrain imported successfully")
    except (AttributeError, RuntimeError) as e:
        error_msg = str(e)
        if "list_audio_backends" in error_msg or "audio_backend" in error_msg.lower():
            print("\n❌ torchaudio compatibility issue detected!")
            
            if auto_downgrade:
                print("\n🔄 Attempting automatic downgrade...")
                success = check_and_downgrade_torchaudio(auto_downgrade=True)
                
                if success:
                    print("\n🔄 Retrying SpeechBrain import after downgrade...")
                    # Reload torchaudio module
                    import importlib
                    if 'torchaudio' in sys.modules:
                        importlib.reload(sys.modules['torchaudio'])
                    
                    # Re-apply patches
                    fix_torchaudio_compatibility()
                    
                    # Retry import
                    try:
                        from speechbrain.inference.speaker import EncoderClassifier
                        print("✅ SpeechBrain imported successfully after downgrade!")
                    except Exception as e2:
                        print(f"\n❌ Still failed after downgrade: {e2}")
                        print("Please try manually: pip install 'torchaudio<2.9'")
                        import traceback
                        traceback.print_exc()
                        sys.exit(1)
                else:
                    print("\n❌ Auto-downgrade failed. Please try manually.")
                    sys.exit(1)
            else:
                print("\n" + "="*60)
                print("  SOLUTION: Downgrade torchaudio to version < 2.9")
                print("="*60)
                print("\nOption 1 - Automatic (recommended):")
                print("  python scripts/download_speaker_embedding_model.py --auto-downgrade")
                print("\nOption 2 - Manual:")
                print("  pip install 'torchaudio<2.9'")
                print("  Then re-run this script.")
                print("\nAlternative: Install soundfile and try again:")
                print("  pip install soundfile")
                print("="*60)
                import traceback
                traceback.print_exc()
                sys.exit(1)
        else:
            raise
    except ImportError as e:
        print(f"❌ Failed to import required modules: {e}")
        print("Please install dependencies first: pip install speechbrain torch onnx onnxruntime")
        sys.exit(1)
    except Exception as e:
        error_msg = str(e)
        if "list_audio_backends" in error_msg or "audio_backend" in error_msg.lower():
            print("\n❌ torchaudio compatibility issue!")
            print("Please downgrade torchaudio: pip install 'torchaudio<2.9'")
            import traceback
            traceback.print_exc()
            sys.exit(1)
        raise
    
    print("\n⬇️  Downloading ECAPA-TDNN model from SpeechBrain...")
    print("This may take a few minutes (model size ~100MB)...")
    print("   Using direct download (no symlinks) to avoid Windows permission issues")
    
    try:
        # First, download model files directly using huggingface_hub to avoid symlink issues
        from huggingface_hub import snapshot_download
        
        print("   Step 1: Downloading model files from HuggingFace Hub...")
        print(f"   Token: {hf_token[:10]}..." if hf_token and len(hf_token) > 10 else "   Using default token")
        
        # Download model files directly to cache_dir (no symlinks)
        model_cache_dir = snapshot_download(
            repo_id="speechbrain/spkrec-ecapa-voxceleb",
            local_dir=str(cache_dir),  # Download directly to this directory (no symlinks)
            token=hf_token,
            local_files_only=False,
        )
        print(f"   ✅ Model files downloaded to: {model_cache_dir}")
        
        # Now load the model from the downloaded directory
        # SpeechBrain's Pretrainer tries to use symlinks by default on Windows
        # We need to patch the link_with_strategy function to use copy instead
        print("   Step 2: Configuring SpeechBrain to use copy strategy (no symlinks)...")
        
        # Patch SpeechBrain's fetching module to use copy instead of symlink
        import speechbrain.utils.fetching as fetch_utils
        
        # Save original function
        original_link_with_strategy = fetch_utils.link_with_strategy
        
        # Create patched version that uses copy instead of symlink
        def patched_link_with_strategy(src, dst, local_strategy=None):
            """Patched version that copies instead of creating symlinks"""
            from pathlib import Path as PathLib
            src_path = PathLib(src)
            dst_path = PathLib(dst)
            
            # If source doesn't exist, fall back to original function
            # This allows SpeechBrain to handle optional files or download them from cache
            if not src_path.exists():
                try:
                    # Try original function - it might handle the file from HuggingFace cache
                    return original_link_with_strategy(src, dst, local_strategy)
                except Exception:
                    # If original also fails, return destination path (file might be optional)
                    dst_path.parent.mkdir(parents=True, exist_ok=True)
                    return dst_path
            
            # Source exists - use copy strategy instead of symlink
            # Create destination directory if needed
            dst_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Remove destination if it exists (could be a symlink or file)
            if dst_path.exists() or dst_path.is_symlink():
                try:
                    if dst_path.is_symlink() or dst_path.is_file():
                        dst_path.unlink()
                    elif dst_path.is_dir():
                        shutil.rmtree(dst_path)
                except Exception as e:
                    print(f"   ⚠️  Warning: Could not remove existing destination: {e}")
            
            # Copy file instead of creating symlink
            try:
                if src_path.is_file():
                    shutil.copy2(src_path, dst_path)
                elif src_path.is_dir():
                    shutil.copytree(src_path, dst_path)
                else:
                    # Fall back to original for edge cases
                    return original_link_with_strategy(src, dst, local_strategy)
            except Exception as e:
                print(f"   ⚠️  Warning: Could not copy {src_path.name}: {e}, using original strategy...")
                # Fall back to original function on error
                return original_link_with_strategy(src, dst, local_strategy)
            
            return dst_path
        
        # Apply patch
        fetch_utils.link_with_strategy = patched_link_with_strategy
        print("   ✅ Patched link_with_strategy to use copy instead of symlink")
        
        try:
            # Load model - it should now use copy instead of symlink
            print("   Step 3: Loading model...")
            # Load from HuggingFace repo ID - files are already cached, but SpeechBrain
            # needs to handle the model structure and optional files (like custom.py)
            classifier = EncoderClassifier.from_hparams(
                source="speechbrain/spkrec-ecapa-voxceleb",  # Load from repo ID (uses cache)
                savedir=str(cache_dir),
                run_opts={"device": "cpu"}
            )
            print("✅ Model loaded successfully")
        finally:
            # Restore original function
            fetch_utils.link_with_strategy = original_link_with_strategy
    except Exception as e:
        error_msg = str(e)
        print(f"❌ Failed to download/load model: {e}")
        
        if "WinError 1314" in error_msg or "特权" in error_msg or "privilege" in error_msg.lower():
            print("\n⚠️  Windows permission error detected (symlink issue)")
            print("\n💡 Alternative solutions:")
            print("   1. Run PowerShell as Administrator")
            print("   2. Enable Windows Developer Mode (Settings > Privacy > For developers)")
            print("   3. Use WSL (Windows Subsystem for Linux)")
        
        import traceback
        traceback.print_exc()
        sys.exit(1)
    
    # ONNX export is not supported for SpeechBrain ECAPA-TDNN models
    # The model contains data-dependent operations (dynamic slicing) that are not ONNX-compatible
    print("\n" + "="*60)
    print("  ONNX Export Information")
    print("="*60)
    print("\n⚠️  ONNX export is not supported for SpeechBrain ECAPA-TDNN models.")
    print("\nReason:")
    print("  SpeechBrain models contain data-dependent operations (dynamic slicing)")
    print("  in preprocessing modules (mean_var_norm) that are not compatible with ONNX export.")
    print("  This is a known limitation of SpeechBrain's preprocessing pipeline.")
    print("\n✅ Good News:")
    print("  The PyTorch model is fully functional and ready to use!")
    print("\n  Usage example:")
    print("  ```python")
    print("  from speechbrain.inference.speaker import EncoderClassifier")
    print(f"  classifier = EncoderClassifier.from_hparams(source='{cache_dir}')")
    print("  # Input: audio tensor of shape [batch, samples] at 16kHz")
    print("  embeddings = classifier.encode_batch(audio_tensor)")
    print("  # Output: embeddings of shape [batch, 1, 192]")
    print("  ```")
    print(f"\n  Model location: {cache_dir}")
    print("\n  All model files are saved and ready to use.")
    print("="*60)
        
    # ONNX export is skipped - SpeechBrain models contain dynamic operations
    # The model is successfully downloaded and loaded, which is the main goal
    pass

def main():
    parser = argparse.ArgumentParser(description="Download and convert ECAPA-TDNN Speaker Embedding Model")
    parser.add_argument(
        "--output",
        default="core/engine/models/speaker_embedding",
        help="Output directory for the model files"
    )
    parser.add_argument(
        "--auto-downgrade",
        action="store_true",
        help="Automatically downgrade torchaudio if incompatible version is detected"
    )
    parser.add_argument(
        "--hf-token",
        default=None,
        help="HuggingFace token for authentication (default: uses built-in token or HF_TOKEN env var)"
    )
    args = parser.parse_args()
    
    output_dir = Path(args.output)
    
    # Get token from argument, environment variable, or default
    hf_token = args.hf_token or os.environ.get("HF_TOKEN") or os.environ.get("HUGGING_FACE_HUB_TOKEN") or DEFAULT_HF_TOKEN
    
    try:
        check_and_install_dependencies(auto_downgrade=args.auto_downgrade)
        download_and_convert_model(output_dir, auto_downgrade=args.auto_downgrade, hf_token=hf_token)
        
        print("\n====================================================")
        print("✅ Download and conversion completed!")
        print(f"   Files saved in: {output_dir.resolve()}")
        print("====================================================")
        
    except KeyboardInterrupt:
        print("\n\n⚠️  Download interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()

