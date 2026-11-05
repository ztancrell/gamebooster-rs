//! Game priority: raise the CPU priority class of the running game.
//!
//! **What it does.** For each `[[game]]` in the config that is running, sets its process priority
//! class (default High). Restore sets running games back to Normal; a game that has exited needs
//! nothing. The game must already be running: start BF6, then boost (or run `boost` again while
//! boosted, which only re-applies priority and power throttling; or use `boost --watch`, which
//! applies them when the game starts, see `watch.rs`).
//!
//! **Windows default.** Every app starts at Normal. The scheduler then shares CPU time between all
//! Normal threads, with a small boost for the foreground window.
//!
//! **Why it helps games.** At High, the game's threads win over every Normal-priority background
//! thread when both are ready to run, so a background burst can't delay the game's frame. It helps
//! when the CPU is busy (4-core Macs, BF6's large maps); on an idle CPU it changes nothing.
//! Avoid Realtime: it can starve input, audio and the driver threads the game itself needs,
//! causing stutter or a frozen system. Launchers and core processes are never touched.
//!
//! **Verify / measure.** Task Manager, Details tab, "Priority" column (or right-click > Set
//! priority) shows High for bf6.exe. Measure frame times in CapFrameX with a background load
//! (for example a big download) at Normal versus High.
//!
//! **Lower other processes (opt-in, `lower_others`).** Like SGB's TPriorityController (spec 2.4a),
//! but gentler: every process owned by the current user that is at Normal is set to Below normal
//! (or Idle, `lower_others_class`), except [`NEVER_LOWER`], exes whose publisher (version-info
//! CompanyName) matches [`NEVER_LOWER_PUBLISHERS`] or `lower_others_exclude_publishers`, the games,
//! the `close` list and `lower_others_exclude`. Each change is recorded (pid, name, creation time, old class) and restore
//! sets the old class back only on the same process (pid and creation time match), so a reused PID
//! is never touched. Processes already raised or lowered are left alone, so this never fights game
//! priority or a choice you made in Task Manager. Verify in Task Manager > Details > Base priority.
//! With `lower_others_mode = "dynamic"` boost lowers nothing; `boost --watch` restrains busy
//! candidates instead (`watch.rs`).

use crate::config::{LowerClass, Priority};
use serde::{Deserialize, Serialize};

/// Built-in exclusions for `lower_others` (case-insensitive; a trailing `*` matches any rest).
/// Shell and Windows session processes (lowering them makes the desktop and input laggy), audio,
/// voice chat, launchers, anti-cheat (EA Javelin = `EAAntiCheat.*`; also Easy Anti-Cheat and
/// BattlEye), ThrottleStop (it fixes the clock lock), OBS, security software and ourselves.
pub const NEVER_LOWER: &[&str] = &[
    "gamebooster.exe", "explorer.exe", "dwm.exe", "csrss.exe", "winlogon.exe", "fontdrvhost.exe",
    "ctfmon.exe", "sihost.exe", "taskhostw.exe", "runtimebroker.exe", "shellexperiencehost.exe",
    "startmenuexperiencehost.exe", "searchhost.exe", "textinputhost.exe", "audiodg.exe",
    "discord*", "steam.exe", "steamwebhelper.exe", "eadesktop.exe", "eabackgroundservice.exe",
    "ealauncher.exe", "eaanticheat*", "javelin*", "easyanticheat*", "beservice*", "throttlestop.exe",
    "obs64.exe", "msmpeng.exe", "securityhealth*", "nissrv.exe",
];

/// Built-in publisher exclusions for `lower_others`: case-insensitive substrings of the exe's
/// version-info CompanyName, so a renamed or new exe from these vendors is never lowered.
/// Launchers/stores and anti-cheat, GPU/driver vendors (Apple: Boot Camp helpers), security
/// software, OBS. Not "Microsoft" (too broad); Defender is covered by `NEVER_LOWER`.
pub const NEVER_LOWER_PUBLISHERS: &[&str] = &[
    "Valve", "Electronic Arts", "Epic Games", "Riot Games", "Ubisoft", "Blizzard", "Discord", "BattlEye",
    "NVIDIA", "Advanced Micro Devices", "Intel Corporation", "Apple Inc",
    "Malwarebytes", "Avast", "AVG Technologies", "ESET", "Kaspersky", "NortonLifeLock", "Gen Digital",
    "Symantec", "Bitdefender", "McAfee", "Sophos", "Trend Micro", "OBS Project",
];

/// Windows' NORMAL_PRIORITY_CLASS (also on non-Windows, for the tests).
pub const NORMAL_CLASS: u32 = 0x20;

