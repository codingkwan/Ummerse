//! 脚本绑定 - 连接宿主与 Wasm 模块的桥梁

/// Wasm 导出函数接口（脚本实现这些函数）
pub mod exports {
    pub const READY: &str = "ready";
    pub const PROCESS: &str = "process";
    pub const PHYSICS_PROCESS: &str = "physics_process";
    pub const ON_SIGNAL: &str = "on_signal";
    pub const EXIT: &str = "exit";
}

/// Wasm 导入函数接口（宿主提供这些函数给脚本）
pub mod imports {
    pub const LOG: &str = "engine_log";
    pub const GET_DELTA: &str = "engine_get_delta";
    pub const EMIT_SIGNAL: &str = "engine_emit_signal";
    pub const MOVE_NODE: &str = "engine_move_node";
    pub const GET_NODE: &str = "engine_get_node";
    pub const INSTANTIATE: &str = "engine_instantiate";
}
