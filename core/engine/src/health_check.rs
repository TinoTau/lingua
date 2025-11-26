//! 健康检查模块
//! 
//! 用于检查 NMT 和 TTS 服务的健康状态

use crate::error::{EngineError, EngineResult};
use reqwest::Client;
use std::time::Duration;
#[cfg(target_os = "windows")]
use tokio::process::Command;

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
        // 从 URL 中提取基础 URL（去掉路径部分）
        // 例如: http://127.0.0.1:5008/translate -> http://127.0.0.1:5008
        let base = if let Some(protocol_pos) = base_url.find("://") {
            let after_protocol = &base_url[protocol_pos + 3..];
            if let Some(path_start) = after_protocol.find('/') {
                let base_end = protocol_pos + 3 + path_start;
                &base_url[..base_end]
            } else {
                base_url
            }
        } else {
            base_url
        };
        // 构建健康检查 URL（确保 base 不以斜杠结尾）
        let health_url_str = if base.ends_with('/') {
            format!("{}health", base)
        } else {
            format!("{}/health", base)
        };
        let health_url = reqwest::Url::parse(&health_url_str)
            .unwrap_or_else(|_| reqwest::Url::parse("http://127.0.0.1:5008/health").unwrap());
        match self.http.get(health_url.clone()).send().await {
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
        // 从 URL 中提取基础 URL（去掉路径部分）
        // 例如: http://127.0.0.1:5005/tts -> http://127.0.0.1:5005
        let base = if let Some(protocol_pos) = base_url.find("://") {
            let after_protocol = &base_url[protocol_pos + 3..];
            if let Some(path_start) = after_protocol.find('/') {
                let base_end = protocol_pos + 3 + path_start;
                &base_url[..base_end]
            } else {
                base_url
            }
        } else {
            base_url
        };
        // 构建健康检查 URL（确保 base 不以斜杠结尾）
        let health_url_str = if base.ends_with('/') {
            format!("{}health", base)
        } else {
            format!("{}/health", base)
        };
        let health_url = reqwest::Url::parse(&health_url_str)
            .unwrap_or_else(|_| reqwest::Url::parse("http://127.0.0.1:5005/health").unwrap());
        match self.http.get(health_url.clone()).send().await {
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
            Err(e) => {
                if Self::check_via_wsl_fallback(&health_url).await {
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
                        error: Some(e.to_string()),
                    }
                }
            }
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

impl HealthChecker {
    #[cfg(target_os = "windows")]
    async fn check_via_wsl_fallback(url: &reqwest::Url) -> bool {
        use std::str;

        let cmd = format!(
            "curl -s -o /dev/null -w \"%{{http_code}}\" {}",
            url.as_str()
        );
        match Command::new("wsl")
            .arg("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                if let Ok(code) = String::from_utf8(output.stdout) {
                    return code.trim() == "200";
                }
                false
            }
            _ => false,
        }
    }

    #[cfg(not(target_os = "windows"))]
    async fn check_via_wsl_fallback(_url: &reqwest::Url) -> bool {
        false
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

