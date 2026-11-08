//! Saved settings and their validation. The JSON shape is unchanged from earlier versions
//! (camelCase), and any field missing from an old file falls back to its default.

use crate::format::FormatOptions;
use crate::hotkeys::{is_reserved, parse_hotkey, same_hotkey};
use serde::{Deserialize, Serialize};

const MAX_AUTO_STOP_SECONDS: u8 = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub dictation_hotkey: String,
    pub agent_hotkey: String,
    pub cleanup: FormatOptions,
    pub warm_on_startup: bool,
    /// Seconds of quiet after speech that end a recording on their own. 0 turns it off.
    pub auto_stop_seconds: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dictation_hotkey: "Control+Alt+Space".into(),
            agent_hotkey: "Control+Alt+A".into(),
            cleanup: FormatOptions::default(),
            warm_on_startup: true,
            auto_stop_seconds: 3,
        }
    }
}

impl Settings {
    /// Checks that are always enforced: hotkey shape, distinct hotkeys, sane time range.
    pub fn validate_shape(&self) -> Result<(), String> {
        parse_hotkey(&self.dictation_hotkey)?;
        parse_hotkey(&self.agent_hotkey)?;
        if same_hotkey(&self.dictation_hotkey, &self.agent_hotkey) {
            return Err("The two hotkeys must be different".into());
        }
        if self.auto_stop_seconds > MAX_AUTO_STOP_SECONDS {
            return Err(format!(
                "Auto-finish must be between 0 and {MAX_AUTO_STOP_SECONDS} seconds"
            ));
        }
        Ok(())
    }

    /// Everything above plus the reserved-shortcut rule, applied when the user saves.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_shape()?;
        for key in [&self.dictation_hotkey, &self.agent_hotkey] {
            if is_reserved(key) {
                return Err(format!(
                    "{key} is used by almost every app. Choose a less common shortcut, such as Ctrl+Alt+Space."
                ));
            }
        }
        Ok(())
    }

    /// Load from untrusted JSON. Missing fields get defaults; a broken file gives the defaults.
    /// An older file that holds a now-reserved shortcut still loads, so the app starts normally
    /// and the user is prompted to change it.
    pub fn from_json_lenient(json: &str) -> Settings {
        match serde_json::from_str::<Settings>(json.trim_start_matches('\u{feff}')) {
            Ok(settings) if settings.validate_shape().is_ok() => settings,
            _ => Settings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(change: impl FnOnce(&mut Settings)) -> Settings {
        let mut settings = Settings::default();
        change(&mut settings);
        settings
    }

    #[test]
    fn defaults_are_valid() {
        assert_eq!(Settings::default().validate(), Ok(()));
        assert_eq!(Settings::default().auto_stop_seconds, 3);
    }

    #[test]
    fn serializes_with_the_same_camel_case_keys_as_before() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        for key in [
            "dictationHotkey",
            "agentHotkey",
            "cleanup",
            "warmOnStartup",
            "autoStopSeconds",
        ] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
        for key in [
            "removeFillers",
            "spokenPunctuation",
            "autoCapitalize",
            "smartFormat",
        ] {
            assert!(json["cleanup"].get(key).is_some(), "missing cleanup.{key}");
        }
    }

    #[test]
    fn accepts_single_key_hotkeys() {
        assert_eq!(
            with(|s| {
                s.dictation_hotkey = "RightAlt".into();
                s.agent_hotkey = "F13".into();
            })
            .validate(),
            Ok(())
        );
    }

    #[test]
    fn rejects_a_lone_letter_and_made_up_keys() {
        for key in ["A", "LeftAlt", "Fn", "RightWin"] {
            assert!(
                with(|s| s.dictation_hotkey = key.into())
                    .validate()
                    .is_err(),
                "should reject {key}"
            );
        }
    }

    #[test]
    fn rejects_an_empty_hotkey_instead_of_silently_resetting() {
        assert!(with(|s| s.agent_hotkey = String::new()).validate().is_err());
    }

    #[test]
    fn rejects_shortcuts_that_would_break_everyday_editing() {
        for key in ["Control+A", "Control+W", "Control+C", "Alt+F4"] {
            let error = with(|s| s.dictation_hotkey = key.into())
                .validate()
                .unwrap_err();
            assert!(error.contains(key), "{error}");
        }
    }

    #[test]
    fn rejects_two_identical_hotkeys() {
        let error = with(|s| s.agent_hotkey = s.dictation_hotkey.clone())
            .validate()
            .unwrap_err();
        assert!(error.to_lowercase().contains("different"));
    }

    #[test]
    fn accepts_zero_to_ten_seconds_and_rejects_more() {
        assert!(with(|s| s.auto_stop_seconds = 0).validate().is_ok());
        assert!(with(|s| s.auto_stop_seconds = 10).validate().is_ok());
        assert!(with(|s| s.auto_stop_seconds = 11).validate().is_err());
    }

    #[test]
    fn an_old_settings_file_gets_the_new_fields_filled_in() {
        let old = r#"{"dictationHotkey":"Control+Alt+Space","agentHotkey":"Control+Alt+A",
            "cleanup":{"removeFillers":true,"spokenPunctuation":false,"autoCapitalize":true},"warmOnStartup":true}"#;
        let loaded = Settings::from_json_lenient(old);
        assert_eq!(loaded.auto_stop_seconds, 3);
        assert!(loaded.cleanup.smart_format);
    }

    #[test]
    fn a_broken_or_hostile_file_gives_the_defaults() {
        assert_eq!(Settings::from_json_lenient("not json"), Settings::default());
        assert_eq!(
            Settings::from_json_lenient(r#"{"agentHotkey": 42}"#),
            Settings::default()
        );
        assert_eq!(
            Settings::from_json_lenient(r#"{"dictationHotkey":"A"}"#),
            Settings::default()
        );
    }

    #[test]
    fn a_file_saved_with_a_byte_order_mark_still_loads() {
        let with_bom = "\u{feff}{\"dictationHotkey\":\"Control+Shift+D\"}";
        assert_eq!(
            Settings::from_json_lenient(with_bom).dictation_hotkey,
            "Control+Shift+D"
        );
    }

    #[test]
    fn an_older_file_with_a_now_reserved_shortcut_still_loads() {
        let old = r#"{"dictationHotkey":"Control+A","agentHotkey":"Control+W"}"#;
        let loaded = Settings::from_json_lenient(old);
        assert_eq!(loaded.dictation_hotkey, "Control+A");
        assert!(loaded.validate().is_err());
    }
}
