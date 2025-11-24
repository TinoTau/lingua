//! 健康检查模块
//! 
//! 用于检查 NMT 和 TTS 服务的健康状态

use crate::error::{EngineError, EngineResult};
use reqwest::Client;
use std::time::Duration;

/// 服务健康状态
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub service_name: String,
    pub url: String,
    pub error: Option<String>,
}

/// 健康检查器
pub struct HealthChecker {
    http: Client,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// 检查 NMT 服务健康状态
    pub async fn check_nmt_service(&self, base_url: &str) -> ServiceHealth {
        let url = format!("{}/health", base_url);
        match self.http.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    ServiceHealth {
                        is_healthy: true,
                        service_name: "NMT".to_string(),
                        url: base_url.to_string(),
                        error: None,
                    }
                } else {
                    ServiceHealth {
                        is_healthy: false,
                        service_name: "NMT".to_string(),
                        url: base_url.to_string(),
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => ServiceHealth {
                is_healthy: false,
                service_name: "NMT".to_string(),
                url: base_url.to_string(),
                error: Some(e.to_string()),
            },
        }
    }

    /// 检查 TTS 服务健康状态
    pub async fn check_tts_service(&self, base_url: &str) -> ServiceHealth {
        let url = format!("{}/health", base_url);
        match self.http.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    ServiceHealth {
                        is_healthy: true,
                        service_name: "TTS".to_string(),
                        url: base_url.to_string(),
                        error: None,
                    }
                } else {
                    ServiceHealth {
                        is_healthy: false,
                        service_name: "TTS".to_string(),
                        url: base_url.to_string(),
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => ServiceHealth {
                is_healthy: false,
                service_name: "TTS".to_string(),
                url: base_url.to_string(),
                error: Some(e.to_string()),
            },
        }
    }

    /// 检查所有服务健康状态
    pub async fn check_all_services(
        &self,
        nmt_url: &str,
        tts_url: &str,
    ) -> (ServiceHealth, ServiceHealth) {
        let (nmt_health, tts_health) = tokio::join!(
            self.check_nmt_service(nmt_url),
            self.check_tts_service(tts_url)
        );
        (nmt_health, tts_health)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

