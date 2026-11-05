//! The restore state: everything `restore` needs, written to disk *before* the first change so a
//! crash, a power cut or a reboot mid-boost can still be undone. Its presence means "boosted".

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// Unix seconds when the boost started.
    pub boosted_at: u64,
    /// Services we stop, in stop order (dependents before the service they depend on).
    pub services: Vec<String>,
    /// Full image paths of the apps we close, relaunched on restore.
    pub processes: Vec<String>,
    pub power: Option<PowerState>,
    pub game_mode: Option<GameModeState>,
    /// Fastest-DNS change (absent in state files from older versions).
    #[serde(default)]
    pub dns: Option<crate::network::DnsState>,
    /// Processes `lower_others` lowered (absent in state files from older versions).
    #[serde(default)]
    pub lowered: Vec<crate::priority::Lowered>,
    /// Accessibility shortcut hotkeys turned off (absent in state files from older versions).
    #[serde(default)]
    pub accessibility: Option<AccessibilityState>,
}

/// Original `dwFlags` of each feature whose hotkey boost turns off; None = left alone.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AccessibilityState {
    pub sticky_keys: Option<u32>,
    pub filter_keys: Option<u32>,
    pub toggle_keys: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerState {
    /// The plan that was active before the boost; restore reactivates it.
    pub previous_scheme: String,
    /// Display brightness (AC, DC) of the previous plan, copied into ours so the screen doesn't jump.
    pub brightness: Option<(u32, u32)>,
    /// (setting GUID, AC, DC) of the previous plan's lid / power button / sleep button actions,
    /// copied into ours (absent in state files from older versions). See `power::KEEP`.
    #[serde(default)]
    pub kept: Vec<(String, u32, u32)>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GameModeState {
    /// The old `AutoGameModeEnabled` value, or None if it didn't exist (restore deletes it then).
    pub previous: Option<u32>,
}

impl State {
    pub fn load(path: &Path) -> Result<Option<State>, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| format!("state file {} is unreadable: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("can't read state file {}: {e}", path.display())),
        }
    }

    /// Write to a temp file and rename, so a crash never leaves a half-written state file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).expect("state serializes");
        std::fs::write(&tmp, json)
            .and_then(|_| std::fs::rename(&tmp, path))
            .map_err(|e| format!("can't write state file {}: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_missing() {
        let path = std::env::temp_dir().join(format!("gamebooster-test-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(State::load(&path).unwrap(), None);

        let st = State {
            boosted_at: 42,
            services: vec!["WSearch".into()],
            processes: vec![r"C:\Apps\OneDrive.exe".into()],
            power: Some(PowerState { previous_scheme: "381b4222-f694-41f0-9685-ff5bb260df2e".into(), brightness: Some((100, 75)), kept: vec![("5ca83367-6e45-459f-a27b-476b1d01c936".into(), 1, 1)] }),
            game_mode: Some(GameModeState { previous: None }),
            dns: Some(crate::network::DnsState { adapter_guid: "{4D36E972-E325-11CE-BFC1-08002BE10318}".into(), adapter: "Wi-Fi".into(), previous_static: None, server: "1.1.1.1".into() }),
            lowered: vec![crate::priority::Lowered { pid: 1234, exe: "chrome.exe".into(), created: 133_000_000_000_000_000, old_class: 0x20 }],
            accessibility: Some(AccessibilityState { sticky_keys: Some(0x1fe), filter_keys: None, toggle_keys: Some(0x3e) }),
        };
        st.save(&path).unwrap();
        assert_eq!(State::load(&path).unwrap(), Some(st));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn old_state_without_new_fields_loads() {
        let st: State = serde_json::from_str(r#"{"boosted_at":1,"services":[],"processes":[],"power":null,"game_mode":null}"#).unwrap();
        assert!(st.lowered.is_empty() && st.dns.is_none() && st.accessibility.is_none());
        let p: PowerState = serde_json::from_str(r#"{"previous_scheme":"x","brightness":null}"#).unwrap();
        assert!(p.kept.is_empty());
    }
}
