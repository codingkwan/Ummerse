//! 节点系统 - Godot 风格的场景树节点

use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 节点唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// 生成新的唯一 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从 UUID 创建
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 获取内部 UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 节点路径 ──────────────────────────────────────────────────────────────────

/// 场景节点路径（类似文件路径）
/// 例如: "Root/Player/Weapon" 或 "../Enemy"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodePath(String);

impl NodePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn root() -> Self {
        Self("/".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 是否为绝对路径（以 / 开头）
    pub fn is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    /// 分割路径段
    pub fn segments(&self) -> Vec<&str> {
        self.0
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 获取最后一段（节点名）
    pub fn name(&self) -> Option<&str> {
        self.segments().last().copied()
    }

    /// 获取父路径
    pub fn parent(&self) -> Option<Self> {
        let segs = self.segments();
        if segs.len() <= 1 {
            None
        } else {
            let parent = segs[..segs.len() - 1].join("/");
            if self.is_absolute() {
                Some(Self(format!("/{}", parent)))
            } else {
                Some(Self(parent))
            }
        }
    }

    /// 追加子路径
    pub fn join(&self, child: &str) -> Self {
        if self.0.ends_with('/') {
            Self(format!("{}{}", self.0, child))
        } else {
            Self(format!("{}/{}", self.0, child))
        }
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NodePath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for NodePath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// ── 节点类型 ──────────────────────────────────────────────────────────────────

/// 节点类型枚举（类似 Godot 的节点类）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // 基础
    Node,
    Node2d,
    Node3d,
    // 渲染
    Sprite2d,
    AnimatedSprite2d,
    MeshInstance3d,
    Camera2d,
    Camera3d,
    DirectionalLight3d,
    PointLight3d,
    SpotLight3d,
    // 物理
    RigidBody2d,
    RigidBody3d,
    StaticBody2d,
    StaticBody3d,
    CharacterBody2d,
    CharacterBody3d,
    // UI
    UiNode,
    UiPanel,
    UiLabel,
    UiButton,
    UiImage,
    UiContainer,
    // 音频
    AudioPlayer,
    // 脚本
    ScriptNode,
    // 自定义
    Custom(String),
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(name) => write!(f, "{}", name),
            other => write!(f, "{:?}", other),
        }
    }
}

// ── 节点基础 trait ────────────────────────────────────────────────────────────

/// 节点基础接口 - 所有场景节点实现此 trait
pub trait Node: Send + Sync + 'static {
    /// 节点类型
    fn node_type(&self) -> NodeType;

    /// 节点名称
    fn name(&self) -> &str;

    /// 节点 ID
    fn id(&self) -> NodeId;

    /// 节点就绪时调用（类似 Godot 的 _ready）
    fn ready(&mut self) {}

    /// 每帧更新（类似 Godot 的 _process）
    fn process(&mut self, _delta: f32) {}

    /// 物理更新（固定时间步）
    fn physics_process(&mut self, _delta: f32) {}

    /// 节点退出场景树时调用
    fn exit_tree(&mut self) {}

    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }
}

// ── 节点元数据 ────────────────────────────────────────────────────────────────

/// 节点元数据（可序列化的节点信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub enabled: bool,
    pub visible: bool,
    pub tags: Vec<String>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl NodeMeta {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            node_type,
            enabled: true,
            visible: true,
            tags: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}
