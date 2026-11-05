//! RAM trim: ask every other process to give back its working set, once. Off by default.
//!
//! **What it does.** Calls `EmptyWorkingSet` on every process it can open (not the games), which
//! moves their in-RAM pages to the standby list or the page file. Prints free physical memory
//! before and after. There is nothing to restore.
//!
//! **Windows default.** Windows trims working sets by itself when free memory runs low, and
//! keeps recently used pages around (standby list) because unused RAM is wasted RAM.
//!
//! **Why it (sometimes) helps games.** Only when RAM is nearly full: BF6 can want 10+ GB, and on
//! an 8 GB Mac a trim before loading a map can avoid paging in the middle of a match. When RAM is
//! not tight it does nothing useful, and it can briefly hurt: the trimmed apps fault their pages
//! back in, causing disk activity. That's why `ram_trim = false` is the default.
//!
//! **Verify / measure.** Task Manager, Performance > Memory: "In use" drops, "Available" rises.
//! Resource Monitor's Memory tab shows "Hard Faults/sec" afterwards. Compare BF6 map-load times and
//! frame-time spikes with and without it on your machine.

#[cfg(windows)]
fn free_mb() -> u64 {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut m) };
    m.ullAvailPhys / (1024 * 1024)
}

#[cfg(windows)]
pub fn trim(cfg: &crate::config::Config) {
    use crate::processes::{list, Process};
    use windows_sys::Win32::System::ProcessStatus::K32EmptyWorkingSet;
    use windows_sys::Win32::System::Threading::{PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA};
    let before = free_mb();
    let trimmed = list()
        .iter()
        .filter(|p| !cfg.is_game(&p.name))
        .filter_map(|p| Process::open(p.pid, PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION))
        .filter(|h| unsafe { K32EmptyWorkingSet(h.0) } != 0)
        .count();
    let after = free_mb();
    println!("  trimmed {trimmed} processes; free RAM {before} MB -> {after} MB");
}
