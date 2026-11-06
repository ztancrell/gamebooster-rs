//! Power plan: a dedicated "gamebooster" plan copied from High performance.
//!
//! **What it does.** Duplicates the built-in High performance plan into our own plan (fixed GUID
//! [`OUR_PLAN`]), writes the values in [`SETTINGS`] for both AC (plugged in) and DC (battery),
//! carries over the current display brightness and the lid / power button / sleep button actions
//! ([`KEEP`]), and activates it. Restore reactivates the plan
//! that was active before and deletes ours.
//!
//! **Windows default.** Boot Camp installs run Balanced: the minimum processor state is 5 %, so
//! cores park and drop to low clocks whenever there is a gap in work, PCIe links (to the GPU and
//! NVMe drive) drop into low-power states (ASPM), and the cooling policy on battery is passive
//! (slow the CPU down before spinning the fans up).
//!
//! **Why it helps games.** A game's work arrives in bursts every frame. On Balanced the CPU has to
//! ramp its clock back up at the start of each burst, and a GPU or NVMe link has to wake from
//! ASPM. Those wake-ups land as frame-time spikes (stutter), even when the average FPS looks fine.
//! Pinning the minimum processor state at 100 % and turning ASPM off removes the ramp. "Active"
//! cooling makes the fans react before the CPU is throttled, which matters on a Mac where the
//! fan curve under Windows is conservative and thermals are tight (iMacs and MacBook Pros reach
//! their limits fast). On Macs with AMD Radeon graphics, the "ATI PowerPlay: maximize
//! performance" setting stops the GPU from downclocking between frames; on other GPUs that
//! setting doesn't exist and is skipped. USB selective suspend is off so a mouse or headset is never put to
//! sleep mid-game (High performance leaves it on). The cost is more heat and battery use, which is why we
//! switch back on restore.
//!
//! **Optional: Turbo Boost off.** The maximum processor state is 100 % unless the config sets
//! `max_processor_state`. On Intel CPUs any value below 100 (99 is enough) stops Windows from
//! requesting Turbo Boost, so the CPU stays at or below its base clock. On a hot i9 MacBook Pro
//! that trades peak clock for less heat and less thermal throttling, often steadier frame times.
//! Verify with `powercfg /query <guid> SUB_PROCESSOR PROCTHROTTLEMAX` (0x63 = 99) and HWiNFO's
//! core clocks; see docs/tweaks/power-plan.md.
//!
//! **Optional: low minimum state.** The minimum processor state is 100 % unless the config sets
//! `min_processor_state`. A low value (5) with `max_processor_state = 99` is the low-power combo for
//! a Mac that gets stuck at 800 MHz by BD PROCHOT; see docs/tweaks/prochot.md.
//!
//! **Optional: frequency cap.** The config's `max_cpu_mhz` sets the hidden "Maximum processor
//! frequency" setting (MHz, 0 = no limit) in our plan. Unlike `max_processor_state = 99` it keeps
//! Turbo Boost but stops the clock at the cap, e.g. 2800 MHz to stay below the point where a
//! 2019 16" MacBook Pro trips BD PROCHOT. With both set, the lower limit wins. Unset = not
//! written (High performance's 0, no limit). Verify with `powercfg /query <guid> SUB_PROCESSOR
//! 75b0ae3f-bce0-45a7-8c89-c9611c25e100` (0xaf0 = 2800); see docs/tweaks/power-plan.md.
//!
//! **800 MHz check.** `status` measures the real clock under one busy thread, like Task Manager:
//! PDH `Processor Frequency` × `% Processor Performance` / 100. No driver needed.
//!
//! **How we do it.** We shell out to `powercfg.exe` instead of calling `powrprof.dll` directly:
//! it is less code, the same commands are what you'd type by hand (see docs/tweaks/power-plan.md),
//! and a setting the machine doesn't have simply makes `powercfg` fail, which we report as
//! "skipped".
//!
//! **Verify / measure.** `powercfg /getactivescheme` shows "gamebooster" while boosted.
//! `powercfg /query <guid> SUB_PROCESSOR PROCTHROTTLEMIN` shows 0x64 (100). In-game, log frame
//! times with PresentMon or CapFrameX in the same BF6 scene with and without the plan and compare
//! the 1 % lows and the frame-time graph spikes, not just the average FPS.

