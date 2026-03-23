//! 引擎统一错误类型

use thiserror::Error;

/// 引擎统一错误
#[derive(Debug, Error)]
pub enum EngineError {
    // ── 系统级 ────────────────────────────────────────────────────────────
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    // ── 节点/场景 ─────────────────────────────────────────────────────────
    #[error("节点未找到: {path}")]
    NodeNotFound { path: String },

    #[error("节点已存在: {name}")]
    NodeAlreadyExists { name: String },

    #[error("无效节点路径: {path}")]
    InvalidNodePath { path: String },

    #[error("场景加载失败: {path}, 原因: {reason}")]
    SceneLoadFailed { path: String, reason: String },

    #[error("场景未找到: {name}")]
    SceneNotFound { name: String },

    // ── 资产 ──────────────────────────────────────────────────────────────
    #[error("资产未找到: {path}")]
    AssetNotFound { path: String },

    #[error("资产加载失败: {path}, 原因: {reason}")]
    AssetLoadFailed { path: String, reason: String },

    #[error("不支持的资产类型: {ext}")]
    UnsupportedAssetType { ext: String },

    // ── 渲染 ──────────────────────────────────────────────────────────────
    #[error("GPU 设备不可用")]
    GpuDeviceUnavailable,

    #[error("着色器编译失败: {name}, 原因: {reason}")]
    ShaderCompileFailed { name: String, reason: String },

    #[error("渲染错误: {0}")]
    RenderError(String),

    // ── 插件 ──────────────────────────────────────────────────────────────
    #[error("插件错误: {name}, 原因: {reason}")]
    PluginError { name: String, reason: String },

    #[error("工具调用失败: {tool}, 原因: {reason}")]
    ToolCallFailed { tool: String, reason: String },

    // ── 脚本 ──────────────────────────────────────────────────────────────
    #[error("脚本执行错误: {0}")]
    ScriptError(String),

    // ── 通用 ──────────────────────────────────────────────────────────────
    #[error("内部错误: {0}")]
    Internal(String),

    #[error("功能未实现: {0}")]
    NotImplemented(String),
}

/// 引擎统一 Result
pub type Result<T> = std::result::Result<T, EngineError>;
