//! Services: stop the background services you list in the config, start them again on restore.
//!
//! **What it does.** For each listed service that is *running*, stops the running services that
//! depend on it first, then the service itself, and waits (up to 30 s each) until it has stopped.
//! The config list is a strict allow-list: a listed service with a running dependent that is *not*
//! also listed is skipped, so nothing unlisted is ever touched. (Superseded: earlier versions
//! stopped unlisted dependents too.)
//! Only services that were running are recorded, and restore starts exactly those, in reverse
//! order (a service before the ones that depend on it). The startup type is never changed, so a
//! reboot also brings everything back.
//!
//! **Windows default.** Dozens of services run all the time: search indexing (`WSearch`),
//! SysMain/Superfetch prefetching (`SysMain`), telemetry (`DiagTrack`), print spooler
//! (`Spooler`), plus third-party updaters (Adobe, Google, Apple Software Update) and the Boot Camp
//! helper services.
//!
//! **Why it helps games.** Each one wakes up now and then to do work: indexing a new file,
//! checking for updates, uploading telemetry. On a 4-core Mac that work competes with the game's
//! threads and shows up as an occasional long frame. Stopping them for the session removes that
//! background noise. The gain depends entirely on what runs on your machine, so measure.
//!
//! **Safety.** Nothing is stopped unless you list it. Don't list services you don't recognise:
//! stopping `Audiosrv`, networking or the Boot Camp/Apple keyboard services will break things
//! until restore. To stop a service that has running dependents, list those too; `--dry-run`
//! shows what will stop and what is skipped.
//!
//! **Verify / measure.** `sc query WSearch` shows STATE STOPPED while boosted. Watch CPU usage of
//! the matching `svchost.exe` in Task Manager, and compare BF6 frame-time spikes in CapFrameX with
//! and without the service stopped.

/// Stop order for the configured services: each running service's running dependents first, then
/// the service itself, without duplicates. Services that aren't running are left out, and so is a
/// service with a running dependent that isn't configured (the list is a strict allow-list).
pub fn stop_order(
    configured: &[String],
    is_running: impl Fn(&str) -> bool,
    running_dependents: impl Fn(&str) -> Vec<String>,
) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    for name in configured.iter().filter(|n| is_running(n)) {
        let deps = running_dependents(name);
        if let Some(d) = deps.iter().find(|d| !configured.iter().any(|c| c.eq_ignore_ascii_case(d))) {
            println!("  skipping service {name}: its running dependent {d} isn't in `services`");
            continue;
        }
        for s in deps.into_iter().chain([name.clone()]) {
            if !order.iter().any(|o| o.eq_ignore_ascii_case(&s)) {
                order.push(s);
            }
        }
    }
    order
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use crate::wide;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Services::*;

    const ERROR_SERVICE_ALREADY_RUNNING: u32 = 1056;
    const ERROR_SERVICE_DISABLED: u32 = 1058;
    const ERROR_SERVICE_NOT_ACTIVE: u32 = 1062;
    const TIMEOUT: Duration = Duration::from_secs(30);

    struct Handle(SC_HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { CloseServiceHandle(self.0) };
        }
    }

    fn open(name: &str, access: u32) -> Result<Handle, String> {
        unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if scm.is_null() {
                return Err(format!("can't open the service manager (error {})", GetLastError()));
            }
            let scm = Handle(scm);
            let h = OpenServiceW(scm.0, wide(name).as_ptr(), access);
            if h.is_null() {
                return Err(format!("can't open service {name} (error {})", GetLastError()));
            }
            Ok(Handle(h))
        }
    }

    fn state(h: &Handle) -> u32 {
        let mut st: SERVICE_STATUS = unsafe { std::mem::zeroed() };
        unsafe { QueryServiceStatus(h.0, &mut st) };
        st.dwCurrentState
    }

    fn is_running(name: &str) -> bool {
        open(name, SERVICE_QUERY_STATUS).is_ok_and(|h| state(&h) == SERVICE_RUNNING)
    }

    /// Running services that depend on `name`, in the order Windows says to stop them.
    fn running_dependents(name: &str) -> Vec<String> {
        let Ok(h) = open(name, SERVICE_ENUMERATE_DEPENDENTS) else { return vec![] };
        let (mut needed, mut count) = (0u32, 0u32);
        unsafe {
            EnumDependentServicesW(h.0, SERVICE_ACTIVE, std::ptr::null_mut(), 0, &mut needed, &mut count);
            if needed == 0 {
                return vec![];
            }
            let mut buf = vec![0u64; needed as usize / 8 + 1]; // u64 for alignment
            let ptr = buf.as_mut_ptr() as *mut ENUM_SERVICE_STATUSW;
            if EnumDependentServicesW(h.0, SERVICE_ACTIVE, ptr, needed, &mut needed, &mut count) == 0 {
                return vec![];
            }
            (0..count as usize).map(|i| crate::from_wide_ptr((*ptr.add(i)).lpServiceName)).collect()
        }
    }

    /// Read-only: the services boost will stop, in stop order.
    pub fn plan(configured: &[String]) -> Vec<String> {
        super::stop_order(configured, is_running, running_dependents)
    }

    fn wait_for(h: &Handle, want: u32) -> bool {
        let start = Instant::now();
        while start.elapsed() < TIMEOUT {
            if state(h) == want {
                return true;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        false
    }

    pub fn stop(name: &str) -> Result<(), String> {
        let h = open(name, SERVICE_STOP | SERVICE_QUERY_STATUS)?;
        let mut st: SERVICE_STATUS = unsafe { std::mem::zeroed() };
        if unsafe { ControlService(h.0, SERVICE_CONTROL_STOP, &mut st) } == 0 {
            let err = unsafe { GetLastError() };
            if err != ERROR_SERVICE_NOT_ACTIVE {
                return Err(format!("can't stop {name} (error {err})"));
            }
        }
        if !wait_for(&h, SERVICE_STOPPED) {
            return Err(format!("{name} didn't stop within {} s", TIMEOUT.as_secs()));
        }
        println!("  stopped service {name}");
        Ok(())
    }

    pub fn start(name: &str) -> Result<(), String> {
        let h = open(name, SERVICE_START | SERVICE_QUERY_STATUS)?;
        if unsafe { StartServiceW(h.0, 0, std::ptr::null()) } == 0 {
            let err = unsafe { GetLastError() };
            if err == ERROR_SERVICE_DISABLED {
                // Disabled since boost (by the user or Windows): can't start it, and that's their call.
                println!("  left service {name} stopped: it is disabled now");
                return Ok(());
            }
            if err != ERROR_SERVICE_ALREADY_RUNNING {
                return Err(format!("can't start {name} (error {err})"));
            }
        }
        println!("  started service {name}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependents_first_no_duplicates_skip_stopped_and_unlisted() {
        let configured: Vec<String> = ["A", "B", "Stopped", "a-child", "shared", "C"].map(String::from).to_vec();
        let order = stop_order(
            &configured,
            |n| n != "Stopped",
            |n| match n {
                "A" => vec!["A-child".into(), "Shared".into()],
                "B" => vec!["shared".into()],
                "C" => vec!["Unlisted".into()],
                _ => vec![],
            },
        );
        assert_eq!(order, ["A-child", "Shared", "A", "B"]);
    }
}