#[cfg(windows)]
use crate::state::PowerState;

/// Our plan's fixed GUID. A fixed ID means a leftover plan from a crashed run is found and
/// replaced by ID, without having to enumerate plans by name.
pub const OUR_PLAN: &str = "a2921d59-9c60-439e-b61a-21c441a0df53";
pub const HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

const SUB_VIDEO: &str = "7516b95f-f776-4464-8c53-06167f40cc99";
const VIDEO_BRIGHTNESS: &str = "aded5e82-b909-4619-9949-f5d71dac0bcb";
const SUB_PROCESSOR: &str = "54533251-82be-4824-96c1-47b60b740d00";
const PROC_MAX: &str = "bc5038f7-23e0-4960-96da-33abaf5935ec";
const PROC_MIN: &str = "893dee8e-2bef-41e0-89c6-b55d0929964c";
const PROC_FREQ_MAX: &str = "75b0ae3f-bce0-45a7-8c89-c9611c25e100";

/// (subgroup, setting, value, meaning). Written for both AC and DC. The minimum and maximum
/// processor states come from the config instead, see [`max_state_meaning`].
pub const SETTINGS: &[(&str, &str, u32, &str)] = &[
    (SUB_PROCESSOR, "94d3a615-a899-4ac5-ae2b-e4d8f634367f", 1, "cooling policy: active (fans before throttling)"),
    ("501a4d13-42af-4429-9fd1-a8218c268e20", "ee12f906-d277-404b-b6da-e5fa1a576df5", 0, "PCIe link state power management off"),
    ("f693fb01-e858-4f00-b20f-f30e12ac06d6", "191f65b5-d45c-4a4f-8aae-1ab8bfd980e6", 1, "AMD ATI PowerPlay: maximize performance (AMD only)"),
    ("0012ee47-9041-4b5d-9b77-535fba8b1442", "6738e2c4-e8a5-4a42-b16a-e040e769756e", 0, "never turn off the disk"),
    ("19cbb8fa-5279-450e-9fac-8a3d5fedd0c1", "12bbebe6-58d6-4636-95bb-3217ef867c1a", 0, "Wi-Fi power saving: maximum performance"),
    ("238c9fa8-0aad-41ed-83f4-97be242c8f20", "29f6c1db-86da-48c5-9fdb-f2b67b1f44da", 0, "never sleep"),
    ("238c9fa8-0aad-41ed-83f4-97be242c8f20", "9d7815a6-7ee4-497e-8888-515a05f02364", 0, "never hibernate"),
    ("fea3413e-7e05-4911-9a71-700331f1c294", "0e796bdb-100d-47d6-a2d5-f7d2daa51f51", 0, "no password on wake"),
    (SUB_VIDEO, "3c0bc021-c8a8-4e07-a973-6b14cbcb2b7e", 0, "never turn off the display"),
    (SUB_VIDEO, "17aaa29b-8b43-4b94-aafe-35f64daaf1ee", 0, "never dim the display"),
    (SUB_VIDEO, "fbd9aa66-9553-4097-ba44-ed6e9d65eab8", 0, "adaptive brightness off"),
    (SUB_VIDEO, "90959d22-d6a1-49b9-af93-bce885ad335b", 0, "unidentified hidden display setting (skipped if absent)"),
    ("9596fb26-9850-41fd-ac3e-f7c3c00afd4b", "03680956-93bc-4294-bba6-4e0f09bb717f", 1, "when sharing media: prevent idle sleep"),
    ("9596fb26-9850-41fd-ac3e-f7c3c00afd4b", "34c7b99f-9a6d-4b3c-8dc7-b6693b78cef4", 0, "when playing video: optimize video quality"),
    ("2a737441-1930-4402-8d77-b2bebba308a3", "48e6b7a6-50f5-4782-a5d4-53bb8f07e226", 0, "USB selective suspend off"),
];

