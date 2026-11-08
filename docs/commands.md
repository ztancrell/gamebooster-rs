# Commands

Put `gamebooster.exe` and your config (`gamebooster.toml`) in one folder, for example
`C:\gamebooster` (see [Install and update](install.md)). Open a terminal **as administrator** (right-click > Run as administrator) from
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
| `boost --watch` | Boosts (or, if already boosted, re-applies game priority), then keeps running in the console until Ctrl+C or the window closes; see [Watch mode](watch.md). Not with `--dry-run`. |
| `auto` | Keeps running in the console: boosts when a configured `[[game]]` starts and restores `auto_restore_delay_secs` (30) after the last one exits; see [Automatic boost](auto.md). Admin. Not with `--dry-run`. |
| `restore` | Undoes the last boost from the state file, then deletes the state file. Also puts back IFEO entries a killed `auto` left (`gamebooster-auto-ifeo.json`). If not boosted, only resets game priority to Normal (and power throttling to Windows). |
| `status` | Prints the saved state (if boosted), the active power plan GUID, its *Maximum processor state* on AC and DC (warns below 100 %, which turns Turbo Boost off, with the `powercfg` fix; only a note when it is gamebooster's own plan at your `max_processor_state`), `AutoGameModeEnabled`, whether hardware-accelerated GPU scheduling is on (`GPU scheduling (HAGS): on | off | not set`, see [optimizer](tweaks/optimizer.md)), whether multiplane overlays are on (`MPO: on (default) | off (OverlayTestMode=5) | off (DisableOverlays=1)`), whether Game DVR background recording is on (warns with the Settings switch to turn it off), whether Windows' Fault Tolerant Heap has flagged a configured game (`Fault Tolerant Heap: no configured game flagged | off`, or a warning naming the exe with the fix: delete its value under `HKLM\SOFTWARE\Microsoft\FTH\State` or run `rundll32.exe fthsvc.dll,FthSysprepSpecialize`), and the CPU clock measured with one busy thread for about a second (warns below 1000 MHz: likely stuck at 800 MHz by [BD PROCHOT](tweaks/prochot.md)). |
| `ping` | Times `ping_samples` (10) TCP connects to each of `ping_hosts`, AWS regional endpoints by default, and prints median, p90, jitter and loss per region, fastest first; marks a region with any loss or jitter above 10 ms. An approximation of distance to game servers, not BF6's own servers; BF6's region is chosen in the game. Changes nothing, no admin needed. See [network](tweaks/network.md#ping). |
| `ping --route [host]` | Per-hop path check like `pathping`: finds the hops to `host` (default: the fastest region) with ICMP, pings each hop `ping_samples` times and prints median and loss, hop 1 labelled as your router (Wi-Fi or Ethernet), plus a hint where persisting loss starts. Changes nothing, no admin needed. See [network](tweaks/network.md#route-check). |
| `init` | Writes the built-in example config as `gamebooster.toml` next to `gamebooster.exe` (or `--config <file>`). Never overwrites an existing file. |
| `update` | Downloads the latest `gamebooster.exe` from the `latest` release and replaces this one; prints old -> new version. No admin needed if you can write to the exe's folder. |
| `snapshot <out.json>` | Saves a read-only snapshot of system state (runs the built-in copy of [capture/snapshot.ps1](../capture/snapshot.ps1) with Windows PowerShell). See [capture](../capture/README.md). |
| `compare <before.json> <after.json>` | Prints what changed between two snapshots (built-in [capture/compare.ps1](../capture/compare.ps1)). |
| `bench [seconds] [--label <name>]` | Records the first running `[[game]]` for 60 s (10 to 3600) with PresentMon and saves `bench\<date>_<time>_<label>.json` (stats, per-10-s FPS, boosted or not, config) and the raw CSV next to gamebooster.exe. Admin. Downloads PresentMon once into `tools\`. See [Measuring](measuring.md). Not working yet on the target Mac (see Measuring). |
| `bench compare <a.json> <b.json>` | Prints two bench runs side by side, the config keys that differ, and whether the avg FPS and 1 % low differences are real or within run-to-run noise. |
| `--version` | Prints the version and the git commit it was built from. |
| `--dry-run` | With `boost` or `restore`: print the plan, change nothing. |
| `--config <file>` | Use another config file. |

Boost and restore without `--dry-run`, and bench, refuse to run without administrator rights.

## What boost does, in order

What a boost does, in this order (restore goes in reverse):

1. **Stop services** you list (running dependents first; a service with a running dependent you
   didn't list is skipped). Restore starts them again.
2. **Close apps** you list. Restore relaunches them.
3. **RAM trim** of all other processes (optional, off by default, not undoable).
4. **Lower other processes** (optional, off by default): your own apps that run at Normal priority
   go to Below normal (or Idle), except the games, launchers, voice chat, anti-cheat, the shell
   and a built-in safe list. Their disk (I/O) priority goes to Low too (`lower_others_io`, default
   on). Restore puts back each one's old priority and I/O priority if it still runs. With
   `lower_others_mode = "dynamic"` boost lowers nothing here; `boost --watch` restrains only busy
   ones while it runs (see [Watch mode](watch.md)).
5. **Power plan**: activates a `gamebooster` plan copied from High performance (CPU minimum 100 %,
   active cooling, PCIe power saving off, USB selective suspend off, AMD PowerPlay max where
   present, no sleep; your
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
9. **Win32PrioritySeparation** `0x2A` (short, fixed quantum, 3:1 foreground boost;
   `priority_separation`, default on, try by feel). Takes effect at once; restore writes back the
   old value, or deletes it if there was none.
10. **Game priority** (High by default) for configured games that are running. Restore sets Normal.
    With `game_priority_method = "registry"` (for anti-cheats that block it, like BF6's) it writes the
    game's IFEO `CpuPriorityClass` instead, which Windows applies at the next game start; restore
    puts the registry back exactly. `auto` writes it when `auto` starts and keeps it until it exits.

Separately, `gamebooster ping` measures latency, jitter and loss to cloud regions near game
servers, and `gamebooster ping --route` checks each hop on the way (your router, your ISP, the far
end). Both change nothing. See [network](tweaks/network.md).

Separately, `gamebooster bench` records the game's frame times with Intel PresentMon and
`bench compare` tells a real difference from run-to-run noise. See [Measuring](measuring.md).
Status: not working yet on the target Mac (BF6's anti-cheat appears to block the capture); see
[docs/measuring.md](measuring.md).
