//! `gamebooster bench`: record the running game's frame times with Intel PresentMon, save the stats
//! as JSON, and compare two runs with run-to-run noise accounted for (docs/measuring.md).
//!
//! PresentMon (MIT, github.com/GameTechDev/PresentMon) reads the DXGI/D3D present events Windows
//! already emits over ETW: nothing is injected into the game. It is downloaded once into `tools\`
//! next to gamebooster.exe and refused unless its SHA-256 matches the pinned one.
use serde::{Deserialize, Serialize};

pub const PRESENTMON_VERSION: &str = "2.6.0";
const PRESENTMON_URL: &str = "https://github.com/GameTechDev/PresentMon/releases/download/v2.6.0/PresentMon-2.6.0-x64.exe";
const PRESENTMON_SHA256: &str = "b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af";
const PRESENTMON_FILE: &str = "PresentMon-2.6.0-x64.exe";
/// Length of the chunks whose average FPS the noise test compares.
pub const CHUNK_SECONDS: f64 = 10.0;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub frames: usize,
    pub avg_fps: f64,
    pub median_ms: f64,
    pub p99_ms: f64,
    pub p999_ms: f64,
    /// FPS at the 99th / 99.9th percentile frame time.
    pub low1_fps: f64,
    pub low01_fps: f64,
    /// Frames longer than twice the median.
    pub stutters: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchResult {
    pub label: String,
    pub game: String,
    pub seconds: u64,
    /// Local time the run started, as in the file name.
    pub started: String,
    /// A boost was active (state file present).
    pub boosted: bool,
    pub stats: Stats,
    pub chunk_seconds: f64,
    pub chunk_fps: Vec<f64>,
    pub chunk_low1_fps: Vec<f64>,
    /// The boost-relevant config keys at bench time.
    pub config: serde_json::Value,
    pub gamebooster: String,
    pub presentmon: String,
}

/// Index of the frame-time column: `MsBetweenPresents` (v2; `msBetweenPresents` in v1 metrics),
/// else v2's `FrameTime`.
pub fn frame_time_column(header: &str) -> Option<usize> {
    let cols: Vec<&str> = header.split(',').map(str::trim).collect();
    ["MsBetweenPresents", "FrameTime"].iter().find_map(|want| cols.iter().position(|c| c.eq_ignore_ascii_case(want)))
}

/// Frame times (ms) from a PresentMon CSV. Rows with `NA` or a non-positive time are skipped.
pub fn parse_csv(text: &str) -> Result<Vec<f64>, String> {
    let mut lines = text.lines();
    let header = lines.next().ok_or("PresentMon wrote an empty CSV")?;
    let col = frame_time_column(header).ok_or_else(|| format!("no MsBetweenPresents or FrameTime column in the CSV header: {header}"))?;
    // ponytail: plain split; PresentMon fields don't contain commas or quotes.
    Ok(lines.filter_map(|l| l.split(',').nth(col)?.trim().parse::<f64>().ok()).filter(|&ms| ms > 0.0).collect())
}

/// Nearest-rank percentile of an ascending slice.
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    let rank = (p / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

pub fn stats(frames_ms: &[f64]) -> Stats {
    if frames_ms.is_empty() {
        return Stats::default();
    }
    let mut s = frames_ms.to_vec();
    s.sort_by(f64::total_cmp);
    let median = percentile(&s, 50.0);
    let (p99, p999) = (percentile(&s, 99.0), percentile(&s, 99.9));
    Stats {
        frames: s.len(),
        avg_fps: 1000.0 * s.len() as f64 / s.iter().sum::<f64>(),
        median_ms: median,
        p99_ms: p99,
        p999_ms: p999,
        low1_fps: 1000.0 / p99,
        low01_fps: 1000.0 / p999,
        stutters: s.iter().filter(|&&ms| ms > 2.0 * median).count(),
    }
}

/// Split frames (in order) into consecutive chunks of `secs` by accumulated frame time. A last
/// chunk shorter than half of `secs` is dropped (too few frames to say anything).
pub fn chunks(frames_ms: &[f64], secs: f64) -> Vec<Vec<f64>> {
    let (mut out, mut cur, mut t) = (Vec::new(), Vec::new(), 0.0);
    for &ms in frames_ms {
        cur.push(ms);
        t += ms;
        if t >= secs * 1000.0 {
            out.push(std::mem::take(&mut cur));
            t = 0.0;
        }
    }
    if t >= secs * 500.0 {
        out.push(cur);
    }
    out
}

fn mean_var(x: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let m = x.iter().sum::<f64>() / n;
    (m, x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n - 1.0))
}