const SUB_BUTTONS: &str = "4f971e89-eebd-4455-a8de-9e59040e7347";

/// (subgroup, setting, meaning): copied from the previous plan (AC and DC) instead of forced, so the
/// user's lid and button actions don't change while boosted. Deliberately differs from SGB, which
/// sets the lid action to "do nothing". A setting the machine lacks is skipped.
pub const KEEP: &[(&str, &str, &str)] = &[
    (SUB_BUTTONS, "5ca83367-6e45-459f-a27b-476b1d01c936", "keep current lid close action"),
    (SUB_BUTTONS, "7648efa3-dd9c-4e3e-b566-50f929386280", "keep current power button action"),
    (SUB_BUTTONS, "96996bc0-ad50-47ec-923b-6f41874dd9eb", "keep current sleep button action"),
];

/// The meaning of a kept setting GUID, for printing.
pub fn kept_meaning(setting: &str) -> &'static str {
    KEEP.iter().find(|k| k.1 == setting).map_or("keep current setting", |k| k.2)
}

/// How boost and --dry-run describe the carried-over display brightness.
pub const BRIGHTNESS_MEANING: &str = "keep current display brightness";

/// The [`KEEP`] settings the previous plan lacks (not on this machine, e.g. Boot Camp Macs have no
/// lid / power / sleep button actions), for printing as skipped.
pub fn absent_kept(kept: &[(String, u32, u32)]) -> Vec<&'static str> {
    KEEP.iter().filter(|k| !kept.iter().any(|(s, ..)| s == k.1)).map(|k| k.2).collect()
}

/// How boost and --dry-run describe the maximum processor state.
pub fn max_state_meaning(pct: u32) -> String {
    format!("maximum processor state {pct} %{}", if pct < 100 { " (Turbo Boost off)" } else { "" })
}

/// How boost and --dry-run describe the frequency cap.
pub fn max_mhz_meaning(mhz: u32) -> String {
    format!("maximum processor frequency {mhz} MHz")
}

/// Under load the CPU should run far above its 800 MHz minimum; below this it looks locked there
/// (BD PROCHOT on a Boot Camp MacBook Pro).
pub fn looks_locked(mhz: f64) -> bool {
    mhz < 1000.0
}

/// Real CPU clock in MHz while one thread busy-spins for about a second, as Task Manager computes
/// it: `Processor Frequency` × `% Processor Performance` / 100 (PDH, no driver).
#[cfg(windows)]
pub fn clock_under_load_mhz() -> Result<f64, String> {
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
    use windows_sys::Win32::System::Performance::*;
    let ok = |what: &str, r: u32| if r == 0 { Ok(()) } else { Err(format!("{what} failed (PDH error 0x{r:08x})")) };
    let stop = AtomicBool::new(false);
    std::thread::scope(|sc| {
        sc.spawn(|| {
            while !stop.load(Relaxed) {
                std::hint::spin_loop();
            }
        });
        // SAFETY: plain PDH calls with valid out-pointers; the query is closed before returning.
        // No `?` out of this block: the spinner must be stopped below or the scope never ends.
        let r = unsafe {
            let mut q = std::ptr::null_mut();
            let r = ok("PdhOpenQuery", PdhOpenQueryW(std::ptr::null(), 0, &mut q)).and_then(|()| {
                let mut c = [std::ptr::null_mut(); 2];
                for (i, path) in [r"\Processor Information(_Total)\Processor Frequency", r"\Processor Information(_Total)\% Processor Performance"].iter().enumerate() {
                    ok(path, PdhAddEnglishCounterW(q, crate::wide(path).as_ptr(), 0, &mut c[i]))?;
                }
                ok("PdhCollectQueryData", PdhCollectQueryData(q))?; // % Performance is a rate: needs two samples
                std::thread::sleep(std::time::Duration::from_secs(1));
                ok("PdhCollectQueryData", PdhCollectQueryData(q))?;
                let mut v = [PDH_FMT_COUNTERVALUE::default(); 2];
                for i in 0..2 {
                    ok("PdhGetFormattedCounterValue", PdhGetFormattedCounterValue(c[i], PDH_FMT_DOUBLE, std::ptr::null_mut(), &mut v[i]))?;
                }
                Ok(v[0].Anonymous.doubleValue * v[1].Anonymous.doubleValue / 100.0)
            });
            if !q.is_null() {
                PdhCloseQuery(q);
            }
            r
        };
        stop.store(true, Relaxed);
        r
    })
}

