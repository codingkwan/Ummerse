//! 材质系统 - PBR 材质 + Unlit 材质

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 材质 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialId(pub Uuid);

impl MaterialId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::new()
    }
}

// ── GPU 数据结构 ──────────────────────────────────────────────────────────────

/// PBR 材质 Uniform（传入 GPU）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PbrMaterialUniform {
    /// 基础颜色（RGBA）
    pub base_color: [f32; 4],
    /// 金属度（0.0 = 非金属，1.0 = 金属）
    pub metallic: f32,
    /// 粗糙度（0.0 = 镜面，1.0 = 完全漫反射）
    pub roughness: f32,
    /// 自发光强度
    pub emissive_strength: f32,
    /// 透明度（0.0 = 完全透明，1.0 = 不透明）
    pub alpha: f32,
}

impl Default for PbrMaterialUniform {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive_strength: 0.0,
            alpha: 1.0,
        }
    }
}

/// 方向光 Uniform（传入 GPU）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct DirectionalLightUniform {
    /// 光线方向（世界空间，归一化，指向光源）
    pub direction: [f32; 3],
    pub _pad0: f32,
    /// 光颜色（RGB）
    pub color: [f32; 3],
    /// 光强度
    pub intensity: f32,
}

impl Default for DirectionalLightUniform {
    fn default() -> Self {
        Self {
            direction: [-0.57735, -0.57735, -0.57735], // 45° 角斜向下
            _pad0: 0.0,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        }
    }
}

// ── PBR 材质 ──────────────────────────────────────────────────────────────────

/// PBR（基于物理的渲染）材质
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMaterial {
    pub id: MaterialId,
    pub name: String,
    /// 基础颜色（线性空间 RGBA）
    pub base_color: [f32; 4],
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 自发光颜色（HDR，强度叠加）
    pub emissive_color: [f32; 3],
    pub emissive_strength: f32,
    /// 透明度模式
    pub alpha_mode: AlphaMode,
    /// Alpha 剪裁阈值（AlphaMode::Mask 时使用）
    pub alpha_cutoff: f32,
    /// 是否双面渲染
    pub double_sided: bool,
    // 纹理路径（由资产系统解析为 GpuTexture）
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub metallic_roughness_texture: Option<String>,
    pub occlusion_texture: Option<String>,
    pub emissive_texture: Option<String>,
}

impl PbrMaterial {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: MaterialId::new(),
            name: name.into(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive_color: [0.0, 0.0, 0.0],
            emissive_strength: 0.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
        }
    }

    /// 创建金属材质
    pub fn metallic(name: impl Into<String>, color: [f32; 3], roughness: f32) -> Self {
        let mut mat = Self::new(name);
        mat.base_color = [color[0], color[1], color[2], 1.0];
        mat.metallic = 1.0;
        mat.roughness = roughness;
        mat
    }

    /// 创建自发光材质
    pub fn emissive(name: impl Into<String>, color: [f32; 3], strength: f32) -> Self {
        let mut mat = Self::new(name);
        mat.emissive_color = color;
        mat.emissive_strength = strength;
        mat
    }

    /// 转换为 GPU Uniform 数据
    pub fn to_uniform(&self) -> PbrMaterialUniform {
        PbrMaterialUniform {
            base_color: self.base_color,
            metallic: self.metallic,
            roughness: self.roughness,
            emissive_strength: self.emissive_strength,
            alpha: self.base_color[3],
        }
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self::new("Default PBR Material")
    }
}

// ── Unlit 材质 ────────────────────────────────────────────────────────────────

/// Unlit 材质（不受光照影响，用于 UI 精灵/粒子等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlitMaterial {
    pub id: MaterialId,
    pub name: String,
    /// 颜色（与纹理相乘）
    pub color: [f32; 4],
    /// 纹理路径
    pub texture: Option<String>,
    /// Alpha 混合模式
    pub alpha_mode: AlphaMode,
}

impl UnlitMaterial {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: MaterialId::new(),
            name: name.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            alpha_mode: AlphaMode::AlphaBlend,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    pub fn with_texture(mut self, path: impl Into<String>) -> Self {
        self.texture = Some(path.into());
        self
    }
}

// ── Material trait ────────────────────────────────────────────────────────────

/// 通用材质 trait
pub trait Material: Send + Sync {
    fn id(&self) -> MaterialId;
    fn name(&self) -> &str;
    fn alpha_mode(&self) -> AlphaMode;
    fn is_opaque(&self) -> bool {
        self.alpha_mode() == AlphaMode::Opaque
    }
}

impl Material for PbrMaterial {
    fn id(&self) -> MaterialId {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

impl Material for UnlitMaterial {
    fn id(&self) -> MaterialId {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// ── 透明度模式 ────────────────────────────────────────────────────────────────

/// 材质透明度模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    /// 完全不透明（忽略 Alpha）
    Opaque,
    /// Alpha 蒙版（Alpha < cutoff 时完全透明）
    Mask,
    /// Alpha 混合（半透明）
    AlphaBlend,
    /// 预乘 Alpha
    PremultipliedAlpha,
}

// ── 内置材质 ──────────────────────────────────────────────────────────────────

/// 内置材质库
pub mod builtins {
    use super::*;

    /// 默认白色 PBR 材质
    pub fn white_pbr() -> PbrMaterial {
        PbrMaterial {
            id: MaterialId(Uuid::nil()),
            name: "White PBR".to_string(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.8,
            ..Default::default()
        }
    }

    /// 默认精灵材质（Unlit，支持透明）
    pub fn sprite() -> UnlitMaterial {
        UnlitMaterial {
            id: MaterialId(Uuid::nil()),
            name: "Sprite".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            alpha_mode: AlphaMode::AlphaBlend,
        }
    }

    /// 网格线（Wireframe Unlit）
    pub fn wireframe(color: [f32; 4]) -> UnlitMaterial {
        UnlitMaterial {
            id: MaterialId::new(),
            name: "Wireframe".to_string(),
            color,
            texture: None,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
