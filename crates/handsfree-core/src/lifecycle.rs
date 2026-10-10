use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct LifecycleSupervisor {
    shutdown_sender: broadcast::Sender<()>,
}

impl Default for LifecycleSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleSupervisor {
    pub fn new() -> Self {
        let (shutdown_sender, _) = broadcast::channel(1);
        Self { shutdown_sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_sender.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_sender.send(());
    }
}
