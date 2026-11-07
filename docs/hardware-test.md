# Hardware test checklist

First real run of `gamebooster` on the Boot Camp Windows install, with BF6. Part A tests our tool.
Part B (optional) boosts with Smart Game Booster (SGB) instead, to fill the unknowns in
[spec section 8](spec.md#8-not-specified--unknown). Takes about 30 minutes plus play time.

In every output below, a line containing `failed`, `could not`, `can't` or `error` is a failure:
copy the whole window text and send it.

## Part A: gamebooster

### Setup

1. First time: download `gamebooster.exe` from the
   [Releases page](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest) into
   `C:\gamebooster` (SmartScreen: **More info** > **Run anyway**). Already installed: skip to step 4
   and run `.\gamebooster.exe update` there first.
   Look for: `Updated: <old> -> <new>` or `Already up to date: ...`; `.\gamebooster.exe --version`
   should show the commit you expect.
2. In the PowerShell window from step 4: `.\gamebooster.exe init`.
   Look for: `Wrote C:\gamebooster\gamebooster.toml ...` (or `already exists; left it unchanged.`).
3. Edit `C:\gamebooster\gamebooster.toml` in Notepad so every tweak gets exercised once:
   - `services = ["WSearch"]` (Windows Search; harmless to stop).
   - `close = ["OneDrive.exe"]`, or any other app you see running in Task Manager > Details that
     you don't need while playing. Leave `[]` if nothing fits.
   - `lower_others = true` (keep `lower_others_class = "below_normal"`).
   - Start BF6, check its name in Task Manager > Details, and fix `exe = "bf6.exe"` if it differs.
     Quit BF6 again.
4. Open PowerShell as administrator (Start, type `PowerShell`, right-click **Windows PowerShell**,
   **Run as administrator**), from your normal Windows account. Then:
   ```powershell
   cd C:\gamebooster
   ```
   Use this one window for everything below.

### Run

5. `.\gamebooster.exe status`
   Look for: `Not boosted.`, `Active power plan: <guid>` (write the GUID down; Balanced is
   `381b4222-f694-41f0-9685-ff5bb260df2e`) and the `AutoGameModeEnabled` value.
6. `.\gamebooster.exe boost --dry-run`
   Look for: `Boost would do (dry run, nothing changed):`, then your service, your app's full
   path, `lower other processes to below normal (old class restored): N processes: <names>`
   (your apps, none of explorer.exe, Discord, Steam, EA app), the power plan list and
   `game priority method: process (SetPriorityClass on the running game)`,
   `priority: bf6.exe -> High while running` and
   `game power throttling (EcoQoS): off while running (handed back to Windows on restore)`.
   Failure: an `error:` line (usually a config typo; the message says which key).
7. Start BF6 and wait in the main menu. Alt-Tab back to PowerShell.
8. `.\gamebooster.exe snapshot 1-before.json`
   Look for: `wrote 1-before.json (...)`. A `note: bcdedit failed` line means the window isn't
   administrator.
9. `.\gamebooster.exe boost`
   Look for, in this order:
   - `Saved restore state to C:\gamebooster\gamebooster-state.json`
   - `Services` / `  stopped service WSearch`
   - `Processes` / `  closed OneDrive.exe (pid ...)`
   - `Ram` / `  off`
   - `LowerOthers` / `  lowered N processes to below normal` (a `(... skipped: access denied, ...
     exited)` tail is fine)
   - `PowerPlan` / about 20 `  set    ...` lines (including `keep current lid close action`,
     `keep current power button action`, `keep current sleep button action`) / `  activated plan gamebooster (a2921d59-9c60-439e-b61a-21c441a0df53)`.
     `skipped (not on this machine)` is fine for the AMD PowerPlay, Wi-Fi and "unidentified hidden
     display" lines, and on Boot Camp Macs for `keep current lid close action`, `keep current power
     button action` and `keep current sleep button action` (Windows there has no such settings, only
     "Start menu power button"); anywhere else, note it. `--dry-run` shows the same skipped lines.
   - `GameMode` / `  AutoGameModeEnabled = 1`
   - `Accessibility` / `  Sticky Keys shortcut off`, `  Filter Keys shortcut off`,
     `  Toggle Keys shortcut off` (`left alone (feature on or shortcut already off)` is fine).
     Then press Shift five times: no Sticky Keys dialog.
   - `Dns` (nothing under it; `fastest_dns` is off by default)
   - `Priority` / `  set bf6.exe (pid ...) to High`. `could not set bf6.exe (pid ...) to High: <OS
     error>` plus a `hint:` line means BF6's anti-cheat blocks it: note it, not fatal (step 18b tries
     the registry method). Then `  bf6.exe: power throttling off` (`... failed: <reason>` is not fatal;
     note the reason). Windows 11: Task Manager > Details, "Power throttling" column says Disabled.
   - `Boosted. Run ...`
10. `.\gamebooster.exe status`
    Look for: `Boosted (since unix time ...)` and
    `Active power plan: a2921d59-9c60-439e-b61a-21c441a0df53 (gamebooster)`.
11. Snapshot and compare:
    ```powershell
    .\gamebooster.exe snapshot 2-boosted.json
    .\gamebooster.exe compare 1-before.json 2-boosted.json | Out-File -Encoding utf8 diff-boost.txt
    notepad diff-boost.txt
    ```
    Expected in `diff-boost.txt`:
    - Power: `~ active scheme: ... -> a2921d59-... gamebooster`, `+ a2921d59-...` under schemes, and
      many settings lines (for example minimum processor state `AC=0x00000064 DC=0x00000064`, 100 %).
    - Services: `~ WSearch: Running / Auto -> Stopped / Auto`. The part after `/` (start type) must
      **not** change; if it does, that's a failure.
    - Processes: `stopped: OneDrive ...`, `stopped: SearchIndexer ...` (WSearch's process) and
      `~ bf6 (pid ...) priority: Normal -> High`, plus `~ <app> (pid ...) priority: Normal ->
      BelowNormal` lines for your other apps (`lower_others`). Other `started:`/`stopped:` lines are usually
      Windows background tasks coming and going.
    - Registry: at most `AutoGameModeEnabled` (only if it wasn't 1 already). Anything else there,
      under BCD or under Windows Update is a failure (we don't touch those).
    - Timer resolution and Memory changes are noise; ignore them.
12. Play a round or two. Note: does it feel the same as with SGB boosted, better or worse? Any
    crash or anti-cheat kick?
    Currently fails (2026-11-05): PresentMon reports `failed to start trace session: access
    denied`; BF6's anti-cheat (EAAntiCheatService) appears to block ETW capture, see
    [measuring](measuring.md#status). Run the step anyway and copy the output.
    Then, in a match, measure (see [measuring](measuring.md)): Alt-Tab, run
    `.\gamebooster.exe bench 30`, switch back to the game within a couple of seconds and keep
    playing the same way until it finishes. The first time it downloads PresentMon into
    `C:\gamebooster\tools`.
    Look for: `<game> 30 s, boosted:` with a frame count and FPS lines, then
    `Saved C:\gamebooster\bench\..._boosted.json (raw frames: ...csv)`.
    Failure: `has SHA-256 ..., expected ...` (download corrupted or changed; run it again once),
    `no frames recorded` (game minimised?) or `PresentMon wrote no CSV` (copy its output).
13. Back to the BF6 main menu (keep the game running), Alt-Tab, then `.\gamebooster.exe restore`.
    Look for:
    - `Priority` / `  set bf6.exe (pid ...) to Normal` and `  bf6.exe: power throttling handed back to Windows`
    - `Dns` / `  nothing to undo`
    - `Accessibility` / `  Sticky Keys flags restored (0x...)` (one line per shortcut boost turned off)
    - `GameMode` / `  AutoGameModeEnabled = <old value>` (or `(deleted)` if it was absent)
    - `PowerPlan` / `  reactivated plan <the GUID from step 5>` / `  deleted plan gamebooster`
    - `LowerOthers` / `  restored the old priority of N processes` (`(... exited)` is fine)
    - `Processes` / `  relaunched C:\...\OneDrive.exe`
    - `Services` / `  started service WSearch`
    - `Restored.`
    Failure: `Some steps failed; the state file is kept ...`. Run `.\gamebooster.exe restore` once
    more; if it still fails, see [Recovery](#recovery).
14. `.\gamebooster.exe status`: `Not boosted.` and the same power plan GUID as in step 5.
15. Snapshot and compare with the first one:
    ```powershell
    .\gamebooster.exe snapshot 3-restored.json
    .\gamebooster.exe compare 1-before.json 3-restored.json | Out-File -Encoding utf8 diff-restore.txt
    notepad diff-restore.txt
    ```
    Expected: `active scheme unchanged`, no changes under power schemes, power settings, services,
    registry, BCD and Windows Update. Under Processes, OneDrive and SearchIndexer show as `stopped`
    and `started` with new pids (they were restarted); that's fine. Anything else under Power, Services or Registry is a
    failure.
    Optional (blocked like step 12 for now), for a first comparison: back in a match, run `.\gamebooster.exe bench 30` once
    more (now unboosted, labelled `normal`), then
    `.\gamebooster.exe bench compare bench\<the normal .json> bench\<the boosted .json>`.
    Look for: the side-by-side table and two `B - A = ...` lines. One pair of runs is only a smoke
    test; real measurements follow [measuring](measuring.md).
16. Optional, frequency cap (`max_cpu_mhz`): set `max_cpu_mhz = 2800` and
    `max_processor_state = 100` in `gamebooster.toml`, then `.\gamebooster.exe boost`.
    Look for: `  set     maximum processor frequency 2800 MHz` under `PowerPlan`, and
    `powercfg /qh SCHEME_CURRENT SUB_PROCESSOR 75b0ae3f-bce0-45a7-8c89-c9611c25e100` shows
    `Current AC Power Setting Index: 0x00000af0` and the same for DC (0xaf0 = 2800). Use `/qh`:
    the setting is hidden and `/query` does not list it.
    Run ThrottleStop's TS Bench and watch the clock in Task Manager or ThrottleStop: it should stay
    at or just under 2.8 GHz (not 4+ GHz, not 0.8 GHz), and *Limits* should not light up `PROCHOT`.
    (The main window's `PROCHOT 100°C` label next to the *Limits* button is TjMax, not a warning.)
    Failure: `skipped (not on this machine)` on that line, or the clock above the cap.
    `.\gamebooster.exe restore`, then remove the two lines (or comment them out).
17. Ping and route check (no admin needed, BF6 can be closed): `.\gamebooster.exe ping`, then
    `.\gamebooster.exe ping --route`.
    Look for, from `ping`: one line per region with `median`, `p90`, `jitter` and `loss` columns
    (e.g. `19 ms    20    0.5  0/10 US East (N. Virginia)`), sorted fastest first.
    From `--route`: `Finding the fastest region ...`, `Route to <host> (<ip>), 10 ICMP echoes per
    hop:`, then numbered hops; hop 1 is your router's address with `your router (Wi-Fi)` (or
    `(Ethernet)`), some hops may show their names. The last lines: either `(destination)` on the last
    hop or `The destination didn't answer ICMP ...`, then one of `No loss on the path.`,
    `Loss only at some intermediate hops ...` or `Loss starts at hop N ...`.
    Failure: `IcmpCreateFile failed`, every hop `*` / `no answer` (ICMP blocked by a firewall), or
    hop 1 not being your router's address (`ipconfig`, *Default Gateway*).
    Also try `.\gamebooster.exe ping --route 1.1.1.1`: it should reach the destination.
18. Watch mode (BF6 closed, not boosted: `status` says `Not boosted.`). In `gamebooster.toml` set
    `lower_others = true` and `lower_others_mode = "dynamic"`. Then `.\gamebooster.exe boost --watch`.
    Look for: under `LowerOthers`, "dynamic: `boost --watch` restrains busy background processes
    while it runs"; under `Priority`, `no configured game is running: the watcher sets it when one
    starts` (with `game_priority_method = "registry"`: `Windows applies the priority when it
    starts; the watcher turns power throttling off`); then `Watching once a second ...` and a `lower others (dynamic, 16 logical CPUs):
    restrain ... >= 10 % busy ... >= 5 % ...` line (12 threads: 10 / 7 %).
    a. Launch BF6 and wait in the main menu. Look for: `Game started:` then `set bf6.exe (pid ...)
       to High` and `bf6.exe: power throttling off` in the watch window; Task Manager > Details
       shows High for bf6.exe.
    b. Alt-Tab to the desktop. Start a CPU-heavy background app that is not excluded, e.g. a
       browser tab running a benchmark, or in a second (non-admin) PowerShell:
       `1..4 | % { Start-Job { while ($true) {} } }`. Put another window (Notepad) in front.
       Look for, within about 3 s: `restrained <name> (pid ..., NN % CPU) to below normal`, and
       Below normal for it in Task Manager > Details (PowerShell jobs run as `powershell.exe`).
    c. Click on that app's window (or the second PowerShell). Look for `released <name> ...: now
       in the foreground`. Click away again: it is restrained again after about 2 s.
    d. Stop the load (`Get-Job | Stop-Job; Get-Job | Remove-Job`). Look for `released ...: quiet
       again`, at least 4 s after it was restrained.
    e. Start the load again, wait for `restrained`, then press Ctrl+C in the watch window.
       Look for: `Stopping the watcher.`, `restored the old priority of N processes`, "Still
       boosted: run `gamebooster restore` when you're done playing."; Task Manager shows Normal
       again. Then `.\gamebooster.exe restore` (ends with `Restored.`), and stop the load.
    Failure: no `restrained` line while the load runs, a restrained app that stays Below normal
    after Ctrl+C, the watch window printing a status line more than every 10 s, or Windows
    reporting the watch window as not responding. Set `lower_others_mode = "once"` (or
    `lower_others = false`) again afterwards.
18b. Registry priority method (if step 9 printed `could not set`). Quit BF6. Add
    `game_priority_method = "registry"` to gamebooster.toml, then `.\gamebooster.exe boost`.
    Look for: `Priority` / `  bf6.exe: IFEO CpuPriorityClass = 3 (High): Windows applies it the next
    time the game starts`. Then:
    a. `reg query "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\bf6.exe\PerfOptions"`
       shows `CpuPriorityClass    REG_DWORD    0x3`.
    b. Start BF6 and wait in the main menu. Task Manager > Details > right-click bf6.exe > Set
       priority shows High. Failure: BF6 refuses to start or shows an anti-cheat error (note the
       exact message, then `restore`).
    c. `.\gamebooster.exe restore`. Look for `  bf6.exe: IFEO CpuPriorityClass deleted, removed
       PerfOptions and key` (if the bf6.exe key didn't exist before), then `Restored.`; the
       `reg query` from (a) now says the key can't be found. Quit BF6 and remove the config line.
18c. Automatic boost (BF6 closed, `status` says `Not boosted.`; keep
    `game_priority_method = "registry"` from 18b for this step). `.\gamebooster.exe auto`.
    Look for: `Game priority (registry): holding the IFEO entries while auto runs`; the `reg query`
    from 18b (a) shows `0x3` before BF6 starts.
    a. Start BF6. Within about a second: `bf6.exe started: boosting`, then the full boost output.
       `(Get-Process bf6).PriorityClass` in a second PowerShell says `High`.
    b. Quit BF6. Look for `last game exited: restoring in 30 s`, then the full restore and
       `restored; waiting for a game`. `status`: `Not boosted.` and the step 5 plan; WSearch
       running; the IFEO entry is still there (held while `auto` runs).
    c. Ctrl+C. The `reg query` now says the key can't be found; no `gamebooster-state.json` or
       `gamebooster-auto-ifeo.json` in `C:\gamebooster`. Remove the config line afterwards.
19. Quit BF6. Send `diff-boost.txt`, `diff-restore.txt`, the three `.json` files, the `bench`
    folder and the console text of steps 9, 12, 13, 16, 17, 18, 18b and 18c.

## Recovery

If `restore` keeps failing, undo by hand in the same administrator PowerShell. Open
`C:\gamebooster\gamebooster-state.json` in Notepad; it lists what was changed.

1. Power plan: `powercfg /setactive <previous_scheme from the file>` (or Balanced:
   `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e`), then
   `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`. Check with `powercfg /getactivescheme`.
2. Services: for each name under `"services"`, last one first: `sc.exe start <name>`
   (in PowerShell `sc` alone is a different command). `sc.exe query <name>` should say `RUNNING`.
3. Apps: start each path under `"processes"` (double-click it in Explorer, or `start "<path>"`).
4. Game Mode: if `"previous"` under `"game_mode"` is a number N:
   `reg add HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled /t REG_DWORD /d N /f`.
   If it is `null`: `reg delete HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled /f`.
5. Accessibility shortcuts: nothing to do; they were changed for this session only, so signing
   out or rebooting brings them back (or turn them on in Settings > Accessibility > Keyboard).
6. Game priority and lowered processes (`"lowered"`, including the watcher's dynamic restraints) need nothing for safety; priorities end when
   each app quits; so does the game's power-throttling opt-out. To put one back now: Task Manager > Details > right-click > Set priority > Normal.
7. IFEO (registry method): for each entry under `"ifeo"`: if `"previous"` is a number N,
   `reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<exe>\PerfOptions" /v CpuPriorityClass /t REG_DWORD /d N /f`;
   if it is `null`, `reg delete "<same key>" /v CpuPriorityClass /f`, then, if `"perf_existed"` is
   false, `reg delete "<same key>" /f` (and, if `"key_existed"` is false and `reg query` of the
   `<exe>` key shows nothing else, delete that key too).
8. Delete `gamebooster-state.json` (otherwise the next `boost` only re-applies priority and power throttling off).

A reboot also brings the services back (their start type is never changed), but not the power plan
or the Game Mode value.

## Part B (optional): SGB, to fill the spec's unknowns

Make sure gamebooster is not boosted (`.\gamebooster.exe status` says `Not boosted.`). Leave SGB's
Boost settings at their defaults, and take a screenshot of its Boost config lists (services and
processes): that alone shows its default lists.

1. Start BF6, wait in the main menu (SGB's priority setting acts only on the running game).
2. `.\gamebooster.exe snapshot sgb-1-before.json`
3. Press **Boost** in SGB and wait until it's done.
4. `.\gamebooster.exe snapshot sgb-2-boosted.json`
5. Press **Restore** in SGB, then `.\gamebooster.exe snapshot sgb-3-restored.json`.
6. Compare:
   ```powershell
   .\gamebooster.exe compare sgb-1-before.json sgb-2-boosted.json | Out-File -Encoding utf8 sgb-diff-boost.txt
   .\gamebooster.exe compare sgb-1-before.json sgb-3-restored.json | Out-File -Encoding utf8 sgb-diff-restore.txt
   ```
7. Send both diffs, the screenshot and the `sgb-*.json` files.

What each part of `sgb-diff-boost.txt` answers (spec section 8 unless noted):

| Diff section | Answers |
|---|---|
| Services, `Running -> Stopped` | The default service list. |
| Services, part after `/` changes (e.g. `Auto -> Manual` or `(delayed)`) | Whether SGB changes start types (ChangeServiceConfig). Also check `sgb-diff-restore.txt`: is it undone? |
| Processes, `stopped:` | The default process list. |
| Processes, `priority:` lines for anything but BF6 | Not expected: TPriorityController is never called (spec 2.4a). Any such line would contradict that. |
| Processes, BF6 `priority:` line | Which priority SGB actually sets. |
| Memory | Only that the RAM trim ran. EmptyWorkingSet vs SetProcessWorkingSetSize and the skip list can't be seen here. |
| Registry, any line other than `AutoGameModeEnabled` | Registry values written beyond Game Mode. MMCSS (`SystemProfile`, incl. `SystemResponsiveness`, `NetworkThrottlingIndex`) and `PriorityControl :: Win32PrioritySeparation` should stay unchanged (spec says ruled out). |
| Power schemes, the `+` line | The SGB plan's GUID: if it is `58231417-2b7b-4575-8231-96cb9cf58b36`, that GUID is the fixed game-plan ID. |
| Power settings, line naming `90959d22-...` | The identity of that hidden display setting (Windows prints its name if it has one). The other lines confirm the section 2.1 values, incl. maximum processor state. |
| Windows Update | Section 2.5: whether `notificationLevel` changes, or `readOnly = True` makes SGB skip it. |
| Timer resolution, affinity lines | Should be none (ruled out); timer changes are often noise from the game itself. |

## Results on the target Mac

2026-11-05: 2019 MacBook Pro, Boot Camp, Windows, i9-9880H with 12 logical CPUs active (2 of its 8
cores disabled by `numproc 12`; recorded at the time as an i7, see
[2026-11-07](#numproc-gpu-power-limit-and-cpu-22x-lock-2026-11-07)), BF6 with EA Javelin anti-cheat.

| Result | What |
|---|---|
| Works | `boost` + `restore` full cycle: power plan (min 5 % / max 99 %), Game Mode, services WSearch / SysMain / Spooler stopped and restarted, state file removed. |
| Works | Fastest DNS: the router (2–3 ms) beat Cloudflare / Quad9 / Google (~20–48 ms), so it was kept. |
| Works | `lower_others` once (25 processes lowered) and dynamic in the watcher (thresholds 10 / 7 / 3 % picked for 12 CPUs; game detected). |
| Works | Power throttling off on `bf6.exe` (the anti-cheat allows it). |
| Works | Game priority, registry method: IFEO `CpuPriorityClass` written, BF6 launched normally, Task Manager shows High. |
| Works | Lid / power / sleep button settings correctly reported as absent on Boot Camp. |
| Works | `ping` (US East ~21 ms); `update` from GitHub releases. |
| Blocked | Game priority, process method: `SetPriorityClass` refused while BF6 runs (anti-cheat). |
| Blocked | `bench`: ETW trace sessions refused system-wide (EAAntiCheatService suspected, unconfirmed). |
| Pending | Accessibility shortcuts always report "left alone": the Shift x5 check is outstanding. |
| Pending | `max_cpu_mhz` clock cap: written (checked with `powercfg /qh`); not yet re-tested now that Turbo is on (no effect was observable while the CPU sat at 22 W). |
| Pending | `ping --route`; `status` clock check under the new build; `restore` output with the registry method. |
| Resolved | CPU held at ~22 W package power: ThrottleStop *Disable Turbo* plus an 80 % *Maximum processor state* in WinUtil's plan, not gamebooster (see below). |

2026-11-06: same Mac (i9-9880H, still 12 logical CPUs under `numproc 12`), BF6, `gamebooster auto` with `game_priority_method = "registry"`.

| Result | What |
|---|---|
| Works | `auto` (build dabbf43): `bf6.exe` detected within ~1 s of launch; full boost (services stopped, gamebooster plan active, DNS kept, power throttling off). After exit: `last game exited: restoring in 30 s`, then full restore in reverse order; `Not boosted.`, Balanced `381b4222-...` active, WSearch running, no IFEO key. |
| Fixed | dabbf43 wrote the IFEO `CpuPriorityClass` at boost time, after BF6 had started, so BF6 ran at Normal; the restore then removed it. 9271794: `auto` holds the IFEO entries for its whole run. |
| Works | `auto` (build 9271794): printed `Game priority (registry): holding the IFEO entries while auto runs` and wrote `CpuPriorityClass = 3` before BF6 launched; `bf6.exe` ran at High (`Get-Process` `PriorityClass`). Exit + 30 s: restored, IFEO entry still held. Ctrl+C: IFEO key removed, no `gamebooster-state.json` or `gamebooster-auto-ifeo.json` left, `Not boosted.` |
| Works | In game (Portal "Spec Ops Survival" custom experience), Turbo on: ~105–108 % processor performance (~2.75 GHz) at ~80 % utility, no 800 MHz BD PROCHOT lock. `status` single-thread clock 4.3 GHz. Played well. |

### CPU clock and power under load (2026-11-05)

i9-9880H, measured with 2 cores disabled by `numproc 12` (6 cores / 12 threads active; recorded at
the time as an i7-9750H, see
[2026-11-07](#numproc-gpu-power-limit-and-cpu-22x-lock-2026-11-07)). Windows plan before boosting: "Ultimate Performance" (from Chris
Titus WinUtil). SMC reset done before these readings.

- **No PROCHOT lock seen.** ThrottleStop's main-window label `PROCHOT 100°C` (right of the
  *Limits* button) is an information label (TjMax), not a throttling indicator; the earlier
  "PROCHOT red" readings came from that label. The actual *Limits* window (CORE / RING / GPU)
  stayed empty during TS Bench. Superseded: earlier notes (prochot-research.md, "What the target
  machine shows") assumed this Mac shows the 800 MHz BD PROCHOT lock; it was not observed.
- **~2.29 GHz under TS Bench.** Multiplier 22–23 x 99.77 MHz, on battery and plugged in
  (right-hand port), ~61 °C, fans quiet. Same with gamebooster's plan (`max_cpu_mhz = 2800`,
  `max_processor_state = 100`) and after `restore` on Ultimate Performance, so neither plan causes
  it. ThrottleStop *Set Multiplier* not ticked.
- **HWiNFO:** CPU Package Power ~22.3 W under full load; static PL1 100 W / PL2 125 W (Apple's
  ceilings); no dynamic PL values shown. So the CPU is held to about half its normal ~45 W budget
  by a dynamic limit (likely the Mac's controller); the limiting reason is not yet identified.
  Superseded: the cause was two settings, not a hardware limit; see *Resolution* below.
- **Next:** the HWiNFO "Performance Limit Reasons" lines that say Yes during TS Bench; then a
  battery report (`powercfg /batteryreport`). Superseded: no limit reason said Yes; see
  *Resolution*.
- **Resolution.** Two settings, neither from gamebooster: ThrottleStop's *Disable Turbo* was
  ticked, and the active WinUtil "Ultimate Performance" plan had *Maximum processor state* 80 %
  (0x50) AC and DC (found with `powercfg /qh scheme_current sub_processor PROCTHROTTLEMAX`).
  gamebooster's own plan writes `max_processor_state` (default 100) itself, so it was not the
  cause; the earlier "same after `restore`" reading was Disable Turbo. Fix: untick *Disable Turbo*
  (and save), then `powercfg /setacvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100`,
  the same with `/setdcvalueindex`, and `powercfg /setactive scheme_current`. Result under TS
  Bench: 4050 MHz, package power up to 85.4 W, temps 54 / 57 / 70 °C, max 100 °C. ThrottleStop's
  PROCHOT indicator now flickers grey / red (brief TjMax touches, normal thermal protection)
  instead of solid red. BD PROCHOT left ticked (enabled, safe). Whether the 800 MHz BD PROCHOT
  lock returns in BF6 with Turbo on: not in the 2026-11-06 session (~2.75 GHz all-core, see the
  table above); fallback `max_cpu_mhz = 3000`.

### numproc, GPU power limit and CPU 22x lock (2026-11-07)

Checked directly on the Mac (WMI, `bcdedit`, an HWiNFO log of one BF6 session).

Facts:

- **CPU.** Intel Core i9-9880H: 8 cores / 16 threads, 2.3 GHz base, up to 4.8 GHz turbo. HWiNFO
  reports the system as Apple MacBookPro16,1 with an AMD Radeon Pro 5500M 8 GB (driver
  32.0.21045.11001, dated 2026-09-24).
- **`numproc 12`.** Windows' BCD had the boot option `numproc 12`, so only 6 cores / 12 threads
  were active. That is why earlier notes recorded an "i7-9750H (6 cores / 12 threads)": every
  measurement before 2026-11-07 is the i9 with 2 cores disabled. Who set it is unknown. Removed on
  2026-11-07 with `bcdedit /deletevalue {current} numproc`, effective from the next reboot (the
  reboot is not confirmed yet).
- **BF6 session, HWiNFO log.** gamebooster boost active, on AC (battery 93 %, charge rate 0 W).
  - GPU: ~99 % busy and at its power limit (*GPU PPT Limit* ~100 %), the PPT limit switching
    between 30 W and 25 W; GPU clock ~300–900 MHz; GPU 72–76 °C, hotspot up to ~84 °C.
  - BF6 mostly 12–20 FPS.
  - CPU locked at 22x (~2.2 GHz), 13–27 W package power, 72–80 °C, with no limit reason flagged
    (no PROCHOT, thermal or PL1/PL2; PL1 100 W / PL2 125 W static).
  - Windows plan: turbo boost mode aggressive, no maximum frequency cap, maximum processor state
    100 %. So neither the plan nor ThrottleStop's *Disable Turbo* (the 2026-11-05 cause, see above)
    explains the 22x lock this time.

Hypothesis (unconfirmed): Apple's firmware shares one power budget between CPU and GPU, and a
charger smaller than the stock 96 W shrinks it. Which charger was used is being checked.

Next:

1. Reboot, then confirm 8 cores / 16 threads in Task Manager (*Performance* > *CPU*: Cores 8,
   Logical processors 16).
2. Replay the same BF6 map on the stock 96 W charger plugged straight in (no hub or adapter). Log
   *GPU PPT Limit*, GPU clock and FPS in HWiNFO and compare with the session above.
3. Check whether the CPU 22x lock persists after the reboot (HWiNFO core multiplier and limit
   reasons, under BF6 and under TS Bench).