/// Two-sided 95 % critical t value; df rounded down (conservative), normal beyond 30.
fn t_crit_95(df: f64) -> f64 {
    const T: [f64; 30] = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145, 2.131,
        2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042];
    let i = df.floor().max(1.0) as usize;
    if i <= 30 { T[i - 1] } else if i <= 60 { 2.021 } else { 1.96 }
}

/// Welch's t-test on two samples (per-chunk FPS of two runs). Returns (difference of means b - a,
/// |t|, critical t at 95 %), or None with fewer than 2 values on either side.
pub fn welch(a: &[f64], b: &[f64]) -> Option<(f64, f64, f64)> {
    if a.len() < 2 || b.len() < 2 {
        return None;
    }
    let ((ma, va), (mb, vb)) = (mean_var(a), mean_var(b));
    let (sa, sb) = (va / a.len() as f64, vb / b.len() as f64);
    let se2 = sa + sb;
    let diff = mb - ma;
    if se2 == 0.0 {
        let t = if diff == 0.0 { 0.0 } else { f64::INFINITY };
        return Some((diff, t, t_crit_95((a.len() + b.len() - 2) as f64)));
    }
    let df = se2 * se2 / (sa * sa / (a.len() - 1) as f64 + sb * sb / (b.len() - 1) as f64);
    Some((diff, diff.abs() / se2.sqrt(), t_crit_95(df)))
}

/// One line judging `b - a` for a metric with per-chunk values.
pub fn judge(name: &str, a: f64, b: f64, chunks_a: &[f64], chunks_b: &[f64]) -> String {
    let d = b - a;
    let pct = if a != 0.0 { 100.0 * d / a } else { 0.0 };
    let verdict = match welch(chunks_a, chunks_b) {
        None => "too few chunks to judge (bench at least 20 s)".to_string(),
        Some((_, t, crit)) if t > crit => format!("real difference (Welch t = {t:.2} > {crit:.2}, ~95 %)"),
        Some((_, t, crit)) => format!("within run-to-run noise (Welch t = {t:.2} <= {crit:.2})"),
    };
    format!("{name}: B - A = {d:+.1} ({pct:+.1} %): {verdict}")
}

/// Warnings that make a comparison doubtful.
pub fn warnings(a: &BenchResult, b: &BenchResult) -> Vec<String> {
    let mut w = Vec::new();
    if !a.game.eq_ignore_ascii_case(&b.game) {
        w.push(format!("different games ({} vs {}): the numbers aren't comparable", a.game, b.game));
    }
    let (lo, hi) = (a.seconds.min(b.seconds) as f64, a.seconds.max(b.seconds) as f64);
    if hi > lo * 1.5 {
        w.push(format!("very different durations ({} s vs {} s): bench both for the same time", a.seconds, b.seconds));
    }
    w
}

/// Config keys whose value differs between the runs.
pub fn config_diff(a: &serde_json::Value, b: &serde_json::Value) -> Vec<String> {
    let (Some(a), Some(b)) = (a.as_object(), b.as_object()) else { return Vec::new() };
    let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter().filter(|k| a.get(*k) != b.get(*k))
        .map(|k| format!("{k}: {} -> {}", a.get(k).unwrap_or(&serde_json::Value::Null), b.get(k).unwrap_or(&serde_json::Value::Null)))
        .collect()
}

