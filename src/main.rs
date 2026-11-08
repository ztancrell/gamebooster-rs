//! gamebooster: boost Windows for a game session, then restore everything.
//!
//! Boost applies the tweaks in [`BOOST_ORDER`]; restore undoes them in reverse. Everything restore
//! needs is saved to the state file before the first change. Each tweak module's doc comment
//! explains what it changes and why; docs/tweaks/ has the same per tweak, with manual commands.
#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

mod accessibility;
mod bench;
mod config;
mod game_mode;
mod gpu_table;
mod install;
mod mac;
mod network;
mod power;
mod priority;
mod processes;
mod ram;
mod services;
mod state;
mod watch;

use config::Config;
use state::{GameModeState, SeparationState, State};
use std::path::{Path, PathBuf};

const USAGE: &str = "\
usage: gamebooster <command> [--dry-run] [--config <file>]

  boost      apply the tweaks enabled in the config (if already boosted: re-apply game priority and power throttling off)
  boost --watch  boost, then keep running until Ctrl+C: game priority when a game starts, dynamic lower_others
  auto       keep running: boost when a configured game starts, restore auto_restore_delay_secs after the last one exits
  restore    undo everything the last boost changed
  status     show whether a boost is active and what it changed
  ping       latency, jitter and loss to cloud regions near game servers, fastest first (changes nothing)
  ping --route [host]  per-hop latency and loss to host (default: the fastest region): router, ISP or far end?
  install    first-run setup (also what a double-click does): copy to C:\\gamebooster-rs, config, Mac power fix, logon task
  uninstall  undo everything install did (GPU table, BF6 Borderless, logon task, config edits, copied files)
  init       write the example config as gamebooster.toml next to gamebooster.exe (never overwrites)
  update     download the latest gamebooster.exe from GitHub and replace this one
  snapshot <out.json>                save a read-only snapshot of system state (capture/snapshot.ps1)
  compare <before.json> <after.json> print what changed between two snapshots (capture/compare.ps1)
  bench [seconds] [--label <name>]  record the running game's frame times with PresentMon (default 60 s)
  bench compare <a.json> <b.json>   compare two bench runs, with a run-to-run noise judgement
  --version  print the version and the git commit it was built from
  --dry-run  print what boost/restore would do, change nothing (no admin needed)
  --config   config file (default: gamebooster.toml next to gamebooster.exe)";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Services,
    Processes,
    Ram,
    LowerOthers,
    PowerPlan,
    GameMode,
    Accessibility,
    Dns,
    PrioritySeparation,
    Priority,
}

/// Boost order, as in the spec. Restore walks it backwards.
const BOOST_ORDER: [Step; 10] = [Step::Services, Step::Processes, Step::Ram, Step::LowerOthers, Step::PowerPlan, Step::GameMode, Step::Accessibility, Step::Dns, Step::PrioritySeparation, Step::Priority];

fn restore_order() -> impl Iterator<Item = Step> {
    BOOST_ORDER.into_iter().rev()
}

struct Args {
    command: String,
    dry_run: bool,
    config: Option<PathBuf>,
    /// File arguments of `snapshot` / `compare` / `bench compare`.
    files: Vec<PathBuf>,
    /// `bench` duration and label.
    seconds: Option<u64>,
    label: Option<String>,
    /// `ping --route [host]`.
    route: bool,
    host: Option<String>,
    /// `boost --watch`.
    watch: bool,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut it = args.into_iter();
    let (mut command, mut dry_run, mut config, mut files) = (None, false, None, Vec::new());
    let (mut seconds, mut label, mut route, mut host, mut watch) = (None, None, false, None, false);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--config" => config = Some(PathBuf::from(it.next().ok_or("--config needs a file")?)),
            "--label" => label = Some(it.next().ok_or("--label needs a name")?),
            "--route" if command.as_deref() == Some("ping") => route = true,
            "--watch" if command.as_deref() == Some("boost") => watch = true,
            _ if route && host.is_none() && !a.starts_with('-') => host = Some(a),
            "--version" | "-V" if command.is_none() => command = Some("version".to_string()),
            "boost" | "auto" | "restore" | "status" | "ping" | "init" | "update" | "snapshot" | "compare" | "bench" | "install" | "uninstall" if command.is_none() => command = Some(a),
            "compare" if command.as_deref() == Some("bench") && seconds.is_none() => command = Some("bench compare".into()),
            _ if command.as_deref() == Some("bench") && seconds.is_none() && a.parse::<u64>().is_ok() => seconds = a.parse().ok(),
            _ if matches!(command.as_deref(), Some("snapshot" | "compare" | "bench compare")) && !a.starts_with('-') => files.push(PathBuf::from(a)),
            _ => return Err(format!("unexpected argument `{a}`")),
        }
    }
    let command = command.ok_or("missing command")?;
    match (command.as_str(), files.len()) {
        ("snapshot", n) if n != 1 => Err("usage: gamebooster snapshot <out.json>".into()),
        ("compare", n) if n != 2 => Err("usage: gamebooster compare <before.json> <after.json>".into()),
        ("bench compare", n) if n != 2 => Err("usage: gamebooster bench compare <a.json> <b.json>".into()),
        ("bench", _) if seconds.is_some_and(|s| !(10..=3600).contains(&s)) => Err("bench seconds: use 10 to 3600".into()),
        (c, _) if c != "bench" && label.is_some() => Err("--label only goes with bench".into()),
        _ if watch && dry_run => Err("--watch can't be combined with --dry-run".into()),
        ("auto" | "install" | "uninstall", _) if dry_run => Err(format!("{command} can't be combined with --dry-run")),
        _ => Ok(Args { command, dry_run, config, files, seconds, label, route, host, watch }),
    }
}

