use crate::traits::{MouseButton, PlatformAdapter, VolumeControl};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::System::Shutdown::LockWorkStation;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, mouse_event, KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, VK_CONTROL, VK_ESCAPE, VK_LWIN,
    VK_MENU, VK_RETURN, VK_SHIFT, VK_SPACE, VK_TAB, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW,
};

#[derive(Debug, Clone, Default)]
pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }

    fn to_wide_null(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn key_name_to_vk(key: &str) -> Option<u8> {
        match key.to_lowercase().as_str() {
            "enter" | "return" => Some(VK_RETURN as u8),
            "tab" => Some(VK_TAB as u8),
            "esc" | "escape" => Some(VK_ESCAPE as u8),
            "space" | "spacebar" => Some(VK_SPACE as u8),
            "ctrl" | "control" => Some(VK_CONTROL as u8),
            "alt" => Some(VK_MENU as u8),
            "shift" => Some(VK_SHIFT as u8),
            "win" | "super" => Some(VK_LWIN as u8),
            "up" => Some(0x26),
            "down" => Some(0x28),
            "left" => Some(0x25),
            "right" => Some(0x27),
            "backspace" => Some(0x08),
            "delete" => Some(0x2E),
            "home" => Some(0x24),
            "end" => Some(0x23),
            "pageup" => Some(0x21),
            "pagedown" => Some(0x22),
            "f1" => Some(0x70),
            "f2" => Some(0x71),
            "f3" => Some(0x72),
            "f4" => Some(0x73),
            "f5" => Some(0x74),
            "f6" => Some(0x75),
            "f7" => Some(0x76),
            "f8" => Some(0x77),
            "f9" => Some(0x78),
            "f10" => Some(0x79),
            "f11" => Some(0x7A),
            "f12" => Some(0x7B),
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                if c.is_ascii_alphanumeric() {
                    Some(c.to_ascii_uppercase() as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl PlatformAdapter for WindowsPlatform {
    fn move_cursor(&self, x: i32, y: i32) -> anyhow::Result<()> {
        unsafe {
            // Direct hardware-level cursor positioning (unaffected by DPI virtualization)
            SetCursorPos(x, y);

            // Also emit mouse_event for hooked background applications
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            if screen_w > 0 && screen_h > 0 {
                let normalized_x = (x as f64 * 65535.0 / screen_w as f64) as i32;
                let normalized_y = (y as f64 * 65535.0 / screen_h as f64) as i32;
                mouse_event(
                    MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                    normalized_x,
                    normalized_y,
                    0,
                    0,
                );
            }
        }
        Ok(())
    }

    fn click(&self, button: MouseButton) -> anyhow::Result<()> {
        unsafe {
            match button {
                MouseButton::Left => {
                    mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                }
                MouseButton::Right => {
                    mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                }
                MouseButton::Middle => {
                    mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
                }
            }
        }
        Ok(())
    }

    fn double_click(&self) -> anyhow::Result<()> {
        self.click(MouseButton::Left)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
        self.click(MouseButton::Left)?;
        Ok(())
    }

    fn mouse_down(&self, button: MouseButton) -> anyhow::Result<()> {
        unsafe {
            match button {
                MouseButton::Left => mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0),
                MouseButton::Right => mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0),
                MouseButton::Middle => mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0),
            }
        }
        Ok(())
    }

    fn mouse_up(&self, button: MouseButton) -> anyhow::Result<()> {
        unsafe {
            match button {
                MouseButton::Left => mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0),
                MouseButton::Right => mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0),
                MouseButton::Middle => mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0),
            }
        }
        Ok(())
    }

    fn scroll(&self, amount: i32) -> anyhow::Result<()> {
        unsafe {
            // Windows wheel delta: 120 per notch
            let delta = amount * 120;
            mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta, 0);
        }
        Ok(())
    }

    fn press_key(&self, key: &str) -> anyhow::Result<()> {
        if let Some(vk) = Self::key_name_to_vk(key) {
            unsafe {
                keybd_event(vk, 0, 0, 0);
                keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
            }
            Ok(())
        } else {
            anyhow::bail!("Unsupported key: {}", key)
        }
    }

    fn type_text(&self, text: &str) -> anyhow::Result<()> {
        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() || ch == ' ' {
                let vk = if ch == ' ' {
                    VK_SPACE as u8
                } else {
                    ch.to_ascii_uppercase() as u8
                };
                let shift_needed = ch.is_ascii_uppercase();
                unsafe {
                    if shift_needed {
                        keybd_event(VK_SHIFT as u8, 0, 0, 0);
                    }
                    keybd_event(vk, 0, 0, 0);
                    keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
                    if shift_needed {
                        keybd_event(VK_SHIFT as u8, 0, KEYEVENTF_KEYUP, 0);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
        Ok(())
    }

    fn hotkey(&self, keys: &[String]) -> anyhow::Result<()> {
        let mut vks = Vec::new();
        for k in keys {
            if let Some(vk) = Self::key_name_to_vk(k) {
                vks.push(vk);
            } else {
                anyhow::bail!("Unknown hotkey component: {}", k);
            }
        }

        unsafe {
            for &vk in &vks {
                keybd_event(vk, 0, 0, 0);
            }
            for &vk in vks.iter().rev() {
                keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
            }
        }
        Ok(())
    }

    fn open_app(&self, app_name: &str) -> anyhow::Result<()> {
        let wide_op = Self::to_wide_null("open");
        let wide_app = Self::to_wide_null(app_name);
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                wide_op.as_ptr(),
                wide_app.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOW,
            );
        }
        Ok(())
    }

    fn open_url(&self, url: &str) -> anyhow::Result<()> {
        let wide_op = Self::to_wide_null("open");
        let wide_url = Self::to_wide_null(url);
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                wide_op.as_ptr(),
                wide_url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOW,
            );
        }
        Ok(())
    }

    fn adjust_volume(&self, control: VolumeControl) -> anyhow::Result<()> {
        let vk = match control {
            VolumeControl::Up => VK_VOLUME_UP as u8,
            VolumeControl::Down => VK_VOLUME_DOWN as u8,
            VolumeControl::Mute | VolumeControl::Unmute => VK_VOLUME_MUTE as u8,
        };
        unsafe {
            keybd_event(vk, 0, 0, 0);
            keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
        }
        Ok(())
    }

    fn lock_screen(&self) -> anyhow::Result<()> {
        unsafe {
            LockWorkStation();
        }
        Ok(())
    }

    fn close_active_window(&self) -> anyhow::Result<()> {
        self.hotkey(&["alt".to_string(), "f4".to_string()])
    }

    fn minimize_window(&self) -> anyhow::Result<()> {
        self.hotkey(&["win".to_string(), "down".to_string()])
    }

    fn maximize_window(&self) -> anyhow::Result<()> {
        self.hotkey(&["win".to_string(), "up".to_string()])
    }
}
