//! 信号系统 - Godot 风格的节点间通信
//!
//! 信号（Signal）是节点发出的通知，其他节点可以连接到信号并接收回调。
//! 与事件总线不同，信号是节点级别的，可精确控制连接/断开。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

/// 信号连接 ID（用于断开连接）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// 信号连接（回调函数 + 是否为单次触发）
struct Connection<T: Clone + Send + Sync + 'static> {
    id: ConnectionId,
    callback: Box<dyn Fn(T) + Send + Sync + 'static>,
    one_shot: bool,
}

/// 信号 - 可连接多个处理器的通知机制
pub struct Signal<T: Clone + Send + Sync + 'static = ()> {
    connections: Mutex<Vec<Connection<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// 创建新信号
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
        }
    }

    /// 连接到信号（返回连接 ID）
    pub fn connect(&self, callback: impl Fn(T) + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        let mut conns = self.connections.lock().unwrap();
        conns.push(Connection {
            id,
            callback: Box::new(callback),
            one_shot: false,
        });
        id
    }

    /// 连接到信号（仅触发一次后自动断开）
    pub fn connect_once(&self, callback: impl Fn(T) + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        let mut conns = self.connections.lock().unwrap();
        conns.push(Connection {
            id,
            callback: Box::new(callback),
            one_shot: true,
        });
        id
    }

    /// 断开连接
    pub fn disconnect(&self, id: ConnectionId) {
        let mut conns = self.connections.lock().unwrap();
        conns.retain(|c| c.id != id);
    }

    /// 发射信号（触发所有连接的回调）
    pub fn emit(&self, value: T) {
        let mut conns = self.connections.lock().unwrap();
        let mut to_remove = Vec::new();

        for conn in conns.iter() {
            (conn.callback)(value.clone());
            if conn.one_shot {
                to_remove.push(conn.id);
            }
        }

        conns.retain(|c| !to_remove.contains(&c.id));
    }

    /// 当前连接数量
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    /// 断开所有连接
    pub fn disconnect_all(&self) {
        self.connections.lock().unwrap().clear();
    }
}

impl<T: Clone + Send + Sync + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── 信号总线 ──────────────────────────────────────────────────────────────────

/// 具名信号总线 - 通过字符串名称管理多个信号
///
/// 用于节点间不需要强类型约束的信号传递（类似 Godot 的 emit_signal("name")）。
pub struct SignalBus {
    /// 信号 → 回调列表
    signals: Mutex<HashMap<String, Vec<(ConnectionId, Box<dyn Fn() + Send + Sync>)>>>,
}

impl SignalBus {
    /// 创建新信号总线
    pub fn new() -> Self {
        Self {
            signals: Mutex::new(HashMap::new()),
        }
    }

    /// 连接具名信号
    pub fn connect(&self, signal: &str, callback: impl Fn() + Send + Sync + 'static) -> ConnectionId {
        let id = ConnectionId::new();
        let mut signals = self.signals.lock().unwrap();
        signals
            .entry(signal.to_string())
            .or_default()
            .push((id, Box::new(callback)));
        id
    }

    /// 断开具名信号的特定连接
    pub fn disconnect(&self, signal: &str, id: ConnectionId) {
        let mut signals = self.signals.lock().unwrap();
        if let Some(handlers) = signals.get_mut(signal) {
            handlers.retain(|(cid, _)| *cid != id);
        }
    }

    /// 发射具名信号
    pub fn emit(&self, signal: &str) {
        let signals = self.signals.lock().unwrap();
        if let Some(handlers) = signals.get(signal) {
            for (_, callback) in handlers {
                callback();
            }
        }
    }

    /// 是否存在该信号的连接
    pub fn has_connections(&self, signal: &str) -> bool {
        let signals = self.signals.lock().unwrap();
        signals.get(signal).map_or(false, |h| !h.is_empty())
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_signal_emit() {
        let signal: Signal<i32> = Signal::new();
        let sum = Arc::new(Mutex::new(0i32));
        let sum_clone = sum.clone();

        signal.connect(move |v| {
            *sum_clone.lock().unwrap() += v;
        });

        signal.emit(10);
        signal.emit(20);

        assert_eq!(*sum.lock().unwrap(), 30);
    }

    #[test]
    fn test_signal_one_shot() {
        let signal: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        signal.connect_once(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        signal.emit(());
        signal.emit(());

        // 只触发一次
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!(signal.connection_count(), 0);
    }

    #[test]
    fn test_signal_disconnect() {
        let signal: Signal<()> = Signal::new();
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let id = signal.connect(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        signal.emit(());
        signal.disconnect(id);
        signal.emit(());

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_signal_bus() {
        let bus = SignalBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        bus.connect("on_ready", move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert!(bus.has_connections("on_ready"));
        bus.emit("on_ready");
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
