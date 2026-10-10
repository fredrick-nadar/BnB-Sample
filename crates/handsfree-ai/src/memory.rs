//! Memory subsystem adapted from AIOS (aios-rs/src/memory.rs).
//! License: Apache-2.0 (Copyright 2024-2026 AGI Research)

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub id: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

impl MemoryNote {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            keywords: vec![],
            tags: vec![],
            timestamp: Utc::now(),
        }
    }
}

pub trait MemoryManager: Send + Sync {
    fn add_memory(&self, note: MemoryNote) -> anyhow::Result<String>;
    fn remove_memory(&self, id: &str) -> anyhow::Result<()>;
    fn get_memory(&self, id: &str) -> Option<MemoryNote>;
    fn list_memories(&self) -> Vec<MemoryNote>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMemoryManager {
    notes: Arc<RwLock<HashMap<String, MemoryNote>>>,
}

impl InMemoryMemoryManager {
    pub fn new() -> Self {
        Self {
            notes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl MemoryManager for InMemoryMemoryManager {
    fn add_memory(&self, note: MemoryNote) -> anyhow::Result<String> {
        let id = note.id.clone();
        self.notes.write().insert(id.clone(), note);
        Ok(id)
    }

    fn remove_memory(&self, id: &str) -> anyhow::Result<()> {
        if self.notes.write().remove(id).is_some() {
            Ok(())
        } else {
            anyhow::bail!("Memory note not found: {}", id)
        }
    }

    fn get_memory(&self, id: &str) -> Option<MemoryNote> {
        self.notes.read().get(id).cloned()
    }

    fn list_memories(&self) -> Vec<MemoryNote> {
        self.notes.read().values().cloned().collect()
    }
}
