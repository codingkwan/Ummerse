//! 引擎系统注册 - Bevy ECS 系统集成
//!
//! 将物理、场景、脚本、音频等子系统注册为 Bevy 系统，
//! 统一调度顺序和数据流。

use ummerse_core::time::Time;

/// 系统集标签（用于调度顺序控制）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemSet {
    /// 输入处理（最先）
    Input,
    /// 脚本 process() 调用
    Script,
    /// 物理步进
    Physics,
    /// 场景树同步
    SceneSync,
    /// 动画更新
    Animation,
    /// 音频更新
    Audio,
    /// 渲染提交（最后）
    Render,
}

/// 物理系统更新（独立于渲染帧率）
pub fn physics_update_system(/* bevy resources */) {
    // TODO: 从 Bevy Resources 获取 PhysicsWorld，执行步进
    // 当 Bevy ECS 完整集成后实现
}

/// 脚本系统更新
pub fn script_update_system(/* bevy resources */) {
    // TODO: 调用所有带 ScriptComponent 的节点的 process()
}

/// 音频系统更新
pub fn audio_update_system(/* bevy resources */) {
    // TODO: 更新音频播放器状态
}

/// 场景树同步（将 SceneNodeData 同步到 Bevy ECS 组件）
pub fn scene_sync_system(/* bevy resources */) {
    // TODO: 同步脏标记节点的变换
}

/// 变换传播系统（计算全局变换）
pub fn transform_propagation_system(/* bevy resources */) {
    // TODO: 从根节点向下传播 TRS 变换
}

/// 相机矩阵更新系统
pub fn camera_update_system(/* bevy resources */) {
    // TODO: 更新相机的 ViewProjection 矩阵
}

/// 引擎系统集定义（统一调度）
pub struct EngineSystems;

impl EngineSystems {
    /// 系统执行顺序说明：
    /// Input -> Script -> Physics -> SceneSync -> Animation -> Audio -> Render
    pub fn schedule_description() -> &'static str {
        "Input → Script → Physics → SceneSync → Animation → Audio → Render"
    }
}
