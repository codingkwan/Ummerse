//! # Ummerse Audio
//!
//! 音频系统，提供：
//! - 音频资产加载（WAV/OGG/MP3）
//! - 音频播放器（支持循环/音量/音调控制）
//! - 3D 空间音频
//! - 音频总线（混音通道）

pub mod bus;
pub mod player;
pub mod spatial;

pub use bus::{AudioBus, AudioBusGraph};
pub use player::{AudioPlayer, PlaybackState};
pub use spatial::SpatialAudioSource;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 音频系统错误
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("音频加载失败: {0}")]
    LoadFailed(String),
    #[error("音频解码失败: {0}")]
    DecodeFailed(String),
    #[error("音频播放失败: {0}")]
    PlaybackFailed(String),
    #[error("音频总线未找到: {0}")]
    BusNotFound(String),
}

/// 音频系统全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 主音量（0.0 ~ 1.0）
    pub master_volume: f32,
    /// 采样率（Hz）
    pub sample_rate: u32,
    /// 缓冲区大小（帧数）
    pub buffer_size: u32,
    /// 最大同时播放音频数
    pub max_concurrent_sounds: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sample_rate: 44100,
            buffer_size: 1024,
            max_concurrent_sounds: 64,
        }
    }
}

/// 音频管理器 - 全局音频系统入口
pub struct AudioManager {
    pub config: AudioConfig,
    bus_graph: AudioBusGraph,
}

impl AudioManager {
    pub fn new(config: AudioConfig) -> Self {
        Self {
            bus_graph: AudioBusGraph::new(),
            config,
        }
    }

    /// 获取音频总线
    pub fn bus(&self, name: &str) -> Option<&AudioBus> {
        self.bus_graph.get_bus(name)
    }

    /// 获取可变音频总线
    pub fn bus_mut(&mut self, name: &str) -> Option<&mut AudioBus> {
        self.bus_graph.get_bus_mut(name)
    }

    /// 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.master_volume = volume.clamp(0.0, 1.0);
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new(AudioConfig::default())
    }
}
