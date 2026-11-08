//! First-run setup and the MacBook Pro 2019 power fix: `install` (also what a double-click runs),
//! `uninstall`, the Mac-only gate, BF6 Borderless, and the install record.
//!
//! **What it does.** Install copies the exe to [`INSTALL_DIR`], writes the config there, and on a
//! 16" MacBook Pro 2019 with the Radeon Pro 5500M (the gate, [`mismatch`]) applies the Mac parts:
//! `max_cpu_mhz = `[`CPU_CAP_MHZ`] in the config (applied by `auto` while BF6 runs, like any boost
//! setting), the GPU soft PowerPlay table ([`crate::gpu_table`], always on) and BF6 in Borderless.
//! Then it registers the `gamebooster` logon task (docs/auto.md) and starts it.
//!
//! **Why.** The Mac's SMC cuts the GPU, then the CPU, once their combined draw passes its warm
//! budget (~65-72 W); 32 W GPU + 1700 MHz CPU stays under it. BF6 in Fullscreen ran at ~20 FPS on
//! the same hardware, Borderless at ~60. See docs/superpowers/specs/2026-11-08-mac-power-fix-design.md.
//!
//! **Safety.** Everything `uninstall` needs (the old GPU table or its absence, the BF6 profile's
//! first-seen values, the old `max_cpu_mhz` line, what install created) goes into the install
//! record ([`RECORD`], in [`INSTALL_DIR`]) before the change. Uninstall is idempotent and keeps the
//! record (and the folder) if a step fails. The GPU table is written only when the gate passes; any
//! error in the gate fails closed.
use serde::{Deserialize, Serialize};

pub const INSTALL_DIR: &str = r"C:\gamebooster-rs";
pub const RECORD: &str = "gamebooster-install.json";
pub const TASK: &str = "gamebooster";
/// CPU cap while BF6 runs. Fallback 1600 MHz if the confirmation run crosses the SMC budget
/// (1800 MHz reached the budget edge after ~7 min).
pub const CPU_CAP_MHZ: u32 = 1700;
pub const BF6: &str = "bf6.exe";
/// Relative to the Documents folder.
pub const PROFILE: &str = r"Battlefield 6\settings\steam\PROFSAVEbf6mp_profile";
/// Borderless: FullscreenMode 2 (1 = Fullscreen), no window borders.
pub const PROFILE_KEYS: [(&str, &str); 2] = [("GstRender.FullscreenMode", "2"), ("GstRender.WindowBordersEnable", "0")];

pub const MAKER: &str = "Apple Inc.";
pub const MODEL: &str = "MacBookPro16,1";
/// Radeon Pro 5500M, Apple subsystem.
pub const GPU_ID: &str = r"PCI\VEN_1002&DEV_7340&SUBSYS_020F106B";
/// The 8 GB card measured (display class key `HardwareInformation.BiosString`).
pub const VBIOS: &str = "113-D32206U1-019";
pub const SKIPPED: &str = "Mac power fix skipped: not a MacBookPro16,1 with the 5500M";

/// What install changed, for `uninstall`. Each backup is the first-seen value: a second install
/// never overwrites it.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Install created [`INSTALL_DIR`]: uninstall deletes the whole folder.
    pub created_folder: bool,
    /// Install copied gamebooster.exe into an existing folder: uninstall deletes the exe.
    pub copied_exe: bool,
    /// Install wrote gamebooster.toml: uninstall deletes it.
    pub config_created: bool,
    /// The config's `max_cpu_mhz` line before install changed it.
    pub config_line: Option<ConfigLine>,
    /// The gate passed: `auto` keeps BF6 in Borderless.
    pub mac: bool,
    pub gpu: Option<GpuBackup>,
    pub profile: Option<ProfileBackup>,
    /// Install registered the logon task.
    pub task: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigLine {
    /// None: there was no active `max_cpu_mhz` line (uninstall removes it).
    pub previous: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GpuBackup {
    /// `SYSTEM\CurrentControlSet\Control\Class\{4d36e968-...}\NNNN`
    pub class_key: String,
    /// The old `PP_PhmSoftPowerPlayTable` as hex; None = absent (uninstall deletes the value).
    pub previous_hex: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileBackup {
    pub path: String,
    /// (key, old value); None = the line was absent (uninstall removes it).
    pub values: Vec<(String, Option<String>)>,
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok())).collect()
}

