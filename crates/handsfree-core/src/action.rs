use handsfree_platform::{MouseButton, PlatformAdapter, VolumeControl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "click")]
    Click {
        #[serde(default = "default_left_button")]
        button: MouseButton,
    },
    #[serde(rename = "double_click")]
    DoubleClick,
    #[serde(rename = "move_cursor")]
    MoveCursor { x: i32, y: i32 },
    #[serde(rename = "mouse_down")]
    MouseDown {
        #[serde(default = "default_left_button")]
        button: MouseButton,
    },
    #[serde(rename = "mouse_up")]
    MouseUp {
        #[serde(default = "default_left_button")]
        button: MouseButton,
    },
    #[serde(rename = "scroll")]
    Scroll { amount: i32 },
    #[serde(rename = "press")]
    Press { key: String },
    #[serde(rename = "type_text")]
    TypeText { text: String },
    #[serde(rename = "hotkey")]
    Hotkey { keys: Vec<String> },
    #[serde(rename = "open_app")]
    OpenApp { name: String },
    #[serde(rename = "open_url")]
    OpenUrl { url: String },
    #[serde(rename = "search_google")]
    SearchGoogle { query: String },
    #[serde(rename = "search_youtube")]
    SearchYoutube { query: String },
    #[serde(rename = "volume")]
    Volume { control: VolumeControl },
    #[serde(rename = "lock_screen")]
    LockScreen,
    #[serde(rename = "close_window")]
    CloseWindow,
    #[serde(rename = "minimize")]
    Minimize,
    #[serde(rename = "maximize")]
    Maximize,
    #[serde(rename = "screenshot")]
    Screenshot,
    #[serde(rename = "mode")]
    Mode { target: String, action: String },
    #[serde(rename = "browser_click_element")]
    BrowserClickElement { target_id: String },
    #[serde(rename = "browser_fill_field")]
    BrowserFillField { target_id: String, text: String },
}

fn default_left_button() -> MouseButton {
    MouseButton::Left
}

pub struct ActionExecutor {
    platform: Arc<dyn PlatformAdapter>,
}

impl ActionExecutor {
    pub fn new(platform: Arc<dyn PlatformAdapter>) -> Self {
        Self { platform }
    }

    pub fn execute(&self, action: &Action) -> anyhow::Result<()> {
        match action {
            Action::Click { button } => self.platform.click(*button),
            Action::DoubleClick => self.platform.double_click(),
            Action::MoveCursor { x, y } => self.platform.move_cursor(*x, *y),
            Action::MouseDown { button } => self.platform.mouse_down(*button),
            Action::MouseUp { button } => self.platform.mouse_up(*button),
            Action::Scroll { amount } => self.platform.scroll(*amount),
            Action::Press { key } => self.platform.press_key(key),
            Action::TypeText { text } => self.platform.type_text(text),
            Action::Hotkey { keys } => self.platform.hotkey(keys),
            Action::OpenApp { name } => self.platform.open_app(name),
            Action::OpenUrl { url } => self.platform.open_url(url),
            Action::SearchGoogle { query } => {
                let url = format!(
                    "https://www.google.com/search?q={}",
                    query.replace(' ', "+")
                );
                self.platform.open_url(&url)
            }
            Action::SearchYoutube { query } => {
                let url = format!(
                    "https://www.youtube.com/results?search_query={}",
                    query.replace(' ', "+")
                );
                self.platform.open_url(&url)
            }
            Action::Volume { control } => self.platform.adjust_volume(control.clone()),
            Action::LockScreen => self.platform.lock_screen(),
            Action::CloseWindow => self.platform.close_active_window(),
            Action::Minimize => self.platform.minimize_window(),
            Action::Maximize => self.platform.maximize_window(),
            Action::Screenshot => self.platform.hotkey(&["win".to_string(), "shift".to_string(), "s".to_string()]),
            Action::Mode { .. } => {
                // Handled in modality coordinator
                Ok(())
            }
            Action::BrowserClickElement { .. } | Action::BrowserFillField { .. } => {
                // Handled via browser bridge
                Ok(())
            }
        }
    }
}
