use crate::action::Action;
use handsfree_platform::{MouseButton, VolumeControl};
use std::collections::HashMap;

pub struct DeterministicRouter {
    site_map: HashMap<&'static str, &'static str>,
    app_map: HashMap<&'static str, &'static str>,
}

impl Default for DeterministicRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterministicRouter {
    pub fn new() -> Self {
        let mut site_map = HashMap::new();
        site_map.insert("google maps", "https://maps.google.com");
        site_map.insert("google drive", "https://drive.google.com");
        site_map.insert("google docs", "https://docs.google.com");
        site_map.insert("google sheets", "https://sheets.google.com");
        site_map.insert("google slides", "https://slides.google.com");
        site_map.insert("google translate", "https://translate.google.com");
        site_map.insert("google news", "https://news.google.com");
        site_map.insert("google colab", "https://colab.research.google.com");
        site_map.insert("youtube music", "https://music.youtube.com");
        site_map.insert("amazon prime", "https://www.primevideo.com");
        site_map.insert("microsoft teams", "https://teams.microsoft.com");
        site_map.insert("stack overflow", "https://stackoverflow.com");
        site_map.insert("khan academy", "https://www.khanacademy.org");
        site_map.insert("google", "https://www.google.com");
        site_map.insert("gmail", "https://mail.google.com");
        site_map.insert("youtube", "https://www.youtube.com");
        site_map.insert("facebook", "https://www.facebook.com");
        site_map.insert("instagram", "https://www.instagram.com");
        site_map.insert("twitter", "https://twitter.com");
        site_map.insert("whatsapp", "https://web.whatsapp.com");
        site_map.insert("netflix", "https://www.netflix.com");
        site_map.insert("github", "https://www.github.com");
        site_map.insert("wikipedia", "https://www.wikipedia.org");
        site_map.insert("spotify", "https://open.spotify.com");
        site_map.insert("linkedin", "https://www.linkedin.com");
        site_map.insert("reddit", "https://www.reddit.com");
        site_map.insert("chatgpt", "https://chat.openai.com");
        site_map.insert("openai", "https://chat.openai.com");
        site_map.insert("gemini", "https://gemini.google.com");
        site_map.insert("weather", "https://www.google.com/search?q=weather+today");

        let mut app_map = HashMap::new();
        app_map.insert("file explorer", "explorer");
        app_map.insert("task manager", "taskmgr");
        app_map.insert("control panel", "control");
        app_map.insert("command prompt", "cmd");
        app_map.insert("powershell", "powershell");
        app_map.insert("ms paint", "mspaint");
        app_map.insert("notepad", "notepad");
        app_map.insert("calculator", "calc");
        app_map.insert("paint", "mspaint");
        app_map.insert("word", "winword");
        app_map.insert("excel", "excel");
        app_map.insert("powerpoint", "powerpnt");
        app_map.insert("explorer", "explorer");
        app_map.insert("calc", "calc");
        app_map.insert("cmd", "cmd");
        app_map.insert("settings", "ms-settings:");
        app_map.insert("snipping tool", "snippingtool");
        app_map.insert("photos", "ms-photos:");
        app_map.insert("brave", "brave");
        app_map.insert("brave browser", "brave");
        app_map.insert("chrome", "chrome");
        app_map.insert("google chrome", "chrome");
        app_map.insert("edge", "msedge");
        app_map.insert("microsoft edge", "msedge");

        Self { site_map, app_map }
    }

