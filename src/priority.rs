//! Game priority: raise the CPU priority class of the running game.
//!
//! **What it does.** For each `[[game]]` in the config that is running, sets its process priority
//! class (default High). Restore sets running games back to Normal; a game that has exited needs
//! nothing. The game must already be running: start BF6, then boost (or run `boost` again while
//! boosted, which only re-applies priority).
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

use crate::config::Priority;

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

/// Set every running process of each game to its priority (or to Normal when `reset`).
/// Returns how many processes were found.
#[cfg(windows)]
pub fn apply(games: &[crate::config::Game], reset: bool) -> usize {
    use crate::processes::{list, Process};
    use windows_sys::Win32::System::Threading::{SetPriorityClass, PROCESS_SET_INFORMATION};
    let mut found = 0;
    for p in list() {
        let Some(game) = games.iter().find(|g| g.exe.eq_ignore_ascii_case(&p.name)) else { continue };
        found += 1;
        let want = if reset { Priority::Normal } else { game.priority };
        let ok = Process::open(p.pid, PROCESS_SET_INFORMATION).is_some_and(|h| unsafe { SetPriorityClass(h.0, class(want)) } != 0);
        println!("  {} {} (pid {}) to {want:?}", if ok { "set" } else { "could not set" }, p.name, p.pid);
    }
    found
}
