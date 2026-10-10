use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeControl {
    Up,
    Down,
    Mute,
    Unmute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAction {
    Press(String),
    Down(String),
    Up(String),
}

pub trait PlatformAdapter: Send + Sync {
    /// Move cursor to absolute screen coordinates
    fn move_cursor(&self, x: i32, y: i32) -> anyhow::Result<()>;

    /// Click mouse button
    fn click(&self, button: MouseButton) -> anyhow::Result<()>;

    /// Double click
    fn double_click(&self) -> anyhow::Result<()>;

    /// Mouse down (for drag initiation)
    fn mouse_down(&self, button: MouseButton) -> anyhow::Result<()>;

    /// Mouse up (for drag completion)
    fn mouse_up(&self, button: MouseButton) -> anyhow::Result<()>;

    /// Scroll vertically
    fn scroll(&self, amount: i32) -> anyhow::Result<()>;

    /// Press a single safe key (e.g. "enter", "tab", "escape", "space")
    fn press_key(&self, key: &str) -> anyhow::Result<()>;

    /// Type string text into active focus
    fn type_text(&self, text: &str) -> anyhow::Result<()>;

    /// Execute hotkey combination (e.g. ["ctrl", "c"], ["alt", "f4"])
    fn hotkey(&self, keys: &[String]) -> anyhow::Result<()>;

    /// Open an application by registered name
    fn open_app(&self, app_name: &str) -> anyhow::Result<()>;

    /// Open a URL in default browser
    fn open_url(&self, url: &str) -> anyhow::Result<()>;

    /// Adjust system volume
    fn adjust_volume(&self, control: VolumeControl) -> anyhow::Result<()>;

    /// Lock workstation
    fn lock_screen(&self) -> anyhow::Result<()>;

    /// Close active window
    fn close_active_window(&self) -> anyhow::Result<()>;

    /// Minimize active window
    fn minimize_window(&self) -> anyhow::Result<()>;

    /// Maximize active window
    fn maximize_window(&self) -> anyhow::Result<()>;
}