/// Human-readable summary of what a state does (for --dry-run and status).
fn describe(st: &State, cfg: &Config) -> String {
    let list = |v: &[String]| if v.is_empty() { "nothing".to_string() } else { v.join(", ") };
    let mut out = format!("  services to stop (in order): {}\n", list(&st.services));
    out += &format!("  apps to close (relaunched on restore): {}\n", list(&st.processes));
    out += &format!("  RAM trim: {}\n", if cfg.ram_trim { "yes (not undoable)" } else { "off" });
    if cfg.lower_others || !st.lowered.is_empty() {
        let mut names: Vec<&str> = st.lowered.iter().map(|p| p.exe.as_str()).collect();
        names.sort_by_key(|n| n.to_ascii_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        let how = if cfg.dynamic_lowering() {
            "dynamic: busy ones to below normal while `boost --watch` runs (old class restored); restrained now".to_string()
        } else {
            format!("to {} (old class restored)", priority::lower_class_name(cfg.lower_others_class))
        };
        let how = if cfg.lower_others_io { format!("{how}, disk I/O priority low") } else { how };
        out += &format!("  lower other processes {how}: {} processes: {}\n", st.lowered.len(), if names.is_empty() { "none".into() } else { names.join(", ") });
    } else {
        out += "  lower other processes: off\n";
    }
    match &st.power {
        Some(p) => {
            out += &format!("  power plan: switch from {} to gamebooster ({}), a copy of High performance with:\n", p.previous_scheme, power::OUR_PLAN);
            for (_, _, v, meaning) in power::SETTINGS {
                out += &format!("      {meaning} (= {v})\n");
            }
            match p.brightness {
                Some((ac, dc)) => out += &format!("      {} (AC = {ac}, DC = {dc})\n", power::BRIGHTNESS_MEANING),
                None => out += &format!("      skipped (not on this machine) {}\n", power::BRIGHTNESS_MEANING),
            }
            for (setting, ac, dc) in &p.kept {
                out += &format!("      {} (AC = {ac}, DC = {dc})\n", power::kept_meaning(setting));
            }
            for meaning in power::absent_kept(&p.kept) {
                out += &format!("      skipped (not on this machine) {meaning}\n");
            }
            let min = cfg.min_processor_state.unwrap_or(100);
            out += &format!("      minimum processor state {min} % (= {min})\n");
            out += &format!("      {} (= {})\n", power::max_state_meaning(cfg.max_processor_state), cfg.max_processor_state);
            if let Some(mhz) = cfg.max_cpu_mhz {
                out += &format!("      {} (= {mhz})\n", power::max_mhz_meaning(mhz));
            }
        }
        None => out += "  power plan: off\n",
    }
    match &st.game_mode {
        Some(g) => out += &format!("  Game Mode: {} = {} -> 1\n", game_mode::VALUE, g.previous.map_or("(absent)".into(), |v| v.to_string())),
        None => out += "  Game Mode: off\n",
    }
    match &st.accessibility {
        Some(a) => {
            for (name, saved) in accessibility::entries(a) {
                out += &match saved {
                    Some(f) => format!("  accessibility: {name} shortcut off (flags {f:#x} restored)\n"),
                    None => format!("  accessibility: {name} left alone (feature on or shortcut already off)\n"),
                };
            }
        }
        None => out += "  accessibility shortcuts: off\n",
    }
    match &st.dns {
        Some(d) => out += &format!("  DNS ({}): {} -> {}\n", d.adapter, d.previous_static.as_deref().unwrap_or("from DHCP"), d.server),
        None if cfg.fastest_dns => out += "  DNS: keep current (no public resolver meaningfully faster)\n",
        None => out += "  DNS: off\n",
    }
    match &st.priority_separation {
        Some(s) => out += &format!("  {}: {} -> {:#x}\n", priority::SEPARATION_VALUE, s.previous.map_or("(absent)".into(), |v| format!("{v:#x}")), priority::SEPARATION),
        None => out += &format!("  {}: off\n", priority::SEPARATION_VALUE),
    }
    out += &format!("  game priority method: {}\n", priority_method_text(cfg));
    if cfg.process_priority() {
        for g in &cfg.games {
            out += &format!("  priority: {} -> {:?} while running\n", g.exe, g.priority);
        }
    }
    for e in &st.ifeo {
        out += &format!("  priority: {} -> {:?} via IFEO {} ({} -> {}{})\n", e.exe, e.priority, priority::CPU_PRIORITY_CLASS,
            e.previous.map_or("absent".into(), |v| v.to_string()), priority::cpu_priority_class(e.priority).unwrap_or(0),
            if e.key_existed { "" } else { "; IFEO key created, removed on restore" });
    }
    out += &format!("  game power throttling (EcoQoS): {}\n", if cfg.game_power_throttling_off { "off while running (handed back to Windows on restore)" } else { "left to Windows" });
    out
}

fn priority_method_text(cfg: &Config) -> &'static str {
    if cfg.process_priority() {
        "process (SetPriorityClass on the running game)"
    } else {
        "registry (IFEO CpuPriorityClass: Windows applies it when the game starts)"
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    // A double-click (no arguments, a console of its own) runs the first-run setup.
    #[cfg(windows)]
    let raw = if raw.is_empty() && mac::own_console() { vec!["install".to_string()] } else { raw };
    let args = match parse_args(raw) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    if args.command == "version" {
        println!("gamebooster {}", install::VERSION);
        return;
    }
    // Best effort: the exe that `update` replaced can only be deleted once it has exited.
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(install::old_path(&exe));
    }
    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn run(_: Args) -> Result<(), String> {
    Err("gamebooster only runs on Windows".into())
}

#[cfg(windows)]
fn run(args: Args) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    let config_path = args.config.unwrap_or_else(|| dir.join("gamebooster.toml"));
    let state_path = dir.join("gamebooster-state.json");
    let f = |i: usize| args.files[i].clone().into_os_string();
    let admin = unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() } != 0;
    match args.command.as_str() {
        "install" | "uninstall" if !admin => {
            println!("Asking for administrator rights (click Yes); the setup continues in a new window.");
            return finish(mac::elevate(&exe, &args.command));
        }
        "install" => return finish(mac::install(&exe)),
        "uninstall" => {
            if let Some(gone) = finish(mac::uninstall())? {
                mac::delete_after_exit(&gone);
            }
            return Ok(());
        }
        "init" => return install::init(&config_path),
        "update" => return install::update(&exe, state_path.exists()),
        "snapshot" => return install::powershell("snapshot", install::SNAPSHOT_PS1, &["-Out".into(), f(0)]),
        "compare" => return install::powershell("compare", install::COMPARE_PS1, &["-Before".into(), f(0), "-After".into(), f(1)]),
        "bench compare" => {
            print!("{}", bench::compare_text(&bench::load(&args.files[0])?, &bench::load(&args.files[1])?));
            return Ok(());
        }
        _ => {}
    }
    let cfg = Config::load(&config_path).map_err(|e| format!("{e}\n(run `gamebooster init` to create it, then edit it)"))?;
    if !matches!(args.command.as_str(), "status" | "ping") && (!args.dry_run || args.command == "bench") && !admin {
        return Err("boost, auto, restore and bench need administrator rights: right-click your terminal, \"Run as administrator\"".into());
    }
    match args.command.as_str() {
        "boost" => {
            boost(&cfg, &state_path, args.dry_run, args.watch, false)?;
            if args.watch { watch::run(&cfg, &state_path, false) } else { Ok(()) }
        }
        "auto" => watch::run(&cfg, &state_path, true),
        "restore" if args.dry_run => {
            restore(&cfg, &state_path, true)?;
            if let Some(h) = State::load(&held_path(&state_path))? {
                println!("Would also put back the IFEO entries `auto` holds: {}", h.ifeo.iter().map(|e| e.exe.as_str()).collect::<Vec<_>>().join(", "));
            }
            Ok(())
        }
        "restore" => {
            restore(&cfg, &state_path, false)?;
            release_ifeo(&state_path) // after a killed `auto`
        }
        "ping" if args.route => network::route(&cfg.ping_hosts, args.host.as_deref(), cfg.ping_samples),
        "ping" => Ok(network::ping(&cfg.ping_hosts, cfg.ping_samples)),
        "bench" => bench::run(&cfg, &dir, state_path.exists(), args.seconds.unwrap_or(60), args.label.as_deref()),
        _ => status(&cfg, &state_path),
    }
}