/// One process `lower_others` changed, enough to put it back safely.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lowered {
    pub pid: u32,
    pub exe: String,
    /// Process creation time (FILETIME as u64): with the pid, identifies the same process.
    pub created: u64,
    /// Priority class before the boost (always Normal today).
    pub old_class: u32,
}

/// In the built-in list, a game, closed by `close`, in `lower_others_exclude`, or its publisher
/// (CompanyName, when known) matches a built-in or `lower_others_exclude_publishers` substring.
pub fn excluded(exe: &str, publisher: Option<&str>, cfg: &crate::config::Config) -> bool {
    let n = exe.to_ascii_lowercase();
    let co = publisher.unwrap_or_default().to_lowercase();
    let by_publisher = |s: &str| !s.is_empty() && co.contains(&s.to_lowercase());
    NEVER_LOWER.iter().any(|p| match p.strip_suffix('*') {
        Some(prefix) => n.starts_with(prefix),
        None => n == *p,
    }) || cfg.is_game(exe)
        || NEVER_LOWER_PUBLISHERS.iter().copied().chain(cfg.lower_others_exclude_publishers.iter().map(String::as_str)).any(by_publisher)
        || cfg.close.iter().chain(&cfg.lower_others_exclude).any(|c| c.eq_ignore_ascii_case(exe))
}

/// Lower only our own user's processes that are at Normal and not excluded.
pub fn should_lower(exe: &str, publisher: Option<&str>, same_user: bool, class: u32, cfg: &crate::config::Config) -> bool {
    same_user && class == NORMAL_CLASS && !excluded(exe, publisher, cfg)
}

pub fn lower_class_name(c: LowerClass) -> &'static str {
    match c {
        LowerClass::BelowNormal => "below normal",
        LowerClass::Idle => "idle",
    }
}

#[cfg(windows)]
pub fn class(p: Priority) -> u32 {
    use windows_sys::Win32::System::Threading::*;
    match p {
        Priority::Realtime => REALTIME_PRIORITY_CLASS,
        Priority::High => HIGH_PRIORITY_CLASS,
        Priority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
        Priority::Normal => NORMAL_PRIORITY_CLASS,
        Priority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
        Priority::Low => IDLE_PRIORITY_CLASS,
    }
}

#[cfg(windows)]
fn lower_class(c: LowerClass) -> u32 {
    use windows_sys::Win32::System::Threading::*;
    match c {
        LowerClass::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
        LowerClass::Idle => IDLE_PRIORITY_CLASS,
    }
}

/// The user SID of a process (as bytes), or None if Windows won't tell us.
#[cfg(windows)]
pub fn user_sid(h: windows_sys::Win32::Foundation::HANDLE) -> Option<Vec<u8>> {
    use windows_sys::Win32::Security::{GetLengthSid, GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    unsafe {
        let mut tok = std::ptr::null_mut();
        if OpenProcessToken(h, TOKEN_QUERY, &mut tok) == 0 {
            return None;
        }
        let tok = crate::processes::Process(tok); // closed on drop
        let mut buf = [0u64; 64]; // u64 for TOKEN_USER's alignment; a SID is at most 68 bytes
        let mut len = 0;
        if GetTokenInformation(tok.0, TokenUser, buf.as_mut_ptr().cast(), std::mem::size_of_val(&buf) as u32, &mut len) == 0 {
            return None;
        }
        let sid = (*(buf.as_ptr() as *const TOKEN_USER)).User.Sid;
        Some(std::slice::from_raw_parts(sid as *const u8, GetLengthSid(sid) as usize).to_vec())
    }
}

#[cfg(windows)]
pub fn created(h: windows_sys::Win32::Foundation::HANDLE) -> Option<u64> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetProcessTimes;
    let z = || FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut c, mut e, mut k, mut u) = (z(), z(), z(), z());
    (unsafe { GetProcessTimes(h, &mut c, &mut e, &mut k, &mut u) } != 0).then(|| (c.dwHighDateTime as u64) << 32 | c.dwLowDateTime as u64)
}

