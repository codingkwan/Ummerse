//! 信号系统 - Godot 风格的节点间通信

use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// 信号参数类型（动态类型）
pub type SignalArgs = Vec<Box<dyn Any + Send + Sync>>;

/// 信号连接回调
pub type SignalCallback = Arc<dyn Fn(&SignalArgs) + Send + Sync>;

/// 信号描述符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signal {
    /// 信号所属节点名
    pub owner: String,
    /// 信号名称
    pub name: String,
}

impl Signal {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }
}

/// 连接 ID，用于断开连接
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

/// 信号总线 - 管理所有节点信号的连接
pub struct SignalBus {
    connections: Mutex<HashMap<Signal, Vec<(ConnectionId, SignalCallback)>>>,
    next_id: Mutex<u64>,
}

impl SignalBus {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
        }
    }

    /// 连接信号到回调函数，返回连接 ID
    pub fn connect(
        &self,
        signal: Signal,
        callback: impl Fn(&SignalArgs) + Send + Sync + 'static,
    ) -> ConnectionId {
        let mut id_guard = self.next_id.lock().unwrap();
        let id = ConnectionId(*id_guard);
        *id_guard += 1;
        drop(id_guard);

        let callback: SignalCallback = Arc::new(callback);
        self.connections
            .lock()
            .unwrap()
            .entry(signal)
            .or_default()
            .push((id, callback));
        id
    }

    /// 断开指定连接
    pub fn disconnect(&self, signal: &Signal, connection_id: ConnectionId) {
        if let Some(connections) = self.connections.lock().unwrap().get_mut(signal) {
            connections.retain(|(id, _)| *id != connection_id);
        }
    }

    /// 发射信号
    pub fn emit(&self, signal: &Signal, args: &SignalArgs) {
        let connections = self.connections.lock().unwrap();
        if let Some(callbacks) = connections.get(signal) {
            for (_, callback) in callbacks {
                callback(args);
            }
        }
    }

    /// 发射无参数信号
    pub fn emit_empty(&self, signal: &Signal) {
        self.emit(signal, &Vec::new());
    }

    /// 清除某个信号的所有连接
    pub fn disconnect_all(&self, signal: &Signal) {
        self.connections.lock().unwrap().remove(signal);
    }

    /// 信号是否有连接
    pub fn is_connected(&self, signal: &Signal) -> bool {
        self.connections
            .lock()
            .unwrap()
            .get(signal)
            .map(|c| !c.is_empty())
            .unwrap_or(false)
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}
