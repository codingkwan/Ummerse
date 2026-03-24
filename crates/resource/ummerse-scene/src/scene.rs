//! 场景资产 - 可序列化的场景数据

use crate::SceneNodeData;
use serde::{Deserialize, Serialize};
use ummerse_core::node::NodeId;

/// 场景资产文件（.uscn 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAsset {
    /// 场景名称
    pub name: String,
    /// 场景版本
    pub version: u32,
    /// 所有节点数据（扁平化存储）
    pub nodes: Vec<SceneNodeData>,
    /// 根节点 ID
    pub root: Option<NodeId>,
    /// 场景元数据
    pub metadata: SceneMetadata,
}

impl SceneAsset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            nodes: Vec::new(),
            root: None,
            metadata: SceneMetadata::default(),
        }
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 序列化为 RON（Rusty Object Notation）
    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }

    /// 从 RON 反序列化
    pub fn from_ron(ron_str: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(ron_str)
    }
}

/// 场景元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    /// 场景使用的资产路径列表
    pub asset_dependencies: Vec<String>,
}

/// 运行时场景实例（结合场景资产与实际状态）
#[derive(Debug)]
pub struct Scene {
    pub asset: SceneAsset,
    /// 场景是否已加载到内存
    pub loaded: bool,
    /// 场景是否当前活跃
    pub active: bool,
}

impl Scene {
    pub fn from_asset(asset: SceneAsset) -> Self {
        Self {
            asset,
            loaded: false,
            active: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.asset.name
    }
}
