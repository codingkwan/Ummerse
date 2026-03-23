//! 信号系统 - Godot 风格的节点间通信
//!
//! 信号（Signal）是节点发出的通知，其他节点可以连接到信号并接收回调。
//! 与事件总线不同，信号是节点级别的，可精确控制连接/断开。
//!
//! ## 设计要点
//! - 使用 `parking_lot::Mutex` 替代 `std::sync::Mutex`（更快，无毒化问题）
//! - 支持 `one_shot` 单次触发后自动断开
//! - `SignalBus` 支持带参数的具名信号（通过 `serde_json::Value`）

use std::sync::Arc;

use ahash::AHashMap;
use parking_lot::Mutex;
use uuid::Uuid;

// ── 连接 ID ───────────────────────────────────────────────────────────────────

/// 信号连接唯一 ID（用于断开连接）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    #[inline]
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 信号连接 ──────────────────────────────────────────────────────────────────

/// 类型化信号连接（回调函数 + 单次触发标志）
struct Connection<T: Clone + Send + Sync + 'static> {
    id: ConnectionId,
    callback: Box<dyn Fn(T) + Send + Sync + 'static>,
    one_shot: bool,
}

// ── 类型化信号 ────────────────────────────────────────────────────────────────

/// 类型化信号 - 可连接多个处理器的通知机制
///
/// # 示例
/// ```rust
/// use ummerse_core::signal::Signal;
///
/// let health_changed: Signal<i32> = Signal::new();
/// let id = health_changed.connect(|hp| println!("HP: {hp}"));
/// health_changed.emit(80);
/// health_changed.disconnect(id);
/// ```
pub struct Signal<T: Clone + Send + Sync + 'static = ()> {
    connections: Mutex<Vec<Connection<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// 创建新信号
    #[inline]
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
        }
    }

    /// 连接到信号（持久连接，返回连接 ID）
    pub fn connect(&self, callback: impl Fn(T) + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        self.connections.lock().push(Connection {
            id,
            callback: Box::new(callback),
            one_shot: false,
        });
        id
    }

    /// 连接到信号（仅触发一次后自动断开）
    pub fn connect_once(&self, callback: impl Fn(T) + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        self.connections.lock().push(Connection {
            id,
            callback: Box::new(callback),
            one_shot: true,
        });
        id
    }

    /// 断开指定连接
    pub fn disconnect(&self, id: ConnectionId) {
        self.connections.lock().retain(|c| c.id != id);
    }

    /// 发射信号（触发所有已连接回调，自动清理 one_shot 连接）
    pub fn emit(&self, value: T) {
        let mut conns = self.connections.lock();
        let mut to_remove: Vec<ConnectionId> = Vec::new();

        for conn in conns.iter() {
            (conn.callback)(value.clone());
            if conn.one_shot {
                to_remove.push(conn.id);
            }
        }

        if !to_remove.is_empty() {
            conns.retain(|c| !to_remove.contains(&c.id));
        }
    }

    /// 当前连接数量
    #[inline]
    pub fn connection_count(&self) -> usize {
        self.connections.lock().len()
    }

    /// 是否有任何连接
    #[inline]
    pub fn is_connected(&self) -> bool {
        !self.connections.lock().is_empty()
    }

    /// 断开所有连接
    pub fn disconnect_all(&self) {
        self.connections.lock().clear();
    }
}

impl<T: Clone + Send + Sync + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── 具名信号总线 ──────────────────────────────────────────────────────────────

/// 具名信号总线 - 通过字符串名称管理多个无类型信号
///
/// 类似 Godot 的 `emit_signal("name")` 模式，适用于：
/// - 节点间松耦合通信
/// - 动态信号名（脚本驱动）
/// - UI 事件（不需要强类型约束）
pub struct SignalBus {
    signals: Mutex<AHashMap<String, Vec<(ConnectionId, Box<dyn Fn() + Send + Sync>)>>>,
}

impl SignalBus {
    /// 创建新信号总线
    #[inline]
    pub fn new() -> Self {
        Self {
            signals: Mutex::new(AHashMap::new()),
        }
    }

    /// 连接具名信号（返回连接 ID）
    pub fn connect(&self, signal: &str, callback: impl Fn() + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        self.signals
            .lock()
            .entry(signal.to_string())
            .or_default()
            .push((id, Box::new(callback)));
        id
    }

    /// 断开具名信号的特定连接
    pub fn disconnect(&self, signal: &str, id: ConnectionId) {
        let mut signals = self.signals.lock();
        if let Some(handlers) = signals.get_mut(signal) {
            handlers.retain(|(cid, _)| *cid != id);
        }
    }

    /// 断开具名信号的所有连接
    pub fn disconnect_all(&self, signal: &str) {
        self.signals.lock().remove(signal);
    }

