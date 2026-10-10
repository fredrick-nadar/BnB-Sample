use crate::events::{EventPayload, Modality, NormalizedEvent};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ModalityArbitrator {
    active_modalities: Arc<Mutex<HashMap<Modality, bool>>>,
    last_event_time: Arc<Mutex<HashMap<Modality, DateTime<Utc>>>>,
    last_gaze_coord: Arc<Mutex<(i32, i32)>>,
    dedup_window_ms: i64,
    emergency_stopped: Arc<Mutex<bool>>,
}

impl ModalityArbitrator {
    pub fn new(dedup_window_ms: i64) -> Self {
        let mut active = HashMap::new();
        active.insert(Modality::Eye, false);
        active.insert(Modality::Gesture, false);
        active.insert(Modality::Voice, true);
        active.insert(Modality::VirtualKeyboard, false);
        active.insert(Modality::StandardInput, true);
        active.insert(Modality::EmergencyStop, true);

        Self {
            active_modalities: Arc::new(Mutex::new(active)),
            last_event_time: Arc::new(Mutex::new(HashMap::new())),
            last_gaze_coord: Arc::new(Mutex::new((-1, -1))),
            dedup_window_ms,
            emergency_stopped: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_active(&self, modality: Modality) -> bool {
        *self
            .active_modalities
            .lock()
            .get(&modality)
            .unwrap_or(&false)
    }

    pub fn set_active(&self, modality: Modality, active: bool) {
        self.active_modalities.lock().insert(modality, active);
    }

    pub fn trigger_emergency_stop(&self) {
        *self.emergency_stopped.lock() = true;
        // Turn off interactive vision and keyboard modes immediately
        self.set_active(Modality::Eye, false);
        self.set_active(Modality::Gesture, false);
        self.set_active(Modality::VirtualKeyboard, false);
    }

    pub fn clear_emergency_stop(&self) {
        *self.emergency_stopped.lock() = false;
        self.set_active(Modality::Eye, true);
        self.set_active(Modality::Gesture, true);
        self.set_active(Modality::VirtualKeyboard, true);
    }

    pub fn is_emergency_stopped(&self) -> bool {
        *self.emergency_stopped.lock()
    }

    /// Arbitrate incoming normalized event: verifies modality active, checks priority and deduplication.
    pub fn should_process(&self, event: &NormalizedEvent) -> bool {
        // Emergency Stop is always processed
        if event.modality == Modality::EmergencyStop
            || matches!(event.payload, EventPayload::EmergencyHalt)
        {
            return true;
        }

        // If emergency stopped, drop all other events
        if self.is_emergency_stopped() {
            return false;
        }

        // Drop if modality is inactive
        if !self.is_active(event.modality) {
            return false;
        }

        // Check coordinates for continuous mouse streaming
        if let EventPayload::GazeMove { x, y, .. } = &event.payload {
            let mut last_coord = self.last_gaze_coord.lock();
            let mut last_times = self.last_event_time.lock();
            let now = Utc::now();

            if let Some(prev) = last_times.get(&event.modality) {
                let elapsed = (now - *prev).num_milliseconds();
                // If stationary (identical coords) within dedup window, drop
                if elapsed < self.dedup_window_ms && *x == last_coord.0 && *y == last_coord.1 {
                    return false;
                }
            }

            *last_coord = (*x, *y);
            last_times.insert(event.modality, now);
            return true;
        }

        // Deduplication check for discrete actions (clicks, hotkeys, macros)
        let mut last_times = self.last_event_time.lock();
        let now = Utc::now();
        if let Some(prev) = last_times.get(&event.modality) {
            let elapsed = (now - *prev).num_milliseconds();
            if elapsed < self.dedup_window_ms {
                return false;
            }
        }

        last_times.insert(event.modality, now);
        true
    }
}