/// `install` / `uninstall`: show an error, then keep a window we own open until Enter.
#[cfg(windows)]
fn finish<T>(r: Result<T, String>) -> Result<T, String> {
    if let Err(e) = &r {
        println!("error: {e}");
    }
    mac::pause();
    if r.is_err() {
        std::process::exit(1);
    }
    r
}

/// Print a failed step and carry on: each tweak is independent.
#[cfg(windows)]
fn warn<T>(what: &str, r: Result<T, String>) -> Option<T> {
    r.map_err(|e| println!("  {what} failed: {e}")).ok()
}

/// What happens to a configured game that isn't running yet. In registry mode Windows sets the
/// priority at game start; only power throttling off is left for the watcher or a second boost.
#[cfg(windows)]
fn no_game_hint(cfg: &Config, watch: bool) -> &'static str {
    match (cfg.process_priority(), cfg.game_power_throttling_off, watch) {
        (false, false, _) => "Windows applies the priority when it starts; nothing else to do",
        (false, true, true) => "Windows applies the priority when it starts; the watcher turns power throttling off",
        (false, true, false) => "Windows applies the priority when it starts; run `gamebooster boost` again once it runs to turn power throttling off (or use `boost --watch`)",
        (true, _, true) => "the watcher sets it when one starts",
        (true, _, false) => "start it, then run `gamebooster boost` again (or use `boost --watch`)",
    }
}

