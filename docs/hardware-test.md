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
     display" lines; anywhere else, note it.
   - `GameMode` / `  AutoGameModeEnabled = 1`
   - `Accessibility` / `  Sticky Keys shortcut off`, `  Filter Keys shortcut off`,
     `  Toggle Keys shortcut off` (`left alone (feature on or shortcut already off)` is fine).
     Then press Shift five times: no Sticky Keys dialog.
   - `Dns` (nothing under it; `fastest_dns` is off by default)
   - `Priority` / `  set bf6.exe (pid ...) to High`. `could not set` means BF6's anti-cheat blocks it:
     note it, not fatal. Then `  bf6.exe: power throttling off` (`... failed: <reason>` is not fatal;
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
    - `Dns` (nothing under it)
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
    `powercfg /query SCHEME_CURRENT SUB_PROCESSOR 75b0ae3f-bce0-45a7-8c89-c9611c25e100` shows
    `Current AC Power Setting Index: 0x00000af0` and the same for DC (0xaf0 = 2800).
    Run ThrottleStop's TS Bench and watch the clock in Task Manager or ThrottleStop: it should stay
    at or just under 2.8 GHz (not 4+ GHz, not 0.8 GHz), and *Limits* should not light up `PROCHOT`.
    Failure: `skipped (not on this machine)` on that line, or the clock above the cap.
    `.\gamebooster.exe restore`, then remove the two lines (or comment them out).
17. Quit BF6. Send `diff-boost.txt`, `diff-restore.txt`, the three `.json` files, the `bench`
    folder and the console text of steps 9, 12, 13 and 16.

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
6. Game priority and lowered processes (`"lowered"`) need nothing for safety; priorities end when
   each app quits; so does the game's power-throttling opt-out. To put one back now: Task Manager > Details > right-click > Set priority > Normal.
7. Delete `gamebooster-state.json` (otherwise the next `boost` only re-applies priority and power throttling off).

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
