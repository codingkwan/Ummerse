//! 渲染图（Render Graph）- 声明式渲染管线编排
//!
//! 参考 Bevy 渲染图设计，提供节点式渲染管线编排：
//! - 渲染节点（RenderNode）：封装单个渲染 Pass
//! - 渲染图（RenderGraph）：DAG 式节点依赖管理
//! - 帧渲染上下文（FrameContext）：每帧传递的 GPU 状态

use std::collections::HashMap;

/// 渲染 Pass 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    /// 深度预处理 Pass
    DepthPrepass,
    /// 3D 不透明物体渲染
    Opaque3d,
    /// 3D 半透明物体渲染（后排序）
    Transparent3d,
    /// 2D 精灵渲染
    Sprite2d,
    /// UI 渲染
    Ui,
    /// 后处理（色调映射等）
    PostProcess,
    /// 阴影贴图生成
    Shadow,
    /// 自定义 Pass
    Custom(u32),
}

/// 渲染资源标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 内置渲染资源 ID
pub mod resources {
    use super::ResourceId;
    pub fn surface() -> ResourceId { ResourceId::new("surface") }
    pub fn hdr_target() -> ResourceId { ResourceId::new("hdr_target") }
    pub fn depth_buffer() -> ResourceId { ResourceId::new("depth_buffer") }
    pub fn shadow_map() -> ResourceId { ResourceId::new("shadow_map") }
    pub fn post_process_output() -> ResourceId { ResourceId::new("post_process_output") }
}

/// 渲染节点 - 封装单个 GPU Pass
pub struct RenderNode {
    pub id: String,
    pub kind: PassKind,
    /// 输入资源依赖
    pub inputs: Vec<ResourceId>,
    /// 输出资源
    pub outputs: Vec<ResourceId>,
    /// 是否启用
    pub enabled: bool,
    /// 执行优先级（数字小先执行）
    pub priority: i32,
}

impl RenderNode {
    pub fn new(id: impl Into<String>, kind: PassKind) -> Self {
        Self {
            id: id.into(),
            kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            priority: 0,
        }
    }

    pub fn with_input(mut self, resource: ResourceId) -> Self {
        self.inputs.push(resource);
        self
    }

    pub fn with_output(mut self, resource: ResourceId) -> Self {
        self.outputs.push(resource);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 渲染图 - 管理所有渲染 Pass 的执行顺序
pub struct RenderGraph {
    nodes: HashMap<String, RenderNode>,
    /// 拓扑排序后的执行顺序
    sorted_nodes: Vec<String>,
    dirty: bool,
}

impl RenderGraph {
    pub fn new() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            sorted_nodes: Vec::new(),
            dirty: true,
        };
        // 注册默认 Pass
        graph.register_defaults();
        graph
    }

    /// 注册默认渲染管线
    fn register_defaults(&mut self) {
        use resources::*;

        self.add_node(
            RenderNode::new("depth_prepass", PassKind::DepthPrepass)
                .with_output(depth_buffer())
                .with_priority(-100),
        );

        self.add_node(
            RenderNode::new("shadow_pass", PassKind::Shadow)
                .with_output(shadow_map())
                .with_priority(-90),
        );

        self.add_node(
            RenderNode::new("opaque_3d", PassKind::Opaque3d)
                .with_input(depth_buffer())
                .with_input(shadow_map())
                .with_output(hdr_target())
                .with_priority(0),
        );

        self.add_node(
            RenderNode::new("transparent_3d", PassKind::Transparent3d)
                .with_input(depth_buffer())
                .with_input(hdr_target())
                .with_output(hdr_target())
                .with_priority(10),
        );

        self.add_node(
            RenderNode::new("sprite_2d", PassKind::Sprite2d)
                .with_output(hdr_target())
                .with_priority(20),
        );

        self.add_node(
            RenderNode::new("post_process", PassKind::PostProcess)
                .with_input(hdr_target())
                .with_output(surface())
                .with_priority(90),
        );

        self.add_node(
            RenderNode::new("ui", PassKind::Ui)
                .with_output(surface())
                .with_priority(100),
        );
    }

    /// 添加渲染节点
    pub fn add_node(&mut self, node: RenderNode) {
        self.nodes.insert(node.id.clone(), node);
        self.dirty = true;
    }

    /// 移除渲染节点
    pub fn remove_node(&mut self, id: &str) -> bool {
        let removed = self.nodes.remove(id).is_some();
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// 启用/禁用节点
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.enabled = enabled;
        }
    }

    /// 获取已排序的执行顺序
    pub fn sorted_passes(&mut self) -> Vec<&RenderNode> {
        if self.dirty {
            self.sort();
        }
        self.sorted_nodes
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .filter(|n| n.enabled)
            .collect()
    }

    /// 拓扑排序（按 priority 排序，未来可改为 Kahn 算法）
    fn sort(&mut self) {
        let mut ids: Vec<String> = self.nodes.keys().cloned().collect();
        ids.sort_by_key(|id| self.nodes[id].priority);
        self.sorted_nodes = ids;
        self.dirty = false;
    }

    /// 获取节点
    pub fn get_node(&self, id: &str) -> Option<&RenderNode> {
        self.nodes.get(id)
    }

    /// 节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ── 帧渲染上下文 ──────────────────────────────────────────────────────────────

/// 每帧渲染时传递给所有 Pass 的共享状态
pub struct FrameContext<'frame> {
    pub device: &'frame wgpu::Device,
    pub queue: &'frame wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub surface_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub delta_time: f32,
    pub frame_index: u64,
}

impl<'frame> FrameContext<'frame> {
    pub fn new(
        device: &'frame wgpu::Device,
        queue: &'frame wgpu::Queue,
        surface_view: wgpu::TextureView,
        width: u32,
        height: u32,
        delta_time: f32,
        frame_index: u64,
    ) -> Self {
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("Frame {} Encoder", frame_index)),
        });
        Self {
            device,
            queue,
            encoder,
            surface_view,
            width,
            height,
            delta_time,
            frame_index,
        }
    }

    /// 提交当前帧的命令缓冲区
    pub fn submit(self) {
        let cmd = self.encoder.finish();
        self.queue.submit(std::iter::once(cmd));
    }

    /// 当前宽高比
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_graph_default_passes() {
        let mut graph = RenderGraph::new();
        let passes = graph.sorted_passes();
        assert!(passes.len() >= 5, "Expected at least 5 default render passes");

        // 检查顺序：depth_prepass 优先级最低（最先执行）
        assert_eq!(passes[0].id, "depth_prepass");
        // UI 最后
        assert_eq!(passes.last().unwrap().id, "ui");
    }

    #[test]
    fn test_disable_pass() {
        let mut graph = RenderGraph::new();
        graph.set_enabled("shadow_pass", false);
        let passes = graph.sorted_passes();
        assert!(!passes.iter().any(|p| p.id == "shadow_pass"));
    }
}
