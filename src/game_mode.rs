//! Game Mode: make sure Windows Game Mode is on.
//!
//! **What it does.** Sets `HKCU\Software\Microsoft\GameBar` value `AutoGameModeEnabled` (DWORD)
//! to 1, after saving the old value (or noting it was absent). Restore writes the old value back,
//! or deletes the value if it didn't exist.
//!
//! **Windows default.** On by default in Windows 10 (since 1903) and 11; when the value is absent
//! Windows treats it as on. So this is usually a no-op that just makes sure nobody turned it off.
//!
//! **Why it helps games.** With Game Mode on, Windows gives the foreground game priority for CPU
//! and GPU scheduling, holds back Windows Update driver installs and restart notifications during
//! play, and lowers the priority of background work. Small but free.
//!
//! **Verify / measure.** Settings > Gaming > Game Mode shows On. Or
//! `reg query HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled`. The value is per user:
//! run gamebooster from your own account (elevated), not from a different admin account.

pub const KEY: &str = r"Software\Microsoft\GameBar";
pub const VALUE: &str = "AutoGameModeEnabled";

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::{KEY, VALUE};
    use crate::wide;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::*;

    /// The current value, or None if it doesn't exist.
    pub fn read() -> Result<Option<u32>, String> {
        crate::reg_dword(HKEY_CURRENT_USER, KEY, VALUE)
    }

    /// Write `Some(v)`, or delete the value for `None`.
    pub fn write(value: Option<u32>) -> Result<(), String> {
        let err = unsafe {
            match value {
                Some(v) => RegSetKeyValueW(HKEY_CURRENT_USER, wide(KEY).as_ptr(), wide(VALUE).as_ptr(), REG_DWORD,
                    &v as *const u32 as *const _, 4),
                None => match RegDeleteKeyValueW(HKEY_CURRENT_USER, wide(KEY).as_ptr(), wide(VALUE).as_ptr()) {
                    ERROR_FILE_NOT_FOUND => ERROR_SUCCESS,
                    e => e,
                },
            }
        };
        if err != ERROR_SUCCESS {
            return Err(format!("can't write {KEY}\\{VALUE} (error {err})"));
        }
        println!("  {VALUE} = {}", value.map_or("(deleted)".into(), |v| v.to_string()));
        Ok(())
    }
}
