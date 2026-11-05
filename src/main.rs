//! gamebooster: boost Windows for a game session, then restore everything.
//!
//! Boost applies the tweaks in [`BOOST_ORDER`]; restore undoes them in reverse. Everything restore
//! needs is saved to the state file before the first change. Each tweak module's doc comment
//! explains what it changes and why; docs/tweaks/ has the same per tweak, with manual commands.
#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

mod config;
mod game_mode;
mod power;
mod priority;
mod processes;
mod ram;
mod services;
mod state;

use config::Config;
use state::{GameModeState, State};
use std::path::{Path, PathBuf};

const USAGE: &str = "\
usage: gamebooster <boost|restore|status> [--dry-run] [--config <file>]

  boost     apply the tweaks enabled in the config (if already boosted: re-apply game priority)
  restore   undo everything the last boost changed
  status    show whether a boost is active and what it changed
  --dry-run print what boost/restore would do, change nothing (no admin needed)
  --config  config file (default: gamebooster.toml next to gamebooster.exe)";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Services,
    Processes,
    Ram,
    PowerPlan,
    GameMode,
    Priority,
}

/// Boost order, as in the spec. Restore walks it backwards.
const BOOST_ORDER: [Step; 6] = [Step::Services, Step::Processes, Step::Ram, Step::PowerPlan, Step::GameMode, Step::Priority];

fn restore_order() -> impl Iterator<Item = Step> {
    BOOST_ORDER.into_iter().rev()
}

struct Args {
    command: String,
    dry_run: bool,
    config: Option<PathBuf>,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut it = args.into_iter();
    let (mut command, mut dry_run, mut config) = (None, false, None);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--config" => config = Some(PathBuf::from(it.next().ok_or("--config needs a file")?)),
            "boost" | "restore" | "status" if command.is_none() => command = Some(a),
            _ => return Err(format!("unexpected argument `{a}`")),
        }
    }
    Ok(Args { command: command.ok_or("missing command")?, dry_run, config })
}

/// Human-readable summary of what a state does (for --dry-run and status).
fn describe(st: &State, cfg: &Config) -> String {
    let list = |v: &[String]| if v.is_empty() { "nothing".to_string() } else { v.join(", ") };
    let mut out = format!("  services to stop (in order): {}\n", list(&st.services));
    out += &format!("  apps to close (relaunched on restore): {}\n", list(&st.processes));
    out += &format!("  RAM trim: {}\n", if cfg.ram_trim { "yes (not undoable)" } else { "off" });
    match &st.power {
        Some(p) => {
            out += &format!("  power plan: switch from {} to gamebooster ({}), a copy of High performance with:\n", p.previous_scheme, power::OUR_PLAN);
            for (_, _, v, meaning) in power::SETTINGS {
                out += &format!("      {meaning} (= {v})\n");
            }
        }
        None => out += "  power plan: off\n",
    }
    match &st.game_mode {
        Some(g) => out += &format!("  Game Mode: {} = {} -> 1\n", game_mode::VALUE, g.previous.map_or("(absent)".into(), |v| v.to_string())),
        None => out += "  Game Mode: off\n",
    }
    for g in &cfg.games {
        out += &format!("  priority: {} -> {:?} while running\n", g.exe, g.priority);
    }
    out
}

fn main() {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
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
    let dir = std::env::current_exe().map_err(|e| e.to_string())?.parent().map(Path::to_path_buf).unwrap_or_default();
    let config_path = args.config.unwrap_or_else(|| dir.join("gamebooster.toml"));
    let state_path = dir.join("gamebooster-state.json");
    let cfg = Config::load(&config_path).map_err(|e| {
        format!("{e}\n(copy gamebooster.example.toml to {} and edit it)", dir.join("gamebooster.toml").display())
    })?;
    let admin = unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() } != 0;
    if args.command != "status" && !args.dry_run && !admin {
        return Err("boost and restore need administrator rights: right-click your terminal, \"Run as administrator\"".into());
    }
    match args.command.as_str() {
        "boost" => boost(&cfg, &state_path, args.dry_run),
        "restore" => restore(&cfg, &state_path, args.dry_run),
        _ => status(&cfg, &state_path),
    }
}

/// Print a failed step and carry on: each tweak is independent.
#[cfg(windows)]
fn warn<T>(what: &str, r: Result<T, String>) -> Option<T> {
    r.map_err(|e| println!("  {what} failed: {e}")).ok()
}

#[cfg(windows)]
fn boost(cfg: &Config, state_path: &Path, dry_run: bool) -> Result<(), String> {
    if State::load(state_path)?.is_some() {
        println!("Already boosted (state in {}). Only re-applying game priority.", state_path.display());
        if !dry_run && priority::apply(&cfg.games, false) == 0 {
            println!("  no configured game is running");
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
            Step::PowerPlan => {
                if let Some(p) = &st.power {
                    warn("power plan", power::boost(p));
                }
            }
            Step::GameMode => {
                if st.game_mode.is_some() {
                    warn("Game Mode", game_mode::write(Some(1)));
                }
            }
            Step::Priority => {
                if priority::apply(&cfg.games, false) == 0 && !cfg.games.is_empty() {
                    println!("  no configured game is running: start it, then run `gamebooster boost` again");
                }
            }
        }
    }
    println!("Boosted. Run `gamebooster restore` when you're done playing.");
    Ok(())
}

#[cfg(windows)]
fn restore(cfg: &Config, state_path: &Path, dry_run: bool) -> Result<(), String> {
    let Some(st) = State::load(state_path)? else {
        println!("Not boosted (no {}). Resetting game priority only.", state_path.display());
        if !dry_run {
            priority::apply(&cfg.games, true);
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
            Step::Priority => { priority::apply(&cfg.games, true); }
            Step::GameMode => {
                if let Some(g) = &st.game_mode {
                    check("Game Mode", game_mode::write(g.previous));
                }
            }
            Step::PowerPlan => {
                if let Some(p) = &st.power {
                    check("power plan", power::restore(p));
                }
            }
            Step::Ram => println!("  nothing to undo"),
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
    let active = power::active_scheme()?;
    println!("Active power plan: {active}{}", if active == power::OUR_PLAN { " (gamebooster)" } else { "" });
    println!("{}: {}", game_mode::VALUE, game_mode::read()?.map_or("(absent, Windows default = on)".into(), |v| v.to_string()));
    Ok(())
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
    fn args() {
        let a = parse_args(["boost", "--dry-run", "--config", "x.toml"].map(String::from)).unwrap();
        assert_eq!((a.command.as_str(), a.dry_run, a.config), ("boost", true, Some(PathBuf::from("x.toml"))));
        assert!(parse_args(["boost", "restore"].map(String::from)).is_err());
        assert!(parse_args(["--dry-run"].map(String::from)).is_err());
        assert!(parse_args(["boost", "--config"].map(String::from)).is_err());
    }

    #[test]
    fn describe_mentions_everything() {
        let cfg = Config::parse(include_str!("../gamebooster.example.toml")).unwrap();
        let st = State {
            services: vec!["WSearch".into()],
            power: Some(state::PowerState { previous_scheme: power::BALANCED.into(), brightness: None }),
            game_mode: Some(GameModeState { previous: None }),
            ..Default::default()
        };
        let d = describe(&st, &cfg);
        for needle in ["WSearch", power::BALANCED, "(absent)", "bf6.exe -> High", "apps to close (relaunched on restore): nothing"] {
            assert!(d.contains(needle), "missing {needle} in:\n{d}");
        }
    }
}
