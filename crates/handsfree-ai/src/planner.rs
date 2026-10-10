use crate::llm::{LLMAdapter, LLMRequest};
use handsfree_core::Action;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const SYSTEM_PROMPT: &str = r#"
You are Handsfree, an autonomous AI assistant embedded in a desktop and browser multimodal accessibility application.
Your job is to interpret the user's spoken, typed, or gestural request and return a JSON object with two keys:
  "actions" - a list of concrete actions to execute in order (max 5 actions)
  "reply"   - a concise, polite sentence describing what you are doing

Available action types (use ONLY these valid JSON action schemas):
1. {"type": "click"} - Left click at current cursor
2. {"type": "right_click"} - Right click
3. {"type": "double_click"} - Double click
4. {"type": "scroll", "amount": 5} - Positive scrolls up, negative scrolls down (e.g. -5 for down)
5. {"type": "hotkey", "keys": ["ctrl", "c"]} - System hotkey combination (e.g. ["win", "shift", "s"] for screenshot, ["ctrl", "t"] for new tab, ["ctrl", "w"] for close tab, ["alt", "tab"] to switch window)
6. {"type": "press", "key": "enter"} - Single key press ("enter", "tab", "esc", "space", "f")
7. {"type": "type_text", "text": "hello"} - Types literal text
8. {"type": "open_app", "name": "notepad"|"calc"|"brave"|"chrome"|"cmd"|"taskmgr"|"snippingtool"} - Launches desktop executable
9. {"type": "open_url", "url": "https://..."} - Navigates browser to URL
10. {"type": "search_google", "query": "..."} - Searches Google
11. {"type": "search_youtube", "query": "..."} - Searches YouTube
12. {"type": "volume", "control": "Up"|"Down"|"Mute"|"Unmute"} - Adjusts system sound
13. {"type": "minimize"} - Minimizes current window
14. {"type": "maximize"} - Maximizes current window
15. {"type": "close_window"} - Closes active window
16. {"type": "lock_screen"} - Locks Windows workstation
17. {"type": "screenshot"} - Triggers Windows screenshot capture
18. {"type": "browser_click_element", "target_id": "hf-..."} - Directly clicks a visible DOM element by its ID from the page context
19. {"type": "browser_fill_field", "target_id": "hf-...", "text": "..."} - Directly fills text into a browser input field by its ID from the page context

Contextual Planning Guidelines:
- If Environment Context provides interactive page elements (e.g. YouTube Subscribe button, Search input), use "browser_click_element" or "browser_fill_field" matching that element's target_id.
- For taking a screenshot or capturing screen, use {"type": "screenshot"} or {"type": "hotkey", "keys": ["win", "shift", "s"]}.
- For opening a channel or video on YouTube, use {"type": "search_youtube", "query": "..."}.
- For conversational questions without computer actions, return an empty "actions" list and answer helpfully in "reply".
- Return ONLY valid JSON matching {"actions": [...], "reply": "..."}. No markdown ticks, no preamble.
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPlan {
    pub actions: Vec<Action>,
    pub reply: String,
}

pub struct AIPlanner {
    adapter: Arc<dyn LLMAdapter>,
    max_steps: usize,
}

impl AIPlanner {
    pub fn new(adapter: Arc<dyn LLMAdapter>, max_steps: usize) -> Self {
        Self { adapter, max_steps }
    }

    pub async fn plan(&self, user_prompt: &str) -> anyhow::Result<AIPlan> {
        self.plan_with_context(user_prompt, None).await
    }

    pub async fn plan_with_context(
        &self,
        user_prompt: &str,
        context: Option<&str>,
    ) -> anyhow::Result<AIPlan> {
        let full_user_prompt = if let Some(ctx) = context {
            format!("User Command: {}\n\nEnvironment Context:\n{}", user_prompt, ctx)
        } else {
            user_prompt.to_string()
        };

        let request = LLMRequest {
            system_prompt: SYSTEM_PROMPT.to_string(),
            user_prompt: full_user_prompt,
            temperature: 0.1,
        };

        let response = self.adapter.infer(&request).await?;
        let cleaned_json = Self::extract_json(&response.content);

        let mut plan: AIPlan = Self::parse_plan(&cleaned_json).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse LLM plan JSON: {} (raw: {})",
                e,
                response.content
            )
        })?;

        if plan.actions.len() > self.max_steps {
            plan.actions.truncate(self.max_steps);
        }

        Ok(plan)
    }

    pub fn parse_plan(raw_json: &str) -> anyhow::Result<AIPlan> {
        let (val, actions_opt) = match serde_json::from_str::<serde_json::Value>(raw_json)? {
            serde_json::Value::Array(arr) => {
                (serde_json::Value::Object(serde_json::Map::new()), Some(arr))
            }
            obj @ serde_json::Value::Object(_) => {
                let arr = obj.get("actions").and_then(|a| a.as_array()).cloned();
                (obj, arr)
            }
            _ => anyhow::bail!("Expected JSON object or array"),
        };

        let mut reply = val
            .get("reply")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let mut actions = Vec::new();
        if let Some(actions_arr) = actions_opt {
            for act_val in actions_arr {
                // If LLM put {"type": "reply", "text": "..."} inside actions array, capture its text
                if let Some(type_str) = act_val.get("type").and_then(|t| t.as_str()) {
                    if type_str == "reply" || type_str == "message" || type_str == "thought" {
                        if reply.is_empty() {
                            if let Some(text) = act_val.get("text").and_then(|t| t.as_str()) {
                                reply = text.to_string();
                            }
                        }
                        continue;
                    }
                }

                // Normalize action Value before deserializing into Action
                let mut normalized = act_val.clone();
                if let Some(obj) = normalized.as_object_mut() {
                    let type_str = obj
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();
                    let has_target_id = obj.contains_key("target_id");

                    // Map {"type": "click", "target_id": "..."} -> Action::BrowserClickElement
                    if has_target_id && (type_str == "click" || type_str == "browser_click") {
                        obj.insert(
                            "type".to_string(),
                            serde_json::json!("browser_click_element"),
                        );
                    }
                    // Map {"type": "fill"|"type_text", "target_id": "..."} -> Action::BrowserFillField
                    else if has_target_id
                        && (type_str == "fill"
                            || type_str == "type"
                            || type_str == "type_text")
                    {
                        obj.insert(
                            "type".to_string(),
                            serde_json::json!("browser_fill_field"),
                        );
                    }
                }

                // Attempt to deserialize individual action, tolerating any stray unknown elements
                match serde_json::from_value::<Action>(normalized) {
                    Ok(action) => actions.push(action),
                    Err(e) => {
                        tracing::warn!(
                            "Skipping unparseable action element: {} (error: {})",
                            act_val,
                            e
                        );
                    }
                }
            }
        }

        if reply.is_empty() {
            reply = "Action planned successfully.".to_string();
        }

        Ok(AIPlan { actions, reply })
    }

    fn extract_json(raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                return trimmed[start..=end].to_string();
            }
        }
        trimmed.to_string()
    }
}
