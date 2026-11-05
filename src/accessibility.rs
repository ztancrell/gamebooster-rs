//! Accessibility shortcuts: turn off the Sticky Keys, Filter Keys and Toggle Keys hotkeys.
//!
//! **What it does.** Clears the "hotkey active" and "confirm hotkey" flags of Sticky Keys (Shift
//! five times), Filter Keys (hold Right Shift 8 s) and Toggle Keys (hold Num Lock 5 s) with
//! `SystemParametersInfoW`, after saving each one's original `dwFlags`. A feature that is currently
//! *on* is left alone entirely; so is one whose hotkey is already off. Restore writes the saved
//! `dwFlags` back exactly, only for the ones we changed.
//!
//! **Session only.** We pass `SPIF_SENDCHANGE` without `SPIF_UPDATEINIFILE`, so nothing is written
//! to the user profile: if gamebooster crashes before restore, signing out or rebooting brings the
//! shortcuts back by itself. (Idea from Razer Cortex, see docs/razer-cortex/README.md.)
//!
//! **Why.** Comfort, not FPS: holding or mashing Shift in a game pops up the Sticky Keys dialog and
//! drops you to the desktop.
//!
//! **Verify.** Settings > Accessibility > Keyboard: "Keyboard shortcut for Sticky keys" is off while
//! boosted. Pressing Shift five times does nothing.

use crate::state::AccessibilityState;

/// The same bit values for all three (`SKF_*`, `FKF_*`, `TKF_*`).
const FEATURE_ON: u32 = 0x1;
const HOTKEY_BITS: u32 = 0x4 | 0x8; // *_HOTKEYACTIVE | *_CONFIRMHOTKEY

/// The flags to write during a boost, or None to leave this feature alone (it is on, or its hotkey
/// is already off).
pub fn hotkey_off(flags: u32) -> Option<u32> {
    (flags & FEATURE_ON == 0 && flags & HOTKEY_BITS != 0).then_some(flags & !HOTKEY_BITS)
}

/// (name, saved flags) for --dry-run, status and restore.
pub fn entries(st: &AccessibilityState) -> [(&'static str, Option<u32>); 3] {
    [("Sticky Keys", st.sticky_keys), ("Filter Keys", st.filter_keys), ("Toggle Keys", st.toggle_keys)]
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::{entries, hotkey_off};
    use crate::state::AccessibilityState;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    /// (get, set, struct size). STICKYKEYS and TOGGLEKEYS are {cbSize, dwFlags}; FILTERKEYS adds
    /// four timing u32s. All start with cbSize, dwFlags, so one [u32; 6] buffer fits each.
    const KINDS: [(u32, u32, u32); 3] = [
        (SPI_GETSTICKYKEYS, SPI_SETSTICKYKEYS, 8),
        (SPI_GETFILTERKEYS, SPI_SETFILTERKEYS, 24),
        (SPI_GETTOGGLEKEYS, SPI_SETTOGGLEKEYS, 8),
    ];

    fn get(get: u32, size: u32) -> Result<[u32; 6], String> {
        let mut buf = [0u32; 6];
        buf[0] = size;
        if unsafe { SystemParametersInfoW(get, size, buf.as_mut_ptr().cast(), 0) } == 0 {
            return Err(format!("SystemParametersInfoW({get:#x}) failed"));
        }
        Ok(buf)
    }

    /// Get the current struct, replace dwFlags, set it for this session only (see module doc).
    fn set_flags(kind: (u32, u32, u32), flags: u32) -> Result<(), String> {
        let mut buf = get(kind.0, kind.2)?;
        buf[1] = flags;
        if unsafe { SystemParametersInfoW(kind.1, kind.2, buf.as_mut_ptr().cast(), SPIF_SENDCHANGE) } == 0 {
            return Err(format!("SystemParametersInfoW({:#x}) failed", kind.1));
        }
        Ok(())
    }

    /// Read-only: the original flags of each feature whose hotkey boost will turn off.
    pub fn plan() -> Result<AccessibilityState, String> {
        let mut saved = [None; 3];
        for (i, k) in KINDS.iter().enumerate() {
            let flags = get(k.0, k.2)?[1];
            saved[i] = hotkey_off(flags).map(|_| flags);
        }
        Ok(AccessibilityState { sticky_keys: saved[0], filter_keys: saved[1], toggle_keys: saved[2] })
    }

    pub fn boost(st: &AccessibilityState) -> Result<(), String> {
        for ((name, saved), kind) in entries(st).into_iter().zip(KINDS) {
            match saved.and_then(hotkey_off) {
                Some(flags) => {
                    set_flags(kind, flags)?;
                    println!("  {name} shortcut off");
                }
                None => println!("  {name}: left alone (feature on or shortcut already off)"),
            }
        }
        Ok(())
    }

    pub fn restore(st: &AccessibilityState) -> Result<(), String> {
        for ((name, saved), kind) in entries(st).into_iter().zip(KINDS) {
            if let Some(flags) = saved {
                set_flags(kind, flags)?;
                println!("  {name} flags restored ({flags:#x})");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_only_hotkey_bits_unless_feature_on() {
        // Windows default Sticky Keys: AVAILABLE|HOTKEYACTIVE|CONFIRMHOTKEY|HOTKEYSOUND|INDICATOR|AUDIBLEFEEDBACK
        assert_eq!(hotkey_off(0x1fe), Some(0x1f2));
        assert_eq!(hotkey_off(0x1ff), None, "feature on: leave alone");
        assert_eq!(hotkey_off(0x1f2), None, "hotkey already off");
        assert_eq!(hotkey_off(0x6), Some(0x2), "only one hotkey bit set");
    }
}