    /// Try resolving phrase deterministically. Returns Some(Action) if matched, None if semantic planning needed.
    pub fn resolve(&self, input: &str) -> Option<Action> {
        let text = input.trim().to_lowercase();
        let cleaned = text
            .replace("handsfree", "")
            .replace("proton", "")
            .trim()
            .to_string();

        // 1. Browser Tab & Navigation
        if cleaned == "new tab" || cleaned == "open new tab" || cleaned == "open tab" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "t".to_string()],
            });
        }
        if cleaned == "close tab" || cleaned == "shut tab" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "w".to_string()],
            });
        }
        if cleaned == "next tab"
            || cleaned == "switch tab"
            || cleaned == "tab forward"
            || cleaned == "tab right"
        {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "tab".to_string()],
            });
        }
        if cleaned == "previous tab"
            || cleaned == "prev tab"
            || cleaned == "tab backward"
            || cleaned == "tab back"
            || cleaned == "tab left"
        {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "shift".to_string(), "tab".to_string()],
            });
        }
        if cleaned == "go back" || cleaned == "previous page" || cleaned == "back page" {
            return Some(Action::Hotkey {
                keys: vec!["alt".to_string(), "left".to_string()],
            });
        }
        if cleaned == "go forward" || cleaned == "next page" {
            return Some(Action::Hotkey {
                keys: vec!["alt".to_string(), "right".to_string()],
            });
        }
        if cleaned == "zoom in" || cleaned == "make bigger" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "+".to_string()],
            });
        }
        if cleaned == "zoom out" || cleaned == "make smaller" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "-".to_string()],
            });
        }

        // 2. Clipboard & File Operations
        if cleaned == "copy" || cleaned == "copy that" || cleaned == "ctrl c" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "c".to_string()],
            });
        }
        if cleaned == "paste" || cleaned == "paste that" || cleaned == "ctrl v" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "v".to_string()],
            });
        }
        if cleaned == "cut" || cleaned == "cut that" || cleaned == "ctrl x" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "x".to_string()],
            });
        }
        if cleaned == "select all" || cleaned == "ctrl a" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "a".to_string()],
            });
        }
        if cleaned == "undo" || cleaned == "ctrl z" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "z".to_string()],
            });
        }
        if cleaned == "redo" || cleaned == "ctrl y" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "y".to_string()],
            });
        }
        if cleaned == "save" || cleaned == "save file" || cleaned == "ctrl s" {
            return Some(Action::Hotkey {
                keys: vec!["ctrl".to_string(), "s".to_string()],
            });
        }

        // 3. Mouse Controls
        if cleaned == "click" || cleaned == "left click" || cleaned == "mouse click" {
            return Some(Action::Click {
                button: MouseButton::Left,
            });
        }
        if cleaned == "right click" {
            return Some(Action::Click {
                button: MouseButton::Right,
            });
        }
        if cleaned == "double click" {
            return Some(Action::DoubleClick);
        }
        if cleaned == "scroll up" || cleaned == "page up" {
            return Some(Action::Scroll { amount: 5 });
        }
        if cleaned == "scroll down" || cleaned == "page down" {
            return Some(Action::Scroll { amount: -5 });
        }

        // 4. Keyboard Keys
        if cleaned == "enter" || cleaned == "press enter" || cleaned == "hit enter" {
            return Some(Action::Press {
                key: "enter".to_string(),
            });
        }
        if cleaned == "tab" || cleaned == "press tab" {
            return Some(Action::Press {
                key: "tab".to_string(),
            });
        }
        if cleaned == "escape" || cleaned == "press escape" || cleaned == "esc" {
            return Some(Action::Press {
                key: "escape".to_string(),
            });
        }
        if cleaned == "space" || cleaned == "press space" {
            return Some(Action::Press {
                key: "space".to_string(),
            });
        }
        if cleaned == "delete" || cleaned == "press delete" {
            return Some(Action::Press {
                key: "delete".to_string(),
            });
        }

        // 5. Typing Text
        if let Some(to_type) = cleaned.strip_prefix("type ") {
            return Some(Action::TypeText {
                text: to_type.trim().to_string(),
            });
        }

        // 6. Window Management
        if cleaned == "minimize" || cleaned == "minimize window" {
            return Some(Action::Minimize);
        }
        if cleaned == "maximize" || cleaned == "maximize window" {
            return Some(Action::Maximize);
        }
        if cleaned == "close window" || cleaned == "close this window" {
            return Some(Action::CloseWindow);
        }
        if cleaned == "lock screen" || cleaned == "lock computer" {
            return Some(Action::LockScreen);
        }

        // 7. System Volume
        if cleaned == "volume up" || cleaned == "increase volume" {
            return Some(Action::Volume {
                control: VolumeControl::Up,
            });
        }
        if cleaned == "volume down" || cleaned == "decrease volume" {
            return Some(Action::Volume {
                control: VolumeControl::Down,
            });
        }
        if cleaned == "mute" || cleaned == "mute volume" {
            return Some(Action::Volume {
                control: VolumeControl::Mute,
            });
        }
        if cleaned == "unmute" {
            return Some(Action::Volume {
                control: VolumeControl::Unmute,
            });
        }

        // 8. Search & Media
        if let Some(query) = cleaned.strip_prefix("search for ") {
            return Some(Action::SearchGoogle {
                query: query.trim().to_string(),
            });
        }
        if let Some(query) = cleaned.strip_prefix("google ") {
            return Some(Action::SearchGoogle {
                query: query.trim().to_string(),
            });
        }
        if let Some(query) = cleaned.strip_prefix("search youtube for ") {
            return Some(Action::SearchYoutube {
                query: query.trim().to_string(),
            });
        }
        if let Some(query) = cleaned.strip_prefix("youtube search ") {
            return Some(Action::SearchYoutube {
                query: query.trim().to_string(),
            });
        }
        if let Some(rest) = cleaned.strip_prefix("open ") {
            if rest.ends_with(" on youtube") {
                let query = rest.trim_end_matches(" on youtube").trim();
                return Some(Action::SearchYoutube {
                    query: query.to_string(),
                });
            }
        }
        if let Some(rest) = cleaned.strip_prefix("search ") {
            if rest.ends_with(" on youtube") {
                let query = rest.trim_end_matches(" on youtube").trim();
                return Some(Action::SearchYoutube {
                    query: query.to_string(),
                });
            }
        }
        if let Some(query) = cleaned.strip_prefix("play ") {
            return Some(Action::SearchYoutube {
                query: query.trim().to_string(),
            });
        }

        // 9. Open Websites (SITE_MAP)
        for (site_name, url) in &self.site_map {
            if cleaned == format!("open {}", site_name) || cleaned == *site_name {
                return Some(Action::OpenUrl {
                    url: url.to_string(),
                });
            }
        }

        // 10. Open Desktop Applications (APP_MAP)
        for (app_name, exe) in &self.app_map {
            if cleaned == format!("open {}", app_name) || cleaned == *app_name {
                return Some(Action::OpenApp {
                    name: exe.to_string(),
                });
            }
        }

        // 11. Mode Switching
        if cleaned == "start eye control" || cleaned == "enable eye" {
            return Some(Action::Mode {
                target: "eye".to_string(),
                action: "start".to_string(),
            });
        }
        if cleaned == "stop eye control" || cleaned == "disable eye" {
            return Some(Action::Mode {
                target: "eye".to_string(),
                action: "stop".to_string(),
            });
        }
        if cleaned == "start gesture control" || cleaned == "enable gesture" {
            return Some(Action::Mode {
                target: "gesture".to_string(),
                action: "start".to_string(),
            });
        }
        if cleaned == "stop gesture control" || cleaned == "disable gesture" {
            return Some(Action::Mode {
                target: "gesture".to_string(),
                action: "stop".to_string(),
            });
        }
        if cleaned == "start virtual keyboard" || cleaned == "open keyboard" {
            return Some(Action::Mode {
                target: "keyboard".to_string(),
                action: "start".to_string(),
            });
        }
        if cleaned == "stop virtual keyboard" || cleaned == "close keyboard" {
            return Some(Action::Mode {
                target: "keyboard".to_string(),
                action: "stop".to_string(),
            });
        }

        None
    }
}
