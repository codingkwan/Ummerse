//! 材质系统 - PBR 和非光照材质

use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

// ── 材质 Trait ────────────────────────────────────────────────────────────────

/// 材质 trait - 所有材质类型实现此接口
pub trait Material: Send + Sync + 'static {
    /// 材质名称
    fn name(&self) -> &str;

    /// 着色器类型
    fn shader_kind(&self) -> ShaderKind;

    /// 序列化为 JSON（用于编辑器和资产系统）
    fn to_json(&self) -> serde_json::Value;

    /// 是否透明
    fn is_transparent(&self) -> bool {
        false
    }

    /// 是否双面渲染
    fn double_sided(&self) -> bool {
        false
    }
}

/// 着色器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderKind {
    /// PBR（物理基础渲染）
    Pbr,
    /// 无光照（Unlit）
    Unlit,
    /// 卡通渲染
    Toon,
    /// 自定义着色器
    Custom,
}

// ── PBR 材质 ──────────────────────────────────────────────────────────────────

/// PBR 材质（物理基础渲染）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMaterial {
    pub name: String,
    /// 基础颜色（RGBA）
    pub base_color: Vec4,
    /// 基础颜色贴图路径
    pub base_color_texture: Option<String>,
    /// 金属度（0.0 = 非金属，1.0 = 全金属）
    pub metallic: f32,
    /// 粗糙度（0.0 = 光滑，1.0 = 粗糙）
    pub roughness: f32,
    /// 金属/粗糙度贴图路径
    pub metallic_roughness_texture: Option<String>,
    /// 法线贴图路径
    pub normal_texture: Option<String>,
    /// 法线贴图强度
    pub normal_scale: f32,
    /// 遮蔽贴图路径（AO）
    pub occlusion_texture: Option<String>,
    /// AO 强度
    pub occlusion_strength: f32,
    /// 自发光颜色
    pub emissive: Vec3,
    /// 自发光贴图
    pub emissive_texture: Option<String>,
    /// 自发光强度
    pub emissive_strength: f32,
    /// Alpha 混合模式
    pub alpha_mode: AlphaMode,
    /// Alpha 截断阈值（用于 AlphaMode::Mask）
    pub alpha_cutoff: f32,
    /// 是否双面
    pub double_sided: bool,
    /// UV 缩放
    pub uv_scale: Vec2,
    /// UV 偏移
    pub uv_offset: Vec2,
}

impl PbrMaterial {
    /// 创建默认白色 PBR 材质
    pub fn default_white() -> Self {
        Self {
            name: "Default".into(),
            base_color: Vec4::ONE,
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            metallic_roughness_texture: None,
            normal_texture: None,
            normal_scale: 1.0,
            occlusion_texture: None,
            occlusion_strength: 1.0,
            emissive: Vec3::ZERO,
            emissive_texture: None,
            emissive_strength: 1.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            uv_scale: Vec2::ONE,
            uv_offset: Vec2::ZERO,
        }
    }

    /// 创建金属材质
    pub fn metallic(color: Vec3) -> Self {
        let mut mat = Self::default_white();
        mat.base_color = color.extend(1.0);
        mat.metallic = 1.0;
        mat.roughness = 0.2;
        mat
    }

    /// 创建自发光材质
    pub fn emissive(color: Vec3, strength: f32) -> Self {
        let mut mat = Self::default_white();
        mat.emissive = color;
        mat.emissive_strength = strength;
        mat.base_color = Vec4::new(0.0, 0.0, 0.0, 1.0);
        mat
    }

    /// 创建玻璃材质（透明）
    pub fn glass(tint: Vec4, roughness: f32) -> Self {
        let mut mat = Self::default_white();
        mat.base_color = tint;
        mat.metallic = 0.0;
        mat.roughness = roughness;
        mat.alpha_mode = AlphaMode::Blend;
        mat
    }
}

impl Material for PbrMaterial {
    fn name(&self) -> &str {
        &self.name
    }

    fn shader_kind(&self) -> ShaderKind {
        ShaderKind::Pbr
    }

    fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Blend)
    }

    fn double_sided(&self) -> bool {
        self.double_sided
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self::default_white()
    }
}

// ── 无光照材质 ────────────────────────────────────────────────────────────────

/// 无光照（Unlit）材质 - 不受光照影响，直接显示颜色/纹理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlitMaterial {
    pub name: String,
    /// 颜色（RGBA）
    pub color: Vec4,
    /// 颜色贴图路径
    pub texture: Option<String>,
    /// UV 缩放
    pub uv_scale: Vec2,
    /// UV 偏移  
    pub uv_offset: Vec2,
    /// Alpha 混合模式
    pub alpha_mode: AlphaMode,
}

impl UnlitMaterial {
    pub fn new(color: Vec4) -> Self {
        Self {
            name: "Unlit".into(),
            color,
            texture: None,
            uv_scale: Vec2::ONE,
            uv_offset: Vec2::ZERO,
            alpha_mode: AlphaMode::Opaque,
        }
    }