#[cfg(windows)]
fn boost(cfg: &Config, state_path: &Path, dry_run: bool, watch: bool, auto: bool) -> Result<(), String> {
    if State::load(state_path)?.is_some() {
        println!("Already boosted (state in {}). Only re-applying game priority and power throttling.", state_path.display());
        if !dry_run && priority::apply(&cfg.games, false, cfg.game_power_throttling_off, cfg.process_priority()) == 0 {
            println!("  no configured game is running: {}", no_game_hint(cfg, watch));
        }
        return Ok(());
    }
    // Read-only: work out everything restore will need.
    let st = State {
        boosted_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()),
        services: services::plan(&cfg.services),
        processes: processes::plan(&cfg.close),
        power: if cfg.power_plan { warn("reading the power plan", power::plan()) } else { None },
        game_mode: if cfg.game_mode {
            warn("reading Game Mode", game_mode::read()).map(|previous| GameModeState { previous })
        } else {
            None
        },
        accessibility: if cfg.accessibility_shortcuts_off { warn("reading accessibility shortcuts", accessibility::plan()) } else { None },
        dns: if cfg.fastest_dns { warn("timing DNS", network::plan()).flatten() } else { None },
        lowered: if cfg.lower_others && !cfg.dynamic_lowering() { priority::plan_lower(cfg) } else { Vec::new() },
        ifeo: if cfg.process_priority() || held_path(state_path).exists() { Vec::new() } else { priority::plan_ifeo(&cfg.games) },
        priority_separation: if cfg.priority_separation {
            warn("reading Win32PrioritySeparation", reg_dword(windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE, priority::PRIORITY_CONTROL, priority::SEPARATION_VALUE))
                .map(|previous| SeparationState { previous })
        } else {
            None
        },
    };
    if dry_run {
        println!("Boost would do (dry run, nothing changed):\n{}", describe(&st, cfg));
        return Ok(());
    }
    st.save(state_path)?; // abort before any change if this fails
    println!("Saved restore state to {}", state_path.display());
    for step in BOOST_ORDER {
        println!("{step:?}");
        match step {
            Step::Services => st.services.iter().for_each(|s| { warn("stopping a service", services::stop(s)); }),
            Step::Processes => processes::close_all(&cfg.close),
            Step::Ram if cfg.ram_trim => ram::trim(cfg),
            Step::Ram => println!("  off"),
            Step::LowerOthers if cfg.dynamic_lowering() => println!("  dynamic: `boost --watch` restrains busy background processes while it runs"),
            Step::LowerOthers if cfg.lower_others => priority::set_others(&st.lowered, Some(cfg.lower_others_class)),
            Step::LowerOthers => println!("  off"),
            Step::PowerPlan => {
                if let Some(p) = &st.power {
                    warn("power plan", power::boost(p, cfg.min_processor_state.unwrap_or(100), cfg.max_processor_state, cfg.max_cpu_mhz));
                }
            }
            Step::GameMode => {
                if st.game_mode.is_some() {
                    warn("Game Mode", game_mode::write(Some(1)));
                }
            }
            Step::Accessibility => {
                if let Some(a) = &st.accessibility {
                    warn("accessibility shortcuts", accessibility::boost(a));
                }
            }
            Step::Dns => match &st.dns {
                Some(d) => { warn("DNS", network::boost(d)); }
                None if cfg.fastest_dns => println!("  kept current server (fastest)"),
                None => println!("  off"),
            },
            Step::PrioritySeparation if st.priority_separation.is_some() => {
                warn("Win32PrioritySeparation", priority::write_separation(Some(priority::SEPARATION)));
            }
            Step::PrioritySeparation => println!("  off"),
            Step::Priority => {
                priority::write_ifeo(&st.ifeo);
                if priority::apply(&cfg.games, false, cfg.game_power_throttling_off, cfg.process_priority()) == 0 && !cfg.games.is_empty() {
                    println!("  no configured game is running: {}", no_game_hint(cfg, watch));
                }
            }
        }
    }
    if auto {
        println!("Boosted. auto restores {} s after the last game exits.", cfg.auto_restore_delay_secs);
    } else {
        println!("Boosted. Run `gamebooster restore` when you're done playing.");
    }
    Ok(())
}

/// `auto` with the registry method holds the games' IFEO entries for its whole run (Windows reads
/// them only at game start, so a boost after the game started is too late). They are journalled in
/// this file (a `State` with only `ifeo`) so `restore` undoes them if auto was killed; while it
/// exists, boost (auto's or a manual one) skips the IFEO step.
fn held_path(state_path: &Path) -> PathBuf {
    state_path.with_file_name("gamebooster-auto-ifeo.json")
}

