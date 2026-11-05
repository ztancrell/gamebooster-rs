# gamebooster

A small Windows game booster, written from scratch in Rust for playing Battlefield 6 under Boot
Camp on a Mac. `boost` applies a handful of well-known tweaks; `restore` undoes all of them. Every
tweak is explained, with the manual commands to try it yourself and how to measure it, so it
doubles as a way to learn what these "booster" apps actually do.

What a boost does, in this order (restore goes in reverse):

1. **Stop services** you list (running dependents first; a service with a running dependent you
   didn't list is skipped). Restore starts them again.
2. **Close apps** you list. Restore relaunches them.
3. **RAM trim** of all other processes (optional, off by default, not undoable).
4. **Power plan**: activates a `gamebooster` plan copied from High performance (CPU minimum 100 %,
   active cooling, PCIe power saving off, AMD PowerPlay max where present, no sleep; optionally
   maximum 99 % to turn Intel Turbo Boost off, and a lower minimum for less heat). Restore
   reactivates your old plan and deletes ours.
5. **Game Mode** on (`HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled = 1`). Restore writes back
   the old value.
6. **Game priority** (High by default) for configured games that are running. Restore sets Normal.

Details, defaults, manual commands and how to measure each one: [docs/tweaks/](docs/tweaks/README.md).
The behaviour follows [docs/spec.md](docs/spec.md).

## Build

Needs Rust. From Linux (cross-compile; needs the `x86_64-pc-windows-gnu` target and mingw-w64):

```sh
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/gamebooster.exe
```

On Windows: `cargo build --release` gives `target\release\gamebooster.exe`.

`cargo test` runs the platform-independent unit tests (config, state file, order, parsers) on any
OS. The binary itself only does anything on Windows.

## Usage

Put `gamebooster.exe` and your config (`gamebooster.toml`) in one folder, for example
`C:\gamebooster`. Open a terminal **as administrator** (right-click > Run as administrator) from
your normal Windows account, then:

```bat
cd C:\gamebooster
gamebooster boost --dry-run
gamebooster boost
gamebooster status
gamebooster restore
```

| Command | What it does |
|---|---|
| `boost` | Applies the enabled tweaks. If already boosted, only re-applies game priority (use this after starting the game). |
| `restore` | Undoes the last boost from the state file, then deletes the state file. If not boosted, only resets game priority to Normal. |
| `status` | Prints the saved state (if boosted), the active power plan GUID, `AutoGameModeEnabled`, and the CPU clock measured with one busy thread for about a second (warns below 1000 MHz: likely stuck at 800 MHz by [BD PROCHOT](docs/tweaks/prochot.md)). |
| `--dry-run` | With `boost` or `restore`: print the plan, change nothing. |
| `--config <file>` | Use another config file. |

Boost and restore without `--dry-run` refuse to run without administrator rights.

## Config

Copy [gamebooster.example.toml](gamebooster.example.toml) to `gamebooster.toml` next to
`gamebooster.exe`. Keys:

| Key | Default | Meaning |
|---|---|---|
| `power_plan` | `true` | Use the gamebooster power plan. |
| `max_processor_state` | `100` | Maximum processor state (%) in that plan, 5 to 100. `99` turns Intel Turbo Boost off (less heat and throttling, lower peak clocks); see [power plan](docs/tweaks/power-plan.md#optional-turbo-boost-off). |
| `min_processor_state` | unset (= `100`) | Minimum processor state (%) in that plan, 0 to 100, not above `max_processor_state`. `5` with `max_processor_state = 99` is the low-power combo for a Mac stuck at 800 MHz; see [power plan](docs/tweaks/power-plan.md#optional-low-minimum-state). |
| `game_mode` | `true` | Make sure Game Mode is on. |
| `ram_trim` | `false` | Trim other processes' RAM once. |
| `services` | `[]` | Service key names to stop (`sc query` names, e.g. `"WSearch"`). The example file has a commented-out suggested list from SGB, pruned; see [services](docs/tweaks/services.md#suggested-list-from-sgb-pruned). |
| `close` | `[]` | Process image names to close (e.g. `"OneDrive.exe"`). |
| `[[game]]` `exe`, `priority` | example: `bf6.exe`, `high` | Games to raise; priority is `realtime`, `high`, `above_normal`, `normal`, `below_normal` or `low`. |

Unknown keys and typos are errors. Protected processes (Windows core, `explorer.exe`,
`taskmgr.exe`, Steam, the EA app, Epic, Defender, Symantec, gamebooster itself) are refused in
`close` and as games. The BF6 executable name `bf6.exe` is the expected one but **unverified**:
check Task Manager > Details while the game runs.

## Safety and restore

- Before the first change, boost writes everything restore needs to
  `gamebooster-state.json` next to `gamebooster.exe` (previous power plan, the services it stops,
  the full paths of the apps it closes, the old Game Mode value). If that write fails, nothing is
  changed. The file's presence means "boosted"; boosting again doesn't overwrite it.
- After a crash, a power cut or a reboot, just run `gamebooster restore`. Restore is idempotent:
  running services, running apps and an already-deleted plan are fine. If starting a service,
  restoring the power plan or writing Game Mode fails, the state
  file is kept so you can run restore again; it is deleted only after a clean restore.
- Nothing is permanent: service startup types aren't touched, so a reboot restarts services too.
  Power plan by hand: `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced) and
  `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`.
- Each tweak is independent: one failing is printed and the others still run.

## Limitations

- Game priority is set only on games already running when you boost. Start the game first, or run
  `boost` again once it's running. There is no background watcher.
- Closed apps are relaunched without their original command-line arguments.
- The power plan values follow the spec; the sleep timeout is 0 (never), an assumption.
- `AutoGameModeEnabled` is per user: run elevated from your own account.
- Not implemented (on purpose): GPU overclock, monitor presets, clipboard clearing, Windows Update
  changes, network acceleration, tray icon cleanup, killing `explorer.exe`.
- Tested only by unit tests and a cross-compile; not yet run on real hardware. First-run checklist:
  [docs/hardware-test.md](docs/hardware-test.md).
