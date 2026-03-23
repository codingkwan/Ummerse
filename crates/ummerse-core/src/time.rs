//! 时间管理 - 帧时间、物理时间、定时器

use std::time::{Duration, Instant};

// ── 全局时间 ──────────────────────────────────────────────────────────────────

/// 引擎全局时间资源
///
/// 每帧由主循环更新，提供 delta time、elapsed time 等时间信息。
#[derive(Debug, Clone)]
pub struct Time {
    /// 上一帧到本帧的时间差（秒，限制后）
    delta: f32,
    /// 原始帧时间差（未限制）
    raw_delta: f32,
    /// 引擎启动后总运行时间（秒）
    elapsed: f64,
    /// 总帧计数
    frame: u64,
    /// 物理固定步长（秒）
    physics_delta: f32,
    /// 启动时刻
    start: Instant,
    /// 上一帧时刻
    last_frame: Option<Instant>,
    /// 最大允许 delta time（防止死亡螺旋）
    max_delta: f32,
    /// 时间缩放（1.0 = 正常，0.5 = 慢动作，2.0 = 快进）
    time_scale: f32,
}

impl Time {
    /// 创建新时间资源
    pub fn new() -> Self {
        Self {
            delta: 0.0,
            raw_delta: 0.0,
            elapsed: 0.0,
            frame: 0,
            physics_delta: 1.0 / 60.0,
            start: Instant::now(),
            last_frame: None,
            max_delta: 0.1,
            time_scale: 1.0,
        }
    }

    /// 每帧开始时更新时间（由主循环调用）
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.raw_delta = if let Some(last) = self.last_frame {
            now.duration_since(last).as_secs_f32()
        } else {
            self.physics_delta
        };
        self.last_frame = Some(now);

        // 限制 delta 防止卡顿导致的"时间追赶"
        self.delta = self.raw_delta.min(self.max_delta) * self.time_scale;
        self.elapsed += self.delta as f64;
        self.frame += 1;
    }

    /// 本帧 delta time（经时间缩放和最大限制处理）
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// 本帧 delta time（以 Duration 形式返回）
    #[inline]
    pub fn delta_duration(&self) -> Duration {
        Duration::from_secs_f32(self.delta)
    }

    /// 原始 delta（未经时间缩放）
    #[inline]
    pub fn raw_delta(&self) -> f32 {
        self.raw_delta
    }

    /// 引擎总运行时间（秒）
    #[inline]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// 引擎总运行时间（Duration）
    #[inline]
    pub fn elapsed_duration(&self) -> Duration {
        Duration::from_secs_f64(self.elapsed)
    }

    /// 当前帧数
    #[inline]
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// 物理步长（秒）
    #[inline]
    pub fn physics_delta(&self) -> f32 {
        self.physics_delta
    }

    /// 设置物理步长
    pub fn set_physics_fps(&mut self, fps: u32) {
        self.physics_delta = 1.0 / fps.max(1) as f32;
    }

    /// 当前时间缩放
    #[inline]
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// 设置时间缩放（0.0 ~ 10.0 合理范围）
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 10.0);
    }

    /// 设置最大 delta time
    pub fn set_max_delta(&mut self, max: f32) {
        self.max_delta = max.max(0.001);
    }

    /// 自启动以来的真实时间（不受 time_scale 影响）
    #[inline]
    pub fn real_elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// 当前 FPS（基于 raw_delta 计算）
    #[inline]
    pub fn fps(&self) -> f32 {
        if self.raw_delta > 0.0 {
            1.0 / self.raw_delta
        } else {
            0.0
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

// ── 定时器 ────────────────────────────────────────────────────────────────────

/// 定时器模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    /// 触发一次后停止
    Once,
    /// 循环触发
    Repeating,
}

/// 定时器 - 按指定间隔触发
///
/// 需要每帧手动调用 `tick(delta)` 更新。
#[derive(Debug, Clone)]
pub struct Timer {
    /// 触发间隔（秒）
    duration: f32,
    /// 累计时间
    elapsed: f32,
    /// 定时器模式
    mode: TimerMode,
    /// 是否已触发（本帧）
    just_finished: bool,
    /// 是否已停止（Once 模式触发后）
    finished: bool,
    /// 是否暂停
    paused: bool,
}

impl Timer {
    /// 创建一次性定时器
    pub fn once(duration_secs: f32) -> Self {
        Self {
            duration: duration_secs.max(f32::EPSILON),
            elapsed: 0.0,
            mode: TimerMode::Once,
            just_finished: false,
            finished: false,
            paused: false,
        }
    }

    /// 创建循环定时器
    pub fn repeating(duration_secs: f32) -> Self {
        Self {
            duration: duration_secs.max(f32::EPSILON),
            elapsed: 0.0,
            mode: TimerMode::Repeating,
            just_finished: false,
            finished: false,
            paused: false,
        }
    }

    /// 每帧更新（传入 delta time）
    pub fn tick(&mut self, delta: f32) {
        if self.paused || self.finished {
            self.just_finished = false;
            return;
        }

        self.elapsed += delta;
        if self.elapsed >= self.duration {
            self.just_finished = true;
            match self.mode {
                TimerMode::Once => {
                    self.finished = true;
                    self.elapsed = self.duration;
                }
                TimerMode::Repeating => {
                    // 保留余量，支持高精度累积
                    self.elapsed %= self.duration;
                }
            }
        } else {
            self.just_finished = false;
        }
    }

    /// 本帧是否触发
    #[inline]
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }

    /// 是否已完成（Once 模式）
    #[inline]
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// 已过时间（秒）
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// 完成进度（0.0 ~ 1.0）
    #[inline]
    pub fn fraction(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }

    /// 剩余时间（秒）
    #[inline]
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }

    /// 暂停/恢复
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// 重置定时器
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.just_finished = false;
        self.finished = false;
    }

    /// 定时器总时长（秒）
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_once() {
        let mut timer = Timer::once(1.0);
        timer.tick(0.5);
        assert!(!timer.just_finished());
        timer.tick(0.6); // 总计 1.1 秒
        assert!(timer.just_finished());
        assert!(timer.finished());
        timer.tick(1.0);
        // Once 触发后不再触发
        assert!(!timer.just_finished());
    }

    #[test]
    fn test_timer_repeating() {
        let mut timer = Timer::repeating(1.0);
        timer.tick(1.1);
        assert!(timer.just_finished());
        timer.tick(0.5);
        assert!(!timer.just_finished());
        timer.tick(0.6);
        assert!(timer.just_finished()); // 第二次触发
    }

    #[test]
    fn test_time_tick() {
        let mut time = Time::new();
        time.tick();
        assert_eq!(time.frame(), 1);
        assert!(time.delta() >= 0.0);
        assert!(time.elapsed() >= 0.0);
    }

    #[test]
    fn test_timer_fraction() {
        let mut timer = Timer::once(2.0);
        timer.tick(1.0);
        assert!((timer.fraction() - 0.5).abs() < f32::EPSILON);
    }
}
