//! The TOML config file: which games to prioritise, which services to stop, which apps to close,
//! and on/off switches for each tweak (so you can measure them one at a time).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Switch to our high-performance power plan during a boost.
    #[serde(default = "yes")]
    pub power_plan: bool,
    /// Make sure Windows Game Mode is on during a boost.
    #[serde(default = "yes")]
    pub game_mode: bool,
    /// Trim the working set of every non-game process once, right after closing apps.
    #[serde(default)]
    pub ram_trim: bool,
    /// Service key names (as in `sc query`, e.g. "WSearch"), stopped during a boost.
    #[serde(default)]
    pub services: Vec<String>,
    /// Process image names (e.g. "OneDrive.exe"), closed during a boost and relaunched on restore.
    #[serde(default)]
    pub close: Vec<String>,
    #[serde(default, rename = "game")]
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub exe: String,
    #[serde(default = "high")]
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Realtime,
    High,
    AboveNormal,
    Normal,
    BelowNormal,
    Low,
}

fn yes() -> bool {
    true
}
fn high() -> Priority {
    Priority::High
}

/// Processes we never close and never re-prioritise: Windows core, the shell, game launchers,
/// anti-virus (including the Symantec parts the spec names) and ourselves.
const PROTECTED: &[&str] = &[
    "system", "registry", "memcompression", "smss.exe", "csrss.exe", "wininit.exe",
    "winlogon.exe", "services.exe", "lsass.exe", "lsaiso.exe", "svchost.exe", "dwm.exe",
    "fontdrvhost.exe", "sihost.exe", "ctfmon.exe", "conhost.exe", "audiodg.exe", "explorer.exe",
    "taskmgr.exe", "steam.exe", "steamservice.exe", "steamwebhelper.exe", "eadesktop.exe",
    "eabackgroundservice.exe", "origin.exe", "epicgameslauncher.exe", "msmpeng.exe", "nissrv.exe",
    "securityhealthservice.exe", "ccsvchst.exe", "smc.exe", "smcgui.exe", "sepliveupdate.exe",
    "gamebooster.exe",
];

pub fn is_protected(exe: &str) -> bool {
    PROTECTED.iter().any(|p| p.eq_ignore_ascii_case(exe))
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, String> {
        let cfg: Config = toml::from_str(text).map_err(|e| e.to_string())?;
        for name in cfg.close.iter().chain(cfg.games.iter().map(|g| &g.exe)) {
            if is_protected(name) {
                return Err(format!("{name} is a protected process and can't be closed or re-prioritised"));
            }
        }
        if let Some(g) = cfg.close.iter().find(|c| cfg.is_game(c)) {
            return Err(format!("{g} is listed both as a game and under `close`"));
        }
        Ok(cfg)
    }

    pub fn load(path: &std::path::Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("can't read config {}: {e}", path.display()))?;
        Config::parse(&text).map_err(|e| format!("config {}: {e}", path.display()))
    }

    pub fn is_game(&self, exe: &str) -> bool {
        self.games.iter().any(|g| g.exe.eq_ignore_ascii_case(exe))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_parses() {
        let cfg = Config::parse(include_str!("../gamebooster.example.toml")).unwrap();
        assert!(cfg.power_plan && cfg.game_mode && !cfg.ram_trim);
        assert!(cfg.services.is_empty() && cfg.close.is_empty());
        assert_eq!(cfg.games[0].exe, "bf6.exe");
        assert_eq!(cfg.games[0].priority, Priority::High);
    }

    #[test]
    fn defaults_and_priority_names() {
        let cfg = Config::parse("[[game]]\nexe = \"a.exe\"\n[[game]]\nexe = \"b.exe\"\npriority = \"above_normal\"").unwrap();
        assert!(cfg.power_plan && cfg.game_mode);
        assert_eq!(cfg.games[0].priority, Priority::High);
        assert_eq!(cfg.games[1].priority, Priority::AboveNormal);
        assert!(cfg.is_game("A.EXE"));
    }

    #[test]
    fn rejects_protected_and_typos() {
        assert!(Config::parse("close = [\"Explorer.exe\"]").is_err());
        assert!(Config::parse("[[game]]\nexe = \"steam.exe\"").is_err());
        assert!(Config::parse("close = [\"bf6.exe\"]\n[[game]]\nexe = \"bf6.exe\"").is_err());
        assert!(Config::parse("servics = []").is_err());
        assert!(Config::parse("[[game]]\nexe = \"x.exe\"\npriority = \"turbo\"").is_err());
    }

    #[test]
    fn protected_is_case_insensitive() {
        assert!(is_protected("SVCHOST.EXE"));
        assert!(!is_protected("OneDrive.exe"));
    }
}
