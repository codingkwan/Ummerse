//! ECS 组件定义 - 场景节点对应的 ECS 组件
//!
//! 使用 bevy_ecs 而非完整 bevy，避免引入不必要的依赖。

use bevy_ecs::prelude::*;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use ummerse_core::node::{NodeId, NodeType};
use ummerse_math::{color::Color, rect::Rect2, transform::{Transform2d, Transform3d}};

/// 节点标识组件
#[derive(Component, Debug, Clone)]
pub struct NodeIdentifier {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
}

/// 2D 节点变换组件
#[derive(Component, Debug, Clone, Default)]
pub struct Transform2dComponent {
    pub local: Transform2d,
    pub global: Transform2d,
    pub dirty: bool,
}

/// 3D 节点变换组件
#[derive(Component, Debug, Clone, Default)]
pub struct Transform3dComponent {
    pub local: Transform3d,
    pub global: Transform3d,
    pub dirty: bool,
}

/// 可见性组件
#[derive(Component, Debug, Clone, Default)]
pub struct VisibilityComponent {
    pub visible: bool,
    pub inherited_visible: bool,
}

/// 2D 精灵组件
#[derive(Component, Debug, Clone)]
pub struct Sprite2dComponent {
    /// 纹理资产路径
    pub texture_path: String,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    /// UV 裁切矩形（None 表示使用整个纹理）
    pub region: Option<Rect2>,
}

impl Default for Sprite2dComponent {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            color: Color::WHITE,
            flip_x: false,
            flip_y: false,
            region: None,
        }
    }
}

/// 相机组件（通用）
#[derive(Component, Debug, Clone)]
pub struct CameraComponent {
    pub is_current: bool,
    pub zoom: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            is_current: false,
            zoom: 1.0,
        }
    }
}

/// 3D 网格实例组件
#[derive(Component, Debug, Clone)]
pub struct MeshInstance3dComponent {
    pub mesh_path: String,
    pub material_path: String,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
}

impl Default for MeshInstance3dComponent {
    fn default() -> Self {
        Self {
            mesh_path: String::new(),
            material_path: String::new(),
            cast_shadow: true,
            receive_shadow: true,
        }
    }
}

/// 刚体类型
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    Dynamic,
    Static,
    Kinematic,
}

/// 2D 刚体组件
#[derive(Component, Debug, Clone)]
pub struct RigidBody2dComponent {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub gravity_scale: f32,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
}

impl Default for RigidBody2dComponent {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
        }
    }
}

/// 音频播放器组件
#[derive(Component, Debug, Clone)]
pub struct AudioPlayerComponent {
    pub audio_path: String,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub autoplay: bool,
    pub playing: bool,
}

impl Default for AudioPlayerComponent {
    fn default() -> Self {
        Self {
            audio_path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            autoplay: false,
            playing: false,
        }
    }
}

/// 脚本组件 - 绑定 Wasm 脚本
#[derive(Component, Debug, Clone)]
pub struct ScriptComponent {
    pub script_path: String,
    pub enabled: bool,
}

impl Default for ScriptComponent {
    fn default() -> Self {
        Self {
            script_path: String::new(),
            enabled: true,
        }
    }
}
