use core_engine::onnx_utils::init_onnx_runtime;

/// 最小测试：确认可以初始化 ONNX Runtime 环境。
#[test]
fn test_init_onnx_runtime_env() {
    init_onnx_runtime().expect("failed to init ONNX Runtime");
}
