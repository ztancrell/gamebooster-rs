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
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerState {
    /// The plan that was active before the boost; restore reactivates it.
    pub previous_scheme: String,
    /// Display brightness (AC, DC) of the previous plan, copied into ours so the screen doesn't jump.
    pub brightness: Option<(u32, u32)>,
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
            power: Some(PowerState { previous_scheme: "381b4222-f694-41f0-9685-ff5bb260df2e".into(), brightness: Some((100, 75)) }),
            game_mode: Some(GameModeState { previous: None }),
            dns: Some(crate::network::DnsState { adapter_guid: "{4D36E972-E325-11CE-BFC1-08002BE10318}".into(), adapter: "Wi-Fi".into(), previous_static: None, server: "1.1.1.1".into() }),
        };
        st.save(&path).unwrap();
        assert_eq!(State::load(&path).unwrap(), Some(st));
        std::fs::remove_file(&path).unwrap();
    }
}