/// The CompanyName from an exe's version info (first translation). Not signature-checked: a spoofed
/// name only makes us skip that exe, which is harmless for an exclusion list.
#[cfg(windows)]
pub fn company_name(path: &str) -> Option<String> {
    use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    let path = crate::wide(path);
    unsafe {
        let size = GetFileVersionInfoSizeW(path.as_ptr(), std::ptr::null_mut());
        let mut buf = vec![0u8; size as usize];
        if size == 0 || GetFileVersionInfoW(path.as_ptr(), 0, size, buf.as_mut_ptr().cast()) == 0 {
            return None;
        }
        let (mut p, mut len) = (std::ptr::null_mut(), 0u32);
        let query = |key: &str, p: &mut *mut core::ffi::c_void, len: &mut u32| VerQueryValueW(buf.as_ptr().cast(), crate::wide(key).as_ptr(), p, len) != 0 && *len > 0;
        if !query("\\VarFileInfo\\Translation", &mut p, &mut len) || len < 4 {
            return None;
        }
        let [lang, cp] = (p as *const [u16; 2]).read_unaligned();
        if !query(&format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\CompanyName"), &mut p, &mut len) {
            return None;
        }
        Some(crate::from_wide(std::slice::from_raw_parts(p as *const u16, len as usize)))
    }
}

/// Read-only: the processes `lower_others` will lower, with what restore needs.
#[cfg(windows)]
pub fn plan_lower(cfg: &crate::config::Config) -> Vec<Lowered> {
    use crate::processes::{list, Process};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetPriorityClass, PROCESS_QUERY_LIMITED_INFORMATION};
    let Some(me) = user_sid(unsafe { GetCurrentProcess() }) else { return Vec::new() };
    let own_pid = std::process::id();
    list()
        .into_iter()
        .filter(|p| p.pid != own_pid && !excluded(&p.name, None, cfg))
        .filter_map(|p| {
            let h = Process::open(p.pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
            let same_user = user_sid(h.0).as_deref() == Some(&me[..]);
            let class = unsafe { GetPriorityClass(h.0) };
            let publisher = p.path.as_deref().and_then(company_name);
            if !should_lower(&p.name, publisher.as_deref(), same_user, class, cfg) {
                return None;
            }
            Some(Lowered { pid: p.pid, created: created(h.0)?, exe: p.name, old_class: class })
        })
        .collect()
}

/// Set each process to `to` (or back to its old class when `to` is None), only if it is still the
/// same process. Prints one summary line; failures are counted, never fatal.
#[cfg(windows)]
pub fn set_others(procs: &[Lowered], to: Option<LowerClass>) {
    use crate::processes::Process;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ACCESS_DENIED};
    use windows_sys::Win32::System::Threading::{SetPriorityClass, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION};
    let (mut done, mut gone, mut denied) = (0, 0, 0);
    for p in procs {
        let Some(h) = Process::open(p.pid, PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION) else {
            match unsafe { GetLastError() } {
                ERROR_ACCESS_DENIED => denied += 1,
                _ => gone += 1, // ERROR_INVALID_PARAMETER: no such pid any more
            }
            continue;
        };
        if created(h.0) != Some(p.created) {
            gone += 1; // the pid now belongs to another process
        } else if unsafe { SetPriorityClass(h.0, to.map_or(p.old_class, lower_class)) } != 0 {
            done += 1;
        } else {
            denied += 1;
        }
    }
    println!("  {}", summary(done, gone, denied, to));
}

/// "lowered 23 processes to below normal (4 skipped: access denied, 1 exited)".
pub fn summary(done: usize, gone: usize, denied: usize, to: Option<LowerClass>) -> String {
    let mut s = match to {
        Some(c) => format!("lowered {done} processes to {}", lower_class_name(c)),
        None => format!("restored the old priority of {done} processes"),
    };
    let mut skipped = Vec::new();
    if denied > 0 {
        skipped.push(format!("{denied} skipped: access denied"));
    }
    if gone > 0 {
        skipped.push(format!("{gone} exited"));
    }
    if !skipped.is_empty() {
        s += &format!(" ({})", skipped.join(", "));
    }
    s
}

/// (ControlMask, StateMask) for `ProcessPowerThrottling`: boost opts the game out of EcoQoS
/// (control execution speed, state off); restore clears both masks, handing control back to Windows.
/// Not IGNORE_TIMER_RESOLUTION: timer-resolution tweaks are rejected (docs/hone/README.md).
pub fn throttling_masks(reset: bool) -> (u32, u32) {
    const EXECUTION_SPEED: u32 = 1; // PROCESS_POWER_THROTTLING_EXECUTION_SPEED
    if reset { (0, 0) } else { (EXECUTION_SPEED, 0) }
}

