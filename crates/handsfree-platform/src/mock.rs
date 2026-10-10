use crate::traits::{MouseButton, PlatformAdapter, VolumeControl};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformCommandRecord {
    MoveCursor(i32, i32),
    Click(MouseButton),
    DoubleClick,
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Scroll(i32),
    PressKey(String),
    TypeText(String),
    Hotkey(Vec<String>),
    OpenApp(String),
    OpenUrl(String),
    AdjustVolume(VolumeControl),
    LockScreen,
    CloseActiveWindow,
    MinimizeWindow,
    MaximizeWindow,
}

#[derive(Debug, Clone, Default)]
pub struct MockPlatform {
    records: Arc<Mutex<Vec<PlatformCommandRecord>>>,
}

impl MockPlatform {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_records(&self) -> Vec<PlatformCommandRecord> {
        self.records.lock().clone()
    }

    pub fn clear(&self) {
        self.records.lock().clear();
    }
}

impl PlatformAdapter for MockPlatform {
    fn move_cursor(&self, x: i32, y: i32) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::MoveCursor(x, y));
        Ok(())
    }

    fn click(&self, button: MouseButton) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::Click(button));
        Ok(())
    }

    fn double_click(&self) -> anyhow::Result<()> {
        self.records.lock().push(PlatformCommandRecord::DoubleClick);
        Ok(())
    }

    fn mouse_down(&self, button: MouseButton) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::MouseDown(button));
        Ok(())
    }

    fn mouse_up(&self, button: MouseButton) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::MouseUp(button));
        Ok(())
    }

    fn scroll(&self, amount: i32) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::Scroll(amount));
        Ok(())
    }

    fn press_key(&self, key: &str) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::PressKey(key.to_string()));
        Ok(())
    }

    fn type_text(&self, text: &str) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::TypeText(text.to_string()));
        Ok(())
    }

    fn hotkey(&self, keys: &[String]) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::Hotkey(keys.to_vec()));
        Ok(())
    }

    fn open_app(&self, app_name: &str) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::OpenApp(app_name.to_string()));
        Ok(())
    }

    fn open_url(&self, url: &str) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::OpenUrl(url.to_string()));
        Ok(())
    }

    fn adjust_volume(&self, control: VolumeControl) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::AdjustVolume(control));
        Ok(())
    }

    fn lock_screen(&self) -> anyhow::Result<()> {
        self.records.lock().push(PlatformCommandRecord::LockScreen);
        Ok(())
    }

    fn close_active_window(&self) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::CloseActiveWindow);
        Ok(())
    }

    fn minimize_window(&self) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::MinimizeWindow);
        Ok(())
    }

    fn maximize_window(&self) -> anyhow::Result<()> {
        self.records
            .lock()
            .push(PlatformCommandRecord::MaximizeWindow);
        Ok(())
    }
}
