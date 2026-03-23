//! 时间管理系统

use std::time::{Duration, Instant};

/// 引擎时间管理
#[derive(Debug)]
pub struct Time {
    /// 当前帧的 delta time（秒）
    delta: f32,
    /// delta time（Duration 格式）
    delta_duration: Duration,
    /// 游戏启动以来的总时间（秒）
    elapsed: f64,
    /// 物理步进时间（秒）
    physics_delta: f32,
    /// 帧计数
    frame_count: u64,
    /// FPS（每秒帧数）
    fps: f32,
    /// 时间缩放（1.0 = 正常速度）
    time_scale: f32,
    /// 上一帧的时刻
    last_frame: Option<Instant>,
    /// FPS 平滑窗口
    fps_samples: Vec<f32>,
}

impl Time {
    pub const PHYSICS_DELTA: f32 = 1.0 / 60.0;

    pub fn new() -> Self {
        Self {
            delta: 0.0,
            delta_duration: Duration::ZERO,
            elapsed: 0.0,
            physics_delta: Self::PHYSICS_DELTA,
            frame_count: 0,
            fps: 0.0,
            time_scale: 1.0,
            last_frame: None,
            fps_samples: Vec::with_capacity(60),
        }
    }

    /// 每帧开始时调用，更新时间状态
    pub fn tick(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_frame {
            self.delta_duration = now.duration_since(last);
            self.delta = self.delta_duration.as_secs_f32() * self.time_scale;
            self.elapsed += self.delta_duration.as_secs_f64();

            // FPS 平滑计算
            if self.delta > 0.0 {
                self.fps_samples.push(1.0 / self.delta_duration.as_secs_f32());
                if self.fps_samples.len() > 60 {
                    self.fps_samples.remove(0);
                }
                self.fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
            }
        }
        self.last_frame = Some(now);
        self.frame_count += 1;
    }

    /// 当前帧 delta time（秒，受时间缩放影响）
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// 原始 delta time（不受时间缩放影响）
    #[inline]
    pub fn raw_delta(&self) -> f32 {
        self.delta_duration.as_secs_f32()
    }

    /// 游戏运行总时间（秒）
    #[inline]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// 物理步进时间（秒）
    #[inline]
    pub fn physics_delta(&self) -> f32 {
        self.physics_delta
    }

    /// 当前帧数
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// 当前 FPS（平滑值）
    #[inline]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// 时间缩放系数
    #[inline]
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// 设置时间缩放（0.0 = 暂停，1.0 = 正常，2.0 = 双倍速）
    #[inline]
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// 设置物理步进时间
    #[inline]
    pub fn set_physics_delta(&mut self, delta: f32) {
        self.physics_delta = delta.clamp(1.0 / 240.0, 1.0 / 10.0);
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
