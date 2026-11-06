# gamebooster-rs

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
   and a built-in safe list. Restore puts back each one's old priority if it still runs. With
   `lower_others_mode = "dynamic"` boost lowers nothing here; `boost --watch` restrains only busy
   ones while it runs (see [Watch mode](#watch-mode)).
5. **Power plan**: activates a `gamebooster` plan copied from High performance (CPU minimum 100 %,
   active cooling, PCIe power saving off, AMD PowerPlay max where present, no sleep; your
   brightness and lid / power button / sleep button actions are copied over unchanged; optionally
   maximum 99 % to turn Intel Turbo Boost off, and a lower minimum for less heat). Restore
   reactivates your old plan and deletes ours.
6. **Game Mode** on (`HKCU\Software\Microsoft\GameBar\AutoGameModeEnabled = 1`). Restore writes back
   the old value.
7. **Accessibility shortcuts off**: the Sticky Keys (Shift five times), Filter Keys (hold Right
   Shift 8 s) and Toggle Keys (hold Num Lock 5 s) hotkeys, so they can't pop up mid-game. A
   feature you have switched on is left alone. Session only; restore writes back the old flags.
8. **Fastest DNS** (optional, off by default): if Cloudflare, Google or Quad9 answers DNS lookups
   clearly faster than your current server, sets it on the active adapter (IPv4). Restore puts back
   the old static list or DHCP.
9. **Game priority** (High by default) for configured games that are running. Restore sets Normal.
   With `game_priority_method = "registry"` (for anti-cheats that block it, like BF6's) it writes the
   game's IFEO `CpuPriorityClass` instead, which Windows applies at the next game start; restore
   puts the registry back exactly. `auto` writes it when `auto` starts and keeps it until it exits.

Separately, `gamebooster ping` measures latency, jitter and loss to cloud regions near game
servers, and `gamebooster ping --route` checks each hop on the way (your router, your ISP, the far
end). Both change nothing. See [network](docs/tweaks/network.md).

Separately, `gamebooster bench` records the game's frame times with Intel PresentMon and
`bench compare` tells a real difference from run-to-run noise. See [Measuring](#measuring).
Status: not working yet on the target Mac (BF6's anti-cheat appears to block the capture); see
[docs/measuring.md](docs/measuring.md).

MacBook Pro 2019 stuck at a low clock in Windows? See [docs/macbook-pro-2019-slow-cpu.md](docs/macbook-pro-2019-slow-cpu.md).
Details, defaults, manual commands and how to measure each one: [docs/tweaks/](docs/tweaks/README.md).
The behaviour follows [docs/spec.md](docs/spec.md).
What Smart Game Booster itself does, feature by feature, with verdicts: [docs/sgb/](docs/sgb/README.md).
The same for PGWARE GameBoost (registry-only, nothing worth adopting): [docs/pgware/](docs/pgware/README.md).
The same for Smart PC Utilities' PC Game Booster (temporary Game Mode, loads the WinRing0 driver for sensors): [docs/smartpcutilities-gb/](docs/smartpcutilities-gb/README.md).
The same for Razer Cortex: [docs/razer-cortex/](docs/razer-cortex/README.md).
The same for Hone (server-driven tweak catalogue, encrypted engine, temporary Game Mode): [docs/hone/](docs/hone/README.md).
The same for ISLC, Intelligent Standby List Cleaner (standby purge and timer resolution, both placebo or harmful on Windows 10/11): [docs/islc/](docs/islc/README.md).
The same for ExitLag (relay "ping reducer" with two kernel packet drivers; helps only on a bad ISP route): [docs/exitlag/](docs/exitlag/README.md).
The same for Process Lasso (ProBalance dynamic priority restraint, read from the binary; the basis of `lower_others_mode = "dynamic"`): [docs/process-lasso/](docs/process-lasso/README.md).
The same for JetBoost (BlueSprig, a rebranded IObit booster from 2012; stops a Windows 8-era service list, hard-kills apps, forces min CPU 100 % even on battery; nothing new to adopt): [docs/jetboost/](docs/jetboost/README.md).
The same for Wise Game Booster (permanent XP-era registry tweaks, kills programs, stops services with a resume list; uploads your game list): [docs/wise-game-booster/](docs/wise-game-booster/README.md).
The same for Tier1Suite (Tier1Settings; 43-entry registry/power catalogue with honest notes and real revert, LocalSystem broker that re-applies, no driver; adoption: a Game DVR check in `status`): [docs/tier1suite/](docs/tier1suite/README.md).
Open-source tools and guides, read from source, with adoption items:
[GameShift](docs/gameshift/README.md) (per-session apply/revert with a crash journal; mostly folklore tweaks, several of which need a reboot to work),
[valleyofdoom PC-Tuning](docs/pc-tuning/README.md) (measure-first guide; keeps idle states on, prefers per-process timer resolution, silent on Nagle/MMCSS),
[Chris Titus WinUtil](docs/winutil/README.md) (gaming-relevant tweaks only; its Ultimate Performance undo resets all power plans).

## Install and update

Every push to `main` that changes the program (not docs-only pushes) is tested and built by GitHub Actions
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
gamebooster boost --watch
gamebooster auto
gamebooster status
gamebooster restore
gamebooster ping
gamebooster ping --route
```

| Command | What it does |
|---|---|
| `boost` | Applies the enabled tweaks. If already boosted, only re-applies game priority and power throttling off (use this after starting the game). |
| `boost --watch` | Boosts (or, if already boosted, re-applies game priority), then keeps running in the console until Ctrl+C or the window closes; see [Watch mode](#watch-mode). Not with `--dry-run`. |
| `auto` | Keeps running in the console: boosts when a configured `[[game]]` starts and restores `auto_restore_delay_secs` (30) after the last one exits; see [Automatic boost](#automatic-boost). Admin. Not with `--dry-run`. |
| `restore` | Undoes the last boost from the state file, then deletes the state file. Also puts back IFEO entries a killed `auto` left (`gamebooster-auto-ifeo.json`). If not boosted, only resets game priority to Normal (and power throttling to Windows). |
| `status` | Prints the saved state (if boosted), the active power plan GUID, its *Maximum processor state* on AC and DC (warns below 100 %, which turns Turbo Boost off, with the `powercfg` fix; only a note when it is gamebooster's own plan at your `max_processor_state`), `AutoGameModeEnabled`, whether hardware-accelerated GPU scheduling is on (`GPU scheduling (HAGS): on | off | not set`, see [optimizer](docs/tweaks/optimizer.md)), and the CPU clock measured with one busy thread for about a second (warns below 1000 MHz: likely stuck at 800 MHz by [BD PROCHOT](docs/tweaks/prochot.md)). |
| `ping` | Times `ping_samples` (10) TCP connects to each of `ping_hosts`, AWS regional endpoints by default, and prints median, p90, jitter and loss per region, fastest first; marks a region with any loss or jitter above 10 ms. An approximation of distance to game servers, not BF6's own servers; BF6's region is chosen in the game. Changes nothing, no admin needed. See [network](docs/tweaks/network.md#ping). |
| `ping --route [host]` | Per-hop path check like `pathping`: finds the hops to `host` (default: the fastest region) with ICMP, pings each hop `ping_samples` times and prints median and loss, hop 1 labelled as your router (Wi-Fi or Ethernet), plus a hint where persisting loss starts. Changes nothing, no admin needed. See [network](docs/tweaks/network.md#route-check). |
| `init` | Writes the built-in example config as `gamebooster.toml` next to `gamebooster.exe` (or `--config <file>`). Never overwrites an existing file. |
| `update` | Downloads the latest `gamebooster.exe` from the `latest` release and replaces this one; prints old -> new version. No admin needed if you can write to the exe's folder. |
| `snapshot <out.json>` | Saves a read-only snapshot of system state (runs the built-in copy of [capture/snapshot.ps1](capture/snapshot.ps1) with Windows PowerShell). See [capture](capture/README.md). |
| `compare <before.json> <after.json>` | Prints what changed between two snapshots (built-in [capture/compare.ps1](capture/compare.ps1)). |
| `bench [seconds] [--label <name>]` | Records the first running `[[game]]` for 60 s (10 to 3600) with PresentMon and saves `bench\<date>_<time>_<label>.json` (stats, per-10-s FPS, boosted or not, config) and the raw CSV next to gamebooster.exe. Admin. Downloads PresentMon once into `tools\`. See [Measuring](#measuring). Not working yet on the target Mac (see Measuring). |
| `bench compare <a.json> <b.json>` | Prints two bench runs side by side, the config keys that differ, and whether the avg FPS and 1 % low differences are real or within run-to-run noise. |
| `--version` | Prints the version and the git commit it was built from. |
| `--dry-run` | With `boost` or `restore`: print the plan, change nothing. |
| `--config <file>` | Use another config file. |

Boost and restore without `--dry-run`, and bench, refuse to run without administrator rights.

## Measuring

Optional: tweaks are judged by feel on the target Mac, because BF6's anti-cheat (assumed) blocks
the capture there. `bench` works on games that allow ETW trace sessions; see
[docs/measuring.md](docs/measuring.md).

With the game running in a match, in an administrator terminal:

```bat
gamebooster bench --label normal
gamebooster boost
gamebooster bench --label boosted
gamebooster bench compare bench\<normal run>.json bench\<boosted run>.json
```

`bench` prints frames, average FPS, median frame time, 1 % and 0.1 % low FPS, stutters (frames
over twice the median) and FPS per 10 s. `bench compare` runs Welch's t-test on the per-10-s
values and says "real difference" or "within run-to-run noise" (~95 %). Do a warm-up run, at least
3 runs per setting alternating normal and boosted, and change one config key at a time:
[docs/measuring.md](docs/measuring.md), which also has the PresentMon (Intel, MIT) third-party
notice. PresentMon 2.6.0 is downloaded from its official GitHub release on first use and only run
if its SHA-256 matches the pinned one.

## Config

`gamebooster init` writes [gamebooster.example.toml](gamebooster.example.toml) (built into the exe)
as `gamebooster.toml` next to `gamebooster.exe`. Keys:

| Key | Default | Meaning |
|---|---|---|
| `power_plan` | `true` | Use the gamebooster power plan. |
| `max_processor_state` | `100` | Maximum processor state (%) in that plan, 5 to 100. `99` turns Intel Turbo Boost off (less heat and throttling, lower peak clocks); see [power plan](docs/tweaks/power-plan.md#optional-turbo-boost-off). |
| `min_processor_state` | unset (= `100`) | Minimum processor state (%) in that plan, 0 to 100, not above `max_processor_state`. `5` with `max_processor_state = 99` is the low-power combo for a Mac stuck at 800 MHz; see [power plan](docs/tweaks/power-plan.md#optional-low-minimum-state). |
| `max_cpu_mhz` | unset (no cap) | Maximum processor frequency (MHz) in that plan, 800 to 6000; `0` or unset = no cap. Turbo stays on up to the cap; with `max_processor_state` below 100 too, the lower limit wins. Try 2600 to 3000 on a Mac that hits BD PROCHOT; see [power plan](docs/tweaks/power-plan.md#optional-frequency-cap). |
| `game_mode` | `true` | Make sure Game Mode is on. |
| `accessibility_shortcuts_off` | `true` | Turn off the Sticky / Filter / Toggle Keys hotkeys while boosted; see [accessibility shortcuts](docs/tweaks/accessibility-shortcuts.md). |
| `game_power_throttling_off` | `true` | Opt each running `[[game]]` out of Windows power throttling (EcoQoS) while boosted; restore hands it back to Windows. See [game priority](docs/tweaks/priority.md#power-throttling-off). |
| `ram_trim` | `false` | Trim other processes' RAM once. |
| `game_priority_method` | `"process"` | How game priority is set: `"process"` (on the running game) or `"registry"` (IFEO `PerfOptions\CpuPriorityClass`, applied by Windows at the next game start; for anti-cheats that block the process way; no `realtime`). See [game priority](docs/tweaks/priority.md#registry-method). |
| `lower_others` | `false` | Lower your other Normal-priority processes during the boost; see [game priority](docs/tweaks/priority.md#lower-other-processes). |
| `lower_others_mode` | `"once"` | `"once"`: boost lowers them all. `"dynamic"`: boost lowers nothing; `boost --watch` (or `auto`) restrains only busy ones, ProBalance-style (Below normal only). See [Watch mode](#watch-mode). |
| `lower_others_class` | `"below_normal"` | `"below_normal"` or `"idle"`. Idle can starve voice chat and streams. Once mode only; dynamic mode always uses Below normal. |
| `lower_others_exclude` | `[]` | Extra image names to leave alone (case-insensitive), on top of the built-in list. |
| `lower_others_exclude_publishers` | `[]` | Extra publishers to leave alone: case-insensitive substrings of the exe's CompanyName (file Properties > Details), on top of the built-in list. |
| `fastest_dns` | `false` | On boost, time DNS lookups against the current server and 1.1.1.1, 8.8.8.8, 9.9.9.9; switch the active adapter's IPv4 DNS to a public one only if it is more than 10 ms and 20 % faster. See [network](docs/tweaks/network.md#fastest-dns). |
| `auto_restore_delay_secs` | `30` | `gamebooster auto`: seconds to wait after the last game exits before restoring; a game starting within that time keeps the boost. `0` = restore at once. |
| `ping_hosts` | 12 AWS regional endpoints | Hosts `gamebooster ping` measures (TCP port 443). |
| `ping_samples` | `10` | Probes per host for `ping` and per hop for `ping --route` (1 to 100). |
| `services` | `[]` | Service key names to stop (`sc query` names, e.g. `"WSearch"`). The example file has a commented-out suggested list from SGB, pruned; see [services](docs/tweaks/services.md#suggested-list-from-sgb-pruned). |
| `close` | `[]` | Process image names to close (e.g. `"OneDrive.exe"`). |
| `[[game]]` `exe`, `priority` | example: `bf6.exe`, `high` | Games to raise; priority is `realtime`, `high`, `above_normal`, `normal`, `below_normal` or `low`. |

Unknown keys and typos are errors. Protected processes (Windows core, `explorer.exe`,
`taskmgr.exe`, Steam, the EA app, Epic, Defender, Symantec, gamebooster itself) are refused in
`close` and as games. The BF6 executable name `bf6.exe` is the expected one but **unverified**:
check Task Manager > Details while the game runs.

## Watch mode

`gamebooster boost --watch` boosts as usual, then stays running in the console, checking once a
second:

- **Games.** When a configured `[[game]]` starts, it sets its priority and power throttling off, so
  you don't have to run `boost` again after launching BF6. When the game exits, both end with it.
- **Dynamic lower others** (`lower_others = true`, `lower_others_mode = "dynamic"`). Like Process
  Lasso's ProBalance: one of your Normal-priority background apps (same filter as `lower_others`,
  and never the window in front or its child processes) goes to Below normal when the whole
  machine is busy and that app has used a big share of it for 0.9 s; it goes back to Normal once it
  is quiet and has been restrained for at least 4.2 s, or at once when you switch to it. The
  thresholds come from Process Lasso's table by logical CPU count (16 threads: machine >= 10 %,
  app >= 5 %, release under 2 %); see [game priority](docs/tweaks/priority.md#dynamic-mode).

It prints one line per change (`restrained chrome.exe (pid 4120, 9 % CPU) to below normal`,
`released chrome.exe (pid 4120) after 6 s: quiet again`) and a status line at most every 10 s when
something changed (`[2:30] CPU 34 %, 1 restrained (chrome.exe), game running`). Ctrl+C or closing
the window releases what the watcher restrained and prints "Still boosted: run `gamebooster
restore` when you're done playing.": the rest of the boost stays until you run `restore`. Each
restraint is recorded in the state file as it happens, so if the watcher is killed, `restore`
still undoes it. If you run `restore` in another window, the watcher notices and stops.

## Automatic boost

`gamebooster auto` (as administrator; leave the window open) runs the same watcher without
boosting first. When a configured `[[game]]` starts it runs the normal `boost` (`bf6.exe started:
boosting`) and then watches like `boost --watch`. When the last game has exited it waits
`auto_restore_delay_secs` (default 30 s; `last game exited: restoring in 30 s`); a game starting
again within that time keeps the boost (a restart or a launcher hand-off). Then it runs the normal
`restore` (`restored; waiting for a game`) and waits for the next game.

- It only restores boosts it made. If you were already boosted when `auto` started (or boost by
  hand while it waits), it watches that boost like `boost --watch` and leaves it for you to restore.
- If you run `restore` in another window while a game is running, it doesn't boost again until all
  games have exited and one starts.
- Ctrl+C or closing the window while it is boosted restores; while it waits, it just exits. Windows
  allows about 5 s after the window is closed: if restore doesn't finish in time, the state file
  stays, so run `gamebooster restore`.
- If a restore step fails, the state file is kept and `auto` treats it as a boost it didn't make:
  run `gamebooster restore`.
- With `game_priority_method = "registry"`, `auto` writes the games' IFEO entries when it starts,
  before any game, because Windows reads them only when the game starts (a boost after that is too
  late). It keeps them across its boosts and restores and puts them back exactly when it exits.
  They are recorded in `gamebooster-auto-ifeo.json`, so if `auto` is killed, `gamebooster restore`
  puts them back (run while `auto` is still open, it ends that hold as well). If a boost is still
  active when `auto` exits (one it didn't make), the entries go to that boost's state file and its
  `restore` puts them back.

## Safety and restore

- Before the first change, boost writes everything restore needs to
  `gamebooster-state.json` next to `gamebooster.exe` (previous power plan, the services it stops,
  the full paths of the apps it closes, the processes it lowers with their old priority, the old
  Game Mode value, the old accessibility shortcut flags, the old DNS setting, the old IFEO entries
  with the registry priority method). If that write fails, nothing is
  changed. The file's presence means "boosted"; boosting again doesn't overwrite it.
- `auto` with the registry priority method records the old IFEO entries in
  `gamebooster-auto-ifeo.json` (same folder) before writing them; `restore` puts them back and
  deletes it. While that file exists, `boost` skips the IFEO step (`auto` already wrote them).
- After a crash, a power cut or a reboot, just run `gamebooster restore`. Restore is idempotent:
  running services, running apps and an already-deleted plan are fine. If starting a service,
  restoring the power plan, writing Game Mode, the accessibility flags or DNS fails, the state
  file is kept so you can run restore again; it is deleted only after a clean restore.
- Nothing is permanent: service startup types aren't touched, so a reboot restarts services too.
  Power plan by hand: `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced) and
  `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`.
  DNS by hand: `netsh interface ip set dns "<adapter>" dhcp`.
- Each tweak is independent: one failing is printed and the others still run.

## Limitations

- Game priority (process method) is set only on games already running when you boost. Start the game first, or run
  `boost` again once it's running, or use `boost --watch` or `auto`. The watcher and `auto` only
  run while their console is open; there is no service or logon task.
- Closed apps are relaunched without their original command-line arguments.
- The power plan values follow the spec; the sleep timeout is 0 (never), an assumption.
- `AutoGameModeEnabled` is per user: run elevated from your own account.
- Not implemented (on purpose): GPU overclock, monitor presets, clipboard clearing, Windows Update
  changes, network acceleration (relay servers; `ping` and fastest DNS are the substitute), tray icon cleanup, killing `explorer.exe`, SGB's other optimizer registry tweaks (see [optimizer](docs/tweaks/optimizer.md)).
- Run on one real machine so far (the target Mac: Boot Camp, BF6); checklist and results in
  [docs/hardware-test.md](docs/hardware-test.md).
