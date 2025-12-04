// core/engine/src/asr_http_client/mod.rs
// HTTP client for faster-whisper ASR service

pub mod client;
pub mod types;

pub use client::AsrHttpClient;
pub use types::{AsrHttpRequest, AsrHttpResponse};

