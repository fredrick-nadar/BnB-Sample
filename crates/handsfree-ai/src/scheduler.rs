//! Task scheduling adapted from AIOS (aios-rs/src/scheduler.rs).
//! License: Apache-2.0 (Copyright 2024-2026 AGI Research)

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub struct TaskRequest {
    pub id: String,
    pub prompt: String,
}

pub trait Scheduler: Send + Sync {
    fn enqueue(&self, task: TaskRequest);
    fn cancel(&self, task_id: &str);
}

#[derive(Clone, Default)]
pub struct AsyncScheduler {
    queue: Arc<Mutex<VecDeque<TaskRequest>>>,
    notify: Arc<Notify>,
}

impl AsyncScheduler {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn next_task(&self) -> TaskRequest {
        loop {
            {
                let mut lock = self.queue.lock();
                if let Some(task) = lock.pop_front() {
                    return task;
                }
            }
            self.notify.notified().await;
        }
    }
}

impl Scheduler for AsyncScheduler {
    fn enqueue(&self, task: TaskRequest) {
        self.queue.lock().push_back(task);
        self.notify.notify_one();
    }

    fn cancel(&self, task_id: &str) {
        self.queue.lock().retain(|t| t.id != task_id);
    }
}
