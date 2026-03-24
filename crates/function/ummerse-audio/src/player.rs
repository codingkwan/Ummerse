//! 音频播放器

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// 停止（未播放）
    Stopped,
    /// 正在播放
    Playing,
    /// 暂停
    Paused,
    /// 播放完成
    Finished,
}

/// 音频播放句柄 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId(Uuid);

impl PlaybackId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlaybackId {
    fn default() -> Self {
        Self::new()
    }
}

/// 音频播放参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackParams {
    /// 音量（0.0 ~ 1.0）
    pub volume: f32,
    /// 音调（0.5 = 低八度，1.0 = 原始，2.0 = 高八度）
    pub pitch: f32,
    /// 是否循环播放
    pub looping: bool,
    /// 开始时间（秒）
    pub start_time: f32,
    /// 渐入时间（秒，0.0 = 无渐入）
    pub fade_in: f32,
    /// 目标音频总线名称
    pub bus: String,
}

impl Default for PlaybackParams {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            start_time: 0.0,
            fade_in: 0.0,
            bus: "SFX".to_string(),
        }
    }
}

/// 音频播放实例
#[derive(Debug)]
pub struct AudioPlayback {
    pub id: PlaybackId,
    pub audio_path: String,
    pub params: PlaybackParams,
    pub state: PlaybackState,
    /// 当前播放位置（秒）
    pub position: f32,
    /// 总时长（秒）
    pub duration: f32,
    /// 当前实际音量（考虑渐入渐出）
    current_volume: f32,
}

impl AudioPlayback {
    pub fn new(audio_path: impl Into<String>, params: PlaybackParams, duration: f32) -> Self {
        Self {
            id: PlaybackId::new(),
            audio_path: audio_path.into(),
            params,
            state: PlaybackState::Stopped,
            position: 0.0,
            duration,
            current_volume: 0.0,
        }
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
        self.position = self.params.start_time;
        if self.params.fade_in <= 0.0 {
            self.current_volume = self.params.volume;
        } else {
            self.current_volume = 0.0;
        }
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    /// 恢复播放
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
        }
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.position = 0.0;
    }

    /// 更新（每帧调用）
    pub fn update(&mut self, delta: f32) {
        if self.state != PlaybackState::Playing {
            return;
        }

        // 渐入处理
        if self.params.fade_in > 0.0 && self.current_volume < self.params.volume {
            self.current_volume = (self.current_volume
                + delta / self.params.fade_in * self.params.volume)
                .min(self.params.volume);
        }

        // 推进播放位置
        self.position += delta * self.params.pitch;

        // 检查是否播放完毕
        if self.position >= self.duration {
            if self.params.looping {
                self.position = self.params.start_time;
            } else {
                self.state = PlaybackState::Finished;
            }
        }
    }

    /// 当前有效音量
    #[inline]
    pub fn effective_volume(&self) -> f32 {
        self.current_volume
    }

    /// 播放进度（0.0 ~ 1.0）
    #[inline]
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.position / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// 是否正在播放
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// 是否播放完成
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.state == PlaybackState::Finished
    }
}

/// 音频播放器组件 - 管理多个播放实例
pub struct AudioPlayer {
    /// 当前所有播放实例
    playbacks: Vec<AudioPlayback>,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("playback_count", &self.playbacks.len())
            .finish_non_exhaustive()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            playbacks: Vec::new(),
        }
    }

    /// 播放音频，返回播放 ID
    pub fn play(
        &mut self,
        audio_path: impl Into<String>,
        params: PlaybackParams,
        duration: f32,
    ) -> PlaybackId {
        let mut playback = AudioPlayback::new(audio_path, params, duration);
        let id = playback.id;
        playback.play();
        self.playbacks.push(playback);
        id
    }

    /// 停止指定播放实例
    pub fn stop(&mut self, id: PlaybackId) {
        if let Some(pb) = self.playbacks.iter_mut().find(|p| p.id == id) {
            pb.stop();
        }
    }

    /// 停止所有播放
    pub fn stop_all(&mut self) {
        for pb in &mut self.playbacks {
            pb.stop();
        }
    }

    /// 暂停指定播放
    pub fn pause(&mut self, id: PlaybackId) {
        if let Some(pb) = self.playbacks.iter_mut().find(|p| p.id == id) {
            pb.pause();
        }
    }

    /// 恢复指定播放
    pub fn resume(&mut self, id: PlaybackId) {
        if let Some(pb) = self.playbacks.iter_mut().find(|p| p.id == id) {
            pb.resume();
        }
    }

    /// 更新所有播放实例，清理已完成的
    pub fn update(&mut self, delta: f32) {
        for pb in &mut self.playbacks {
            pb.update(delta);
        }
        // 清理已停止/完成的播放
        self.playbacks
            .retain(|pb| pb.state != PlaybackState::Finished && pb.state != PlaybackState::Stopped);
    }

    /// 当前活跃播放数量
    pub fn active_count(&self) -> usize {
        self.playbacks.iter().filter(|p| p.is_playing()).count()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
