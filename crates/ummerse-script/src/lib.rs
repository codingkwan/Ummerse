//! # Ummerse Script
//!
//! WebAssembly 脚本运行时，支持：
//! - Wasmtime（桌面平台）作为 Wasm 宿主
//! - wasm-bindgen（Web 平台）
//! - 脚本 API 绑定（通过 WIT 接口）

pub mod runtime;
pub mod api;
pub mod binding;

pub use runtime::ScriptRuntime;
pub use api::ScriptApi;

use serde::{Deserialize, Serialize};

/// 脚本元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMeta {
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
}

/// 脚本执行错误
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("脚本加载失败: {0}")]
    LoadFailed(String),
    #[error("脚本执行错误: {0}")]
    ExecutionError(String),
    #[error("函数未找到: {0}")]
    FunctionNotFound(String),
    #[error("类型转换错误: {0}")]
    TypeMismatch(String),
}