/// Why this machine fails the gate, or None if it is the Mac the fix was measured on.
pub fn mismatch(maker: &str, model: &str, gpu_id: Option<&str>, vbios: &str) -> Option<String> {
    if maker.trim() != MAKER || model.trim() != MODEL {
        return Some(format!("this is a {} {}", maker.trim(), model.trim()));
    }
    let Some(id) = gpu_id else { return Some("no Radeon Pro 5500M (PCI\\VEN_1002&DEV_7340) found".into()) };
    if !id.to_ascii_uppercase().starts_with(GPU_ID) {
        return Some(format!("GPU {id} is not the Apple 5500M ({GPU_ID})"));
    }
    if !vbios.contains(VBIOS) {
        return Some(format!("GPU VBIOS {:?} is not {VBIOS}", vbios.trim()));
    }
    None
}

/// The first 5500M instance id in `pnputil /enum-devices /class Display` output (labels are
/// localised, the id isn't).
pub fn find_gpu_instance(pnputil: &str) -> Option<String> {
    pnputil.split_whitespace().find(|t| t.to_ascii_uppercase().starts_with(r"PCI\VEN_1002&DEV_7340")).map(String::from)
}

/// Set (Some) or remove (None) the top-level `key = ...` line of a TOML file: replace an active
/// one, else insert after a commented `# key =` example, else before the first table header.
/// Returns the new text and the old active line.
pub fn set_config_line(text: &str, key: &str, new: Option<&str>) -> (String, Option<String>) {
    let mut lines: Vec<String> = text.split_inclusive('\n').map(String::from).collect();
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let is = |l: &str| l.trim_start().strip_prefix(key).is_some_and(|r| r.trim_start().starts_with('='));
    let top = lines.iter().position(|l| l.trim_start().starts_with('[')).unwrap_or(lines.len());
    let active = lines[..top].iter().position(|l| is(l));
    let old = active.map(|i| lines[i].trim_end().to_string());
    match (active, new) {
        (Some(i), Some(n)) => lines[i] = format!("{n}{eol}"),
        (Some(i), None) => drop(lines.remove(i)),
        (None, Some(n)) => {
            let at = lines[..top].iter().position(|l| is(l.trim_start().trim_start_matches('#'))).map_or(top, |i| i + 1);
            if at > 0 && !lines[at - 1].ends_with('\n') {
                lines[at - 1].push_str(eol);
            }
            lines.insert(at, format!("{n}{eol}"));
        }
        (None, None) => {}
    }
    (lines.concat(), old)
}

/// Set (Some) or remove (None) `Key value` lines of a BF6 profile; a missing key is appended.
/// Returns the new text and each key's old value (first occurrence; None = absent).
pub fn edit_profile(text: &str, want: &[(String, Option<String>)]) -> (String, Vec<(String, Option<String>)>) {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.split_inclusive('\n').map(String::from).collect();
    let mut old = Vec::new();
    for (key, value) in want {
        let i = lines.iter().position(|l| l.trim_end().split_once(' ').map_or(l.trim_end(), |(k, _)| k) == key);
        old.push((key.clone(), i.map(|i| lines[i].trim_end().split_once(' ').map_or("", |(_, v)| v).to_string())));
        match (i, value) {
            (Some(i), Some(v)) => lines[i] = format!("{key} {v}{eol}"),
            (Some(i), None) => drop(lines.remove(i)),
            (None, Some(v)) => {
                if let Some(last) = lines.last_mut().filter(|l| !l.ends_with('\n')) {
                    last.push_str(eol);
                }
                lines.push(format!("{key} {v}{eol}"));
            }
            (None, None) => {}
        }
    }
    (lines.concat(), old)
}

