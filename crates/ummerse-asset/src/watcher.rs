//! 资产热重载监视器（仅非 Wasm 平台）

/// 资产变化事件
#[derive(Debug, Clone)]
pub struct AssetChangedEvent {
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// 文件监视器（桌面平台）
#[cfg(not(target_arch = "wasm32"))]
pub struct AssetWatcher {
    sender: std::sync::mpsc::Sender<AssetChangedEvent>,
    receiver: std::sync::Mutex<std::sync::mpsc::Receiver<AssetChangedEvent>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl AssetWatcher {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            sender,
            receiver: std::sync::Mutex::new(receiver),
        }
    }

    /// 轮询变化事件（非阻塞）
    pub fn poll_events(&self) -> Vec<AssetChangedEvent> {
        let receiver = self.receiver.lock().unwrap();
        let mut events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        events
    }

    /// 发送变化事件（供测试和内部使用）
    pub fn send_event(&self, event: AssetChangedEvent) {
        let _ = self.sender.send(event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AssetWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Wasm 平台无热重载
#[cfg(target_arch = "wasm32")]
pub struct AssetWatcher;

#[cfg(target_arch = "wasm32")]
impl AssetWatcher {
    pub fn new() -> Self {
        Self
    }

    pub fn poll_events(&self) -> Vec<AssetChangedEvent> {
        Vec::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for AssetWatcher {
    fn default() -> Self {
        Self::new()
    }
}