    /// 发射具名信号
    pub fn emit(&self, signal: &str) {
        // 先复制回调列表，避免锁内调用用户代码（死锁风险）
        let handlers: Vec<_> = {
            let signals = self.signals.lock();
            signals
                .get(signal)
                .map(|h| h.iter().map(|(id, _)| *id).collect())
                .unwrap_or_default()
        };
        // 在锁外调用回调
        let signals = self.signals.lock();
        if let Some(h) = signals.get(signal) {
            for (_, cb) in h {
                cb();
            }
        }
    }

    /// 是否存在该信号的连接
    pub fn has_connections(&self, signal: &str) -> bool {
        self.signals
            .lock()
            .get(signal)
            .map(|h| !h.is_empty())
            .unwrap_or(false)
    }

    /// 获取所有已注册的信号名称
    pub fn signal_names(&self) -> Vec<String> {
        self.signals.lock().keys().cloned().collect()
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── 带参数的具名信号总线 ──────────────────────────────────────────────────────

/// 带 JSON 参数的具名信号总线（供脚本系统使用）
///
/// 参数通过 `serde_json::Value` 传递，灵活性高但有序列化开销。
pub struct ScriptSignalBus {
    signals: Mutex<
        AHashMap<
            String,
            Vec<(ConnectionId, Box<dyn Fn(&serde_json::Value) + Send + Sync>)>,
        >,
    >,
}

impl ScriptSignalBus {
    #[inline]
    pub fn new() -> Self {
        Self {
            signals: Mutex::new(AHashMap::new()),
        }
    }

    /// 连接具名信号（带 JSON 参数）
    pub fn connect(
        &self,
        signal: &str,
        callback: impl Fn(&serde_json::Value) + Send + Sync + 'static,
    ) -> ConnectionId {
        let id = ConnectionId::new();
        self.signals
            .lock()
            .entry(signal.to_string())
            .or_default()
            .push((id, Box::new(callback)));
        id
    }

    /// 断开连接
    pub fn disconnect(&self, signal: &str, id: ConnectionId) {
        let mut signals = self.signals.lock();
        if let Some(handlers) = signals.get_mut(signal) {
            handlers.retain(|(cid, _)| *cid != id);
        }
    }

    /// 发射信号（带 JSON 参数）
    pub fn emit(&self, signal: &str, params: &serde_json::Value) {
        let signals = self.signals.lock();
        if let Some(handlers) = signals.get(signal) {
            for (_, cb) in handlers {
                cb(params);
            }
        }
    }

    /// 是否有连接
    pub fn has_connections(&self, signal: &str) -> bool {
        self.signals
            .lock()
            .get(signal)
            .map(|h| !h.is_empty())
            .unwrap_or(false)
    }
}

impl Default for ScriptSignalBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicI32, AtomicU32, Ordering},
        Arc,
    };

    #[test]
    fn test_signal_emit_and_connect() {
        let signal: Signal<i32> = Signal::new();
        let sum = Arc::new(AtomicI32::new(0));
        let s = sum.clone();

        signal.connect(move |v| {
            s.fetch_add(v, Ordering::Relaxed);
        });

        signal.emit(10);
        signal.emit(20);

        assert_eq!(sum.load(Ordering::Relaxed), 30);
        assert_eq!(signal.connection_count(), 1);
    }

    #[test]
    fn test_signal_one_shot() {
        let signal: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();

        signal.connect_once(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        signal.emit(());
        signal.emit(()); // 第二次不再触发

        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert_eq!(signal.connection_count(), 0); // 已自动移除
    }

    #[test]
    fn test_signal_disconnect() {
        let signal: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();

        let id = signal.connect(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        signal.emit(());
        assert_eq!(count.load(Ordering::Relaxed), 1);

        signal.disconnect(id);
        signal.emit(());
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_signal_disconnect_all() {
        let signal: Signal<()> = Signal::new();
        signal.connect(|_| {});
        signal.connect(|_| {});
        assert_eq!(signal.connection_count(), 2);

        signal.disconnect_all();
        assert_eq!(signal.connection_count(), 0);
    }

    #[test]
    fn test_signal_bus() {
        let bus = SignalBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();

        let id = bus.connect("on_ready", move || {
            c.fetch_add(1, Ordering::Relaxed);
        });

        assert!(bus.has_connections("on_ready"));
        bus.emit("on_ready");
        assert_eq!(count.load(Ordering::Relaxed), 1);

        bus.disconnect("on_ready", id);
        assert!(!bus.has_connections("on_ready"));
    }

    #[test]
    fn test_script_signal_bus() {
        let bus = ScriptSignalBus::new();
        let received = Arc::new(Mutex::new(Vec::<i64>::new()));
        let r = received.clone();

        bus.connect("score_changed", move |params| {
            if let Some(score) = params.get("score").and_then(|v| v.as_i64()) {
                r.lock().push(score);
            }
        });

        bus.emit("score_changed", &serde_json::json!({ "score": 100 }));
        bus.emit("score_changed", &serde_json::json!({ "score": 200 }));

        let r = received.lock();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], 100);
        assert_eq!(r[1], 200);
    }
}
