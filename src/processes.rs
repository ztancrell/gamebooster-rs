//! Close apps: end the background apps you list in the config, relaunch them on restore.
//!
//! **What it does.** Finds every running process whose image name is in `close` (for example
//! `OneDrive.exe`), records each one's full path, then terminates them all. Restore starts each
//! recorded path again unless it is already running. Relaunching goes through `explorer.exe`, so
//! the apps start as your normal user and not with the administrator rights gamebooster runs with.
//!
//! **Windows default.** Everything in the tray and Startup apps keeps running all session: cloud
//! sync (OneDrive, Dropbox, iCloud), chat apps (Discord, Teams), updaters, RGB/peripheral tools,
//! browser background processes.
//!
//! **Why it helps games.** These apps use CPU in bursts (a sync, a notification, a redraw), hold
//! RAM, and some draw overlays that hook into the game's rendering. Closing them gives the game the
//! whole CPU and more free memory, which is most noticeable on a 4-core Mac with 8 or 16 GB.
//!
//! **Safety.** Only names you list are touched. The game, launchers (EA app, Steam, Epic),
//! Windows core processes, `explorer.exe` and anti-virus are protected and rejected in the config
//! (see `config::is_protected`). Unsaved work in a closed app is lost, as with End task.
//! Command-line arguments are not restored: apps come back as if started from the Start menu.
//!
//! **Verify / measure.** Task Manager's Processes tab before and after. Compare CPU usage with
//! the game idle in the menu, and BF6 frame times in CapFrameX with the apps open and closed.

/// One running process.
pub struct Proc {
    pub pid: u32,
    pub name: String,
    /// Full image path; None when Windows won't tell us (protected or system processes).
    pub path: Option<String>,
}

/// Unique image paths of the processes that `close` names, in first-seen order.
pub fn paths_to_close(procs: &[Proc], close: &[String]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for p in procs.iter().filter(|p| close.iter().any(|c| c.eq_ignore_ascii_case(&p.name))) {
        if let Some(path) = &p.path {
            if !paths.iter().any(|q| q.eq_ignore_ascii_case(path)) {
                paths.push(path.clone());
            }
        }
    }
    paths
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::Proc;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
    use windows_sys::Win32::System::Threading::*;

    /// An open process handle, closed on drop.
    pub struct Process(pub HANDLE);
    impl Drop for Process {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }
    impl Process {
        pub fn open(pid: u32, access: PROCESS_ACCESS_RIGHTS) -> Option<Process> {
            let h = unsafe { OpenProcess(access, 0, pid) };
            (!h.is_null()).then_some(Process(h))
        }
    }

    fn image_path(pid: u32) -> Option<String> {
        let p = Process::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(p.0, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) };
        (ok != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
    }

    pub fn list() -> Vec<Proc> {
        let mut out = Vec::new();
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap == INVALID_HANDLE_VALUE {
                return out;
            }
            let mut e: PROCESSENTRY32W = std::mem::zeroed();
            e.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            let mut more = Process32FirstW(snap, &mut e) != 0;
            while more {
                let name = crate::from_wide(&e.szExeFile);
                out.push(Proc { pid: e.th32ProcessID, path: image_path(e.th32ProcessID), name });
                more = Process32NextW(snap, &mut e) != 0;
            }
            CloseHandle(snap);
        }
        out
    }

    /// Read-only: the paths boost will close (and restore will relaunch).
    pub fn plan(close: &[String]) -> Vec<String> {
        super::paths_to_close(&list(), close)
    }

    /// Terminate every running process named in `close` (a fresh list, so new instances go too).
    pub fn close_all(close: &[String]) {
        for p in list().iter().filter(|p| close.iter().any(|c| c.eq_ignore_ascii_case(&p.name))) {
            let ok = Process::open(p.pid, PROCESS_TERMINATE).is_some_and(|h| unsafe { TerminateProcess(h.0, 1) } != 0);
            println!("  {} {} (pid {})", if ok { "closed" } else { "could not close" }, p.name, p.pid);
        }
    }

    /// Start each path again unless something is already running from it.
    pub fn relaunch(paths: &[String]) {
        let running = list();
        for path in paths {
            if running.iter().any(|p| p.path.as_deref().is_some_and(|q| q.eq_ignore_ascii_case(path))) {
                println!("  already running: {path}");
                continue;
            }
            // explorer.exe starts it un-elevated, as the logged-in user. Its exit code is meaningless.
            match std::process::Command::new("explorer.exe").arg(path).spawn() {
                Ok(_) => println!("  relaunched {path}"),
                Err(e) => println!("  could not relaunch {path}: {e}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_listed_names_dedupes_paths() {
        let p = |pid, name: &str, path: Option<&str>| Proc { pid, name: name.into(), path: path.map(String::from) };
        let procs = [
            p(1, "OneDrive.exe", Some(r"C:\OD\OneDrive.exe")),
            p(2, "onedrive.exe", Some(r"c:\od\onedrive.exe")),
            p(3, "Discord.exe", Some(r"C:\D\Discord.exe")),
            p(4, "Discord.exe", None),
            p(5, "bf6.exe", Some(r"C:\Games\bf6.exe")),
        ];
        let close = vec!["ONEDRIVE.EXE".to_string(), "discord.exe".to_string()];
        assert_eq!(paths_to_close(&procs, &close), [r"C:\OD\OneDrive.exe", r"C:\D\Discord.exe"]);
    }
}
