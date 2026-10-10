use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BridgeAuthenticator {
    expected_token: Arc<RwLock<String>>,
}

impl Default for BridgeAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeAuthenticator {
    pub fn new() -> Self {
        let token = Uuid::new_v4().to_string();
        Self {
            expected_token: Arc::new(RwLock::new(token)),
        }
    }

    pub fn with_token(token: &str) -> Self {
        Self {
            expected_token: Arc::new(RwLock::new(token.to_string())),
        }
    }

    pub fn get_token(&self) -> String {
        self.expected_token.read().clone()
    }

    pub fn rotate_token(&self) -> String {
        let new_token = Uuid::new_v4().to_string();
        *self.expected_token.write() = new_token.clone();
        new_token
    }

    pub fn validate_token(&self, candidate: &str) -> bool {
        let expected = self.expected_token.read();
        candidate == *expected
    }
}
