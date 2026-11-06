//! `boost --watch`: after the boost, keep running in the console until Ctrl+C or the console closes.
//!
//! **What it does.** Once a second it looks at the running processes and:
//! - sets game priority and power throttling off on a configured game that just started (so you no
//!   longer need to run `boost` again after launching BF6). When a game exits, both end with the
//!   process; there is nothing to reset.
//! - with `lower_others = true` and `lower_others_mode = "dynamic"`: restrains busy background
//!   processes ProBalance-style (Process Lasso). A candidate (same filter as `lower_others`, see
//!   [`crate::priority::should_lower`], and never the foreground window's process or its
//!   descendants) is set to Below normal when the whole machine is at least [`PROBALANCE`]'s
//!   "machine" % busy and the process has used at least its "process" % of the machine for
//!   [`SUSTAIN_MS`]. It goes back to Normal once it is under the "release" % and has been restrained
//!   for at least [`MIN_HOLD_MS`], or at once when it (or its parent) becomes the foreground window.
//!   Below normal only, never Idle.
//!
//! **Safety.** Every restraint is written to the state file's `lowered` journal (pid, creation time,
//! exe, old class) before the priority changes, and removed on release, so a crash or a killed
//! watcher leaves `gamebooster restore` able to undo it. On exit the watcher releases what it
//! restrained and leaves everything else boosted. If the state file disappears (restore ran in
//! another console), the watcher stops without touching anything.
//!
//! **`gamebooster auto`** runs the same loop without a boost first: [`auto_step`] boosts (the normal
//! `boost`) when a configured game starts and runs the normal `restore` `auto_restore_delay_secs`
//! after the last one exits. It only restores boosts it made; Ctrl+C or closing the console while
//! it is boosted restores (restore keeps the state file until it has finished). With the registry
//! priority method auto writes the games' IFEO entries when it starts and holds them until it exits
//! (`crate::hold_ifeo`): Windows reads them only at game start, after which auto's boost is too late.

use std::collections::HashSet;

/// Process Lasso's ProBalance defaults by logical CPU count, normal (non-hybrid) branch:
/// (machine busy %, process share of the machine %, release below %). Row i is for i + 1 logical
/// CPUs; 33 and more use the last row. Source: decomp/process-lasso/governor/notes.md, "ProBalance
/// defaults by logical CPU count" (ProcessGovernor.exe 19.0, `.rdata` tables 0x140115230-0x1401153b0).
/// The hybrid-CPU branch (P/E cores) is not reproduced.
pub const PROBALANCE: [(u8, u8, u8); 32] = [
    (75, 64, 15), (45, 32, 7), (40, 20, 5), (23, 15, 5), // 1-4
    (18, 10, 5), (18, 10, 5), (17, 9, 5), (11, 8, 4), // 5-8
    (11, 8, 3), (10, 8, 3), (10, 7, 3), (10, 7, 3), // 9-12
    (10, 6, 2), (10, 6, 2), (10, 5, 2), (10, 5, 2), // 13-16
    (9, 5, 1), (9, 5, 1), (9, 5, 1), (9, 4, 1), // 17-20
    (9, 4, 1), (8, 4, 1), (8, 4, 1), (8, 4, 1), // 21-24
    (8, 3, 1), (8, 3, 1), (7, 3, 1), (7, 3, 1), // 25-28
    (7, 3, 1), (7, 2, 1), (7, 2, 1), (7, 2, 1), // 29-32
];
/// Process Lasso's fallback when the CPU count is unknown (same source).
const UNKNOWN_CPUS: (u8, u8, u8) = (70, 20, 5);
/// Time over the process threshold before a restraint (`TimeOverQuotaBeforeRestraint`).
pub const SUSTAIN_MS: u64 = 900;
/// Minimum time restrained before a quiet process is released (`MinimumTimeOfRestraint`; no maximum).
pub const MIN_HOLD_MS: u64 = 4200;
/// Sampling interval (Process Lasso's core refresh rate is about the same).
pub const TICK_MS: u64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    pub system: f64,
    pub process: f64,
    pub release: f64,
}