/// Side-by-side table plus judgement for `bench compare`.
pub fn compare_text(a: &BenchResult, b: &BenchResult) -> String {
    let yn = |v: bool| if v { "yes" } else { "no" }.to_string();
    let rows: [(&str, String, String); 11] = [
        ("label", a.label.clone(), b.label.clone()),
        ("game", a.game.clone(), b.game.clone()),
        ("started", a.started.clone(), b.started.clone()),
        ("duration s", a.seconds.to_string(), b.seconds.to_string()),
        ("boosted", yn(a.boosted), yn(b.boosted)),
        ("frames", a.stats.frames.to_string(), b.stats.frames.to_string()),
        ("avg FPS", format!("{:.1}", a.stats.avg_fps), format!("{:.1}", b.stats.avg_fps)),
        ("median ms", format!("{:.2}", a.stats.median_ms), format!("{:.2}", b.stats.median_ms)),
        ("1% low FPS", format!("{:.1}", a.stats.low1_fps), format!("{:.1}", b.stats.low1_fps)),
        ("0.1% low FPS", format!("{:.1}", a.stats.low01_fps), format!("{:.1}", b.stats.low01_fps)),
        ("stutters", a.stats.stutters.to_string(), b.stats.stutters.to_string()),
    ];
    let mut out = format!("{:<14}{:<22}{}\n", "", "A", "B");
    for (k, x, y) in rows {
        out += &format!("{k:<14}{x:<22}{y}\n");
    }
    for d in config_diff(&a.config, &b.config) {
        out += &format!("config        {d}\n");
    }
    out += "\n";
    out += &judge("avg FPS", a.stats.avg_fps, b.stats.avg_fps, &a.chunk_fps, &b.chunk_fps);
    out += "\n";
    out += &judge("1% low FPS", a.stats.low1_fps, b.stats.low1_fps, &a.chunk_low1_fps, &b.chunk_low1_fps);
    out += "\n";
    for w in warnings(a, b) {
        out += &format!("warning: {w}\n");
    }
    out
}

/// Keep a label safe as part of a file name.
pub fn clean_label(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

/// The SHA-256 from `certutil -hashfile <f> SHA256` output (older Windows puts spaces between bytes).
pub fn parse_certutil(out: &str) -> Option<String> {
    out.lines().map(|l| l.replace(' ', "").to_ascii_lowercase()).find(|l| l.len() == 64 && l.bytes().all(|c| c.is_ascii_hexdigit()))
}

pub fn load(path: &std::path::Path) -> Result<BenchResult, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} isn't a bench result: {e}", path.display()))
}

#[cfg(windows)]
pub use sys::*;

#[cfg(windows)]
mod sys {
    use super::*;
    use crate::config::Config;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn sha256(path: &Path) -> Result<String, String> {
        let out = Command::new("certutil.exe").arg("-hashfile").arg(path).arg("SHA256").output()
            .map_err(|e| format!("can't run certutil.exe: {e}"))?;
        parse_certutil(&String::from_utf8_lossy(&out.stdout)).ok_or_else(|| format!("certutil couldn't hash {}", path.display()))
    }

