//! Context management and snapshotting adapted from AIOS (aios-rs/src/context.rs).
//! License: Apache-2.0 (Copyright 2024-2026 AGI Research)

use parking_lot::RwLock;
use std::sync::Arc;

pub trait ContextManager: Send + Sync {
    fn record_interaction(&self, user_prompt: &str, system_reply: &str);
    fn get_recent_history(&self, limit: usize) -> Vec<(String, String)>;
    fn clear(&self);
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryContextManager {
    history: Arc<RwLock<Vec<(String, String)>>>,
}

impl InMemoryContextManager {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl ContextManager for InMemoryContextManager {
    fn record_interaction(&self, user_prompt: &str, system_reply: &str) {
        self.history
            .write()
            .push((user_prompt.to_string(), system_reply.to_string()));
    }

    fn get_recent_history(&self, limit: usize) -> Vec<(String, String)> {
        let lock = self.history.read();
        let len = lock.len();
        if len <= limit {
            lock.clone()
        } else {
            lock[len - limit..].to_vec()
        }
    }

    fn clear(&self) {
        self.history.write().clear();
    }
}
