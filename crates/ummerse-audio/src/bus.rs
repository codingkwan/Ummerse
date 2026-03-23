//! 音频总线（混音通道）

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 音频效果器 trait
pub trait AudioEffect: Send + Sync + 'static {
    fn name(&self) -> &str;
    /// 处理音频采样（原地处理）
    fn process(&mut self, samples: &mut [f32], sample_rate: u32);
    /// 是否启用
    fn enabled(&self) -> bool {
        true
    }
}

/// 音频总线 - 类似混音台的通道
pub struct AudioBus {
    pub name: String,
    /// 总线音量（0.0 ~ 1.0+，可超过 1.0 增益）
    pub volume: f32,
    /// 是否静音
    pub muted: bool,
    /// 声像（-1.0 = 左，0.0 = 中，1.0 = 右）
    pub panning: f32,
    /// 发送到的父总线名称（None = 主输出）
    pub send_to: Option<String>,
    /// 挂载的音频效果器链
    effects: Vec<Box<dyn AudioEffect>>,
}

impl AudioBus {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: 1.0,
            muted: false,
            panning: 0.0,
            send_to: None,
            effects: Vec::new(),
        }
    }

    /// 添加音频效果器
    pub fn add_effect(&mut self, effect: impl AudioEffect) {
        self.effects.push(Box::new(effect));
    }

    /// 移除指定名称的效果器
    pub fn remove_effect(&mut self, name: &str) {
        self.effects.retain(|e| e.name() != name);
    }

    /// 处理音频数据（应用所有效果器和音量）
    pub fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        if self.muted {
            samples.fill(0.0);
            return;
        }

        // 应用所有效果器
        for effect in &mut self.effects {
            if effect.enabled() {
                effect.process(samples, sample_rate);
            }
        }

        // 应用音量和声像（交错立体声：左/右/左/右...）
        let left_gain = self.volume * (1.0 - self.panning.max(0.0));
        let right_gain = self.volume * (1.0 + self.panning.min(0.0));

        for (i, sample) in samples.iter_mut().enumerate() {
            let gain = if i % 2 == 0 { left_gain } else { right_gain };
            *sample *= gain;
        }
    }
}

impl std::fmt::Debug for AudioBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBus")
            .field("name", &self.name)
            .field("volume", &self.volume)
            .field("muted", &self.muted)
            .field("panning", &self.panning)
            .field("send_to", &self.send_to)
            .finish()
    }
}

// ── 内置效果器 ────────────────────────────────────────────────────────────────

/// 简单均衡器（低/中/高频增益）
pub struct EqEffect {
    pub name: String,
    pub low_gain: f32,
    pub mid_gain: f32,
    pub high_gain: f32,
    pub enabled: bool,
}

impl EqEffect {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
            enabled: true,
        }
    }
}

impl AudioEffect for EqEffect {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(&mut self, _samples: &mut [f32], _sample_rate: u32) {
        // 简化实现：直接应用全频增益（完整版需要 IIR 滤波器）
        // TODO: 实现真正的 EQ 滤波器
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

/// 混响效果器（Reverb）
pub struct ReverbEffect {
    pub name: String,
    /// 混响量（0.0 ~ 1.0）
    pub wet: f32,
    /// 干信号量（0.0 ~ 1.0）
    pub dry: f32,
    /// 房间大小（0.0 ~ 1.0）
    pub room_size: f32,
    /// 阻尼（0.0 ~ 1.0）
    pub damping: f32,
    pub enabled: bool,
    delay_buffer: Vec<f32>,
    write_pos: usize,
}

impl ReverbEffect {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            wet: 0.3,
            dry: 0.7,
            room_size: 0.5,
            damping: 0.5,
            enabled: true,
            delay_buffer: vec![0.0; 44100], // 1s @ 44100 Hz
            write_pos: 0,
        }
    }
}

impl AudioEffect for ReverbEffect {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let delay_samples = (self.room_size * sample_rate as f32 * 0.1) as usize;
        let delay_samples = delay_samples.max(1).min(self.delay_buffer.len() - 1);

        for sample in samples.iter_mut() {
            let read_pos = (self.write_pos + self.delay_buffer.len() - delay_samples)
                % self.delay_buffer.len();
            let delayed = self.delay_buffer[read_pos];

            self.delay_buffer[self.write_pos] = *sample + delayed * self.room_size * (1.0 - self.damping);
            self.write_pos = (self.write_pos + 1) % self.delay_buffer.len();

            *sample = *sample * self.dry + delayed * self.wet;
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

// ── 音频总线图 ─────────────────────────────────────────────────────────────────

/// 音频总线图 - 管理所有混音通道
pub struct AudioBusGraph {
    buses: HashMap<String, AudioBus>,
    /// 总线的处理顺序（拓扑排序后）
    order: Vec<String>,
}

impl AudioBusGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            buses: HashMap::new(),
            order: Vec::new(),
        };
        // 默认创建主总线
        graph.add_bus(AudioBus::new("Master"));
        graph.add_bus(AudioBus {
            name: "Music".to_string(),
            volume: 1.0,
            muted: false,
            panning: 0.0,
            send_to: Some("Master".to_string()),
            effects: Vec::new(),
        });
        graph.add_bus(AudioBus {
            name: "SFX".to_string(),
            volume: 1.0,
            muted: false,
            panning: 0.0,
            send_to: Some("Master".to_string()),
            effects: Vec::new(),
        });
        graph
    }

    /// 添加总线
    pub fn add_bus(&mut self, bus: AudioBus) {
        let name = bus.name.clone();
        self.buses.insert(name.clone(), bus);
        self.rebuild_order();
    }

    /// 获取总线
    pub fn get_bus(&self, name: &str) -> Option<&AudioBus> {
        self.buses.get(name)
    }

    /// 获取可变总线
    pub fn get_bus_mut(&mut self, name: &str) -> Option<&mut AudioBus> {
        self.buses.get_mut(name)
    }

    /// 移除总线
    pub fn remove_bus(&mut self, name: &str) -> bool {
        if name == "Master" {
            return false; // 不允许删除主总线
        }
        let removed = self.buses.remove(name).is_some();
        if removed {
            self.rebuild_order();
        }
        removed
    }

    /// 重建处理顺序（叶总线先处理，主总线最后）
    fn rebuild_order(&mut self) {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit(
            name: &str,
            buses: &HashMap<String, AudioBus>,
            order: &mut Vec<String>,
            visited: &mut std::collections::HashSet<String>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name.to_string());
            // 先处理子总线
            for (bus_name, bus) in buses {
                if bus.send_to.as_deref() == Some(name) {
                    visit(bus_name, buses, order, visited);
                }
            }
            order.push(name.to_string());
        }

        visit("Master", &self.buses, &mut order, &mut visited);
        self.order = order;
    }

    /// 处理顺序
    pub fn processing_order(&self) -> &[String] {
        &self.order
    }
}

impl Default for AudioBusGraph {
    fn default() -> Self {
        Self::new()
    }
}
