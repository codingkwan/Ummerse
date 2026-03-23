//! Wasm 插件运行时
//!
//! 使用 wasmtime 在沙箱中运行 Wasm 插件，
//! 插件通过 JSON 消息协议与引擎通信。

use crate::{
    manifest::PluginManifest,
    protocol::{ToolCall, ToolResult},
    PluginError, Result,
};

/// Wasm 插件实例（包含 wasmtime 运行时）
///
/// 仅在非 Wasm 目标平台上编译（桌面端）。
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmPlugin {
    /// 插件清单
    pub manifest: PluginManifest,
    /// wasmtime 引擎（共享，跨实例复用）
    engine: wasmtime::Engine,
    /// wasmtime Store（持有实例状态）
    store: wasmtime::Store<WasmPluginState>,
    /// wasmtime 实例
    instance: wasmtime::Instance,
}

/// Wasm 插件实例内部状态
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmPluginState {
    /// 日志输出缓冲
    pub log_buffer: Vec<String>,
    /// 待发送的工具调用请求
    pub pending_calls: Vec<ToolCall>,
    /// 已接收的工具调用结果
    pub received_results: Vec<ToolResult>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmPlugin {
    /// 从 Wasm 字节码创建插件实例
    pub fn new(
        manifest: PluginManifest,
        wasm_bytes: &[u8],
    ) -> Result<Self> {
        use wasmtime::*;

        // 创建 wasmtime 引擎（启用 WASM 组件模型）
        let mut config = Config::new();
        config.wasm_component_model(true);
        // 启用燃料计量（限制无限循环）
        config.consume_fuel(true);

        let engine = Engine::new(&config).map_err(|e| PluginError::LoadFailed {
            name: manifest.name.clone(),
            reason: format!("Failed to create Wasm engine: {e}"),
        })?;

        let module = Module::new(&engine, wasm_bytes).map_err(|e| PluginError::LoadFailed {
            name: manifest.name.clone(),
            reason: format!("Failed to compile Wasm module: {e}"),
        })?;

        let state = WasmPluginState {
            log_buffer: Vec::new(),
            pending_calls: Vec::new(),
            received_results: Vec::new(),
        };

        let mut store = Store::new(&engine, state);
        // 设置初始燃料（限制计算量，防止插件占用过多 CPU）
        store.set_fuel(u64::MAX).ok();

        // 创建链接器并注册宿主函数
        let mut linker = Linker::new(&engine);
        Self::register_host_functions(&mut linker)?;

        let instance = linker.instantiate(&mut store, &module).map_err(|e| {
            PluginError::LoadFailed {
                name: manifest.name.clone(),
                reason: format!("Failed to instantiate Wasm module: {e}"),
            }
        })?;

        Ok(Self { manifest, engine, store, instance })
    }

    /// 从文件路径加载 Wasm 插件
    pub fn from_file(manifest: PluginManifest, path: &std::path::Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|e| PluginError::LoadFailed {
            name: manifest.name.clone(),
            reason: format!("Failed to read Wasm file '{}': {e}", path.display()),
        })?;
        Self::new(manifest, &bytes)
    }

    /// 调用插件的初始化函数（_start 或 ummerse_init）
    pub fn initialize(&mut self) -> Result<()> {
        // 尝试调用 ummerse_init，不存在则跳过
        if let Ok(init_fn) = self.instance.get_typed_func::<(), ()>(&mut self.store, "ummerse_init") {
            init_fn.call(&mut self.store, ()).map_err(|e| PluginError::LoadFailed {
                name: self.manifest.name.clone(),
                reason: format!("ummerse_init failed: {e}"),
            })?;
            tracing::info!(plugin = %self.manifest.name, "Plugin initialized via ummerse_init");
        } else {
            tracing::debug!(plugin = %self.manifest.name, "No ummerse_init found, skipping");
        }
        Ok(())
    }

    /// 向插件传递工具调用结果（通过共享内存）
    pub fn deliver_result(&mut self, result: ToolResult) {
        self.store.data_mut().received_results.push(result);
    }

    /// 获取插件发起的待处理工具调用
    pub fn take_pending_calls(&mut self) -> Vec<ToolCall> {
        std::mem::take(&mut self.store.data_mut().pending_calls)
    }

    /// 获取插件日志输出
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().log_buffer)
    }

    /// 注册宿主函数（提供给 Wasm 插件调用的 API）
    fn register_host_functions(linker: &mut wasmtime::Linker<WasmPluginState>) -> Result<()> {
        use wasmtime::*;

        // ummerse_log(level: i32, msg_ptr: i32, msg_len: i32)
        linker.func_wrap(
            "ummerse",
            "log",
            |mut caller: Caller<'_, WasmPluginState>, level: i32, ptr: i32, len: i32| {
                if let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let mut buf = vec![0u8; len as usize];
                    if memory.read(&caller, ptr as usize, &mut buf).is_ok() {
                        if let Ok(msg) = std::str::from_utf8(&buf) {
                            let level_str = match level {
                                0 => "ERROR",
                                1 => "WARN",
                                2 => "INFO",
                                3 => "DEBUG",
                                _ => "TRACE",
                            };
                            caller.data_mut().log_buffer.push(format!("[{level_str}] {msg}"));
                        }
                    }
                }
            },
        )
        .map_err(|e| PluginError::LoadFailed {
            name: "host".to_string(),
            reason: format!("Failed to register host function 'log': {e}"),
        })?;

        // ummerse_tool_call(name_ptr: i32, name_len: i32, params_ptr: i32, params_len: i32)
        linker.func_wrap(
            "ummerse",
            "tool_call",
            |mut caller: Caller<'_, WasmPluginState>,
             name_ptr: i32, name_len: i32,
             params_ptr: i32, params_len: i32| {
                let result: i32 = if let Some(memory) =
                    caller.get_export("memory").and_then(|e| e.into_memory())
                {
                    let mut name_buf = vec![0u8; name_len as usize];
                    let mut params_buf = vec![0u8; params_len as usize];
                    if memory.read(&caller, name_ptr as usize, &mut name_buf).is_ok()
                        && memory.read(&caller, params_ptr as usize, &mut params_buf).is_ok()
                    {
                        if let (Ok(name), Ok(params_str)) = (
                            std::str::from_utf8(&name_buf),
                            std::str::from_utf8(&params_buf),
                        ) {
                            if let Ok(params) = serde_json::from_str::<serde_json::Value>(params_str) {
                                let call = ToolCall::new(name, params);
                                caller.data_mut().pending_calls.push(call);
                                0 // success
                            } else {
                                -1 // JSON parse error
                            }
                        } else {
                            -2 // UTF-8 error
                        }
                    } else {
                        -3 // memory read error
                    }
                } else {
                    -4 // no memory export
                };
                result
            },
        )
        .map_err(|e| PluginError::LoadFailed {
            name: "host".to_string(),
            reason: format!("Failed to register host function 'tool_call': {e}"),
        })?;

        Ok(())
    }
}

// ── 非桌面平台占位实现 ────────────────────────────────────────────────────────

/// Wasm 目标平台下的占位结构（Wasm 中不运行 wasmtime）
#[cfg(target_arch = "wasm32")]
pub struct WasmPlugin {
    pub manifest: PluginManifest,
}

#[cfg(target_arch = "wasm32")]
impl WasmPlugin {
    pub fn new(manifest: PluginManifest, _wasm_bytes: &[u8]) -> Result<Self> {
        Err(PluginError::LoadFailed {
            name: manifest.name.clone(),
            reason: "WasmPlugin is not supported on the wasm32 target".to_string(),
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        Err(PluginError::LoadFailed {
            name: self.manifest.name.clone(),
            reason: "WasmPlugin is not supported on the wasm32 target".to_string(),
        })
    }

    pub fn take_pending_calls(&mut self) -> Vec<ToolCall> {
        Vec::new()
    }

    pub fn take_logs(&mut self) -> Vec<String> {
        Vec::new()
    }

    pub fn deliver_result(&mut self, _result: ToolResult) {}
}