pub fn thresholds(cpus: usize) -> Thresholds {
    let (system, process, release) = if cpus == 0 { UNKNOWN_CPUS } else { PROBALANCE[cpus.min(32) - 1] };
    Thresholds { system: system.into(), process: process.into(), release: release.into() }
}

/// Percent of the whole machine: CPU time used over wall time times logical CPUs (same units).
pub fn cpu_percent(cpu_delta: u64, elapsed: u64, cpus: usize) -> f64 {
    if elapsed == 0 || cpus == 0 {
        return 0.0;
    }
    cpu_delta as f64 * 100.0 / (elapsed as f64 * cpus as f64)
}

/// Machine busy % from `GetSystemTimes` deltas (kernel time includes idle time).
pub fn busy_percent(idle: u64, kernel: u64, user: u64) -> f64 {
    let total = kernel + user;
    if total == 0 { 0.0 } else { total.saturating_sub(idle) as f64 * 100.0 / total as f64 }
}

/// `root` and all its descendants, from (pid, parent pid) pairs. Root 0 (no foreground window)
/// gives nothing. A stale parent pid (Windows reuses pids) can only add extra processes, which are
/// then left alone: harmless.
pub fn descendants(root: u32, parents: &[(u32, u32)]) -> HashSet<u32> {
    let mut set = HashSet::new();
    if root == 0 {
        return set;
    }
    set.insert(root);
    loop {
        let before = set.len();
        for &(pid, parent) in parents {
            if pid != parent && set.contains(&parent) {
                set.insert(pid);
            }
        }
        if set.len() == before {
            return set;
        }
    }
}

/// Restraint state of one process. Times are milliseconds since the watcher started.
#[derive(Debug, Default)]
pub struct Track {
    /// When the process first went over the process threshold (with the machine busy enough).
    pub over_since: Option<u64>,
    pub restrained_at: Option<u64>,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Restrain,
    Release,
}

impl Track {
    /// One sample. `foreground`: the process is the foreground window's process or a descendant.
    pub fn step(&mut self, now: u64, system: f64, process: f64, foreground: bool, t: Thresholds) -> Option<Event> {
        if let Some(at) = self.restrained_at {
            if foreground || (process < t.release && now.saturating_sub(at) >= MIN_HOLD_MS) {
                *self = Track::default();
                return Some(Event::Release);
            }
            return None;
        }
        if foreground || system < t.system || process < t.process {
            self.over_since = None;
            return None;
        }
        let since = *self.over_since.get_or_insert(now);
        (now.saturating_sub(since) >= SUSTAIN_MS).then(|| {
            self.restrained_at = Some(now);
            Event::Restrain
        })
    }
}

/// `gamebooster auto`: where it stands. Times are milliseconds since the watcher started.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Auto {
    /// Not boosted: boost when a configured game starts.
    Idle,
    /// Our boost, a game running.
    Boosted,
    /// Our boost, no game running since this time: restore once the delay has passed.
    Grace(u64),
    /// A boost auto didn't make (state file found): watch it like `boost --watch`, never restore it.
    Manual,
    /// Restore ran elsewhere while a game was running: don't boost again until no game runs.
    Disarmed,
}

#[derive(Debug, PartialEq)]
pub enum AutoDo {
    Boost,
    Restore,
}

/// One tick of `auto`. `boosted`: the state file exists.
pub fn auto_step(s: Auto, now: u64, game: bool, boosted: bool, delay_ms: u64) -> (Auto, Option<AutoDo>) {
    use Auto::*;
    match s {
        Idle if boosted => (Manual, None),
        Idle if game => (Boosted, Some(AutoDo::Boost)),
        Idle => (Idle, None),
        Disarmed if !game => (Idle, None),
        Disarmed => (Disarmed, None),
        _ if !boosted => (if game { Disarmed } else { Idle }, None), // restore ran elsewhere
        Manual => (Manual, None),
        Boosted | Grace(_) if game => (Boosted, None),
        Boosted => auto_step(Grace(now), now, game, boosted, delay_ms),
        Grace(t) if now.saturating_sub(t) >= delay_ms => (Idle, Some(AutoDo::Restore)),
        Grace(t) => (Grace(t), None),
    }
}

#[cfg(windows)]
pub use sys::run;

