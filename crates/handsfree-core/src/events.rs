use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Eye,
    Gesture,
    Voice,
    VirtualKeyboard,
    StandardInput,
    EmergencyStop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventPayload {
    GazeMove { x: i32, y: i32, confidence: f32 },
    DwellTrigger { x: i32, y: i32 },
    BlinkTrigger { eye: String, duration_sec: f32 },
    GestureDetected { gesture: String, confidence: f32 },
    KeyboardInput { key: String },
    VoicePhrase { text: String, confidence: f32 },
    Scroll { delta_y: i32 },
    EmergencyHalt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub modality: Modality,
    pub payload: EventPayload,
    pub confidence: f32,
}

impl NormalizedEvent {
    pub fn new(modality: Modality, payload: EventPayload, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            modality,
            payload,
            confidence,
        }
    }
}