    /// The pinned PresentMon in `tools\`, downloaded on first use; refused if its hash is wrong.
    fn presentmon(dir: &Path) -> Result<PathBuf, String> {
        let tools = dir.join("tools");
        let exe = tools.join(PRESENTMON_FILE);
        if !exe.exists() {
            std::fs::create_dir_all(&tools).map_err(|e| format!("can't create {}: {e}", tools.display()))?;
            let part = tools.join(format!("{PRESENTMON_FILE}.part"));
            println!("PresentMon {PRESENTMON_VERSION} (Intel, MIT licence) isn't in {} yet; downloading {PRESENTMON_URL}", tools.display());
            let ok = Command::new("curl.exe").args(["-fL", "--progress-bar", "-o"]).arg(&part).arg(PRESENTMON_URL).status()
                .map_err(|e| format!("can't run curl.exe (needs Windows 10 1803 or later): {e}"))?.success();
            if !ok {
                let _ = std::fs::remove_file(&part);
                return Err("PresentMon download failed (see curl's message above)".into());
            }
            std::fs::rename(&part, &exe).map_err(|e| format!("can't move the download into place: {e}"))?;
        }
        let got = sha256(&exe)?;
        if got != PRESENTMON_SHA256 {
            let _ = std::fs::remove_file(&exe);
            return Err(format!("{} has SHA-256 {got}, expected {PRESENTMON_SHA256}; deleted it and won't run it. Run bench again to re-download.", exe.display()));
        }
        Ok(exe)
    }

