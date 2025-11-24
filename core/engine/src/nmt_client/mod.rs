//! NMT 客户端模块
//!
//! 提供统一的 NMT 客户端接口，支持本地 Python 服务和远程 API。

mod local_m2m100;
mod remote;
mod types;
mod adapter;

pub use local_m2m100::LocalM2m100HttpClient;
pub use remote::RemoteNmtHttpClient;
pub use types::{NmtClient, NmtTranslateRequest, NmtTranslateResponse};
pub use adapter::NmtClientAdapter;