/// `auto` start: journal, then write, each game's IFEO entry. A file left by a killed auto and a
/// manual boost's entries keep their old values (taken over from the state file, so the boost's
/// restore leaves them to auto).
#[cfg(windows)]
fn hold_ifeo(cfg: &Config, state_path: &Path) -> Result<(), String> {
    let path = held_path(state_path);
    let mut held = State::load(&path)?.unwrap_or_default();
    let mut st = State::load(state_path)?.filter(|st| !st.ifeo.is_empty());
    if let Some(st) = &mut st {
        priority::take_over(&mut held.ifeo, std::mem::take(&mut st.ifeo));
    }
    priority::take_over(&mut held.ifeo, priority::plan_ifeo(&cfg.games));
    held.save(&path)?; // before the state file loses its entries: a crash in between leaves both (restoring twice is harmless)
    if let Some(st) = st {
        st.save(state_path)?;
    }
    println!("Game priority (registry): holding the IFEO entries while auto runs ({})", path.display());
    priority::write_ifeo(&held.ifeo);
    Ok(())
}

/// Put back the IFEO entries `auto` held (auto exit, or `restore` after a killed auto). If a boost is
/// still active they are handed to its state file instead, so its `restore` puts them back.
#[cfg(windows)]
fn release_ifeo(state_path: &Path) -> Result<(), String> {
    let path = held_path(state_path);
    let Some(held) = State::load(&path)? else { return Ok(()) };
    match State::load(state_path)? {
        Some(mut st) => {
            priority::take_over(&mut st.ifeo, held.ifeo);
            st.save(state_path)?;
            println!("IFEO entries auto held: left to the active boost; `gamebooster restore` puts them back");
        }
        None => {
            println!("Putting back the IFEO entries auto held:");
            priority::restore_ifeo(&held.ifeo)?; // on failure the file stays for another `restore`
        }
    }
    std::fs::remove_file(&path).map_err(|e| format!("can't delete {}: {e}", path.display()))
}

#[cfg(windows)]
fn restore(cfg: &Config, state_path: &Path, dry_run: bool) -> Result<(), String> {
    let Some(st) = State::load(state_path)? else {
        println!("Not boosted (no {}). Resetting game priority and power throttling only.", state_path.display());
        if !dry_run {
            priority::apply(&cfg.games, true, cfg.game_power_throttling_off, cfg.process_priority());
        }
        return Ok(());
    };
    if dry_run {
        println!("Restore would undo (dry run, nothing changed):\n{}", describe(&st, cfg));
        return Ok(());
    }
    let mut ok = true;
    let mut check = |what: &str, r: Result<(), String>| ok &= warn(what, r).is_some();
    for step in restore_order() {
        println!("{step:?}");
        match step {
            Step::Priority => {
                check("IFEO", priority::restore_ifeo(&st.ifeo));
                if priority::apply(&cfg.games, true, cfg.game_power_throttling_off, cfg.process_priority()) == 0 && st.ifeo.is_empty() {
                    println!("  nothing to undo");
                }
            }
            Step::GameMode => {
                if let Some(g) = &st.game_mode {
                    check("Game Mode", game_mode::write(g.previous));
                }
            }
            Step::Accessibility => match &st.accessibility {
                Some(a) if accessibility::entries(a).iter().any(|(_, saved)| saved.is_some()) => check("accessibility shortcuts", accessibility::restore(a)),
                _ => println!("  nothing to undo"),
            },
            Step::Dns => match &st.dns {
                Some(d) => check("DNS", network::restore(d)),
                None => println!("  nothing to undo"),
            },
            Step::PrioritySeparation => match &st.priority_separation {
                Some(s) => check("Win32PrioritySeparation", priority::write_separation(s.previous)),
                None => println!("  nothing to undo"),
            },
            Step::PowerPlan => {
                if let Some(p) = &st.power {
                    check("power plan", power::restore(p));
                }
            }
            Step::Ram => println!("  nothing to undo"),
            Step::LowerOthers if st.lowered.is_empty() => println!("  nothing to undo"),
            Step::LowerOthers => priority::set_others(&st.lowered, None),
            Step::Processes => processes::relaunch(&st.processes),
            Step::Services => st.services.iter().rev().for_each(|s| check("starting a service", services::start(s))),
        }
    }
    if ok {
        std::fs::remove_file(state_path).map_err(|e| format!("can't delete {}: {e}", state_path.display()))?;
        println!("Restored.");
    } else {
        println!("Some steps failed; the state file is kept, so you can run `gamebooster restore` again.");
    }
    Ok(())
}

