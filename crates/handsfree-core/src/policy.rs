use crate::action::Action;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskTier {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingConfirmation {
    pub id: Uuid,
    pub action: Action,
    pub prompt: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub risk_tier: RiskTier,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, Default)]
pub struct PolicyEngine {
    pending_confirmations: Arc<Mutex<Vec<PendingConfirmation>>>,
    audit_log: Arc<Mutex<Vec<AuditEntry>>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            pending_confirmations: Arc::new(Mutex::new(Vec::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn classify_risk(&self, action: &Action) -> RiskTier {
        match action {
            Action::Scroll { .. }
            | Action::Screenshot
            | Action::OpenUrl { .. }
            | Action::SearchGoogle { .. }
            | Action::SearchYoutube { .. }
            | Action::Volume { .. }
            | Action::Mode { .. } => RiskTier::Low,

            Action::Click { .. }
            | Action::DoubleClick
            | Action::MoveCursor { .. }
            | Action::MouseDown { .. }
            | Action::MouseUp { .. }
            | Action::Press { .. }
            | Action::TypeText { .. }
            | Action::Hotkey { .. }
            | Action::OpenApp { .. }
            | Action::Minimize
            | Action::Maximize
            | Action::BrowserClickElement { .. }
            | Action::BrowserFillField { .. } => RiskTier::Moderate,

            Action::CloseWindow | Action::LockScreen => RiskTier::High,
        }
    }

    pub fn requires_confirmation(&self, action: &Action) -> bool {
        self.classify_risk(action) == RiskTier::High
    }

    pub fn request_confirmation(
        &self,
        action: Action,
        prompt: &str,
        target: &str,
        timeout_sec: i64,
    ) -> PendingConfirmation {
        let now = Utc::now();
        let confirmation = PendingConfirmation {
            id: Uuid::new_v4(),
            action,
            prompt: prompt.to_string(),
            target: target.to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::seconds(timeout_sec),
        };

        let mut lock = self.pending_confirmations.lock();
        // Clear any expired confirmations
        lock.retain(|c| c.expires_at > now);
        lock.push(confirmation.clone());

        confirmation
    }

    pub fn resolve_confirmation(&self, confirmed: bool) -> Option<Action> {
        let mut lock = self.pending_confirmations.lock();
        let now = Utc::now();
        lock.retain(|c| c.expires_at > now);

        if let Some(item) = lock.pop() {
            if confirmed {
                self.record_audit(&item.action, RiskTier::High, "Confirmed", &item.target);
                Some(item.action)
            } else {
                self.record_audit(&item.action, RiskTier::High, "Cancelled", &item.target);
                None
            }
        } else {
            None
        }
    }

    pub fn has_pending_confirmation(&self) -> bool {
        let mut lock = self.pending_confirmations.lock();
        let now = Utc::now();
        lock.retain(|c| c.expires_at > now);
        !lock.is_empty()
    }

    pub fn record_audit(&self, action: &Action, risk: RiskTier, status: &str, details: &str) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action_type: format!("{:?}", action),
            risk_tier: risk,
            status: status.to_string(),
            details: Self::redact_sensitive_text(details),
        };
        self.audit_log.lock().push(entry);
    }

    pub fn get_audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.lock().clone()
    }

    fn redact_sensitive_text(text: &str) -> String {
        // Redact potential API keys or tokens
        let re = regex::Regex::new(r"(?i)(key|token|secret|password)=([^\s&]+)").unwrap();
        re.replace_all(text, "$1=REDACTED").to_string()
    }
}
