//! 健康检查模块测试

use core_engine::health_check::{HealthChecker, ServiceHealth};

#[tokio::test]
async fn test_health_checker_creation() {
    let checker = HealthChecker::new();
    // 应该能成功创建
    assert!(true);
}

#[tokio::test]
async fn test_check_nmt_service_invalid_url() {
    let checker = HealthChecker::new();
    let health = checker.check_nmt_service("http://127.0.0.1:99999").await;
    
    // 无效的 URL 应该返回不健康状态
    assert!(!health.is_healthy);
    assert_eq!(health.service_name, "NMT");
    assert!(health.error.is_some());
}

#[tokio::test]
async fn test_check_tts_service_invalid_url() {
    let checker = HealthChecker::new();
    let health = checker.check_tts_service("http://127.0.0.1:99999").await;
    
    // 无效的 URL 应该返回不健康状态
    assert!(!health.is_healthy);
    assert_eq!(health.service_name, "TTS");
    assert!(health.error.is_some());
}

#[tokio::test]
async fn test_check_all_services() {
    let checker = HealthChecker::new();
    let (nmt_health, tts_health) = checker
        .check_all_services("http://127.0.0.1:99999", "http://127.0.0.1:99998")
        .await;
    
    // 两个服务都应该不健康（因为端口不存在）
    assert!(!nmt_health.is_healthy);
    assert!(!tts_health.is_healthy);
}

#[tokio::test]
async fn test_service_health_structure() {
    let health = ServiceHealth {
        is_healthy: true,
        service_name: "Test".to_string(),
        url: "http://test.com".to_string(),
        error: None,
    };
    
    assert!(health.is_healthy);
    assert_eq!(health.service_name, "Test");
    assert_eq!(health.url, "http://test.com");
    assert!(health.error.is_none());
}