#[cfg(windows)]
mod sys {
    use super::*;
    use crate::config::Config;
    use crate::priority::{self, Lowered, NORMAL_CLASS};
    use crate::processes::{self, Process};
    use crate::state::State;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{FILETIME, HANDLE};
    use windows_sys::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_C_EVENT};
    use windows_sys::Win32::System::Threading::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    static STOP: AtomicBool = AtomicBool::new(false);
    static DONE: AtomicBool = AtomicBool::new(false);

    unsafe extern "system" fn on_ctrl(kind: u32) -> windows_sys::core::BOOL {
        STOP.store(true, SeqCst);
        if kind != CTRL_C_EVENT && kind != CTRL_BREAK_EVENT {
            // Console closed / sign-out / shutdown: Windows ends us when this returns (after about
            // 5 s at most), so wait here for the main loop's cleanup.
            for _ in 0..45 {
                if DONE.load(SeqCst) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        1
    }

    fn ft(f: FILETIME) -> u64 {
        (f.dwHighDateTime as u64) << 32 | f.dwLowDateTime as u64
    }
    fn zero() -> FILETIME {
        FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 }
    }

    /// Kernel + user CPU time of a process (100 ns).
    fn cpu_time(h: HANDLE) -> Option<u64> {
        let (mut c, mut e, mut k, mut u) = (zero(), zero(), zero(), zero());
        (unsafe { GetProcessTimes(h, &mut c, &mut e, &mut k, &mut u) } != 0).then(|| ft(k) + ft(u))
    }

    /// (idle, kernel, user) of the whole machine (100 ns).
    fn system_times() -> (u64, u64, u64) {
        let (mut i, mut k, mut u) = (zero(), zero(), zero());
        unsafe { GetSystemTimes(&mut i, &mut k, &mut u) };
        (ft(i), ft(k), ft(u))
    }

    fn foreground_pid() -> u32 {
        let mut pid = 0;
        let w = unsafe { GetForegroundWindow() };
        if !w.is_null() {
            unsafe { GetWindowThreadProcessId(w, &mut pid) };
        }
        pid
    }

    /// Change the state file's `lowered` journal (atomic write). Fails if the state file is gone.
    fn journal(path: &Path, f: impl FnOnce(&mut Vec<Lowered>)) -> Result<(), String> {
        let mut st = State::load(path)?.ok_or("not boosted any more")?;
        f(&mut st.lowered);
        st.save(path)
    }

    /// One process seen by the watcher. `h` is Some only for restraint candidates; holding the
    /// handle also keeps Windows from reusing the pid while we track it.
    struct Entry {
        exe: String,
        created: u64,
        h: Option<Process>,
        cpu: u64,
        track: Track,
    }

    /// `auto`: boost when a game starts and restore after the last one exits (see [`auto_step`]).
    pub fn run(cfg: &Config, state_path: &Path, auto: bool) -> Result<(), String> {
        if auto && cfg.games.is_empty() {
            return Err("auto needs at least one [[game]] in the config".into());
        }
        let delay = cfg.auto_restore_delay_secs;
        let cpus = std::thread::available_parallelism().map_or(0, |n| n.get());
        let t = thresholds(cpus);
        let dynamic = cfg.dynamic_lowering();
        unsafe { SetConsoleCtrlHandler(Some(on_ctrl), 1) };
        if auto {
            println!("Auto: boost when a configured game starts, restore {delay} s after the last one exits.");
            println!("Ctrl+C or closing this window restores a boost auto made, then exits.");
        } else {
            println!("Watching once a second until Ctrl+C or this window closes.");
        }
        println!("  games: priority and power throttling set when a configured game starts");
        if dynamic {
            println!("  lower others (dynamic, {cpus} logical CPUs): restrain to below normal when the machine is >= {} % busy and a process uses >= {} % for {:.1} s; release under {} % after >= {:.1} s, or when it comes to the foreground",
                t.system, t.process, SUSTAIN_MS as f64 / 1000.0, t.release, MIN_HOLD_MS as f64 / 1000.0);
            if cfg.lower_others_class == crate::config::LowerClass::Idle {
                println!("  note: dynamic mode uses below normal only; lower_others_class = \"idle\" is ignored");
            }
        } else {
            println!("  lower others: {}", if cfg.lower_others { "once (done by boost); set lower_others_mode = \"dynamic\" for the watcher to do it" } else { "off" });
        }
        if auto && !cfg.process_priority() {
            crate::hold_ifeo(cfg, state_path)?;
        }
        let me = priority::user_sid(unsafe { GetCurrentProcess() }).unwrap_or_default();
        let own_pid = std::process::id();
        let start = Instant::now();
        let (mut prev_at, mut prev_sys) = (start, system_times());
        let mut procs: HashMap<u32, Entry> = HashMap::new();
        let mut games: HashSet<u32> = HashSet::new();
        let (mut last_status, mut status_at) = (String::new(), None::<Instant>);
        let mut mode = Auto::Idle;
        let stopped_by_restore = loop {
            for _ in 0..TICK_MS / 100 {
                if !STOP.load(SeqCst) {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            if STOP.load(SeqCst) {
                break false;
            }
            // ponytail: a restore landing between this check and a journal write can leave a fresh
            // state file behind; narrow window, a second `restore` cleans it up.
            if !auto && !state_path.exists() {
                println!("Not boosted any more (restore ran): stopping the watcher.");
                break true;
            }
            let now = Instant::now();
            let ms = now.duration_since(start).as_millis() as u64;
            let elapsed = now.duration_since(prev_at).as_nanos() as u64 / 100;
            prev_at = now;
            let sys = system_times();
            let busy = busy_percent(sys.0 - prev_sys.0, sys.1 - prev_sys.1, sys.2 - prev_sys.2);
            prev_sys = sys;
            let list = processes::list();

            // Games: apply to a newly started one (re-applying to the others is harmless).
            let running: HashSet<u32> = list.iter().filter(|p| cfg.is_game(&p.name)).map(|p| p.pid).collect();
            let mut fresh = running.iter().any(|p| !games.contains(p));
            if auto {
                let (next, act) = auto_step(mode, ms, !running.is_empty(), state_path.exists(), delay * 1000);
                let game = list.iter().find(|p| running.contains(&p.pid)).map_or("game", |p| p.name.as_str());
                match (mode, next, act) {
                    (_, _, Some(AutoDo::Boost)) => {
                        println!("{game} started: boosting");
                        if let Err(e) = crate::boost(cfg, state_path, false, true, true) {
                            println!("boost failed: {e}");
                        }
                        fresh = false; // boost just applied game priority
                    }
                    (_, _, Some(AutoDo::Restore)) => {
                        println!("{}", if mode == Auto::Boosted { "last game exited: restoring".to_string() } else { format!("no game for {delay} s: restoring") });
                        if let Err(e) = crate::restore(cfg, state_path, false) {
                            println!("restore failed: {e}");
                        }
                        if !state_path.exists() {
                            println!("restored; waiting for a game");
                        } // else: the next tick sees the kept state file as a boost auto didn't make
                    }
                    (Auto::Boosted, Auto::Grace(_), _) => println!("last game exited: restoring in {delay} s"),
                    (Auto::Grace(_), Auto::Boosted, _) => println!("{game} started again: keeping the boost"),
                    (_, Auto::Manual, _) => println!("Already boosted, not by auto: watching it like `boost --watch`; auto won't restore it"),
                    (_, Auto::Disarmed, _) => println!("Not boosted any more (restore ran): waiting until the games exit"),
                    (Auto::Disarmed, Auto::Idle, _) => println!("games exited; waiting for a game"),
                    (Auto::Boosted | Auto::Grace(_) | Auto::Manual, Auto::Idle, None) => println!("Not boosted any more (restore ran); waiting for a game"),
                    _ => {}
                }
                mode = next;
            }
            let boosted = state_path.exists();
            if fresh && boosted {
                println!("Game started:");
                priority::apply(&cfg.games, false, cfg.game_power_throttling_off, cfg.process_priority());
            }
            if !auto && games.iter().any(|p| !running.contains(p)) && running.is_empty() {
                println!("Game exited (its priority and power throttling ended with it).");
            }
            games = running;

            if !boosted {
                procs.clear(); // restore reset what was restrained; nothing to journal into
            } else if dynamic {
                let alive: HashSet<u32> = list.iter().map(|p| p.pid).collect();
                procs.retain(|pid, e| {
                    let keep = alive.contains(pid);
                    if !keep && e.track.restrained_at.is_some() {
                        let (pid, created) = (*pid, e.created);
                        if let Err(err) = journal(state_path, |l| l.retain(|x| !(x.pid == pid && x.created == created))) {
                            println!("  journal update failed: {err}");
                        }
                    }
                    keep
                });
                let parents: Vec<(u32, u32)> = list.iter().map(|p| (p.pid, p.parent)).collect();
                let fg = descendants(foreground_pid(), &parents);
                for p in &list {
                    if procs.get(&p.pid).is_some_and(|e| !e.exe.eq_ignore_ascii_case(&p.name)) {
                        procs.remove(&p.pid); // pid reused by another exe
                    }
                    let e = procs.entry(p.pid).or_insert_with(|| new_entry(p, &me, own_pid, cfg));
                    let Some(h) = &e.h else { continue };
                    let Some(cpu) = cpu_time(h.0) else { continue };
                    let pct = cpu_percent(cpu.saturating_sub(e.cpu), elapsed, cpus);
                    e.cpu = cpu;
                    let class = unsafe { GetPriorityClass(h.0) };
                    if e.track.restrained_at.is_none() && class != NORMAL_CLASS {
                        e.track = Track::default(); // someone else changed it: not ours to touch
                        continue;
                    }
                    let held = e.track.restrained_at.map(|at| ms.saturating_sub(at));
                    let foreground = fg.contains(&p.pid);
                    match e.track.step(ms, busy, pct, foreground, t) {
                        Some(Event::Restrain) => {
                            let entry = Lowered { pid: p.pid, exe: p.name.clone(), created: e.created, old_class: class };
                            let ok = journal(state_path, |l| l.push(entry)).map_err(|err| format!("journal failed: {err}"))
                                .and_then(|_| (unsafe { SetPriorityClass(h.0, BELOW_NORMAL_PRIORITY_CLASS) } != 0).then_some(()).ok_or_else(|| std::io::Error::last_os_error().to_string()));
                            match ok {
                                Ok(()) => println!("  restrained {} (pid {}, {pct:.0} % CPU) to below normal", p.name, p.pid),
                                Err(err) => {
                                    println!("  could not restrain {} (pid {}): {err}", p.name, p.pid);
                                    e.track = Track::default();
                                    let (pid, created) = (p.pid, e.created);
                                    let _ = journal(state_path, |l| l.retain(|x| !(x.pid == pid && x.created == created)));
                                }
                            }
                        }
                        Some(Event::Release) => {
                            // Only undo our own change: leave it if someone set another class meanwhile.
                            let ok = if class != BELOW_NORMAL_PRIORITY_CLASS || unsafe { SetPriorityClass(h.0, NORMAL_CLASS) } != 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
                            let (pid, created) = (p.pid, e.created);
                            let j = journal(state_path, |l| l.retain(|x| !(x.pid == pid && x.created == created)));
                            let why = if foreground { "now in the foreground" } else { "quiet again" };
                            let secs = held.unwrap_or(0) as f64 / 1000.0;
                            match (ok, j) {
                                (Ok(()), Ok(())) => println!("  released {} (pid {}) after {secs:.0} s: {why}", p.name, p.pid),
                                (Err(err), _) => println!("  could not release {} (pid {}): {err}", p.name, p.pid),
                                (Ok(()), Err(err)) => println!("  released {} (pid {}), journal update failed: {err}", p.name, p.pid),
                            }
                        }
                        None => {}
                    }
                }
            }

            // Status: only when something changed, at most every 10 s.
            let mut names: Vec<&str> = procs.values().filter(|e| e.track.restrained_at.is_some()).map(|e| e.exe.as_str()).collect();
            names.sort_unstable();
            let key = format!("{} restrained{}, game {}", names.len(), if names.is_empty() { String::new() } else { format!(" ({})", names.join(", ")) },
                if games.is_empty() { "not running" } else { "running" });
            if key != last_status && status_at.is_none_or(|at| now.duration_since(at) >= Duration::from_secs(10)) {
                let s = ms / 1000;
                println!("[{}:{:02}] CPU {busy:.0} %, {key}", s / 60, s % 60);
                (last_status, status_at) = (key, Some(now));
            }
        };

        if matches!(mode, Auto::Boosted | Auto::Grace(_)) {
            // If Windows ends us mid-restore (console closed), the state file stays for a later `restore`.
            println!("Stopping: restoring the boost auto made.");
            if let Err(e) = crate::restore(cfg, state_path, false) {
                println!("restore failed: {e}");
            }
        } else if !stopped_by_restore && state_path.exists() {
            let held: Vec<Lowered> = procs.iter().filter(|(_, e)| e.track.restrained_at.is_some())
                .map(|(&pid, e)| Lowered { pid, exe: e.exe.clone(), created: e.created, old_class: NORMAL_CLASS }).collect();
            println!("Stopping the watcher.");
            if !held.is_empty() {
                priority::set_others(&held, None);
                if let Err(err) = journal(state_path, |l| l.retain(|x| !held.iter().any(|h| h.pid == x.pid && h.created == x.created))) {
                    println!("  journal update failed: {err} (`gamebooster restore` will retry them)");
                }
            }
            println!("Still boosted: run `gamebooster restore` when you're done playing.");
        }
        if auto {
            if let Err(e) = crate::release_ifeo(state_path) {
                println!("IFEO restore failed: {e} (`gamebooster restore` will retry)");
            }
        }
        DONE.store(true, SeqCst);
        Ok(())
    }

    /// First sight of a process: a restraint candidate gets a handle, everything else is cached as
    /// "not a candidate" (so the publisher is read once per process, not every second).
    fn new_entry(p: &processes::Proc, me: &[u8], own_pid: u32, cfg: &Config) -> Entry {
        let mut e = Entry { exe: p.name.clone(), created: 0, h: None, cpu: 0, track: Track::default() };
        if p.pid == own_pid || p.pid == 0 || me.is_empty() || priority::excluded(&p.name, None, cfg) {
            return e;
        }
        let Some(h) = Process::open(p.pid, PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION) else { return e };
        let same_user = priority::user_sid(h.0).as_deref() == Some(me);
        let publisher = if same_user { p.path.as_deref().and_then(priority::company_name) } else { None };
        // Class is checked every tick, so pass Normal here: only the user and the exclusions decide.
        if !priority::should_lower(&p.name, publisher.as_deref(), same_user, NORMAL_CLASS, cfg) {
            return e;
        }
        let (Some(created), Some(cpu)) = (priority::created(h.0), cpu_time(h.0)) else { return e };
        (e.created, e.cpu, e.h) = (created, cpu, Some(h));
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_table() {
        assert_eq!(thresholds(16), Thresholds { system: 10.0, process: 5.0, release: 2.0 }); // 2019 MBP 16"
        assert_eq!(thresholds(12), Thresholds { system: 10.0, process: 7.0, release: 3.0 });
        assert_eq!(thresholds(8), Thresholds { system: 11.0, process: 8.0, release: 4.0 });
        assert_eq!(thresholds(1), Thresholds { system: 75.0, process: 64.0, release: 15.0 });
        assert_eq!(thresholds(64), thresholds(32));
        assert_eq!(thresholds(32), Thresholds { system: 7.0, process: 2.0, release: 1.0 });
        assert_eq!(thresholds(0), Thresholds { system: 70.0, process: 20.0, release: 5.0 });
        // Every column only falls (or stays) as the CPU count grows.
        assert!(PROBALANCE.windows(2).all(|w| w[1].0 <= w[0].0 && w[1].1 <= w[0].1 && w[1].2 <= w[0].2));
    }

    #[test]
    fn percentages() {
        assert_eq!(cpu_percent(10_000_000, 10_000_000, 16), 100.0 / 16.0); // one thread flat out for 1 s
        assert_eq!(cpu_percent(5, 0, 4), 0.0);
        assert_eq!(busy_percent(75, 100, 0), 25.0); // kernel time includes idle
        assert_eq!(busy_percent(0, 0, 0), 0.0);
    }

    #[test]
    fn descendant_tree() {
        let parents = [(4, 0), (100, 4), (200, 100), (300, 200), (400, 4), (500, 500), (600, 700), (700, 600)];
        assert_eq!(descendants(100, &parents), HashSet::from([100, 200, 300]));
        assert_eq!(descendants(300, &parents), HashSet::from([300]));
        assert_eq!(descendants(600, &parents), HashSet::from([600, 700])); // a loop from pid reuse ends
        assert!(descendants(0, &parents).is_empty()); // no foreground window
    }

    #[test]
    fn restrain_and_release() {
        let t = thresholds(16); // 10 / 5 / 2 %
        let mut tr = Track::default();
        assert_eq!(tr.step(0, 50.0, 9.0, false, t), None); // first sample over: not yet 900 ms
        assert_eq!(tr.step(1000, 50.0, 9.0, false, t), Some(Event::Restrain));
        assert_eq!(tr.step(2000, 50.0, 0.0, false, t), None); // quiet, but held < 4.2 s
        assert_eq!(tr.step(5000, 50.0, 0.0, false, t), None);
        assert_eq!(tr.step(6000, 50.0, 3.0, false, t), None); // above the 2 % release threshold
        assert_eq!(tr.step(6000, 5.0, 1.0, false, t), Some(Event::Release)); // machine idle too: release check
        assert_eq!(tr.restrained_at, None);

        // A spike, a dip, a spike: never 900 ms in a row.
        let mut tr = Track::default();
        for (i, p) in [9.0, 1.0, 9.0, 1.0].into_iter().enumerate() {
            assert_eq!(tr.step(i as u64 * 1000, 50.0, p, false, t), None);
        }
        // Machine not busy enough: never restrained, however busy the process.
        let mut tr = Track::default();
        assert_eq!(tr.step(0, 9.0, 9.0, false, t), None);
        assert_eq!(tr.step(1000, 9.0, 9.0, false, t), None);
        // Foreground: never restrained; becoming foreground releases at once.
        let mut tr = Track::default();
        assert_eq!(tr.step(0, 50.0, 9.0, true, t), None);
        assert_eq!(tr.step(1000, 50.0, 9.0, true, t), None);
        assert_eq!(tr.step(2000, 50.0, 9.0, false, t), None);
        assert_eq!(tr.step(3000, 50.0, 9.0, false, t), Some(Event::Restrain));
        assert_eq!(tr.step(3500, 50.0, 30.0, true, t), Some(Event::Release));
    }

    #[test]
    fn auto_grace_and_rearm() {
        use Auto::*;
        let d = 30_000;
        assert_eq!(auto_step(Idle, 0, false, false, d), (Idle, None));
        assert_eq!(auto_step(Idle, 0, true, false, d), (Boosted, Some(AutoDo::Boost))); // game starts
        assert_eq!(auto_step(Boosted, 1000, true, true, d), (Boosted, None));
        assert_eq!(auto_step(Boosted, 5000, false, true, d), (Grace(5000), None)); // last game exits
        assert_eq!(auto_step(Grace(5000), 20_000, false, true, d), (Grace(5000), None));
        assert_eq!(auto_step(Grace(5000), 20_000, true, true, d), (Boosted, None)); // restart within the grace
        assert_eq!(auto_step(Grace(5000), 35_000, false, true, d), (Idle, Some(AutoDo::Restore))); // grace over
        assert_eq!(auto_step(Boosted, 5000, false, true, 0), (Idle, Some(AutoDo::Restore))); // delay 0: at once
        // Boosted by hand (state file at start): watched, never restored, even with no game.
        assert_eq!(auto_step(Idle, 0, true, true, d), (Manual, None));
        assert_eq!(auto_step(Manual, 99_000, false, true, d), (Manual, None));
        assert_eq!(auto_step(Manual, 99_000, false, false, d), (Idle, None)); // restored by hand
        // `restore` in another window while the game runs: no boost until all games are gone.
        assert_eq!(auto_step(Boosted, 1000, true, false, d), (Disarmed, None));
        assert_eq!(auto_step(Grace(0), 1000, true, false, d), (Disarmed, None));
        assert_eq!(auto_step(Disarmed, 2000, true, false, d), (Disarmed, None));
        assert_eq!(auto_step(Disarmed, 3000, false, false, d), (Idle, None));
        assert_eq!(auto_step(Grace(0), 1000, false, false, d), (Idle, None)); // restored by hand in the grace
    }
}
