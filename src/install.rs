//! Getting gamebooster onto Windows and keeping it current: `update`, `init`, and the capture
//! scripts from capture/ embedded in the exe (`snapshot`, `compare`). Std only.
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Crate version plus the git short hash it was built from (build.rs).
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GB_GIT_HASH"), ")");
const URL: &str = "https://github.com/ztancrell/gamebooster-rs/releases/download/latest/gamebooster.exe";
pub const SNAPSHOT_PS1: &str = include_str!("../capture/snapshot.ps1");
pub const COMPARE_PS1: &str = include_str!("../capture/compare.ps1");
const EXAMPLE: &str = include_str!("../gamebooster.example.toml");

/// Where `update` moves the running exe (Windows can rename it, not delete it).
pub fn old_path(exe: &Path) -> PathBuf {
    let mut p = exe.as_os_str().to_owned();
    p.push(".old");
    p.into()
}

/// A Windows program starts with the DOS header "MZ"; an empty file or an HTML error page doesn't.
pub fn looks_like_exe(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ")
}

/// Download the latest release next to `exe`, check it, swap it in.
pub fn update(exe: &Path, boosted: bool) -> Result<(), String> {
    // .exe extension so it can be started for the --version check.
    let new = exe.with_extension("new.exe");
    println!("current: gamebooster {VERSION}\ndownloading {URL}");
    // Unique query: some GitHub edges cache the `latest` redirect and point at a deleted asset (404).
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let ok = Command::new("curl.exe").args(["-fL", "--progress-bar", "-o"]).arg(&new).arg(format!("{URL}?t={t}")).status()
        .map_err(|e| format!("can't run curl.exe (needs Windows 10 1803 or later): {e}"))?
        .success();
    let r = if ok { swap(exe, &new, boosted) } else { Err("download failed (see curl's message above)".into()) };
    let _ = fs::remove_file(&new); // already gone after a successful swap
    r
}

fn swap(exe: &Path, new: &Path, boosted: bool) -> Result<(), String> {
    let bytes = fs::read(new).map_err(|e| format!("can't read {}: {e}", new.display()))?;
    if !looks_like_exe(&bytes) {
        return Err(format!("the download isn't a Windows program ({} bytes); nothing changed", bytes.len()));
    }
    let out = Command::new(new).arg("--version").output().map_err(|e| format!("can't run the download: {e}"))?;
    let version = String::from_utf8_lossy(&out.stdout).trim().trim_start_matches("gamebooster ").to_string();
    if !out.status.success() || version.is_empty() {
        return Err("the downloaded gamebooster doesn't run; nothing changed".into());
    }
    if version == VERSION {
        println!("Already up to date: {VERSION}");
        return Ok(());
    }
    let old = old_path(exe);
    let _ = fs::remove_file(&old);
    fs::rename(exe, &old).map_err(|e| format!("can't move {} aside: {e}", exe.display()))?;
    if let Err(e) = fs::rename(new, exe) {
        let _ = fs::rename(&old, exe);
        return Err(format!("can't put the new version in place: {e}; nothing changed"));
    }
    println!("Updated: {VERSION} -> {version}");
    if boosted {
        println!("note: a boost is active; `gamebooster restore` works the same with the new version.");
    }
    Ok(())
}

/// Write the example config to `path`, never over an existing file.
pub fn init(path: &Path) -> Result<(), String> {
    use std::io::Write;
    match fs::OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut f) => {
            f.write_all(EXAMPLE.as_bytes()).map_err(|e| format!("can't write {}: {e}", path.display()))?;
            println!("Wrote {}. Edit it (notepad {0}), then run `gamebooster boost --dry-run`.", path.display());
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!("{} already exists; left it unchanged.", path.display());
            Ok(())
        }
        Err(e) => Err(format!("can't create {}: {e}", path.display())),
    }
}

/// Run an embedded capture script with Windows PowerShell; its output goes straight to the console.
pub fn powershell(name: &str, script: &str, args: &[OsString]) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("gamebooster-{name}-{}.ps1", std::process::id()));
    fs::write(&tmp, script).map_err(|e| format!("can't write {}: {e}", tmp.display()))?;
    let st = Command::new("powershell.exe").args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]).arg(&tmp).args(args).status();
    let _ = fs::remove_file(&tmp);
    match st {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("{name}.ps1 failed ({s})")),
        Err(e) => Err(format!("can't run powershell.exe: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mz_check() {
        assert!(looks_like_exe(b"MZ\x90\x00"));
        assert!(!looks_like_exe(b""));
        assert!(!looks_like_exe(b"M"));
        assert!(!looks_like_exe(b"<!DOCTYPE html>"));
    }

    #[test]
    fn old_path_appends() {
        assert_eq!(old_path(Path::new("C:/gb/gamebooster.exe")), PathBuf::from("C:/gb/gamebooster.exe.old"));
        assert_eq!(Path::new("C:/gb/gamebooster.exe").with_extension("new.exe"), PathBuf::from("C:/gb/gamebooster.new.exe"));
    }

    #[test]
    fn version_has_hash() {
        assert!(VERSION.starts_with(env!("CARGO_PKG_VERSION")) && VERSION.ends_with(')'), "{VERSION}");
    }
}