/// The Borderless values to write.
pub fn borderless() -> Vec<(String, Option<String>)> {
    PROFILE_KEYS.iter().map(|(k, v)| (k.to_string(), Some(v.to_string()))).collect()
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::*;
    use crate::config::Config;
    use crate::gpu_table;
    use crate::state::{load_json, save_json};
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{RegGetValueW, RegSetKeyValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, REG_BINARY, RRF_RT_ANY};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const CLASS: &str = r"SYSTEM\CurrentControlSet\Control\Class\";

    fn dir() -> PathBuf {
        PathBuf::from(INSTALL_DIR)
    }
    fn record_path() -> PathBuf {
        dir().join(RECORD)
    }
    pub fn load_record() -> Result<Option<Record>, String> {
        load_json(&record_path(), "install record")
    }
    fn save(rec: &Record) -> Result<(), String> {
        save_json(rec, &record_path(), "install record")
    }

    /// A registry value's raw bytes (any type), or None if it doesn't exist.
    fn reg_get(root: HKEY, key: &str, value: &str) -> Result<Option<Vec<u8>>, String> {
        let (k, v) = (crate::wide(key), crate::wide(value));
        let mut size = 0u32;
        let mut err = unsafe { RegGetValueW(root, k.as_ptr(), v.as_ptr(), RRF_RT_ANY, std::ptr::null_mut(), std::ptr::null_mut(), &mut size) };
        if err == ERROR_SUCCESS {
            let mut buf = vec![0u8; size as usize];
            err = unsafe { RegGetValueW(root, k.as_ptr(), v.as_ptr(), RRF_RT_ANY, std::ptr::null_mut(), buf.as_mut_ptr().cast(), &mut size) };
            if err == ERROR_SUCCESS {
                buf.truncate(size as usize);
                return Ok(Some(buf));
            }
        }
        match err {
            ERROR_FILE_NOT_FOUND => Ok(None),
            e => Err(format!("can't read {key}\\{value} (error {e})")),
        }
    }

    /// A REG_SZ (or UTF-16 REG_BINARY, as `HardwareInformation.BiosString` often is) as a string.
    fn reg_string(root: HKEY, key: &str, value: &str) -> Result<Option<String>, String> {
        Ok(reg_get(root, key, value)?.map(|b| {
            let u: Vec<u16> = b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            crate::from_wide(&u)
        }))
    }

    /// Write `Some(bytes)` as REG_BINARY, or delete the value for None (absent is fine).
    fn reg_set_binary(key: &str, value: &str, data: Option<&[u8]>) -> Result<(), String> {
        let Some(d) = data else { return crate::reg_set_dword(HKEY_LOCAL_MACHINE, key, value, None) };
        let err = unsafe { RegSetKeyValueW(HKEY_LOCAL_MACHINE, crate::wide(key).as_ptr(), crate::wide(value).as_ptr(), REG_BINARY, d.as_ptr().cast(), d.len() as u32) };
        if err == ERROR_SUCCESS { Ok(()) } else { Err(format!("can't write {key}\\{value} (error {err})")) }
    }

    pub struct Gpu {
        pub instance_id: String,
        /// Full display class key path under HKLM.
        pub class_key: String,
    }

    fn gpu_instance() -> Result<Option<String>, String> {
        let out = Command::new("pnputil.exe").args(["/enum-devices", "/class", "Display"]).output().map_err(|e| format!("can't run pnputil: {e}"))?;
        Ok(find_gpu_instance(&String::from_utf8_lossy(&out.stdout)))
    }

    /// The Mac gate: Apple MacBookPro16,1 + Apple 5500M + VBIOS 113-D32206U1-019. Any error fails closed.
    pub fn gate() -> Result<Gpu, String> {
        let bios = r"HARDWARE\DESCRIPTION\System\BIOS";
        let maker = reg_string(HKEY_LOCAL_MACHINE, bios, "SystemManufacturer")?.unwrap_or_default();
        let model = reg_string(HKEY_LOCAL_MACHINE, bios, "SystemProductName")?.unwrap_or_default();
        if let Some(why) = mismatch(&maker, &model, Some(GPU_ID), VBIOS) {
            return Err(why); // not the Mac: don't even look at the GPU
        }
        let id = gpu_instance()?;
        let mut class_key = String::new();
        let mut vbios = String::new();
        if let Some(id) = &id {
            let driver = reg_string(HKEY_LOCAL_MACHINE, &format!(r"SYSTEM\CurrentControlSet\Enum\{id}"), "Driver")?.ok_or("the GPU has no driver key")?;
            class_key = format!("{CLASS}{driver}");
            vbios = reg_string(HKEY_LOCAL_MACHINE, &class_key, "HardwareInformation.BiosString")?.unwrap_or_default();
        }
        match mismatch(&maker, &model, id.as_deref(), &vbios) {
            Some(why) => Err(why),
            None => Ok(Gpu { instance_id: id.unwrap_or_default(), class_key }),
        }
    }

    /// Restart the GPU so the driver reads its PowerPlay values again. False: reboot instead.
    fn restart_gpu(id: &str) -> bool {
        println!("  restarting the GPU: the screen goes black for a few seconds");
        match Command::new("pnputil.exe").args(["/restart-device", id]).output() {
            Ok(o) if o.status.success() => true,
            Ok(o) => {
                println!("  pnputil /restart-device failed: {}", String::from_utf8_lossy(&o.stdout).trim());
                false
            }
            Err(e) => {
                println!("  can't run pnputil: {e}");
                false
            }
        }
    }

    /// The GPU table step: back up the old value into the record, write the table, restart the GPU.
    fn gpu_step(rec: &mut Record, gpu: &Gpu) -> Result<String, String> {
        let table = gpu_table::build(&gpu.class_key)?;
        let current = reg_get(HKEY_LOCAL_MACHINE, &gpu.class_key, gpu_table::VALUE)?;
        if current.as_deref() == Some(&table[..]) {
            return Ok(format!("{} W (already set)", gpu_table::GPU_WATTS));
        }
        if rec.gpu.is_none() {
            rec.gpu = Some(GpuBackup { class_key: gpu.class_key.clone(), previous_hex: current.as_deref().map(hex) });
            save(rec)?; // before the write
        }
        reg_set_binary(&gpu.class_key, gpu_table::VALUE, Some(&table))?;
        Ok(if restart_gpu(&gpu.instance_id) { format!("{} W, always on", gpu_table::GPU_WATTS) } else { format!("{} W after a reboot", gpu_table::GPU_WATTS) })
    }

    fn profile_path() -> Option<PathBuf> {
        let docs = reg_string(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders", "Personal").ok().flatten()
            .or_else(|| std::env::var("USERPROFILE").ok().map(|p| format!(r"{p}\Documents")))?;
        Some(PathBuf::from(docs).join(PROFILE))
    }

    fn bf6_running() -> bool {
        crate::processes::list().iter().any(|p| p.name.eq_ignore_ascii_case(BF6))
    }

    /// Put BF6 in Borderless (only while it isn't running; first-seen values backed up first).
    fn borderless_step(rec: &mut Record) -> Result<&'static str, String> {
        let path = profile_path().ok_or("can't find the Documents folder")?;
        let text = match std::fs::read(&path) {
            Ok(b) => String::from_utf8(b).map_err(|_| format!("{} isn't plain text; left alone", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok("set after BF6 has run once (no profile yet)"),
            Err(e) => return Err(format!("can't read {}: {e}", path.display())),
        };
        let (new, old) = edit_profile(&text, &borderless());
        if new == text {
            return Ok("on");
        }
        if bf6_running() {
            return Ok("set after BF6 closes");
        }
        if rec.profile.is_none() {
            rec.profile = Some(ProfileBackup { path: path.display().to_string(), values: old });
            save(rec)?;
        }
        write_atomic(&path, &new)?;
        Ok("set")
    }

    fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
        let tmp = path.with_extension("gamebooster.tmp");
        std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, path)).map_err(|e| format!("can't write {}: {e}", path.display()))
    }

    /// `auto` start and after each BF6 exit: put Borderless back if BF6 (or Steam Cloud) changed it.
    pub fn borderless_recheck() {
        let mut rec = match load_record() {
            Ok(Some(r)) if r.mac => r,
            Ok(_) => return,
            Err(e) => return println!("BF6 Borderless: {e}"),
        };
        match borderless_step(&mut rec) {
            Ok("on") => {}
            Ok(m) => println!("BF6 Borderless: {m}"),
            Err(e) => println!("BF6 Borderless failed: {e}"),
        }
    }

    fn task_registered() -> bool {
        Command::new("schtasks.exe").args(["/query", "/tn", TASK]).output().is_ok_and(|o| o.status.success())
    }

    /// The logon task from docs/auto.md, replacing one with the same name, then started.
    const TASK_PS1: &str = r#"param($Exe, $Dir)
