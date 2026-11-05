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
4. **Lower other processes** (optional, off by default): your own apps that run at Normal priority
   go to Below normal (or Idle), except the games, launchers, voice chat, anti-cheat, the shell
   and a built-in safe list. Restore puts back each one's old priority if it still runs.
5. **Power plan**: activates a `gamebooster` plan copied from High performance (CPU minimum 100 %,
   active cooling, PCIe power saving off, AMD PowerPlay max where present, no sleep; optionally
   maximum 99 % to turn Intel Turbo Boost off, and a lower minimum for less heat). Restore
   reactivates your old plan and deletes ours.
6. **Game Mode** on (`HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled = 1`). Restore writes back
   the old value.
7. **Fastest DNS** (optional, off by default): if Cloudflare, Google or Quad9 answers DNS lookups
   clearly faster than your current server, sets it on the active adapter (IPv4). Restore puts back
   the old static list or DHCP.
8. **Game priority** (High by default) for configured games that are running. Restore sets Normal.

Separately, `gamebooster ping` measures latency to cloud regions near game servers and changes
nothing. See [network](docs/tweaks/network.md).

Details, defaults, manual commands and how to measure each one: [docs/tweaks/](docs/tweaks/README.md).
The behaviour follows [docs/spec.md](docs/spec.md).

## Install and update

Every push to `main` is tested and built by GitHub Actions
([.github/workflows/release.yml](.github/workflows/release.yml)) and published as `gamebooster.exe`
on the rolling [`latest` release](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest).
That one exe is all Windows needs: the example config and the capture scripts are built in.

First install:

1. Download `gamebooster.exe` from the
   [Releases page](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest) and put it in
   `C:\gamebooster`. Windows SmartScreen may warn the first time you run it: **More info** >
   **Run anyway**. If PowerShell still refuses it, run `Unblock-File C:\gamebooster\gamebooster.exe`.
2. In a terminal: `cd C:\gamebooster`, then `gamebooster init`. It writes `gamebooster.toml` next to
   the exe (it never overwrites an existing one); edit it in Notepad.

Later: `gamebooster update` downloads the latest `gamebooster.exe` with `curl.exe` (Windows 10 1803
or later), checks it is a Windows program that runs, and swaps it in; it prints the old and new
version (`gamebooster --version`: crate version and git commit). The replaced exe is kept as
`gamebooster.exe.old` until the next command deletes it. Updating while boosted is fine: the state
file stays compatible, so `restore` works with the new version.

## Build

Needs Rust. From Linux (cross-compile; needs the `x86_64-pc-windows-gnu` target and mingw-w64):

```sh
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/gamebooster.exe
```

On Windows: `cargo build --release` gives `target\release\gamebooster.exe`.

`cargo test` runs the platform-independent unit tests (config, state file, order, parsers) on any
OS. The binary itself only does anything on Windows. `build.rs` embeds the git short hash (or
`GITHUB_SHA` in CI) for `--version`.

## Usage

