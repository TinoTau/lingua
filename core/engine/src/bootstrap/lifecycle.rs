//! CoreEngine 生命周期管理
//! 
//! 包含启动和关闭相关的方法

use crate::error::EngineResult;
use crate::health_check::HealthChecker;
use crate::telemetry::TelemetryDatum;

use super::core::CoreEngine;

impl CoreEngine {
    /// 启动 CoreEngine
    pub async fn boot(&self) -> EngineResult<()> {
        self.event_bus.start().await?;
        let config = self.config.load().await?;
        self.cache.warm_up().await?;
        self.asr.initialize().await?;
        self.nmt.initialize().await?;
        
        // 健康检查：检查 NMT 和 TTS 服务（带重试机制，等待服务就绪）
        if let (Some(nmt_url), Some(tts_url)) = (&self.nmt_service_url, &self.tts_service_url) {
            let checker = HealthChecker::new();
            
            // 等待服务就绪，最多重试 15 次，每次间隔 1 秒（总共最多 15 秒）
            const MAX_RETRIES: u32 = 15;
            const RETRY_DELAY_MS: u64 = 1000;
            
            let mut nmt_healthy = false;
            let mut tts_healthy = false;
            let mut final_attempt = 0;
            
            eprintln!("[INFO] Waiting for NMT and TTS services to be ready...");
            
            for attempt in 1..=MAX_RETRIES {
                final_attempt = attempt;
                let (nmt_health, tts_health) = checker.check_all_services(nmt_url, tts_url).await;
                
                nmt_healthy = nmt_health.is_healthy;
                tts_healthy = tts_health.is_healthy;
                
                if nmt_healthy && tts_healthy {
                    // 所有服务都健康，退出重试循环
                    break;
                }
                
                if attempt < MAX_RETRIES {
                    // 等待后重试（不打印中间结果，避免日志混乱）
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }
            }
            
            // 报告最终状态
            if nmt_healthy {
                eprintln!("[INFO] NMT service health check passed: {} (attempt {}/{})", nmt_url, final_attempt, MAX_RETRIES);
            } else {
                eprintln!("[WARN] NMT service is not healthy after {} attempts: {} - Please ensure the service is running", final_attempt, nmt_url);
                // 不阻止启动，但记录警告
            }
            
            if tts_healthy {
                eprintln!("[INFO] TTS service health check passed: {} (attempt {}/{})", tts_url, final_attempt, MAX_RETRIES);
            } else {
                eprintln!("[WARN] TTS service is not healthy after {} attempts: {} - Please ensure the service is running", final_attempt, tts_url);
                // 不阻止启动，但记录警告
            }
        }
        
        self.telemetry
            .record(TelemetryDatum {
                name: "core_engine.boot".to_string(),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        self.telemetry
            .record(TelemetryDatum {
                name: format!("core_engine.mode.{}", config.mode),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        Ok(())
    }

    /// 关闭 CoreEngine
    pub async fn shutdown(&self) -> EngineResult<()> {
        self.asr.finalize().await?;
        self.nmt.finalize().await?;
        self.tts.close().await?;
        self.cache.purge().await?;
        self.event_bus.stop().await?;
        self.telemetry
            .record(TelemetryDatum {
                name: "core_engine.shutdown".to_string(),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        Ok(())
    }
}