#[cfg(windows)]
fn status(cfg: &Config, state_path: &Path) -> Result<(), String> {
    match State::load(state_path)? {
        Some(st) => println!("Boosted (since unix time {}). Restore will undo:\n{}", st.boosted_at, describe(&st, cfg)),
        None => println!("Not boosted."),
    }
    if let Some(h) = State::load(&held_path(state_path))? {
        println!("auto holds IFEO entries (put back when auto exits, or by `gamebooster restore`): {}", h.ifeo.iter().map(|e| e.exe.as_str()).collect::<Vec<_>>().join(", "));
    }
    println!("Game priority method: {}", priority_method_text(cfg));
    let active = power::active_scheme()?;
    println!("Active power plan: {active}{}", if active == power::OUR_PLAN { " (gamebooster)" } else { "" });
    match power::max_state(&active) {
        Ok((ac, dc)) => {
            println!("Maximum processor state: {ac} % (AC), {dc} % (DC)");
            let name = power::active_scheme_name().unwrap_or_else(|_| active.clone());
            if let Some(msg) = power::max_state_warning(&name, active == power::OUR_PLAN, cfg.max_processor_state, ac, dc) {
                println!("{msg}");
            }
        }
        Err(e) => println!("Maximum processor state: couldn't read ({e})"),
    }
    println!("{}: {}", game_mode::VALUE, game_mode::read()?.map_or("(absent, Windows default = on)".into(), |v| v.to_string()));
    // Read only: HAGS needs a reboot and driver support, so boost doesn't touch it (docs/tweaks/optimizer.md).
    let hags = reg_dword(windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE, HAGS_KEY, "HwSchMode")?;
    println!("GPU scheduling (HAGS): {}", hags_text(hags));
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    let overlay = reg_dword(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\Dwm", "OverlayTestMode")?;
    println!("MPO: {}", mpo_text(overlay, reg_dword(HKEY_LOCAL_MACHINE, HAGS_KEY, "DisableOverlays")?));
    let dvr = r"Software\Microsoft\Windows\CurrentVersion\GameDVR";
    println!("{}", dvr_text(reg_dword(HKEY_CURRENT_USER, r"System\GameConfigStore", "GameDVR_Enabled")?,
        reg_dword(HKEY_CURRENT_USER, dvr, "AppCaptureEnabled")?, reg_dword(HKEY_CURRENT_USER, dvr, "HistoricalCaptureEnabled")?));
    // Read only: Fault Tolerant Heap shims an exe after repeated crashes (docs/oss-sweep-2026/README.md).
    if reg_dword(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\FTH", "Enabled")? == Some(0) {
        println!("Fault Tolerant Heap: off");
    } else {
        match fth_flagged(cfg, &reg_value_names(HKEY_LOCAL_MACHINE, FTH_STATE_KEY)?).as_slice() {
            [] => println!("Fault Tolerant Heap: no configured game flagged"),
            exes => println!("warning: Fault Tolerant Heap has flagged {} (Windows slows its heap after repeated crashes). \
                Fix: delete its value under HKLM\\{FTH_STATE_KEY}, or run: rundll32.exe fthsvc.dll,FthSysprepSpecialize", exes.join(", ")),
        }
    }
    mac::status();
    match power::clock_under_load_mhz() {
        Ok(mhz) => {
            println!("CPU clock under load: {mhz:.0} MHz");
            if power::looks_locked(mhz) {
                println!("  warning: the CPU looks locked at its minimum clock, likely BD PROCHOT. See docs/tweaks/prochot.md");
            }
        }
        Err(e) => println!("CPU clock under load: couldn't measure ({e})"),
    }
    Ok(())
}

const FTH_STATE_KEY: &str = r"SOFTWARE\Microsoft\FTH\State";

/// FTH names each flagged app by its full exe path; return the file names that are configured games.
fn fth_flagged(cfg: &Config, value_names: &[String]) -> Vec<String> {
    value_names.iter().map(|n| n.rsplit('\\').next().unwrap_or(n).to_string()).filter(|exe| cfg.is_game(exe)).collect()
}

const HAGS_KEY: &str = r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers";

/// `HwSchMode`: 2 = on, 1 = off, absent = Windows default.
fn hags_text(v: Option<u32>) -> String {
    match v {
        Some(2) => "on".into(),
        Some(1) => "off".into(),
        Some(n) => format!("unknown value {n}"),
        None => "not set (Windows default)".into(),
    }
}

/// Multiplane overlays (read only, docs/tweaks/optimizer.md): `OverlayTestMode = 5` or
/// `DisableOverlays = 1` turns them off; anything else leaves them on.
fn mpo_text(overlay_test_mode: Option<u32>, disable_overlays: Option<u32>) -> String {
    match (overlay_test_mode, disable_overlays) {
        (None, None) => "on (default)".into(),
        (Some(5), _) => "off (OverlayTestMode=5)".into(),
        (_, Some(1)) => "off (DisableOverlays=1)".into(),
        (o, d) => format!("on (OverlayTestMode={}, DisableOverlays={})", show(o), show(d)),
    }
}

fn show(v: Option<u32>) -> String {
    v.map_or("absent".into(), |v| v.to_string())
}

/// Game DVR background recording (read only): on when `HistoricalCaptureEnabled = 1` and neither
/// `GameDVR_Enabled` nor `AppCaptureEnabled` is 0 (absent = on for those two, off for the first).
fn dvr_text(game_dvr: Option<u32>, app_capture: Option<u32>, historical: Option<u32>) -> String {
    let on = historical == Some(1) && game_dvr != Some(0) && app_capture != Some(0);
    let mut s = format!("Game DVR background recording: {} (GameDVR_Enabled={}, AppCaptureEnabled={}, HistoricalCaptureEnabled={})",
        if on { "on" } else { "off" }, show(game_dvr), show(app_capture), show(historical));
    if on {
        s += "\n  warning: Windows keeps recording the last minutes of play (GPU encoder and disk). Turn it off: \
              Settings > Gaming > Captures > \"Record in the background while I'm playing a game\" = Off";
    }
    s
}

/// A REG_DWORD value, or None if it doesn't exist.
#[cfg(windows)]
fn reg_dword(root: windows_sys::Win32::System::Registry::HKEY, key: &str, value: &str) -> Result<Option<u32>, String> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{RegGetValueW, RRF_RT_REG_DWORD};
    let (mut data, mut size) = (0u32, 4u32);
    let err = unsafe {
        RegGetValueW(root, wide(key).as_ptr(), wide(value).as_ptr(), RRF_RT_REG_DWORD,
            std::ptr::null_mut(), &mut data as *mut u32 as *mut _, &mut size)
    };
    match err {
        ERROR_SUCCESS => Ok(Some(data)),
        ERROR_FILE_NOT_FOUND => Ok(None),
        e => Err(format!("can't read {key}\\{value} (error {e})")),
    }
}

