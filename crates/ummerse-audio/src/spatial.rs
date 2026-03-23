//! 3D 空间音频

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// 3D 空间音频源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioSource {
    /// 音频文件路径
    pub audio_path: String,
    /// 世界空间位置
    pub position: Vec3,
    /// 最大听距（超出后无声）
    pub max_distance: f32,
    /// 参考距离（在此距离内音量为最大）
    pub ref_distance: f32,
    /// 衰减模型
    pub attenuation: AttenuationModel,
    /// 基础音量
    pub volume: f32,
    /// 是否循环
    pub looping: bool,
    /// 音频总线名称
    pub bus: String,
}

impl SpatialAudioSource {
    pub fn new(audio_path: impl Into<String>, position: Vec3) -> Self {
        Self {
            audio_path: audio_path.into(),
            position,
            max_distance: 50.0,
            ref_distance: 1.0,
            attenuation: AttenuationModel::InverseDistance,
            volume: 1.0,
            looping: false,
            bus: "SFX".to_string(),
        }
    }

    /// 计算在给定监听器位置处的有效音量
    pub fn volume_at(&self, listener_pos: Vec3) -> f32 {
        let distance = self.position.distance(listener_pos);
        if distance >= self.max_distance {
            return 0.0;
        }
        let effective_distance = distance.max(self.ref_distance);
        let attenuation = self.attenuation.compute(effective_distance, self.ref_distance, self.max_distance);
        self.volume * attenuation
    }

    /// 计算立体声声像（-1.0 = 全左，1.0 = 全右）
    pub fn panning_at(&self, listener_pos: Vec3, listener_forward: Vec3, listener_right: Vec3) -> f32 {
        let to_source = (self.position - listener_pos).normalize_or_zero();
        // 投影到监听者右方向
        let pan = to_source.dot(listener_right).clamp(-1.0, 1.0);
        pan
    }
}

/// 音频衰减模型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttenuationModel {
    /// 线性衰减（距离线性降低）
    Linear,
    /// 逆距离衰减（更真实）
    InverseDistance,
    /// 指数衰减（非常快速降低）
    Exponential,
    /// 无衰减（全距离等音量）
    None,
}

impl AttenuationModel {
    /// 计算给定距离的衰减系数（0.0 ~ 1.0）
    pub fn compute(&self, distance: f32, ref_distance: f32, max_distance: f32) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Linear => {
                1.0 - (distance - ref_distance) / (max_distance - ref_distance).max(f32::EPSILON)
            }
            Self::InverseDistance => {
                ref_distance / (ref_distance + (distance - ref_distance))
            }
            Self::Exponential => {
                (ref_distance / distance).powi(2)
            }
        }
        .clamp(0.0, 1.0)
    }
}

/// 3D 音频监听器（通常跟随相机）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl AudioListener {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            forward: Vec3::NEG_Z,
            right: Vec3::X,
            up: Vec3::Y,
        }
    }

    /// 更新朝向（基于旋转四元数）
    pub fn set_orientation(&mut self, forward: Vec3, up: Vec3) {
        self.forward = forward.normalize_or_zero();
        self.up = up.normalize_or_zero();
        self.right = self.forward.cross(self.up).normalize_or_zero();
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}
