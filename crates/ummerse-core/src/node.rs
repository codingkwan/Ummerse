//! 节点系统 - Godot 风格的场景树节点
//!
//! 提供节点 ID、路径、类型和基础 trait，
//! 供场景树（ummerse-scene）使用。

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ── 节点 ID ───────────────────────────────────────────────────────────────────

/// 节点唯一标识符（基于 UUID v4）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// 生成新的唯一 ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从已有 UUID 创建
    #[inline]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 获取内部 UUID（只读）
    #[inline]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// 判断是否为 nil UUID（零值）
    #[inline]
    pub fn is_nil(&self) -> bool {
        self.0.is_nil()
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
///
/// - 绝对路径以 `/` 开头，如 `/Root/Player/Weapon`
/// - 相对路径不以 `/` 开头，如 `Player/Weapon`
/// - 父节点引用使用 `..`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodePath(String);

impl NodePath {
    /// 从字符串创建路径（不验证格式）
    #[inline]
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// 根路径 `/`
    #[inline]
    pub fn root() -> Self {
        Self("/".into())
    }

    /// 获取路径字符串
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 是否为绝对路径（以 `/` 开头）
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.0.starts_with('/')
    }

    /// 分割路径段，过滤空段
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

    /// 获取父路径（如果有）
    pub fn parent(&self) -> Option<Self> {
        let segs = self.segments();
        if segs.len() <= 1 {
            return None;
        }
        let parent = segs[..segs.len() - 1].join("/");
        Some(Self(if self.is_absolute() {
            format!("/{parent}")
        } else {
            parent
        }))
    }

    /// 追加子路径段
    pub fn join(&self, child: &str) -> Self {
        if self.0.ends_with('/') {
            Self(format!("{}{child}", self.0))
        } else {
            Self(format!("{}/{child}", self.0))
        }
    }

    /// 判断是否为根路径
    #[inline]
    pub fn is_root(&self) -> bool {
        self.0 == "/"
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

/// 节点类型枚举（对应 Godot 的节点类层次）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // ── 基础节点 ─────────────────────────────────────────────────────────
    Node,
    Node2d,
    Node3d,

    // ── 2D 渲染 ──────────────────────────────────────────────────────────
    Sprite2d,
    AnimatedSprite2d,
    Camera2d,
    TileMap,
    Label,
    CanvasLayer,

    // ── 3D 渲染 ──────────────────────────────────────────────────────────
    MeshInstance3d,
    Camera3d,
    DirectionalLight3d,
    PointLight3d,
    SpotLight3d,
    OmniLight3d,
    Environment3d,
    Decal,

    // ── 2D 物理 ──────────────────────────────────────────────────────────
    RigidBody2d,
    StaticBody2d,
    CharacterBody2d,
    Area2d,
    CollisionShape2d,
    RayCast2d,

    // ── 3D 物理 ──────────────────────────────────────────────────────────
    RigidBody3d,
    StaticBody3d,
    CharacterBody3d,
    Area3d,
    CollisionShape3d,
    RayCast3d,

    // ── UI ───────────────────────────────────────────────────────────────
    UiNode,
    UiPanel,
    UiLabel,
    UiButton,
    UiImage,
    UiContainer,
    UiScrollContainer,
    UiLineEdit,
    UiTextEdit,
    UiCheckbox,
    UiSlider,

    // ── 音频 ─────────────────────────────────────────────────────────────
    AudioStreamPlayer,
    AudioStreamPlayer2d,
    AudioStreamPlayer3d,

    // ── 动画 ─────────────────────────────────────────────────────────────
    AnimationPlayer,
    AnimationTree,

    // ── 脚本 ─────────────────────────────────────────────────────────────
    ScriptNode,

    // ── 自定义 ───────────────────────────────────────────────────────────
    /// 用户自定义节点类型
    Custom(String),
}

impl NodeType {
    /// 是否为 2D 节点
    #[must_use]
    pub fn is_2d(&self) -> bool {
        matches!(
            self,
            Self::Node2d
                | Self::Sprite2d
                | Self::AnimatedSprite2d
                | Self::Camera2d
                | Self::TileMap
                | Self::RigidBody2d
                | Self::StaticBody2d
                | Self::CharacterBody2d
                | Self::Area2d
                | Self::CollisionShape2d
                | Self::RayCast2d
                | Self::AudioStreamPlayer2d
        )
    }

    /// 是否为 3D 节点
    #[must_use]
    pub fn is_3d(&self) -> bool {
        matches!(
            self,
            Self::Node3d
                | Self::MeshInstance3d
                | Self::Camera3d
                | Self::DirectionalLight3d
                | Self::PointLight3d
                | Self::SpotLight3d
                | Self::OmniLight3d
                | Self::RigidBody3d
                | Self::StaticBody3d
                | Self::CharacterBody3d
                | Self::Area3d
                | Self::CollisionShape3d
                | Self::RayCast3d
                | Self::AudioStreamPlayer3d
        )
    }

    /// 是否为 UI 节点
    #[must_use]
    pub fn is_ui(&self) -> bool {
        matches!(
            self,
            Self::UiNode
                | Self::UiPanel
                | Self::UiLabel
                | Self::UiButton
                | Self::UiImage
                | Self::UiContainer
                | Self::UiScrollContainer
                | Self::UiLineEdit
                | Self::UiTextEdit
                | Self::UiCheckbox
                | Self::UiSlider
        )
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(name) => write!(f, "{name}"),
            other => write!(f, "{other:?}"),
        }
    }
}