/// Value names under a key; a missing key gives an empty list.
#[cfg(windows)]
fn reg_value_names(root: windows_sys::Win32::System::Registry::HKEY, key: &str) -> Result<Vec<String>, String> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{RegCloseKey, RegEnumValueW, RegOpenKeyExW, KEY_READ};
    let mut h = std::ptr::null_mut();
    match unsafe { RegOpenKeyExW(root, wide(key).as_ptr(), 0, KEY_READ, &mut h) } {
        ERROR_SUCCESS => {}
        ERROR_FILE_NOT_FOUND => return Ok(vec![]),
        e => return Err(format!("can't open {key} (error {e})")),
    }
    let (mut names, mut buf) = (vec![], vec![0u16; 16384]); // 16383 = max value name length
    let err = loop {
        let mut len = buf.len() as u32;
        let e = unsafe {
            RegEnumValueW(h, names.len() as u32, buf.as_mut_ptr(), &mut len, std::ptr::null(),
                std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut())
        };
        if e != ERROR_SUCCESS { break e; }
        names.push(String::from_utf16_lossy(&buf[..len as usize]));
    };
    unsafe { RegCloseKey(h) };
    if err == ERROR_NO_MORE_ITEMS { Ok(names) } else { Err(format!("can't list {key} (error {err})")) }
}

/// Write `Some(v)` as a REG_DWORD (creating the key), or delete the value for None (absent is fine).
#[cfg(windows)]
fn reg_set_dword(root: windows_sys::Win32::System::Registry::HKEY, key: &str, value: &str, v: Option<u32>) -> Result<(), String> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{RegDeleteKeyValueW, RegSetKeyValueW, REG_DWORD};
    let err = unsafe {
        match v {
            Some(v) => RegSetKeyValueW(root, wide(key).as_ptr(), wide(value).as_ptr(), REG_DWORD, &v as *const u32 as *const _, 4),
            None => match RegDeleteKeyValueW(root, wide(key).as_ptr(), wide(value).as_ptr()) {
                ERROR_FILE_NOT_FOUND => ERROR_SUCCESS,
                e => e,
            },
        }
    };
    if err == ERROR_SUCCESS { Ok(()) } else { Err(format!("can't write {key}\\{value} (error {err})")) }
}

#[cfg(windows)]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

#[cfg(windows)]
fn from_wide(buf: &[u16]) -> String {
    let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..n])
}