$ErrorActionPreference = 'Stop'
$u = "$env:COMPUTERNAME\$env:USERNAME"
Unregister-ScheduledTask -TaskName gamebooster -Confirm:$false -ErrorAction SilentlyContinue
$a = New-ScheduledTaskAction -Execute $Exe -Argument 'auto' -WorkingDirectory $Dir
$t = New-ScheduledTaskTrigger -AtLogOn -User $u
$p = New-ScheduledTaskPrincipal -UserId $u -LogonType Interactive -RunLevel Highest
$s = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit 0 -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName gamebooster -Action $a -Trigger $t -Principal $p -Settings $s | Out-Null
Start-ScheduledTask gamebooster
"#;

    /// End a running `auto` task and undo a boost or IFEO hold it left (ending it skips its
    /// restore). True when nothing is left to restore.
    fn stop_auto() -> bool {
        let _ = Command::new("schtasks.exe").args(["/end", "/tn", TASK]).output();
        let state = dir().join("gamebooster-state.json");
        if !state.exists() && !crate::held_path(&state).exists() {
            return true;
        }
        match Config::load(&dir().join("gamebooster.toml")) {
            Ok(cfg) => {
                println!("Undoing the boost the stopped `auto` left:");
                let _ = crate::warn("restore", crate::restore(&cfg, &state, false));
                let _ = crate::warn("IFEO restore", crate::release_ifeo(&state));
            }
            Err(e) => println!("  a boost is still active but {e}"),
        }
        let clean = !state.exists() && !crate::held_path(&state).exists();
        if !clean {
            println!("  a boost is still active: run `{INSTALL_DIR}\\gamebooster.exe restore` as administrator");
        }
        clean
    }

    fn copy_exe(exe: &Path, target: &Path) -> Result<(), String> {
        let mut last = String::new();
        for _ in 0..10 {
            // the ended task's process may need a moment to let go of the file
            match std::fs::copy(exe, target) {
                Ok(_) => return Ok(()),
                Err(e) => last = e.to_string(),
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Err(format!("can't copy gamebooster.exe to {}: {last} (close any gamebooster window and run it again)", target.display()))
    }

    fn throttlestop_warning() {
        if crate::processes::list().iter().any(|p| p.name.eq_ignore_ascii_case("ThrottleStop.exe")) {
            println!("warning: ThrottleStop is running: it overrides gamebooster's CPU cap (max_cpu_mhz). Close it and turn off its start with Windows; a power limit it set stays until a reboot.");
        }
    }

    /// First-run setup (no arguments / double-click, or `gamebooster install`).
    pub fn install(exe: &Path) -> Result<(), String> {
        println!("gamebooster {}: setting up\n", crate::install::VERSION);
        stop_auto();
        let dir = dir();
        let mut rec = load_record()?.unwrap_or_else(|| Record { created_folder: !dir.exists(), ..Default::default() });
        std::fs::create_dir_all(&dir).map_err(|e| format!("can't create {}: {e}", dir.display()))?;
        let target = dir.join("gamebooster.exe");
        if !exe.as_os_str().to_string_lossy().eq_ignore_ascii_case(&target.to_string_lossy()) {
            if !rec.created_folder && !target.exists() && !record_path().exists() {
                rec.copied_exe = true;
            }
            save(&rec)?;
            copy_exe(exe, &target)?;
        }
        let config = dir.join("gamebooster.toml");
        if !config.exists() {
            rec.config_created = true;
            save(&rec)?;
            crate::install::write_example(&config)?;
        }
        save(&rec)?;
        let mut summary = vec![];
        match gate() {
            Ok(gpu) => {
                println!("This is a 16\" MacBook Pro 2019 with the Radeon Pro 5500M: applying the Mac power fix.");
                rec.mac = true;
                save(&rec)?;
                match set_cpu_cap(&mut rec, &config) {
                    Ok(()) => summary.push(format!("CPU limited to {CPU_CAP_MHZ} MHz while BF6 runs (max_cpu_mhz in {}), full speed otherwise", config.display())),
                    Err(e) => summary.push(format!("CPU limit not set: {e}")),
                }
                match gpu_step(&mut rec, &gpu) {
                    Ok(m) => summary.push(format!("GPU power limit: {m}")),
                    Err(e) => summary.push(format!("GPU power limit: not set ({e})")),
                }
                match borderless_step(&mut rec) {
                    Ok("on" | "set") => summary.push("BF6 display mode: Borderless (much faster than Fullscreen on this Mac)".into()),
                    Ok(m) => summary.push(format!("BF6 display mode: Borderless is {m}")),
                    Err(e) => summary.push(format!("BF6 display mode: not changed ({e})")),
                }
            }
            Err(why) => println!("{SKIPPED}\n  ({why})"),
        }
        println!("Registering the logon task `{TASK}` (runs `gamebooster auto` when you log on) and starting it");
        crate::install::powershell("task", TASK_PS1, &["-Exe".into(), target.clone().into_os_string(), "-Dir".into(), dir.clone().into_os_string()])?;
        rec.task = true;
        save(&rec)?;
        summary.push("the normal boost (power plan, Game Mode, priority, ...) whenever BF6 starts, undone 30 s after it closes".into());
        println!("\nDone. What gamebooster set up:");
        for s in &summary {
            println!("  - {s}");
        }
        println!("It runs by itself: a small minimised window \"gamebooster auto\" starts when you log on; leave it open.");
        println!("Just open Steam and play BF6.");
        println!("To undo everything: open a terminal as administrator and run  {INSTALL_DIR}\\gamebooster.exe uninstall");
        throttlestop_warning();
        Ok(())
    }

    fn set_cpu_cap(rec: &mut Record, config: &Path) -> Result<(), String> {
        let text = std::fs::read_to_string(config).map_err(|e| format!("can't read {}: {e}", config.display()))?;
        let (new, old) = set_config_line(&text, "max_cpu_mhz", Some(&format!("max_cpu_mhz = {CPU_CAP_MHZ}")));
        let cfg = Config::parse(&new).map_err(|e| format!("the config would not parse ({e}); left unchanged"))?;
        if !cfg.is_game(BF6) {
            println!("  note: {BF6} isn't a [[game]] in {}, so `auto` won't boost for it", config.display());
        }
        if new == text {
            return Ok(());
        }
        if rec.config_line.is_none() {
            rec.config_line = Some(ConfigLine { previous: old });
            save(rec)?;
        }
        write_atomic(config, &new)
    }

    /// `uninstall`: undo everything install did, from the install record. Returns what to delete
    /// once this process has exited (the running exe can't delete itself).
    pub fn uninstall() -> Result<Option<PathBuf>, String> {
        let Some(rec) = load_record()? else {
            println!("Not installed: no {}. Nothing to undo.", record_path().display());
            return Ok(None);
        };
        let mut ok = stop_auto();
        let mut step = |what: &str, r: Result<String, String>| match r {
            Ok(m) => println!("{what}: {m}"),
            Err(e) => {
                println!("{what} failed: {e}");
                ok = false;
            }
        };
        if rec.task {
            step("Logon task", Command::new("schtasks.exe").args(["/delete", "/tn", TASK, "/f"]).output().map_err(|e| e.to_string())
                .and_then(|o| if o.status.success() || !task_registered() { Ok("removed".into()) } else { Err(String::from_utf8_lossy(&o.stderr).trim().to_string()) }));
        }
        if let Some(g) = &rec.gpu {
            step("GPU power table", (|| -> Result<String, String> {
                let previous = g.previous_hex.as_deref().map(|h| unhex(h).ok_or("the backup in the install record is damaged")).transpose()?;
                reg_set_binary(&g.class_key, gpu_table::VALUE, previous.as_deref())?;
                let id = gpu_instance()?.ok_or("restored, but the GPU wasn't found to restart it: reboot")?;
                Ok(format!("{} ({})", if previous.is_some() { "previous table put back" } else { "removed" },
                    if restart_gpu(&id) { "applied" } else { "reboot to apply" }))
            })());
        }
        if let Some(p) = &rec.profile {
            step("BF6 display mode", (|| -> Result<String, String> {
                if bf6_running() {
                    return Err("BF6 is running: close it and run uninstall again".into());
                }
                let path = Path::new(&p.path);
                match std::fs::read_to_string(path) {
                    Ok(text) => write_atomic(path, &edit_profile(&text, &p.values).0).map(|_| "previous setting put back".into()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("profile gone; nothing to put back".into()),
                    Err(e) => Err(format!("can't read {}: {e}", path.display())),
                }
            })());
        }
        let config = dir().join("gamebooster.toml");
        if rec.config_created && !rec.created_folder {
            step("Config", std::fs::remove_file(&config).or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
                .map(|_| "removed".into()).map_err(|e| format!("can't delete {}: {e}", config.display())));
        } else if let Some(c) = rec.config_line.as_ref().filter(|_| !rec.config_created) {
            step("Config max_cpu_mhz", std::fs::read_to_string(&config).map_err(|e| format!("can't read {}: {e}", config.display()))
                .and_then(|text| write_atomic(&config, &set_config_line(&text, "max_cpu_mhz", c.previous.as_deref()).0))
                .map(|_| c.previous.clone().unwrap_or_else(|| "removed".into())));
        }
        if !ok {
            println!("Some steps failed; {} is kept, so you can run uninstall again.", record_path().display());
            return Ok(None);
        }
        std::fs::remove_file(record_path()).map_err(|e| format!("can't delete {}: {e}", record_path().display()))?;
        let gone = if rec.created_folder { Some(dir()) } else if rec.copied_exe { Some(dir().join("gamebooster.exe")) } else { None };
        match &gone {
            Some(p) => println!("Uninstalled. {} is deleted a few seconds after this window closes (if it's still there, delete it by hand).", p.display()),
            None => println!("Uninstalled. {INSTALL_DIR} was there before install, so it stays."),
        }
        Ok(gone)
    }

    /// Delete `path` (the install folder or exe) from a hidden cmd once this process has exited.
    pub fn delete_after_exit(path: &Path) {
        let cmd = if path.is_dir() { "rd /s /q" } else { "del /f /q" };
        let r = Command::new("cmd.exe").raw_arg(format!("/c ping -n 4 127.0.0.1 >nul & {cmd} \"{}\"", path.display()))
            .current_dir(r"C:\").creation_flags(CREATE_NO_WINDOW).spawn();
        if let Err(e) = r {
            println!("can't schedule deleting {}: {e}; delete it by hand", path.display());
        }
    }

    /// `status`: whether the Mac fix is in place, the logon task, ThrottleStop.
    pub fn status() {
        let rec = load_record().unwrap_or_else(|e| {
            println!("Install record: {e}");
            None
        });
        println!("Installed in {INSTALL_DIR}: {}", if rec.is_some() { "yes" } else { "no" });
        match gate() {
            Ok(gpu) => {
                let table = reg_get(HKEY_LOCAL_MACHINE, &gpu.class_key, gpu_table::VALUE);
                println!("Mac power fix: GPU power table ({}): {}", gpu_table::VALUE, match table {
                    Ok(Some(t)) => format!("present ({} bytes)", t.len()),
                    Ok(None) => "absent (stock driver limits)".into(),
                    Err(e) => format!("couldn't read ({e})"),
                });
                let bl = profile_path().map(|p| match std::fs::read_to_string(&p) {
                    Ok(t) if edit_profile(&t, &borderless()).0 == t => "on".to_string(),
                    Ok(t) => format!("off ({})", edit_profile(&t, &borderless()).1.iter().map(|(k, v)| format!("{k}={}", v.as_deref().unwrap_or("absent"))).collect::<Vec<_>>().join(", ")),
                    Err(_) => "no BF6 profile yet".into(),
                });
                println!("Mac power fix: BF6 Borderless: {}", bl.unwrap_or_else(|| "can't find the Documents folder".into()));
            }
            Err(why) => println!("Mac power fix: not this Mac ({why})"),
        }
        println!("Logon task `{TASK}`: {}", if task_registered() { "registered" } else { "not registered" });
        throttlestop_warning();
    }

    /// True if this process owns its console (double-click, task or elevated relaunch) rather
    /// than running inside a terminal.
    pub fn own_console() -> bool {
        let mut ids = [0u32; 2];
        unsafe { windows_sys::Win32::System::Console::GetConsoleProcessList(ids.as_mut_ptr(), 2) == 1 }
    }

    /// Keep a window we own open until the user has read it.
    pub fn pause() {
        if own_console() {
            println!("\nPress Enter to close this window.");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    }

    /// Relaunch this exe elevated (UAC prompt) with `args`. Ok: the elevated copy has started.
    pub fn elevate(exe: &Path, args: &str) -> Result<(), String> {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let h = unsafe { ShellExecuteW(std::ptr::null_mut(), crate::wide("runas").as_ptr(), crate::wide(&exe.to_string_lossy()).as_ptr(), crate::wide(args).as_ptr(), std::ptr::null(), SW_SHOWNORMAL) };
        if h as usize > 32 { Ok(()) } else { Err("gamebooster needs administrator rights: run it again and click Yes".into()) }
    }

    /// `auto`'s console: a plain title, and minimised when it has its own window (the logon task).
    pub fn auto_console() {
        use windows_sys::Win32::System::Console::{GetConsoleWindow, SetConsoleTitleW};
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWMINNOACTIVE};
        unsafe { SetConsoleTitleW(crate::wide("gamebooster auto: boosts BF6 while it runs. Closing this window undoes the boost").as_ptr()) };
        if own_console() {
            unsafe { ShowWindow(GetConsoleWindow(), SW_SHOWMINNOACTIVE) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_matches_only_the_measured_mac() {
        let id = r"PCI\VEN_1002&DEV_7340&SUBSYS_020F106B&REV_40\6&2A1B3C4D&0&00000008";
        assert_eq!(mismatch("Apple Inc.", "MacBookPro16,1", Some(id), "113-D32206U1-019"), None);
        assert_eq!(mismatch("Apple Inc.", "MacBookPro16,1", Some(&id.to_ascii_lowercase()), "113-D32206U1-019\0"), None);
        assert!(mismatch("Apple Inc.", "MacBookPro15,1", Some(id), VBIOS).is_some());
        assert!(mismatch("Dell Inc.", MODEL, Some(id), VBIOS).is_some());
        assert!(mismatch(MAKER, MODEL, None, VBIOS).is_some());
        assert!(mismatch(MAKER, MODEL, Some(r"PCI\VEN_1002&DEV_7340&SUBSYS_0B0C1028&REV_C1\x"), VBIOS).is_some());
        assert!(mismatch(MAKER, MODEL, Some(id), "113-D32206U1-020").is_some());
        assert!(mismatch(MAKER, MODEL, Some(id), "").is_some());
    }

    #[test]
    fn pnputil_parsing() {
        let out = "Microsoft PnP Utility\r\n\r\nInstance ID:                PCI\\VEN_8086&DEV_3E9B&SUBSYS_00000000&REV_02\\3&11583659&0&10\r\n\
                   Device Description:         Intel(R) UHD Graphics 630\r\n\r\nInstance ID:                PCI\\VEN_1002&DEV_7340&SUBSYS_020F106B&REV_40\\6&1&0&00000008\r\n\
                   Device Description:         AMD Radeon Pro 5500M\r\n";
        assert_eq!(find_gpu_instance(out).as_deref(), Some(r"PCI\VEN_1002&DEV_7340&SUBSYS_020F106B&REV_40\6&1&0&00000008"));
        assert_eq!(find_gpu_instance("Instance ID: PCI\\VEN_10DE&DEV_1F91\\x"), None);
    }

    #[test]
    fn config_line_set_and_revert() {
        let ex = include_str!("../gamebooster.example.toml");
        let (set, old) = set_config_line(ex, "max_cpu_mhz", Some("max_cpu_mhz = 1700"));
        assert_eq!(old, None);
        assert!(set.contains("# max_cpu_mhz = 2800\nmax_cpu_mhz = 1700\n"), "inserted after the example");
        assert_eq!(crate::config::Config::parse(&set).unwrap().max_cpu_mhz, Some(1700));
        assert_eq!(set_config_line(&set, "max_cpu_mhz", None), (ex.to_string(), Some("max_cpu_mhz = 1700".into())));
        let (s2, old) = set_config_line("max_cpu_mhz = 3000\r\n[[game]]\r\nexe = \"bf6.exe\"\r\n", "max_cpu_mhz", Some("max_cpu_mhz = 1700"));
        assert_eq!((s2.as_str(), old.as_deref()), ("max_cpu_mhz = 1700\r\n[[game]]\r\nexe = \"bf6.exe\"\r\n", Some("max_cpu_mhz = 3000")));
        // no example line: before the first table, never inside [[game]]
        let (s3, _) = set_config_line("power_plan = true\n[[game]]\nexe = \"bf6.exe\"", "max_cpu_mhz", Some("max_cpu_mhz = 1700"));
        assert_eq!(s3, "power_plan = true\nmax_cpu_mhz = 1700\n[[game]]\nexe = \"bf6.exe\"");
        assert_eq!(set_config_line("game_mode = true", "max_cpu_mhz", Some("max_cpu_mhz = 1700")).0, "game_mode = true\nmax_cpu_mhz = 1700\n");
        assert_eq!(set_config_line("max_cpu_mhz_x = 1\n", "max_cpu_mhz", None).1, None);
    }

    #[test]
    fn profile_borderless_and_restore() {
        let text = "GstRender.Dx12Enabled 1\r\nGstRender.FullscreenMode 1\r\nGstRender.ResolutionScale 1.000000\r\n";
        let (set, old) = edit_profile(text, &borderless());
        assert_eq!(set, "GstRender.Dx12Enabled 1\r\nGstRender.FullscreenMode 2\r\nGstRender.ResolutionScale 1.000000\r\nGstRender.WindowBordersEnable 0\r\n");
        assert_eq!(old, [("GstRender.FullscreenMode".to_string(), Some("1".to_string())), ("GstRender.WindowBordersEnable".to_string(), None)]);
        assert_eq!(edit_profile(&set, &borderless()).0, set, "already Borderless: unchanged");
        assert_eq!(edit_profile(&set, &old).0, text, "backup puts it back exactly");
        assert_eq!(edit_profile("GstRender.FullscreenModeX 1", &borderless()).0.lines().count(), 3, "whole key only");
    }

    #[test]
    fn record_roundtrip_and_hex() {
        assert_eq!(unhex(&hex(&[0, 0xab, 0xff, 7])), Some(vec![0, 0xab, 0xff, 7]));
        assert_eq!(unhex("abc"), None);
        assert_eq!(unhex("zz"), None);
        let path = std::env::temp_dir().join(format!("gamebooster-install-test-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(crate::state::load_json::<Record>(&path, "install record").unwrap(), None);
        let rec = Record {
            created_folder: true,
            config_line: Some(ConfigLine { previous: None }),
            mac: true,
            gpu: Some(GpuBackup { class_key: r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000".into(), previous_hex: Some(hex(&[1, 2])) }),
            profile: Some(ProfileBackup { path: r"C:\Users\a\Documents\x".into(), values: vec![("GstRender.FullscreenMode".into(), Some("1".into()))] }),
            task: true,
            ..Default::default()
        };
        crate::state::save_json(&rec, &path, "install record").unwrap();
        assert_eq!(crate::state::load_json(&path, "install record").unwrap(), Some(rec));
        std::fs::remove_file(&path).unwrap();
    }
}
