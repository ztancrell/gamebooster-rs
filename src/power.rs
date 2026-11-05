//! Power plan: a dedicated "gamebooster" plan copied from High performance.
//!
//! **What it does.** Duplicates the built-in High performance plan into our own plan (fixed GUID
//! [`OUR_PLAN`]), writes the values in [`SETTINGS`] for both AC (plugged in) and DC (battery),
//! carries over the current display brightness, and activates it. Restore reactivates the plan
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
//! setting doesn't exist and is skipped. The cost is more heat and battery use, which is why we
//! switch back on restore.
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

/// (subgroup, setting, value, meaning). Written for both AC and DC.
pub const SETTINGS: &[(&str, &str, u32, &str)] = &[
    ("54533251-82be-4824-96c1-47b60b740d00", "893dee8e-2bef-41e0-89c6-b55d0929964c", 100, "minimum processor state 100 %"),
    ("54533251-82be-4824-96c1-47b60b740d00", "bc5038f7-23e0-4960-96da-33abaf5935ec", 100, "maximum processor state 100 %"),
    ("54533251-82be-4824-96c1-47b60b740d00", "94d3a615-a899-4ac5-ae2b-e4d8f634367f", 1, "cooling policy: active (fans before throttling)"),
    ("501a4d13-42af-4429-9fd1-a8218c268e20", "ee12f906-d277-404b-b6da-e5fa1a576df5", 0, "PCIe link state power management off"),
    ("f693fb01-e858-4f00-b20f-f30e12ac06d6", "191f65b5-d45c-4a4f-8aae-1ab8bfd980e6", 1, "AMD ATI PowerPlay: maximize performance (AMD only)"),
    ("0012ee47-9041-4b5d-9b77-535fba8b1442", "6738e2c4-e8a5-4a42-b16a-e040e769756e", 0, "never turn off the disk"),
    ("19cbb8fa-5279-450e-9fac-8a3d5fedd0c1", "12bbebe6-58d6-4636-95bb-3217ef867c1a", 0, "Wi-Fi power saving: maximum performance"),
    ("238c9fa8-0aad-41ed-83f4-97be242c8f20", "29f6c1db-86da-48c5-9fdb-f2b67b1f44da", 0, "never sleep"),
    ("238c9fa8-0aad-41ed-83f4-97be242c8f20", "9d7815a6-7ee4-497e-8888-515a05f02364", 0, "never hibernate"),
    ("4f971e89-eebd-4455-a8de-9e59040e7347", "5ca83367-6e45-459f-a27b-476b1d01c936", 0, "closing the lid does nothing"),
    ("fea3413e-7e05-4911-9a71-700331f1c294", "0e796bdb-100d-47d6-a2d5-f7d2daa51f51", 0, "no password on wake"),
    (SUB_VIDEO, "3c0bc021-c8a8-4e07-a973-6b14cbcb2b7e", 0, "never turn off the display"),
    (SUB_VIDEO, "17aaa29b-8b43-4b94-aafe-35f64daaf1ee", 0, "never dim the display"),
    (SUB_VIDEO, "fbd9aa66-9553-4097-ba44-ed6e9d65eab8", 0, "adaptive brightness off"),
    (SUB_VIDEO, "90959d22-d6a1-49b9-af93-bce885ad335b", 0, "unidentified hidden display setting (skipped if absent)"),
    ("9596fb26-9850-41fd-ac3e-f7c3c00afd4b", "03680956-93bc-4294-bba6-4e0f09bb717f", 1, "when sharing media: prevent idle sleep"),
    ("9596fb26-9850-41fd-ac3e-f7c3c00afd4b", "34c7b99f-9a6d-4b3c-8dc7-b6693b78cef4", 0, "when playing video: optimize video quality"),
];

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
    Ok(PowerState { previous_scheme, brightness })
}

#[cfg(windows)]
pub fn boost(st: &PowerState) -> Result<(), String> {
    delete_ours()?;
    if powercfg(&["/duplicatescheme", HIGH_PERFORMANCE, OUR_PLAN]).is_err() {
        println!("  High performance plan missing, copying Balanced instead");
        powercfg(&["/duplicatescheme", BALANCED, OUR_PLAN])?;
    }
    powercfg(&["/changename", OUR_PLAN, "gamebooster", "Temporary plan created by gamebooster; removed by `gamebooster restore`."])?;
    let mut values: Vec<(&str, &str, u32, u32, &str)> = SETTINGS.iter().map(|&(g, s, v, m)| (g, s, v, v, m)).collect();
    if let Some((ac, dc)) = st.brightness {
        values.push((SUB_VIDEO, VIDEO_BRIGHTNESS, ac, dc, "keep current display brightness"));
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
        assert_eq!(parse_guid(OUR_PLAN).as_deref(), Some(OUR_PLAN));
    }
}
