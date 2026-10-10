use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticElement {
    pub id: String,
    pub role: String,
    pub name: String,
    #[serde(default)]
    pub tag_name: String,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub is_clickable: bool,
    #[serde(default)]
    pub is_input: bool,
    #[serde(default)]
    pub is_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSnapshot {
    pub generation_id: u64,
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub title: String,
    pub elements: Vec<SemanticElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum BridgeCommand {
    #[serde(rename = "auth")]
    Auth { token: String },

    #[serde(rename = "page_snapshot")]
    Snapshot(PageSnapshot),

    #[serde(rename = "semantic_action")]
    SemanticAction {
        id: Uuid,
        target_id: String,
        action_type: String,
        value: Option<String>,
    },

    #[serde(rename = "action_result")]
    ActionResult {
        id: Uuid,
        success: bool,
        message: Option<String>,
    },

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeEnvelope {
    pub id: Uuid,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub command: BridgeCommand,
}

impl BridgeEnvelope {
    pub fn new(command: BridgeCommand) -> Self {
        Self {
            id: Uuid::new_v4(),
            version: 1,
            timestamp: Utc::now(),
            command,
        }
    }
}
