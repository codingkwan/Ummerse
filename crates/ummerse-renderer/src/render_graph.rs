//! 渲染图（Render Graph）- 声明式渲染管线调度

use std::collections::HashMap;

/// 渲染通道（Pass）标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPassId(String);

impl RenderPassId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// 渲染通道描述
pub struct RenderPassDesc {
    pub id: RenderPassId,
    pub label: String,
    /// 此通道依赖的其他通道（决定执行顺序）
    pub depends_on: Vec<RenderPassId>,
}

/// 渲染图 - 管理渲染通道的依赖关系和执行顺序
pub struct RenderGraph {
    passes: HashMap<RenderPassId, RenderPassDesc>,
    /// 执行顺序（拓扑排序后）
    order: Vec<RenderPassId>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// 添加渲染通道
    pub fn add_pass(&mut self, desc: RenderPassDesc) {
        self.passes.insert(desc.id.clone(), desc);
        self.rebuild_order();
    }

    /// 重建拓扑排序执行顺序（简化版 Kahn 算法）
    fn rebuild_order(&mut self) {
        let mut in_degree: HashMap<&RenderPassId, usize> = HashMap::new();
        for (id, desc) in &self.passes {
            in_degree.entry(id).or_insert(0);
            for dep in &desc.depends_on {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<&RenderPassId> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(id, _)| *id)
            .collect();
        queue.sort_by_key(|id| id.0.as_str());

        let mut order = Vec::new();
        while let Some(id) = queue.first().cloned() {
            queue.remove(0);
            order.push(id.clone());
            if let Some(desc) = self.passes.get(id) {
                for dep in &desc.depends_on {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d = d.saturating_sub(1);
                        if *d == 0 {
                            queue.push(dep);
                        }
                    }
                }
            }
        }
        self.order = order;
    }

    /// 获取执行顺序
    pub fn execution_order(&self) -> &[RenderPassId] {
        &self.order
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ── 内置通道 ID ───────────────────────────────────────────────────────────────

pub mod passes {
    use super::RenderPassId;

    pub fn shadow() -> RenderPassId { RenderPassId::new("shadow") }
    pub fn opaque_3d() -> RenderPassId { RenderPassId::new("opaque_3d") }
    pub fn transparent_3d() -> RenderPassId { RenderPassId::new("transparent_3d") }
    pub fn sprite_2d() -> RenderPassId { RenderPassId::new("sprite_2d") }
    pub fn ui() -> RenderPassId { RenderPassId::new("ui") }
    pub fn post_process() -> RenderPassId { RenderPassId::new("post_process") }
    pub fn tonemapping() -> RenderPassId { RenderPassId::new("tonemapping") }
}