Put `gamebooster.exe` and your config (`gamebooster.toml`) in one folder, for example
`C:\gamebooster` (see [Install and update](#install-and-update)). Open a terminal **as administrator** (right-click > Run as administrator) from
your normal Windows account, then:

```bat
cd C:\gamebooster
gamebooster boost --dry-run
gamebooster boost
gamebooster status
gamebooster restore
gamebooster ping
```

| Command | What it does |
|---|---|
| `boost` | Applies the enabled tweaks. If already boosted, only re-applies game priority (use this after starting the game). |
| `restore` | Undoes the last boost from the state file, then deletes the state file. If not boosted, only resets game priority to Normal. |
| `status` | Prints the saved state (if boosted), the active power plan GUID, `AutoGameModeEnabled`, whether hardware-accelerated GPU scheduling is on (`GPU scheduling (HAGS): on | off | not set`, see [optimizer](docs/tweaks/optimizer.md)), and the CPU clock measured with one busy thread for about a second (warns below 1000 MHz: likely stuck at 800 MHz by [BD PROCHOT](docs/tweaks/prochot.md)). |
| `ping` | Measures latency (median of 5 TCP connects) to `ping_hosts`, AWS regional endpoints by default, and prints them fastest first. An approximation of distance to game servers, not BF6's own servers; BF6's region is chosen in the game. Changes nothing, no admin needed. See [network](docs/tweaks/network.md#ping). |
| `init` | Writes the built-in example config as `gamebooster.toml` next to `gamebooster.exe` (or `--config <file>`). Never overwrites an existing file. |
| `update` | Downloads the latest `gamebooster.exe` from the `latest` release and replaces this one; prints old -> new version. No admin needed if you can write to the exe's folder. |
| `snapshot <out.json>` | Saves a read-only snapshot of system state (runs the built-in copy of [capture/snapshot.ps1](capture/snapshot.ps1) with Windows PowerShell). See [capture](capture/README.md). |
| `compare <before.json> <after.json>` | Prints what changed between two snapshots (built-in [capture/compare.ps1](capture/compare.ps1)). |
| `--version` | Prints the version and the git commit it was built from. |
| `--dry-run` | With `boost` or `restore`: print the plan, change nothing. |
| `--config <file>` | Use another config file. |

Boost and restore without `--dry-run` refuse to run without administrator rights.

## Config

`gamebooster init` writes [gamebooster.example.toml](gamebooster.example.toml) (built into the exe)
as `gamebooster.toml` next to `gamebooster.exe`. Keys:

| Key | Default | Meaning |
|---|---|---|
| `power_plan` | `true` | Use the gamebooster power plan. |
| `max_processor_state` | `100` | Maximum processor state (%) in that plan, 5 to 100. `99` turns Intel Turbo Boost off (less heat and throttling, lower peak clocks); see [power plan](docs/tweaks/power-plan.md#optional-turbo-boost-off). |
| `min_processor_state` | unset (= `100`) | Minimum processor state (%) in that plan, 0 to 100, not above `max_processor_state`. `5` with `max_processor_state = 99` is the low-power combo for a Mac stuck at 800 MHz; see [power plan](docs/tweaks/power-plan.md#optional-low-minimum-state). |
| `game_mode` | `true` | Make sure Game Mode is on. |
| `ram_trim` | `false` | Trim other processes' RAM once. |
| `lower_others` | `false` | Lower your other Normal-priority processes during the boost; see [game priority](docs/tweaks/priority.md#lower-other-processes). |
| `lower_others_class` | `"below_normal"` | `"below_normal"` or `"idle"`. Idle can starve voice chat and streams. |
| `lower_others_exclude` | `[]` | Extra image names to leave alone (case-insensitive), on top of the built-in list. |
| `fastest_dns` | `false` | On boost, time DNS lookups against the current server and 1.1.1.1, 8.8.8.8, 9.9.9.9; switch the active adapter's IPv4 DNS to a public one only if it is more than 10 ms and 20 % faster. See [network](docs/tweaks/network.md#fastest-dns). |
| `ping_hosts` | 12 AWS regional endpoints | Hosts `gamebooster ping` measures (TCP port 443). |
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
  the full paths of the apps it closes, the processes it lowers with their old priority, the old
  Game Mode value, the old DNS setting). If that write fails, nothing is
  changed. The file's presence means "boosted"; boosting again doesn't overwrite it.
- After a crash, a power cut or a reboot, just run `gamebooster restore`. Restore is idempotent:
  running services, running apps and an already-deleted plan are fine. If starting a service,
  restoring the power plan, writing Game Mode or restoring DNS fails, the state
  file is kept so you can run restore again; it is deleted only after a clean restore.
- Nothing is permanent: service startup types aren't touched, so a reboot restarts services too.
  Power plan by hand: `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced) and
  `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`.
  DNS by hand: `netsh interface ip set dns "<adapter>" dhcp`.
- Each tweak is independent: one failing is printed and the others still run.

## Limitations

- Game priority is set only on games already running when you boost. Start the game first, or run
  `boost` again once it's running. There is no background watcher.
- Closed apps are relaunched without their original command-line arguments.
- The power plan values follow the spec; the sleep timeout is 0 (never), an assumption.
- `AutoGameModeEnabled` is per user: run elevated from your own account.
- Not implemented (on purpose): GPU overclock, monitor presets, clipboard clearing, Windows Update
  changes, network acceleration (relay servers; `ping` and fastest DNS are the substitute), tray icon cleanup, killing `explorer.exe`, SGB's other optimizer registry tweaks (see [optimizer](docs/tweaks/optimizer.md)).
- Tested only by unit tests and a cross-compile; not yet run on real hardware. First-run checklist:
  [docs/hardware-test.md](docs/hardware-test.md).
