# Hardware test checklist

First real run of `gamebooster` on the Boot Camp Windows install, with BF6. Part A tests our tool.
Part B (optional) boosts with Smart Game Booster (SGB) instead, to fill the unknowns in
[spec section 8](spec.md#8-not-specified--unknown). Takes about 30 minutes plus play time.

In every output below, a line containing `failed`, `could not`, `can't` or `error` is a failure:
copy the whole window text and send it.

## Part A: gamebooster

### Setup

1. On Linux, in the repo, build:
   ```sh
   cargo build --release --target x86_64-pc-windows-gnu
   ```
   Result: `target/x86_64-pc-windows-gnu/release/gamebooster.exe`.
2. Copy to the Windows side (USB stick or shared folder), all into `C:\gamebooster`:
   - `gamebooster.exe`
   - `gamebooster.example.toml`, renamed to `gamebooster.toml`
   - the whole `capture` folder, so you get `C:\gamebooster\capture\snapshot.ps1`
3. Edit `C:\gamebooster\gamebooster.toml` in Notepad so every tweak gets exercised once:
   - `services = ["WSearch"]` (Windows Search; harmless to stop).
   - `close = ["OneDrive.exe"]`, or any other app you see running in Task Manager > Details that
     you don't need while playing. Leave `[]` if nothing fits.
   - Start BF6, check its name in Task Manager > Details, and fix `exe = "bf6.exe"` if it differs.
     Quit BF6 again.
4. Open PowerShell as administrator (Start, type `PowerShell`, right-click **Windows PowerShell**,
   **Run as administrator**), from your normal Windows account. Then:
   ```powershell
   Set-ExecutionPolicy -Scope Process Bypass
   cd C:\gamebooster
   ```
   Use this one window for everything below.

### Run

5. `.\gamebooster.exe status`
   Look for: `Not boosted.`, `Active power plan: <guid>` (write the GUID down; Balanced is
   `381b4222-f694-41f0-9685-ff5bb260df2e`) and the `AutoGameModeEnabled` value.
6. `.\gamebooster.exe boost --dry-run`
   Look for: `Boost would do (dry run, nothing changed):`, then your service, your app's full
   path, the power plan list and `priority: bf6.exe -> High while running`.
   Failure: an `error:` line (usually a config typo; the message says which key).
7. Start BF6 and wait in the main menu. Alt-Tab back to PowerShell.
8. `.\capture\snapshot.ps1 -Out 1-before.json`
   Look for: `wrote 1-before.json (...)`. A `note: bcdedit failed` line means the window isn't
   administrator.
9. `.\gamebooster.exe boost`
   Look for, in this order:
   - `Saved restore state to C:\gamebooster\gamebooster-state.json`
   - `Services` / `  stopped service WSearch`
   - `Processes` / `  closed OneDrive.exe (pid ...)`
   - `Ram` / `  off`
   - `PowerPlan` / about 17 `  set    ...` lines / `  activated plan gamebooster (a2921d59-9c60-439e-b61a-21c441a0df53)`.
     `skipped (not on this machine)` is fine for the AMD PowerPlay, Wi-Fi and "unidentified hidden
     display" lines; anywhere else, note it.
   - `GameMode` / `  AutoGameModeEnabled = 1`
   - `Priority` / `  set bf6.exe (pid ...) to High`. `could not set` means BF6's anti-cheat blocks it:
     note it, not fatal.
   - `Boosted. Run ...`
10. `.\gamebooster.exe status`
    Look for: `Boosted (since unix time ...)` and
    `Active power plan: a2921d59-9c60-439e-b61a-21c441a0df53 (gamebooster)`.
11. Snapshot and compare:
    ```powershell
    .\capture\snapshot.ps1 -Out 2-boosted.json
    .\capture\compare.ps1 -Before 1-before.json -After 2-boosted.json | Out-File -Encoding utf8 diff-boost.txt
    notepad diff-boost.txt
    ```
    Expected in `diff-boost.txt`:
    - Power: `~ active scheme: ... -> a2921d59-... gamebooster`, `+ a2921d59-...` under schemes, and
      many settings lines (for example minimum processor state `AC=0x00000064 DC=0x00000064`, 100 %).
    - Services: `~ WSearch: Running / Auto -> Stopped / Auto`. The part after `/` (start type) must
      **not** change; if it does, that's a failure.
    - Processes: `stopped: OneDrive ...`, `stopped: SearchIndexer ...` (WSearch's process) and
      `~ bf6 (pid ...) priority: Normal -> High`. Other `started:`/`stopped:` lines are usually
      Windows background tasks coming and going.
    - Registry: at most `AutoGameModeEnabled` (only if it wasn't 1 already). Anything else there,
      under BCD or under Windows Update is a failure (we don't touch those).
    - Timer resolution and Memory changes are noise; ignore them.
12. Play a round or two. Note: does it feel the same as with SGB boosted, better or worse? Any
    crash or anti-cheat kick?
13. Back to the BF6 main menu (keep the game running), Alt-Tab, then `.\gamebooster.exe restore`.
    Look for:
    - `Priority` / `  set bf6.exe (pid ...) to Normal`
    - `GameMode` / `  AutoGameModeEnabled = <old value>` (or `(deleted)` if it was absent)
    - `PowerPlan` / `  reactivated plan <the GUID from step 5>` / `  deleted plan gamebooster`
    - `Processes` / `  relaunched C:\...\OneDrive.exe`
    - `Services` / `  started service WSearch`
    - `Restored.`
    Failure: `Some steps failed; the state file is kept ...`. Run `.\gamebooster.exe restore` once
    more; if it still fails, see [Recovery](#recovery).
14. `.\gamebooster.exe status`: `Not boosted.` and the same power plan GUID as in step 5.
15. Snapshot and compare with the first one:
    ```powershell
    .\capture\snapshot.ps1 -Out 3-restored.json
    .\capture\compare.ps1 -Before 1-before.json -After 3-restored.json | Out-File -Encoding utf8 diff-restore.txt
    notepad diff-restore.txt
    ```
    Expected: `active scheme unchanged`, no changes under power schemes, power settings, services,
    registry, BCD and Windows Update. Under Processes, OneDrive and SearchIndexer show as `stopped`
    and `started` with new pids (they were restarted); that's fine. Anything else under Power, Services or Registry is a
    failure.
16. Quit BF6. Send `diff-boost.txt`, `diff-restore.txt`, the three `.json` files and the console
    text of steps 9 and 13.

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
5. Game priority needs nothing; it ends when BF6 quits.
6. Delete `gamebooster-state.json` (otherwise the next `boost` only re-applies priority).

A reboot also brings the services back (their start type is never changed), but not the power plan
or the Game Mode value.

## Part B (optional): SGB, to fill the spec's unknowns

Make sure gamebooster is not boosted (`.\gamebooster.exe status` says `Not boosted.`). Leave SGB's
Boost settings at their defaults, and take a screenshot of its Boost config lists (services and
processes): that alone shows its default lists.

1. Start BF6, wait in the main menu (SGB's priority setting acts only on the running game).
2. `.\capture\snapshot.ps1 -Out sgb-1-before.json`
3. Press **Boost** in SGB and wait until it's done.
4. `.\capture\snapshot.ps1 -Out sgb-2-boosted.json`
5. Press **Restore** in SGB, then `.\capture\snapshot.ps1 -Out sgb-3-restored.json`.
6. Compare:
   ```powershell
   .\capture\compare.ps1 -Before sgb-1-before.json -After sgb-2-boosted.json | Out-File -Encoding utf8 sgb-diff-boost.txt
   .\capture\compare.ps1 -Before sgb-1-before.json -After sgb-3-restored.json | Out-File -Encoding utf8 sgb-diff-restore.txt
   ```
7. Send both diffs, the screenshot and the `sgb-*.json` files.

What each part of `sgb-diff-boost.txt` answers (spec section 8 unless noted):

| Diff section | Answers |
|---|---|
| Services, `Running -> Stopped` | The default service list. |
| Services, part after `/` changes (e.g. `Auto -> Manual` or `(delayed)`) | Whether SGB changes start types (ChangeServiceConfig). Also check `sgb-diff-restore.txt`: is it undone? |
| Processes, `stopped:` | The default process list. |
| Processes, `priority:` lines for anything but BF6 | Whether A10 lowers other processes (TPriorityController). |
| Processes, BF6 `priority:` line | Which priority SGB actually sets. |
| Memory | Only that the RAM trim ran. EmptyWorkingSet vs SetProcessWorkingSetSize and the skip list can't be seen here. |
| Registry, any line other than `AutoGameModeEnabled` | Registry values written beyond Game Mode. MMCSS (`SystemProfile`, incl. `SystemResponsiveness`, `NetworkThrottlingIndex`) and `PriorityControl :: Win32PrioritySeparation` should stay unchanged (spec says ruled out). |
| Power schemes, the `+` line | The SGB plan's GUID: if it is `58231417-2b7b-4575-8231-96cb9cf58b36`, that GUID is the fixed game-plan ID. |
| Power settings, line naming `90959d22-...` | The identity of that hidden display setting (Windows prints its name if it has one). The other lines confirm the section 2.1 values, incl. maximum processor state. |
| Windows Update | Section 2.5: whether `notificationLevel` changes, or `readOnly = True` makes SGB skip it. |
| Timer resolution, affinity lines | Should be none (ruled out); timer changes are often noise from the game itself. |