// ── 节点基础 trait ────────────────────────────────────────────────────────────

/// 节点基础接口 - 所有场景节点实现此 trait
///
/// 参考 Godot 的节点生命周期钩子（`_ready`/`_process`/`_physics_process`）。
pub trait Node: Send + Sync + 'static {
    /// 节点类型标识
    fn node_type(&self) -> NodeType;

    /// 节点名称
    fn name(&self) -> &str;

    /// 节点唯一 ID
    fn id(&self) -> NodeId;

    /// 节点进入场景树时调用（类似 Godot 的 `_ready`）
    fn ready(&mut self) {}

    /// 每帧逻辑更新（类似 Godot 的 `_process`）
    ///
    /// - `delta`: 本帧时间（秒）
    fn process(&mut self, _delta: f32) {}

    /// 固定步长物理更新（类似 Godot 的 `_physics_process`）
    ///
    /// - `delta`: 物理步长（秒，通常为 1/60）
    fn physics_process(&mut self, _delta: f32) {}

    /// 节点退出场景树时调用（类似 Godot 的 `_exit_tree`）
    fn exit_tree(&mut self) {}

    /// 节点是否启用（禁用时跳过 process/physics_process）
    fn is_enabled(&self) -> bool {
        true
    }
}

// ── 节点元数据 ────────────────────────────────────────────────────────────────

/// 节点元数据（可序列化的节点状态快照）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    /// 节点 ID
    pub id: NodeId,
    /// 节点名称
    pub name: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 是否启用
    pub enabled: bool,
    /// 是否可见
    pub visible: bool,
    /// 标签列表（用于分组查询）
    pub tags: Vec<String>,
    /// 父节点 ID（根节点为 None）
    pub parent: Option<NodeId>,
    /// 子节点 ID 列表（有序）
    pub children: Vec<NodeId>,
}

impl NodeMeta {
    /// 创建节点元数据
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

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// 判断是否含有指定标签
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_path_segments() {
        let path = NodePath::new("/Root/Player/Weapon");
        assert_eq!(path.segments(), vec!["Root", "Player", "Weapon"]);
        assert_eq!(path.name(), Some("Weapon"));
    }

    #[test]
    fn test_node_path_parent() {
        let path = NodePath::new("/Root/Player/Weapon");
        let parent = path.parent().unwrap();
        assert_eq!(parent.as_str(), "/Root/Player");
    }

    #[test]
    fn test_node_path_join() {
        let path = NodePath::new("/Root");
        let child = path.join("Player");
        assert_eq!(child.as_str(), "/Root/Player");
    }

    #[test]
    fn test_node_type_classification() {
        assert!(NodeType::Sprite2d.is_2d());
        assert!(NodeType::MeshInstance3d.is_3d());
        assert!(NodeType::UiButton.is_ui());
        assert!(!NodeType::Node.is_2d());
    }

    #[test]
    fn test_node_id_unique() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }
}
