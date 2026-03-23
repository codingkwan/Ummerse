//! Wasm 脚本运行时

use crate::ScriptError;

/// 脚本运行时（跨平台抽象）
pub struct ScriptRuntime {
    #[cfg(not(target_arch = "wasm32"))]
    engine: Option<WasmtimeEngine>,
}

#[cfg(not(target_arch = "wasm32"))]
struct WasmtimeEngine {
    engine: wasmtime::Engine,
}

impl ScriptRuntime {
    pub fn new() -> Result<Self, ScriptError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let config = wasmtime::Config::new();
            let engine = wasmtime::Engine::new(&config)
                .map_err(|e| ScriptError::LoadFailed(e.to_string()))?;
            Ok(Self {
                engine: Some(WasmtimeEngine { engine }),
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self {})
        }
    }

    /// 加载并执行 Wasm 脚本字节
    pub fn load_module(&self, _wasm_bytes: &[u8]) -> Result<ScriptModule, ScriptError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(engine) = &self.engine {
                let _module = wasmtime::Module::new(&engine.engine, _wasm_bytes)
                    .map_err(|e| ScriptError::LoadFailed(e.to_string()))?;
                return Ok(ScriptModule { name: "module".to_string() });
            }
        }
        Ok(ScriptModule { name: "stub".to_string() })
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            #[cfg(not(target_arch = "wasm32"))]
            engine: None,
        })
    }
}

/// 已加载的脚本模块
pub struct ScriptModule {
    pub name: String,
}