/// # Safety
/// `p` must point to a NUL-terminated UTF-16 string.
#[cfg(windows)]
unsafe fn from_wide_ptr(p: *const u16) -> String {
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(p, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_is_reverse_of_boost() {
        let mut r: Vec<Step> = restore_order().collect();
        r.reverse();
        assert_eq!(r, BOOST_ORDER);
        assert_eq!(restore_order().next(), Some(Step::Priority));
        assert_eq!(restore_order().last(), Some(Step::Services));
    }

    #[test]
    fn fth_matches_file_name() {
        let cfg = Config::parse("[[game]]\nexe = \"bf6.exe\"").unwrap();
        let names = [r"C:\Games\BF6\BF6.EXE".to_string(), r"C:\Windows\notepad.exe".into(), "bf6.exe".into()];
        assert_eq!(fth_flagged(&cfg, &names), ["BF6.EXE", "bf6.exe"]);
        assert!(fth_flagged(&cfg, &[]).is_empty());
    }

    #[test]
    fn hags_values() {
        assert_eq!(hags_text(Some(2)), "on");
        assert_eq!(hags_text(Some(1)), "off");
        assert_eq!(hags_text(None), "not set (Windows default)");
        assert_eq!(hags_text(Some(7)), "unknown value 7");
    }

    #[test]
    fn mpo_and_dvr_values() {
        assert_eq!(mpo_text(None, None), "on (default)");
        assert_eq!(mpo_text(Some(5), None), "off (OverlayTestMode=5)");
        assert_eq!(mpo_text(None, Some(1)), "off (DisableOverlays=1)");
        assert_eq!(mpo_text(Some(0), None), "on (OverlayTestMode=0, DisableOverlays=absent)");
        assert!(dvr_text(Some(0), Some(0), Some(0)).starts_with("Game DVR background recording: off (GameDVR_Enabled=0"));
        assert!(!dvr_text(None, None, None).contains("warning"));
        assert!(dvr_text(None, None, Some(1)).contains("on (") && dvr_text(None, None, Some(1)).contains("warning"));
        assert!(!dvr_text(Some(0), None, Some(1)).contains("warning"));
    }

    #[test]
    fn args() {
        let a = parse_args(["boost", "--dry-run", "--config", "x.toml"].map(String::from)).unwrap();
        assert_eq!((a.command.as_str(), a.dry_run, a.config), ("boost", true, Some(PathBuf::from("x.toml"))));
        assert!(parse_args(["boost", "restore"].map(String::from)).is_err());
        assert!(parse_args(["--dry-run"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--config"].map(String::from)).is_err());
        assert_eq!(parse_args(["--version"].map(String::from)).unwrap().command, "version");
        assert_eq!(parse_args(["update"].map(String::from)).unwrap().command, "update");
        let s = parse_args(["snapshot", "a.json"].map(String::from)).unwrap();
        assert_eq!((s.command.as_str(), s.files), ("snapshot", vec![PathBuf::from("a.json")]));
        assert_eq!(parse_args(["compare", "a.json", "b.json"].map(String::from)).unwrap().files.len(), 2);
        assert!(parse_args(["snapshot"].map(String::from)).is_err());
        assert!(parse_args(["compare", "a.json"].map(String::from)).is_err());
        assert!(parse_args(["boost", "a.json"].map(String::from)).is_err());
        assert!(parse_args(["init", "x"].map(String::from)).is_err());
        let b = parse_args(["bench", "30", "--label", "power-off"].map(String::from)).unwrap();
        assert_eq!((b.command.as_str(), b.seconds, b.label.as_deref()), ("bench", Some(30), Some("power-off")));
        assert_eq!(parse_args(["bench"].map(String::from)).unwrap().seconds, None);
        let c = parse_args(["bench", "compare", "a.json", "b.json"].map(String::from)).unwrap();
        assert_eq!((c.command.as_str(), c.files.len()), ("bench compare", 2));
        assert!(parse_args(["bench", "compare", "a.json"].map(String::from)).is_err());
        assert!(parse_args(["bench", "5"].map(String::from)).is_err());
        assert!(parse_args(["bench", "30", "40"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--label", "x"].map(String::from)).is_err());
        let p = parse_args(["ping", "--route", "example.com"].map(String::from)).unwrap();
        assert_eq!((p.route, p.host.as_deref()), (true, Some("example.com")));
        assert_eq!(parse_args(["ping", "--route"].map(String::from)).unwrap().host, None);
        assert!(parse_args(["ping", "example.com"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--route"].map(String::from)).is_err());
        assert!(parse_args(["ping", "--route", "a", "b"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--watch"].map(String::from)).unwrap().watch);
        assert!(parse_args(["restore", "--watch"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--watch", "--dry-run"].map(String::from)).is_err());
        assert_eq!(parse_args(["auto"].map(String::from)).unwrap().command, "auto");
        assert!(parse_args(["auto", "--dry-run"].map(String::from)).is_err());
        assert!(parse_args(["auto", "--watch"].map(String::from)).is_err());
        assert_eq!(parse_args(["install"].map(String::from)).unwrap().command, "install");
        assert_eq!(parse_args(["uninstall"].map(String::from)).unwrap().command, "uninstall");
        assert!(parse_args(["uninstall", "--dry-run"].map(String::from)).is_err());
        assert!(parse_args(["install", "x"].map(String::from)).is_err());
        assert!(parse_args(Vec::<String>::new()).is_err());
    }

    #[test]
    fn describe_mentions_everything() {
        let cfg = Config::parse(include_str!("../gamebooster.example.toml")).unwrap();
        let st = State {
            services: vec!["WSearch".into()],
            power: Some(state::PowerState { previous_scheme: power::BALANCED.into(), brightness: None, kept: vec![(power::KEEP[0].1.into(), 1, 0)] }),
            accessibility: Some(state::AccessibilityState { sticky_keys: Some(0x1fe), ..Default::default() }),
            game_mode: Some(GameModeState { previous: None }),
            priority_separation: Some(SeparationState { previous: None }),
            ..Default::default()
        };
        let d = describe(&st, &cfg);
        assert!(d.contains("game priority method: process"), "{d}");
        let reg = Config::parse("game_priority_method = \"registry\"\n[[game]]\nexe = \"bf6.exe\"").unwrap();
        let ifeo = State { ifeo: vec![priority::Ifeo { exe: "bf6.exe".into(), priority: config::Priority::High, key_existed: false, perf_existed: false, previous: None }], ..Default::default() };
        let r = describe(&ifeo, &reg);
        for needle in ["game priority method: registry", "bf6.exe -> High via IFEO CpuPriorityClass (absent -> 3; IFEO key created, removed on restore)"] {
            assert!(r.contains(needle), "missing {needle} in:\n{r}");
        }
        assert!(!r.contains("-> High while running"), "{r}");
        for needle in ["WSearch", power::BALANCED, "(absent)", "bf6.exe -> High", "minimum processor state 100 %", "maximum processor state 100 %", "apps to close (relaunched on restore): nothing", "DNS: off", "lower other processes: off", "keep current lid close action (AC = 1, DC = 0)", "skipped (not on this machine) keep current power button action", "skipped (not on this machine) keep current display brightness", "Sticky Keys shortcut off (flags 0x1fe restored)", "Filter Keys left alone", "Win32PrioritySeparation: (absent) -> 0x2a"] {
            assert!(d.contains(needle), "missing {needle} in:\n{d}");
        }
    }
}
