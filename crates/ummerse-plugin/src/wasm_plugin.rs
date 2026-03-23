//! Wasm 插件支持 - 在 Wasmtime 沙盒中运行插件

use crate::{Result, PluginError, manifest::PluginManifest};

/// Wasm 插件实例（桌面平台，Wasmtime 运行时）
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmPluginInstance {
    pub manifest: PluginManifest,
    engine: wasmtime::Engine,
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmPluginInstance {
    /// 从 Wasm 字节码创建插件实例
    pub fn new(manifest: PluginManifest, wasm_bytes: &[u8]) -> Result<Self> {
        let config = wasmtime::Config::new();
        let engine = wasmtime::Engine::new(&config).map_err(|e| PluginError::LoadFailed {
            name: manifest.id.clone(),
            reason: e.to_string(),
        })?;

        // 验证 Wasm 模块
        wasmtime::Module::validate(&engine, wasm_bytes).map_err(|e| PluginError::LoadFailed {
            name: manifest.id.clone(),
            reason: format!("Invalid Wasm module: {}", e),
        })?;

        Ok(Self { manifest, engine })
    }

    /// 从文件加载并创建插件实例
    pub fn from_file(manifest: PluginManifest, path: &str) -> Result<Self> {
        let wasm_bytes = std::fs::read(path).map_err(|e| PluginError::LoadFailed {
            name: manifest.id.clone(),
            reason: format!("Failed to read Wasm file '{}': {}", path, e),
        })?;
        Self::new(manifest, &wasm_bytes)
    }

    /// 调用 Wasm 导出函数
    pub fn call_function(&self, _func_name: &str, _args: &[u8]) -> Result<Vec<u8>> {
        // TODO: 完整实现需要创建 Store 和 Instance
        // 当前为占位实现
        Err(PluginError::Communication(
            "WasmPlugin::call_function not fully implemented yet".to_string()
        ))
    }
}

/// Wasm 插件描述符（平台无关）
pub struct WasmPluginDescriptor {
    pub manifest: PluginManifest,
    pub wasm_path: std::path::PathBuf,
}

impl WasmPluginDescriptor {
    pub fn new(manifest: PluginManifest, wasm_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            manifest,
            wasm_path: wasm_path.into(),
        }
    }

    /// 从清单目录加载描述符（自动查找 plugin.json 和 .wasm 文件）
    pub fn from_dir(dir: &std::path::Path) -> Option<Self> {
        let manifest_path = dir.join("plugin.json");
        if !manifest_path.exists() {
            return None;
        }

        let manifest_json = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest = PluginManifest::from_json(&manifest_json).ok()?;

        let wasm_path = if let Some(entry) = &manifest.wasm_entry {
            dir.join(entry)
        } else {
            dir.join(format!("{}.wasm", manifest.id.replace('.', "_")))
        };

        Some(Self { manifest, wasm_path })
    }
}

/// 扫描目录中的所有 Wasm 插件
pub fn scan_plugins(dir: &std::path::Path) -> Vec<WasmPluginDescriptor> {
    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(descriptor) = WasmPluginDescriptor::from_dir(&path) {
                    plugins.push(descriptor);
                }
            }
        }
    }

    plugins
}