    /// Local time as `YYYY-MM-DD_HHMMSS` (safe in a file name).
    fn now_local() -> String {
        let mut t: windows_sys::Win32::Foundation::SYSTEMTIME = unsafe { std::mem::zeroed() };
        unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
        format!("{:04}-{:02}-{:02}_{:02}{:02}{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
    }

    fn config_snapshot(cfg: &Config) -> serde_json::Value {
        serde_json::json!({
            "power_plan": cfg.power_plan,
            "min_processor_state": cfg.min_processor_state.unwrap_or(100),
            "max_processor_state": cfg.max_processor_state,
            "max_cpu_mhz": cfg.max_cpu_mhz.unwrap_or(0),
            "game_mode": cfg.game_mode,
            "accessibility_shortcuts_off": cfg.accessibility_shortcuts_off,
            "ram_trim": cfg.ram_trim,
            "services": cfg.services.len(),
            "close": cfg.close.len(),
            "lower_others": cfg.lower_others,
            "lower_others_mode": format!("{:?}", cfg.lower_others_mode).to_lowercase(),
            "lower_others_class": crate::priority::lower_class_name(cfg.lower_others_class),
            "fastest_dns": cfg.fastest_dns,
            "game_power_throttling_off": cfg.game_power_throttling_off,
            "game_priority_method": format!("{:?}", cfg.game_priority_method).to_lowercase(),
            "games": cfg.games.iter().map(|g| format!("{} {:?}", g.exe, g.priority)).collect::<Vec<_>>(),
        })
    }

    pub fn run(cfg: &Config, dir: &Path, boosted: bool, seconds: u64, label: Option<&str>) -> Result<(), String> {
        let running = crate::processes::list();
        let game = cfg.games.iter().find_map(|g| running.iter().find(|p| p.name.eq_ignore_ascii_case(&g.exe)))
            .map(|p| p.name.clone())
            .ok_or("no configured game is running: start it (see [[game]] in the config), get into a match, then run bench")?;
        let pm = presentmon(dir)?;
        let label = clean_label(label.unwrap_or(if boosted { "boosted" } else { "normal" }));
        let started = now_local();
        let out_dir = dir.join("bench");
        std::fs::create_dir_all(&out_dir).map_err(|e| format!("can't create {}: {e}", out_dir.display()))?;
        let csv = out_dir.join(format!("{started}_{label}.csv"));
        println!("Recording {game} for {seconds} s with PresentMon {PRESENTMON_VERSION}. Switch to the game now and play the same scene.");
        let out = Command::new(&pm)
            .args(["--process_name", &game, "--timed", &seconds.to_string(), "--terminate_after_timed", "--terminate_on_proc_exit",
                "--no_console_stats", "--session_name", "gamebooster_bench", "--stop_existing_session", "--output_file"])
            .arg(&csv)
            .output()
            .map_err(|e| format!("can't run {}: {e}", pm.display()))?;
        let text = std::fs::read_to_string(&csv).map_err(|e| {
            format!("PresentMon wrote no CSV ({e}); its output:\n{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
        })?;
        let frames = parse_csv(&text)?;
        if frames.is_empty() {
            return Err(format!("no frames recorded from {game}: is it rendering (not minimised)? Raw CSV: {}", csv.display()));
        }
        let ch = chunks(&frames, CHUNK_SECONDS);
        let r = BenchResult {
            label: label.clone(),
            game,
            seconds,
            started: started.clone(),
            boosted,
            stats: stats(&frames),
            chunk_seconds: CHUNK_SECONDS,
            chunk_fps: ch.iter().map(|c| stats(c).avg_fps).collect(),
            chunk_low1_fps: ch.iter().map(|c| stats(c).low1_fps).collect(),
            config: config_snapshot(cfg),
            gamebooster: crate::install::VERSION.into(),
            presentmon: PRESENTMON_VERSION.into(),
        };
        let json = out_dir.join(format!("{started}_{label}.json"));
        let body = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?;
        std::fs::write(&json, body).map_err(|e| format!("can't write {}: {e}", json.display()))?;
        let s = &r.stats;
        println!("{} {} s, {}:", r.game, r.seconds, if boosted { "boosted" } else { "not boosted" });
        println!("  {} frames, avg {:.1} FPS, median {:.2} ms", s.frames, s.avg_fps, s.median_ms);
        println!("  1% low {:.1} FPS, 0.1% low {:.1} FPS, stutters (> 2x median) {}", s.low1_fps, s.low01_fps, s.stutters);
        println!("  avg FPS per {CHUNK_SECONDS} s: {}", r.chunk_fps.iter().map(|f| format!("{f:.1}")).collect::<Vec<_>>().join(" "));
        println!("Saved {} (raw frames: {})", json.display(), csv.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_detection() {
        assert_eq!(frame_time_column("Application,ProcessID,MsBetweenPresents,MsInPresentAPI"), Some(2));
        assert_eq!(frame_time_column("Application,TimeInSeconds,msBetweenPresents"), Some(2));
        assert_eq!(frame_time_column("Application,FrameTime,CPUBusy"), Some(1));
        assert_eq!(frame_time_column("Application,MsCPUBusy"), None);
    }

    #[test]
    fn csv_parse() {
        let csv = "Application,ProcessID,MsBetweenPresents\r\nbf6.exe,42,16.5\r\nbf6.exe,42,NA\r\nbf6.exe,42,0\r\nbf6.exe,42,17.0\r\n";
        assert_eq!(parse_csv(csv).unwrap(), vec![16.5, 17.0]);
        assert!(parse_csv("Application,X\nbf6.exe,1").is_err());
        assert!(parse_csv("").is_err());
    }

    #[test]
    fn percentiles_and_lows() {
        // 990 frames of 10 ms and 10 frames of 40 ms: 1 % low is the 99th percentile = 10 ms.
        let mut f = vec![10.0; 990];
        f.extend([40.0; 10]);
        let s = stats(&f);
        assert_eq!(s.frames, 1000);
        assert_eq!(s.median_ms, 10.0);
        assert_eq!(s.p99_ms, 10.0);
        assert_eq!(s.p999_ms, 40.0);
        assert!((s.low1_fps - 100.0).abs() < 1e-9 && (s.low01_fps - 25.0).abs() < 1e-9);
        assert!((s.avg_fps - 1000.0 / 10.3).abs() < 1e-9);
        assert_eq!(s.stutters, 10);
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 50.0), 2.0);
        assert_eq!(percentile(&[5.0], 99.9), 5.0);
        assert_eq!(stats(&[]), Stats::default());
    }

    #[test]
    fn stutter_threshold_is_strict() {
        assert_eq!(stats(&[10.0, 10.0, 10.0, 20.0, 20.1]).stutters, 1);
    }

    #[test]
    fn chunking() {
        let c = chunks(&[5000.0; 5], 10.0); // 25 s: two full chunks, 5 s tail kept (half a chunk)
        assert_eq!(c.iter().map(Vec::len).collect::<Vec<_>>(), [2, 2, 1]);
        let c = chunks(&[1000.0; 24], 10.0); // 24 s: 4 s tail dropped
        assert_eq!(c.len(), 2);
        assert!(chunks(&[], 10.0).is_empty());
    }

    #[test]
    fn noise_test() {
        let a = [70.0, 71.0, 69.5, 70.5, 70.2, 69.8];
        let shifted: Vec<f64> = a.iter().map(|x| x + 5.0).collect();
        let (d, t, crit) = welch(&a, &shifted).unwrap();
        assert!((d - 5.0).abs() < 1e-9 && t > crit);
        let noisy = [60.0, 80.0, 65.0, 78.0, 62.0, 77.0];
        let (_, t, crit) = welch(&a, &noisy).unwrap();
        assert!(t <= crit, "t = {t}");
        assert!(welch(&[1.0], &a).is_none());
        assert_eq!(welch(&[1.0, 1.0], &[1.0, 1.0]).unwrap().1, 0.0);
        assert!(judge("avg FPS", 70.0, 75.0, &a, &shifted).contains("real difference"));
        assert!(judge("avg FPS", 70.0, 70.3, &a, &noisy).contains("within run-to-run noise"));
        assert!(judge("x", 1.0, 2.0, &[1.0], &[2.0]).contains("too few"));
        assert_eq!(t_crit_95(5.7), 2.571);
        assert_eq!(t_crit_95(0.3), 12.706);
        assert_eq!(t_crit_95(500.0), 1.96);
    }

    fn result(game: &str, seconds: u64, fps: f64) -> BenchResult {
        BenchResult {
            label: "x".into(), game: game.into(), seconds, started: String::new(), boosted: false,
            stats: Stats { avg_fps: fps, ..Default::default() }, chunk_seconds: CHUNK_SECONDS,
            chunk_fps: vec![fps; 3], chunk_low1_fps: vec![fps / 2.0; 3],
            config: serde_json::json!({"power_plan": false, "services": 0}), gamebooster: String::new(), presentmon: String::new(),
        }
    }

    #[test]
    fn compare_warns_and_diffs() {
        let a = result("bf6.exe", 60, 70.0);
        let mut b = result("BF6.exe", 60, 80.0);
        b.config["power_plan"] = true.into();
        assert!(warnings(&a, &b).is_empty());
        assert_eq!(warnings(&a, &result("other.exe", 120, 70.0)).len(), 2);
        assert_eq!(config_diff(&a.config, &b.config), ["power_plan: false -> true"]);
        let t = compare_text(&a, &b);
        assert!(t.contains("avg FPS: B - A = +10.0 (+14.3 %): real difference"), "{t}");
        assert!(t.contains("config        power_plan: false -> true"), "{t}");
        // round-trips through JSON
        let back: BenchResult = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(back.chunk_fps, a.chunk_fps);
    }

    #[test]
    fn certutil_and_labels() {
        let new = "SHA256 hash of C:\\gb\\tools\\pm.exe:\r\nb2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af\r\nCertUtil: -hashfile command completed successfully.\r\n";
        assert_eq!(parse_certutil(new).as_deref(), Some(PRESENTMON_SHA256));
        let old = "SHA256 hash of file x:\r\nB2 A7 06 BC 6A D4 75 74 9E 3B 7E 34 09 26 3A A1 E6 90 6D 45 BD CF 99 3F 6D BC 0F 66 01 88 F1 AF\r\nCertUtil: done\r\n";
        assert_eq!(parse_certutil(old).as_deref(), Some(PRESENTMON_SHA256));
        assert_eq!(parse_certutil("CertUtil: error"), None);
        assert_eq!(clean_label("power plan/off"), "power_plan_off");
    }
}