    pub fn with_texture(mut self, path: impl Into<String>) -> Self {
        self.texture = Some(path.into());
        self
    }

    pub fn white() -> Self {
        Self::new(Vec4::ONE)
    }

    pub fn transparent(color: Vec4) -> Self {
        let mut mat = Self::new(color);
        mat.alpha_mode = AlphaMode::Blend;
        mat
    }
}

impl Material for UnlitMaterial {
    fn name(&self) -> &str {
        &self.name
    }

    fn shader_kind(&self) -> ShaderKind {
        ShaderKind::Unlit
    }

    fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Blend)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl Default for UnlitMaterial {
    fn default() -> Self {
        Self::white()
    }
}

// ── 2D 精灵材质 ───────────────────────────────────────────────────────────────

/// 2D 精灵材质（优化用于批渲染）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteMaterial {
    pub name: String,
    /// 颜色调制（RGBA）
    pub modulate: Vec4,
    /// 纹理路径
    pub texture: Option<String>,
    /// 是否启用纹理图集（Atlas）
    pub atlas: Option<SpriteAtlas>,
    /// 混合模式
    pub blend_mode: BlendMode,
}

/// 纹理图集信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAtlas {
    /// 图集纹理路径
    pub texture: String,
    /// 总列数
    pub columns: u32,
    /// 总行数
    pub rows: u32,
    /// 单格宽度（像素）
    pub frame_width: u32,
    /// 单格高度（像素）
    pub frame_height: u32,
}

impl SpriteAtlas {
    /// 计算第 n 帧的 UV 坐标（归一化）
    pub fn frame_uvs(&self, frame: u32) -> [Vec2; 4] {
        let col = frame % self.columns;
        let row = frame / self.columns;
        // 图集总尺寸
        let total_w = (self.columns * self.frame_width) as f32;
        let total_h = (self.rows * self.frame_height) as f32;
        let u0 = (col * self.frame_width) as f32 / total_w;
        let v0 = (row * self.frame_height) as f32 / total_h;
        let u1 = u0 + self.frame_width as f32 / total_w;
        let v1 = v0 + self.frame_height as f32 / total_h;
        [
            Vec2::new(u0, v0), // 左上
            Vec2::new(u1, v0), // 右上
            Vec2::new(u1, v1), // 右下
            Vec2::new(u0, v1), // 左下
        ]
    }

    /// 总帧数
    pub fn frame_count(&self) -> u32 {
        self.columns * self.rows
    }
}

impl SpriteMaterial {
    pub fn new() -> Self {
        Self {
            name: "Sprite".into(),
            modulate: Vec4::ONE,
            texture: None,
            atlas: None,
            blend_mode: BlendMode::Alpha,
        }
    }

    pub fn with_texture(mut self, path: impl Into<String>) -> Self {
        self.texture = Some(path.into());
        self
    }
}

impl Default for SpriteMaterial {
    fn default() -> Self {
        Self::new()
    }
}

// ── 枚举类型 ──────────────────────────────────────────────────────────────────

/// Alpha 混合模式（参考 glTF 规范）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    /// 完全不透明
    Opaque,
    /// Alpha 遮罩（小于 cutoff 的像素完全透明）
    Mask,
    /// Alpha 混合（半透明）
    Blend,
}

/// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    /// 正常 Alpha 混合
    Alpha,
    /// 加法混合（粒子效果）
    Additive,
    /// 乘法混合
    Multiply,
    /// 不透明
    Opaque,
}

// ── 材质注册表 ────────────────────────────────────────────────────────────────

/// 材质注册表 - 缓存已创建的材质
pub struct MaterialRegistry {
    pbr_materials: std::collections::HashMap<String, PbrMaterial>,
    unlit_materials: std::collections::HashMap<String, UnlitMaterial>,
    sprite_materials: std::collections::HashMap<String, SpriteMaterial>,
}

impl MaterialRegistry {
    pub fn new() -> Self {
        Self {
            pbr_materials: std::collections::HashMap::new(),
            unlit_materials: std::collections::HashMap::new(),
            sprite_materials: std::collections::HashMap::new(),
        }
    }

    pub fn add_pbr(&mut self, material: PbrMaterial) {
        self.pbr_materials.insert(material.name.clone(), material);
    }

    pub fn add_unlit(&mut self, material: UnlitMaterial) {
        self.unlit_materials.insert(material.name.clone(), material);
    }

    pub fn add_sprite(&mut self, material: SpriteMaterial) {
        self.sprite_materials
            .insert(material.name.clone(), material);
    }

    pub fn get_pbr(&self, name: &str) -> Option<&PbrMaterial> {
        self.pbr_materials.get(name)
    }

    pub fn get_unlit(&self, name: &str) -> Option<&UnlitMaterial> {
        self.unlit_materials.get(name)
    }

    pub fn get_sprite(&self, name: &str) -> Option<&SpriteMaterial> {
        self.sprite_materials.get(name)
    }
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self::new()
    }
}
