use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
}

#[async_trait]
pub trait LLMAdapter: Send + Sync {
    async fn infer(&self, request: &LLMRequest) -> anyhow::Result<LLMResponse>;
}

/// Simple offline Echo / Mock LLM
pub struct EchoLLM;

#[async_trait]
impl LLMAdapter for EchoLLM {
    async fn infer(&self, request: &LLMRequest) -> anyhow::Result<LLMResponse> {
        Ok(LLMResponse {
            content: format!(
                r#"{{"actions": [], "reply": "Echo: {}"}}"#,
                request.user_prompt
            ),
        })
    }
}

/// Generic OpenAI-compatible HTTP Adapter (covers OpenAI, Groq, and Ollama)
pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl LLMAdapter for OpenAICompatibleProvider {
    async fn infer(&self, request: &LLMRequest) -> anyhow::Result<LLMResponse> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
            "temperature": request.temperature,
            "response_format": {"type": "json_object"}
        });

        let mut req = self.client.post(&endpoint).json(&payload);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Provider API error ({}): {}", status, err_text);
        }

        let json_resp: serde_json::Value = resp.json().await?;
        let content = json_resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LLMResponse { content })
    }
}

/// Google Gemini HTTP Adapter
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl LLMAdapter for GeminiProvider {
    async fn infer(&self, request: &LLMRequest) -> anyhow::Result<LLMResponse> {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {"text": format!("{}\n\nUser: {}", request.system_prompt, request.user_prompt)}
                    ]
                }
            ],
            "generationConfig": {
                "temperature": request.temperature,
                "responseMimeType": "application/json"
            }
        });

        let resp = self.client.post(&endpoint).json(&payload).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error ({}): {}", status, err_text);
        }

        let json_resp: serde_json::Value = resp.json().await?;
        let content = json_resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LLMResponse { content })
    }
}
