//! 游戏主循环管理
//!
//! 实现固定时间步物理 + 可变帧率渲染的经典游戏循环，
//! 参考 "Fix Your Timestep!" 文章的半固定步进模式。

use std::time::{Duration, Instant};

/// 主循环配置
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// 目标渲染帧率（0 = 不限制）
    pub target_fps: u32,
    /// 物理更新频率（Hz）
    pub physics_fps: u32,
    /// 最大允许帧时间（防止死亡螺旋）
    pub max_delta: f32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            physics_fps: 60,
            max_delta: 0.1,
        }
    }
}

/// 帧状态 - 每帧传递给系统的时间信息
#[derive(Debug, Clone, Copy)]
pub struct FrameState {
    /// 本帧渲染 delta（秒）
    pub delta: f32,
    /// 物理固定步长（秒）
    pub physics_delta: f32,
    /// 本帧需要执行的物理步数
    pub physics_steps: u32,
    /// 物理插值因子（0.0 ~ 1.0，用于平滑渲染）
    pub alpha: f32,
    /// 总运行时间（秒）
    pub elapsed: f64,
    /// 帧计数
    pub frame: u64,
}

/// 游戏主循环控制器
#[derive(Debug)]
pub struct GameLoop {
    config: LoopConfig,
    /// 物理步长（秒）
    physics_delta: f32,
    /// 目标帧时长（None = 不限制）
    target_frame_time: Option<Duration>,
    /// 上一帧时刻
    last_time: Option<Instant>,
    /// 物理时间累积器
    accumulator: f32,
    /// 总运行时间
    elapsed: f64,
    /// 帧计数
    frame: u64,
}

impl GameLoop {
    pub fn new(config: LoopConfig) -> Self {
        let physics_delta = 1.0 / config.physics_fps.max(1) as f32;
        let target_frame_time = if config.target_fps > 0 {
            Some(Duration::from_secs_f64(1.0 / config.target_fps as f64))
        } else {
            None
        };
        Self {
            config,
            physics_delta,
            target_frame_time,
            last_time: None,
            accumulator: 0.0,
            elapsed: 0.0,
            frame: 0,
        }
    }

    /// 开始一帧，返回帧状态
    ///
    /// 这是半固定步进算法核心：
    /// 1. 测量真实 delta time
    /// 2. 限制最大 delta（防止死亡螺旋）
    /// 3. 累加到物理累积器
    /// 4. 计算需要执行几步物理
    /// 5. 计算渲染插值因子
    pub fn begin_frame(&mut self) -> FrameState {
        let now = Instant::now();

        // 计算真实 delta time
        let raw_delta = if let Some(last) = self.last_time {
            now.duration_since(last).as_secs_f32()
        } else {
            self.physics_delta // 第一帧使用物理步长
        };
        self.last_time = Some(now);

        // 限制最大 delta（防止死亡螺旋）
        let delta = raw_delta.min(self.config.max_delta);
        self.elapsed += delta as f64;

        // 累加到物理累积器
        self.accumulator += delta;

        // 计算本帧应执行的物理步数
        let physics_steps = (self.accumulator / self.physics_delta) as u32;
        let physics_steps = physics_steps.min(8); // 最多 8 步防止卡顿雪崩
        self.accumulator -= physics_steps as f32 * self.physics_delta;
        self.accumulator = self.accumulator.max(0.0);

        // 渲染插值因子（剩余累积器 / 步长）
        let alpha = self.accumulator / self.physics_delta;

        self.frame += 1;

        FrameState {
            delta,
            physics_delta: self.physics_delta,
            physics_steps,
            alpha,
            elapsed: self.elapsed,
            frame: self.frame,
        }
    }

    /// 结束一帧，必要时休眠以维持目标帧率
    pub fn end_frame(&self, frame_start: Instant) {
        if let Some(target) = self.target_frame_time {
            let elapsed = frame_start.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        }
    }

    /// 重置循环状态（场景切换时使用）
    pub fn reset(&mut self) {
        self.last_time = None;
        self.accumulator = 0.0;
    }

    /// 当前帧计数
    #[inline]
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// 总运行时间（秒）
    #[inline]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new(LoopConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_config_defaults() {
        let config = LoopConfig::default();
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.physics_fps, 60);
    }

    #[test]
    fn test_game_loop_begin_frame() {
        let mut game_loop = GameLoop::new(LoopConfig::default());
        let state = game_loop.begin_frame();
        assert_eq!(state.frame, 1);
        assert!(state.delta > 0.0);
        assert!(state.alpha >= 0.0 && state.alpha <= 1.0);
    }
}
