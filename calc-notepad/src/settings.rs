use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub font_size: f32,
    pub always_on_top: bool,
    pub text: String,
    /// Код языка интерфейса: "ru" | "en". `serde(default)` — чтобы старые файлы без
    /// этого поля читались.
    #[serde(default = "default_lang")]
    pub lang: String,
}
fn default_lang() -> String {
    "ru".to_string()
}
impl Default for Settings {
    fn default() -> Self {
        Settings { font_size: 14.0, always_on_top: false, text: String::new(), lang: default_lang() }
    }
}
fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("irish", "green", "chista-notepad")
        .map(|d| d.config_dir().join("state.json"))
}
pub fn load() -> Settings {
    config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
pub fn save(s: &Settings) {
    if let Some(p) = config_path() {
        if let Some(dir) = p.parent() { let _ = std::fs::create_dir_all(dir); }
        if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write(p, j); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_roundtrip() {
        let s = Settings { font_size: 16.0, always_on_top: true, text: "x=1".into(), lang: "en".into() };
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.font_size, 16.0);
        assert!(back.always_on_top);
        assert_eq!(back.text, "x=1");
        assert_eq!(back.lang, "en");
    }
    #[test]
    fn old_settings_without_lang_default_to_ru() {
        let back: Settings = serde_json::from_str(r#"{"font_size":14.0,"always_on_top":false,"text":""}"#).unwrap();
        assert_eq!(back.lang, "ru");
    }
}