/// First GUID in `powercfg` output, lowercased (e.g. from "Power Scheme GUID: 381b...  (Balanced)").
pub fn parse_guid(text: &str) -> Option<String> {
    let is_guid = |s: &str| {
        s.len() == 36
            && s.char_indices().all(|(i, c)| if [8, 13, 18, 23].contains(&i) { c == '-' } else { c.is_ascii_hexdigit() })
    };
    (0..text.len().saturating_sub(35))
        .filter_map(|i| text.get(i..i + 36))
        .find(|s| is_guid(s))
        .map(str::to_ascii_lowercase)
}

/// The current (AC, DC) index from `powercfg /query` of one setting: the last two `0x` numbers.
/// Labels are localised, the hex values are not.
pub fn parse_ac_dc(text: &str) -> Option<(u32, u32)> {
    let hex: Vec<u32> = text
        .lines()
        .filter_map(|l| l.rsplit_once("0x"))
        .filter_map(|(_, h)| u32::from_str_radix(h.trim(), 16).ok())
        .collect();
    match hex[..] {
        [.., ac, dc] => Some((ac, dc)),
        _ => None,
    }
}

#[cfg(windows)]
fn powercfg(args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("powercfg.exe").args(args).output().map_err(|e| format!("powercfg: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(text)
    } else {
        Err(format!("powercfg {}: {}", args.join(" "), text.trim()))
    }
}

#[cfg(windows)]
pub fn active_scheme() -> Result<String, String> {
    parse_guid(&powercfg(&["/getactivescheme"])?).ok_or("can't parse `powercfg /getactivescheme`".into())
}

/// The status line under "Maximum processor state" when the active plan caps it below 100 % (that
/// turns Turbo Boost off; an 80 % cap held the target Mac at ~22 W, docs/hardware-test.md). Our own
/// plan at the configured `max_processor_state` is deliberate, so that is a note, not a warning.
pub fn max_state_warning(name: &str, ours: bool, configured: u32, ac: u32, dc: u32) -> Option<String> {
    if ac >= 100 && dc >= 100 {
        None
    } else if ours && ac == configured && dc == configured {
        Some(format!("  note: set by max_processor_state = {configured} (Turbo Boost off while boosted)"))
    } else {
        Some(format!(
            "  warning: active plan '{name}' caps Maximum processor state at {ac}% (AC) / {dc}% (DC); below 100 turns Turbo Boost off. \
             Fix: powercfg /setacvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100 (and /setdcvalueindex), \
             then powercfg /setactive scheme_current"
        ))
    }
}

/// The active plan's name, e.g. "Balanced" from "Power Scheme GUID: 381b...  (Balanced)".
#[cfg(windows)]
pub fn active_scheme_name() -> Result<String, String> {
    let text = powercfg(&["/getactivescheme"])?;
    Ok(text.trim().rsplit_once('(').map_or(text.trim(), |(_, n)| n.trim_end_matches(')')).to_string())
}

/// (AC, DC) Maximum processor state of a plan, in %.
#[cfg(windows)]
pub fn max_state(scheme: &str) -> Result<(u32, u32), String> {
    parse_ac_dc(&powercfg(&["/query", scheme, SUB_PROCESSOR, PROC_MAX])?).ok_or("can't parse `powercfg /query`".into())
}

#[cfg(windows)]
fn plan_exists(guid: &str) -> bool {
    powercfg(&["/list"]).is_ok_and(|t| t.to_ascii_lowercase().contains(guid))
}

/// Read-only: what restore will need.
#[cfg(windows)]
pub fn plan() -> Result<PowerState, String> {
    let active = active_scheme()?;
    let previous_scheme = if active == OUR_PLAN { BALANCED.to_string() } else { active };
    let brightness = powercfg(&["/query", &previous_scheme, SUB_VIDEO, VIDEO_BRIGHTNESS]).ok().and_then(|t| parse_ac_dc(&t));
    let kept = KEEP.iter()
        .filter_map(|&(sub, setting, _)| {
            let (ac, dc) = powercfg(&["/query", &previous_scheme, sub, setting]).ok().and_then(|t| parse_ac_dc(&t))?;
            Some((setting.to_string(), ac, dc))
        })
        .collect();
    Ok(PowerState { previous_scheme, brightness, kept })
}

#[cfg(windows)]
pub fn boost(st: &PowerState, min_state: u32, max_state: u32, max_mhz: Option<u32>) -> Result<(), String> {
    delete_ours()?;
    if powercfg(&["/duplicatescheme", HIGH_PERFORMANCE, OUR_PLAN]).is_err() {
        println!("  High performance plan missing, copying Balanced instead");
        powercfg(&["/duplicatescheme", BALANCED, OUR_PLAN])?;
    }
    powercfg(&["/changename", OUR_PLAN, "gamebooster", "Temporary plan created by gamebooster; removed by `gamebooster restore`."])?;
    let (min_meaning, max_meaning) = (format!("minimum processor state {min_state} %"), max_state_meaning(max_state));
    let mhz_meaning = max_mhz.map(max_mhz_meaning).unwrap_or_default();
    let mut values: Vec<(&str, &str, u32, u32, &str)> = SETTINGS.iter().map(|&(g, s, v, m)| (g, s, v, v, m)).collect();
    values.push((SUB_PROCESSOR, PROC_MIN, min_state, min_state, &min_meaning));
    values.push((SUB_PROCESSOR, PROC_MAX, max_state, max_state, &max_meaning));
    if let Some(mhz) = max_mhz {
        values.push((SUB_PROCESSOR, PROC_FREQ_MAX, mhz, mhz, &mhz_meaning));
    }
    if let Some((ac, dc)) = st.brightness {
        values.push((SUB_VIDEO, VIDEO_BRIGHTNESS, ac, dc, BRIGHTNESS_MEANING));
    } else {
        println!("  skipped (not on this machine) {BRIGHTNESS_MEANING}");
    }
    for meaning in absent_kept(&st.kept) {
        println!("  skipped (not on this machine) {meaning}");
    }
    for (setting, ac, dc) in &st.kept {
        values.push((SUB_BUTTONS, setting.as_str(), *ac, *dc, kept_meaning(setting)));
    }
    for (sub, setting, ac, dc, meaning) in values {
        let ok = powercfg(&["/setacvalueindex", OUR_PLAN, sub, setting, &ac.to_string()]).is_ok()
            & powercfg(&["/setdcvalueindex", OUR_PLAN, sub, setting, &dc.to_string()]).is_ok();
        println!("  {} {meaning}", if ok { "set    " } else { "skipped (not on this machine)" });
    }
    powercfg(&["/setactive", OUR_PLAN])?;
    println!("  activated plan gamebooster ({OUR_PLAN})");
    Ok(())
}

#[cfg(windows)]
pub fn restore(st: &PowerState) -> Result<(), String> {
    if powercfg(&["/setactive", &st.previous_scheme]).is_err() {
        println!("  previous plan {} is gone, activating Balanced", st.previous_scheme);
        powercfg(&["/setactive", BALANCED])?;
    }
    println!("  reactivated plan {}", active_scheme()?);
    delete_ours()
}

/// Delete our plan if it exists; Windows refuses to delete the active plan, so leave it first.
#[cfg(windows)]
fn delete_ours() -> Result<(), String> {
    if !plan_exists(OUR_PLAN) {
        return Ok(());
    }
    if active_scheme()? == OUR_PLAN {
        powercfg(&["/setactive", BALANCED])?;
    }
    powercfg(&["/delete", OUR_PLAN])?;
    println!("  deleted plan gamebooster");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_active_scheme() {
        let out = "Power Scheme GUID: 381B4222-F694-41F0-9685-FF5BB260DF2E  (Balanced)\r\n";
        assert_eq!(parse_guid(out).as_deref(), Some(BALANCED));
        assert_eq!(parse_guid("no guid here"), None);
    }

    #[test]
    fn parses_ac_dc_from_query() {
        let out = "Power Setting GUID: aded5e82-b909-4619-9949-f5d71dac0bcb  (Display brightness)\r\n\
                   \x20 Minimum Possible Setting: 0x00000000\r\n\
                   \x20 Maximum Possible Setting: 0x00000064\r\n\
                   \x20 Possible Settings increment: 0x00000001\r\n\
                   \x20 Possible Settings units: %\r\n\
                   Current AC Power Setting Index: 0x00000064\r\n\
                   Current DC Power Setting Index: 0x0000004b\r\n";
        assert_eq!(parse_ac_dc(out), Some((100, 75)));
        assert_eq!(parse_ac_dc("nothing"), None);
    }

    #[test]
    fn settings_are_well_formed() {
        for (sub, setting, _, _) in SETTINGS {
            assert_eq!(parse_guid(sub).as_deref(), Some(*sub));
            assert_eq!(parse_guid(setting).as_deref(), Some(*setting));
        }
        for (sub, setting, _) in KEEP {
            assert_eq!(parse_guid(sub).as_deref(), Some(*sub));
            assert_eq!(parse_guid(setting).as_deref(), Some(*setting));
            assert!(SETTINGS.iter().all(|s| s.1 != *setting), "{setting} is both forced and kept");
        }
        assert_eq!(kept_meaning("7648efa3-dd9c-4e3e-b566-50f929386280"), "keep current power button action");
        assert_eq!(absent_kept(&[(KEEP[0].1.into(), 1, 1)]), ["keep current power button action", "keep current sleep button action"]);
        assert_eq!(parse_guid(OUR_PLAN).as_deref(), Some(OUR_PLAN));
        assert_eq!(parse_guid(PROC_MAX).as_deref(), Some(PROC_MAX));
        assert_eq!(parse_guid(PROC_MIN).as_deref(), Some(PROC_MIN));
        assert_eq!(parse_guid(PROC_FREQ_MAX).as_deref(), Some(PROC_FREQ_MAX));
    }

    #[test]
    fn max_state_says_turbo_off_below_100() {
        assert_eq!(max_state_meaning(100), "maximum processor state 100 %");
        assert_eq!(max_state_meaning(99), "maximum processor state 99 % (Turbo Boost off)");
    }

    #[test]
    fn warns_on_capped_max_state() {
        assert_eq!(max_state_warning("Balanced", false, 100, 100, 100), None);
        assert!(max_state_warning("Balanced", false, 100, 80, 100).unwrap().contains("warning: active plan 'Balanced' caps Maximum processor state at 80% (AC) / 100% (DC)"));
        assert!(max_state_warning("gamebooster", true, 99, 99, 99).unwrap().contains("note: set by max_processor_state = 99"));
        assert!(max_state_warning("gamebooster", true, 99, 80, 80).unwrap().contains("warning"));
    }

    #[test]
    fn clock_lock_threshold() {
        assert!(looks_locked(798.0) && !looks_locked(2300.0) && !looks_locked(1000.0));
    }
}
