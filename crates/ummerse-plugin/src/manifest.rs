//! 插件清单 - 描述插件元数据和能力声明

use serde::{Deserialize, Serialize};

/// 插件能力声明（权限系统）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// 读取项目文件
    ReadFiles,
    /// 写入项目文件
    WriteFiles,
    /// 执行命令（受限沙盒）
    ExecuteCommand,
    /// 访问场景树 API
    SceneAccess,
    /// 访问资产 API
    AssetAccess,
    /// 修改节点属性
    NodeMutation,
    /// 网络访问（仅通过代理）
    NetworkAccess,
    /// 访问编辑器 UI
    EditorUi,
    /// AI 推理调用（外部 LLM API）
    AiInference,
    /// 自定义能力（扩展）
    Custom(String),
}

/// 插件清单（plugin.toml / plugin.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件唯一标识符（反向域名格式）
    pub id: String,
    /// 插件名称（用于展示）
    pub name: String,
    /// 版本（语义化版本）
    pub version: String,
    /// 作者
    pub author: String,
    /// 描述
    pub description: String,
    /// 主页 URL
    pub homepage: Option<String>,
    /// 存储库 URL
    pub repository: Option<String>,
    /// 所需权限
    pub capabilities: Vec<PluginCapability>,
    /// 插件入口（Wasm 文件路径）
    pub wasm_entry: Option<String>,
    /// 插件脚本入口（可选的 Lua/JS 桥接脚本）
    pub script_entry: Option<String>,
    /// 引擎版本兼容范围（SemVer 范围）
    pub engine_version: String,
    /// 依赖的其他插件 ID
    pub dependencies: Vec<String>,
    /// 插件提供的工具列表
    pub tools: Vec<ToolManifest>,
    /// 插件提供的命令列表
    pub commands: Vec<CommandManifest>,
}

impl PluginManifest {
    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 从 TOML 反序列化
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 检查是否有指定能力
    pub fn has_capability(&self, cap: &PluginCapability) -> bool {
        self.capabilities.contains(cap)
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: "com.example.plugin".to_string(),
            name: "Example Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Unknown".to_string(),
            description: "An Ummerse plugin".to_string(),
            homepage: None,
            repository: None,
            capabilities: Vec::new(),
            wasm_entry: None,
            script_entry: None,
            engine_version: "^0.1".to_string(),
            dependencies: Vec::new(),
            tools: Vec::new(),
            commands: Vec::new(),
        }
    }
}

/// 工具清单（描述单个工具的元数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// 工具名称（snake_case）
    pub name: String,
    /// 工具描述（供 AI 理解用途）
    pub description: String,
    /// 参数 Schema（JSON Schema 格式）
    pub parameters: serde_json::Value,
    /// 是否需要用户确认
    pub requires_approval: bool,
}

/// 命令清单（编辑器命令面板中的命令）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandManifest {
    /// 命令 ID（唯一）
    pub id: String,
    /// 命令标题（显示在命令面板）
    pub title: String,
    /// 命令描述
    pub description: String,
    /// 快捷键（可选，如 "Ctrl+Shift+P"）
    pub keybinding: Option<String>,
}