/// Opt a process out of power throttling (or hand it back to Windows when `reset`).
/// Needs Windows 10 1709+; fails with access denied or invalid parameter otherwise.
#[cfg(windows)]
fn set_throttling(h: windows_sys::Win32::Foundation::HANDLE, reset: bool) -> Result<(), String> {
    use windows_sys::Win32::System::Threading::{ProcessPowerThrottling, SetProcessInformation, PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_STATE};
    let (control, state) = throttling_masks(reset);
    let s = PROCESS_POWER_THROTTLING_STATE { Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION, ControlMask: control, StateMask: state };
    if unsafe { SetProcessInformation(h, ProcessPowerThrottling, (&s as *const PROCESS_POWER_THROTTLING_STATE).cast(), std::mem::size_of_val(&s) as u32) } != 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// Set every running process of each game to its priority (or to Normal when `reset`), and, with
/// `throttling_off`, opt it out of power throttling (EcoQoS; `reset` hands it back to Windows).
/// Restore matches by exe name, like priority: the masks it sets are Windows' default, so a game
/// relaunched since the boost (or a reused pid, which can't be a game exe) is left as Windows has it.
/// Returns how many processes were found.
#[cfg(windows)]
pub fn apply(games: &[crate::config::Game], reset: bool, throttling_off: bool) -> usize {
    use crate::processes::{list, Process};
    use windows_sys::Win32::System::Threading::{SetPriorityClass, PROCESS_SET_INFORMATION};
    let mut found = 0;
    for p in list() {
        let Some(game) = games.iter().find(|g| g.exe.eq_ignore_ascii_case(&p.name)) else { continue };
        found += 1;
        let want = if reset { Priority::Normal } else { game.priority };
        let h = Process::open(p.pid, PROCESS_SET_INFORMATION);
        let ok = h.as_ref().is_some_and(|h| unsafe { SetPriorityClass(h.0, class(want)) } != 0);
        println!("  {} {} (pid {}) to {want:?}", if ok { "set" } else { "could not set" }, p.name, p.pid);
        if throttling_off {
            let what = if reset { "power throttling handed back to Windows" } else { "power throttling off" };
            match h.as_ref().map_or_else(|| Err(std::io::Error::last_os_error().to_string()), |h| set_throttling(h.0, reset)) {
                Ok(()) => println!("  {}: {what}", p.name),
                Err(e) => println!("  {}: {what} failed: {e}", p.name),
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn exclusions() {
        let cfg = Config::parse("close = [\"OneDrive.exe\"]\nlower_others_exclude = [\"Spotify.exe\"]\n[[game]]\nexe = \"bf6.exe\"").unwrap();
        for name in ["BF6.exe", "onedrive.exe", "spotify.EXE", "Explorer.EXE", "Discord.exe", "DiscordPTB.exe",
            "EAAntiCheat.GameServiceLauncher.exe", "SecurityHealthSystray.exe", "obs64.exe", "gamebooster.exe"] {
            assert!(excluded(name, None, &cfg), "{name} not excluded");
        }
        for name in ["chrome.exe", "OneDriveSetup.exe", "Teams.exe", "explorer2.exe"] {
            assert!(!excluded(name, None, &cfg), "{name} excluded");
        }
    }

    #[test]
    fn publisher_exclusions() {
        let cfg = Config::parse("lower_others_exclude_publishers = [\"Spotify AB\"]").unwrap();
        for co in ["NVIDIA Corporation", "Valve Corp.", "Epic Games, Inc.", "spotify ab"] {
            assert!(excluded("renamed.exe", Some(co), &cfg), "{co} not excluded");
        }
        for co in [None, Some(""), Some("Microsoft Corporation"), Some("Google LLC")] {
            assert!(!excluded("chrome.exe", co, &cfg), "{co:?} excluded");
        }
    }

    #[test]
    fn lowers_only_own_normal_processes() {
        let cfg = Config::parse("").unwrap();
        assert!(should_lower("chrome.exe", None, true, NORMAL_CLASS, &cfg));
        assert!(!should_lower("chrome.exe", None, false, NORMAL_CLASS, &cfg)); // another user / SYSTEM
        assert!(!should_lower("chrome.exe", None, true, 0x80, &cfg)); // already High
        assert!(!should_lower("chrome.exe", None, true, 0x4000, &cfg)); // already Below normal
        assert!(!should_lower("steam.exe", None, true, NORMAL_CLASS, &cfg));
    }

    #[test]
    fn summary_text() {
        assert_eq!(summary(23, 0, 4, Some(LowerClass::BelowNormal)), "lowered 23 processes to below normal (4 skipped: access denied)");
        assert_eq!(summary(5, 2, 0, None), "restored the old priority of 5 processes (2 exited)");
        assert_eq!(summary(1, 0, 0, Some(LowerClass::Idle)), "lowered 1 processes to idle");
    }

    #[test]
    fn throttling() {
        assert_eq!(throttling_masks(false), (1, 0)); // opt out: control execution speed, keep it off
        assert_eq!(throttling_masks(true), (0, 0)); // restore: Windows decides again
    }
}
