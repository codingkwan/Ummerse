//! 脚本引擎 API 定义 - 供 Wasm 脚本调用的宿主函数
//!
//! 这些函数通过 WIT 接口或直接内存映射暴露给 Wasm 脚本。
//! 参考 Godot 的 GDScript API 设计，提供：
//! - 日志/调试 API
//! - 节点操作 API
//! - 输入查询 API
//! - 数学工具 API
//! - 时间 API

use serde::{Deserialize, Serialize};

/// 脚本 API - 提供给脚本访问引擎功能的接口
pub struct ScriptApi;

impl ScriptApi {
    // ── 日志 API ──────────────────────────────────────────────────────────

    /// 日志输出
    pub fn log(level: &str, message: &str) {
        match level {
            "error" => tracing::error!(target: "script", "{}", message),
            "warn" => tracing::warn!(target: "script", "{}", message),
            "info" => tracing::info!(target: "script", "{}", message),
            _ => tracing::debug!(target: "script", "{}", message),
        }
    }

    /// 打印调试信息（类 Godot print()）
    pub fn print(message: &str) {
        tracing::info!(target: "script", "{}", message);
    }

    /// 打印警告
    pub fn push_warning(message: &str) {
        tracing::warn!(target: "script", "⚠️ {}", message);
    }

    /// 打印错误
    pub fn push_error(message: &str) {
        tracing::error!(target: "script", "❌ {}", message);
    }

    // ── 时间 API ──────────────────────────────────────────────────────────

    /// 获取当前 Unix 时间戳（毫秒）
    pub fn get_time_msec() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    // ── 数学工具 API ──────────────────────────────────────────────────────

    /// 线性插值
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }

    /// 平滑步进
    pub fn smoothstep(a: f32, b: f32, t: f32) -> f32 {
        let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// 钳制值
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
}

// ── 宿主函数接口（Host Imports）────────────────────────────────────────────────

/// 脚本可调用的引擎宿主函数描述（用于生成 WIT 绑定）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostFunction {
    /// 函数名称
    pub name: String,
    /// 参数列表
    pub params: Vec<HostParam>,
    /// 返回值类型
    pub returns: Option<HostType>,
    /// 描述
    pub description: String,
    /// 是否需要审批（危险操作）
    pub requires_approval: bool,
}

/// 宿主函数参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostParam {
    pub name: String,
    pub ty: HostType,
    pub description: String,
}

/// 宿主函数值类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostType {
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Json,
    Void,
}

/// 内置宿主函数注册表
pub fn builtin_host_functions() -> Vec<HostFunction> {
    vec![
        HostFunction {
            name: "engine_log".into(),
            params: vec![
                HostParam {
                    name: "level".into(),
                    ty: HostType::String,
                    description: "日志级别 (info/warn/error)".into(),
                },
                HostParam {
                    name: "message".into(),
                    ty: HostType::String,
                    description: "日志消息".into(),
                },
            ],
            returns: None,
            description: "输出日志消息到引擎控制台".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "get_node".into(),
            params: vec![HostParam {
                name: "path".into(),
                ty: HostType::String,
                description: "节点路径".into(),
            }],
            returns: Some(HostType::Json),
            description: "根据路径获取场景节点数据".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "set_node_position".into(),
            params: vec![
                HostParam {
                    name: "node_id".into(),
                    ty: HostType::String,
                    description: "节点 ID".into(),
                },
                HostParam {
                    name: "x".into(),
                    ty: HostType::F32,
                    description: "X 坐标".into(),
                },
                HostParam {
                    name: "y".into(),
                    ty: HostType::F32,
                    description: "Y 坐标".into(),
                },
            ],
            returns: None,
            description: "设置 2D 节点位置".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "emit_signal".into(),
            params: vec![
                HostParam {
                    name: "signal_name".into(),
                    ty: HostType::String,
                    description: "信号名".into(),
                },
                HostParam {
                    name: "args_json".into(),
                    ty: HostType::Json,
                    description: "信号参数（JSON数组）".into(),
                },
            ],
            returns: None,
            description: "发射信号".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "get_delta".into(),
            params: vec![],
            returns: Some(HostType::F32),
            description: "获取当前帧 delta time（秒）".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "is_action_pressed".into(),
            params: vec![HostParam {
                name: "action".into(),
                ty: HostType::String,
                description: "输入动作名".into(),
            }],
            returns: Some(HostType::Bool),
            description: "检查输入动作是否被按住".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "play_sound".into(),
            params: vec![
                HostParam {
                    name: "path".into(),
                    ty: HostType::String,
                    description: "音频资产路径".into(),
                },
                HostParam {
                    name: "volume".into(),
                    ty: HostType::F32,
                    description: "音量 (0.0~1.0)".into(),
                },
            ],
            returns: None,
            description: "播放音效".into(),
            requires_approval: false,
        },
        HostFunction {
            name: "queue_free".into(),
            params: vec![HostParam {
                name: "node_id".into(),
                ty: HostType::String,
                description: "要删除的节点 ID".into(),
            }],
            returns: None,
            description: "标记节点为待删除（在帧末删除）".into(),
            requires_approval: false,
        },
    ]
}

// ── 脚本调用上下文 ────────────────────────────────────────────────────────────

/// 脚本调用上下文 - 传递给脚本函数的运行时状态
#[derive(Debug, Clone)]
pub struct ScriptContext {
    /// 当前脚本所挂载的节点 ID
    pub node_id: String,
    /// 当前 delta time
    pub delta: f32,
    /// 累计运行时间
    pub elapsed: f64,
    /// 帧计数
    pub frame: u64,
}

impl ScriptContext {
    pub fn new(node_id: impl Into<String>, delta: f32, elapsed: f64, frame: u64) -> Self {
        Self {
            node_id: node_id.into(),
            delta,
            elapsed,
            frame,
        }
    }
}

/// 脚本调用结果
#[derive(Debug, Clone)]
pub enum ScriptCallResult {
    /// 成功，返回值（JSON 编码）
    Ok(Option<serde_json::Value>),
    /// 脚本运行时错误
    RuntimeError(String),
    /// 宿主函数调用（需要引擎处理后继续执行）
    HostCall {
        function: String,
        args: serde_json::Value,
    },
}
