# Game priority

**What it is.** Sets the CPU priority class of each running `[[game]]` process (default High).
Restore sets running games back to Normal; nothing is needed if the game has exited. The game has
to be running when this step runs: start BF6, then run `gamebooster boost`. If you boosted
before starting the game, run `gamebooster boost` again: while boosted, it only re-applies
priority. Or use `gamebooster boost --watch`: it sets the priority itself when the game starts
(see [Dynamic mode](#dynamic-mode)).

**Windows default.** Every app starts at Normal priority; the scheduler shares CPU time between
all Normal threads, with a small boost for the window in front.

**Why it helps.** At High, the game's threads run before every Normal-priority background thread
that is ready at the same moment, so a background burst can't delay a frame. It only matters when
the CPU is busy (4-core Macs, large BF6 maps); on an idle CPU it changes nothing. Don't use
Realtime: it can starve input, audio and the driver threads the game itself depends on, which
causes stutter or even a frozen system.

**Exact setting.**

```toml
[[game]]
exe = "bf6.exe"     # check the real name in Task Manager > Details while BF6 runs
priority = "high"   # realtime, high, above_normal, normal, below_normal, low
```

`steam.exe`, `explorer.exe`, launchers and Windows core processes are refused as games.

## Try and undo by hand

Task Manager > Details > right-click `bf6.exe` > Set priority > High. Or in PowerShell (admin):

```powershell
(Get-Process bf6).PriorityClass = 'High'
(Get-Process bf6).PriorityClass = 'Normal'   # undo
```

Windows forgets the priority when the game exits; that is also an undo.

## Registry method

`game_priority_method = "registry"`. For anti-cheats that refuse `SetPriorityClass` from outside
(BF6's EA Javelin: boost prints `could not set bf6.exe (pid ...) to High: Access is denied. (os
error 5)` while power throttling off still works).

**Verified** on the target Mac with BF6, 2026-11-05: the IFEO value was written, BF6 launched
normally and Task Manager showed High. The process method is blocked there by the anti-cheat
([results](../hardware-test.md#results-on-the-target-mac)).

**What it does.** On boost, for each `[[game]]`, writes the DWORD
`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<exe>\PerfOptions\CpuPriorityClass`
(1 = low/Idle, 2 = normal, 3 = high, 5 = below_normal, 6 = above_normal; `realtime` is refused by
the config). Windows applies it itself when the game starts, so nothing opens the game process.
It takes effect the next time the game starts: if the game is running at boost, restart it.
The process method's `SetPriorityClass` is skipped; power throttling off still applies to the
running game.

**With `gamebooster auto`.** auto boosts only after the game has started, which is too late for
this value. So auto writes the entries when it starts (before any game) and keeps them across its
boosts and restores; its own boosts skip the IFEO step. When auto exits it puts them back exactly
(same records as below), or, if a boost it didn't make is still active, hands them to that boost's
state file. The records are kept in `gamebooster-auto-ifeo.json` next to the state file, so if auto
is killed, `gamebooster restore` puts them back. Started on top of an active boost, auto takes that
boost's IFEO records over (the old values are kept, never re-read from our own writes).

**Restore.** The state file records, per game, whether the `<exe>` key and `PerfOptions` existed and
the old value. Restore writes the old value back (or deletes it), then deletes `PerfOptions` and
the `<exe>` key only if boost created them and they are empty. Other IFEO values (Debugger,
MitigationOptions, ...) are never touched. A running game keeps its priority until it exits.

**Risk.** Low but nonzero: IFEO is also where debuggers and mitigations are configured, and an
anti-cheat could object to any entry for its game. Watch for launch errors; if BF6 won't start,
run `gamebooster restore`.

**By hand.**

```
reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\bf6.exe\PerfOptions" /v CpuPriorityClass /t REG_DWORD /d 3 /f
reg delete "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\bf6.exe\PerfOptions" /f
```

(The delete removes the whole `PerfOptions` key; only use it if it didn't exist before.)

## Measure

- Task Manager > Details, column "Base priority" (right-click a column header > Select columns).
- CapFrameX with some background load running (a big download or a file copy), once at Normal and
  once at High ([how](README.md#how-to-measure-any-tweak)).

## Power throttling off

**What it is.** On by default (`game_power_throttling_off = true`). In the same step as the
priority, each running `[[game]]` process is opted out of Windows power throttling (EcoQoS):
`SetProcessInformation(ProcessPowerThrottling)` with `ControlMask = EXECUTION_SPEED`,
`StateMask = 0`. Restore sets both masks to 0, which hands the decision back to Windows. Like
priority, it needs the game running: start it, then boost (or `boost` again while boosted).

**Windows default.** Windows decides per process; when it thinks an app is in the background
(minimised, behind another window, on battery) it may run it on slow, efficient settings.

**Why it helps.** Little for a focused, fullscreen game. It stops throttling when BF6 is behind
another window or Alt-Tabbed, mostly on battery. Low value on the Mac (no hybrid cores), but harmless.

**Safety.** Process-scoped: nothing is stored on disk, and it ends when the game exits, so a crash
mid-boost is harmless. Needs Windows 10 1709+; access denied (anti-cheat) or an older Windows is
printed (`power throttling off failed: ...`) and the boost carries on. The timer-resolution flag
(`IGNORE_TIMER_RESOLUTION`) is deliberately not set: timer-resolution tweaks are rejected
([Hone research](../hone/README.md)).

**Verify / measure.** Task Manager > Details, "Power throttling" column (Windows 11) shows Disabled
for `bf6.exe` while boosted. Measure with `gamebooster bench`, `game_power_throttling_off` true vs
false, ideally on battery with the game windowed.

## Lower other processes

**What it is.** Optional, off by default (`lower_others = true` turns it on). During a boost, every
process owned by you (same user account; SYSTEM and service processes are never touched) that runs
at Normal priority is set to Below normal, or Idle with `lower_others_class = "idle"`. Processes
already at another priority are left alone, so it never undoes game priority or a priority you set
by hand. Restore puts each one back to its old priority, but only if the same process still runs
(same pid and creation time), so a pid that Windows reused for a new process is never touched.
Processes that have exited are skipped. Boost and restore print one line each, for example
`lowered 23 processes to below normal (4 skipped: access denied)`; failures never stop the boost.
`--dry-run` lists the processes it would lower.

Smart Game Booster 6.0 has code that would do the same with Idle, but never calls it (spec 2.4a;
Superseded: this page said SGB does it). Idle is aggressive: an Idle app only runs
when nothing else wants the CPU, so voice chat, music or a stream can stutter while the game is
busy. Below normal still yields to the game but keeps those apps responsive.

**Never lowered** (`NEVER_LOWER` in `src/priority.rs`, case-insensitive, `*` = any ending): the
configured games, the apps in `close`, `lower_others_exclude`, and `gamebooster.exe`,
`explorer.exe`, `dwm.exe`, `csrss.exe`, `winlogon.exe`, `fontdrvhost.exe`, `ctfmon.exe`,
`sihost.exe`, `taskhostw.exe`, `RuntimeBroker.exe`, `ShellExperienceHost.exe`,
`StartMenuExperienceHost.exe`, `SearchHost.exe`, `TextInputHost.exe`, `audiodg.exe`, `Discord*`,
`steam.exe`, `steamwebhelper.exe`, `EADesktop.exe`, `EABackgroundService.exe`, `EALauncher.exe`,
`EAAntiCheat*` (EA Javelin, BF6's anti-cheat), `javelin*`, `EasyAntiCheat*`, `BEService*`
(BattlEye), `ThrottleStop.exe`, `obs64.exe`, `MsMpEng.exe`, `SecurityHealth*`, `NisSrv.exe`. The
anti-cheat names are from the vendors' usual process names, not checked on a BF6 install; most of
them run as SYSTEM and would be skipped anyway.

**Never lowered by publisher** (`NEVER_LOWER_PUBLISHERS` in `src/priority.rs`), a safety net so a
renamed or new exe from a protected vendor is skipped too (idea from Smart PC Utilities' exception
list, see docs/smartpcutilities-gb/README.md). The publisher is the CompanyName in the exe's version
info (Properties > Details, "Company"); an exe is skipped when its CompanyName contains, ignoring
case, one of: `Valve`, `Electronic Arts`, `Epic Games` (also Easy Anti-Cheat), `Riot Games`,
`Ubisoft`, `Blizzard`, `Discord`, `BattlEye`, `NVIDIA`, `Advanced Micro Devices`,
`Intel Corporation`, `Apple Inc` (Boot Camp helpers), `Malwarebytes`, `Avast`, `AVG Technologies`,
`ESET`, `Kaspersky`, `NortonLifeLock`, `Gen Digital`, `Symantec`, `Bitdefender`, `McAfee`, `Sophos`,
`Trend Micro`, `OBS Project`, or a string in `lower_others_exclude_publishers`. Not "Microsoft"
(that would cover most of Windows); Defender is in the name list. The signature is not checked: an
exe that fakes a CompanyName only gets skipped, which is harmless here. Exes without version info
are matched by name only.

**Exact setting.**

```toml
lower_others = true
lower_others_mode = "once"            # or "dynamic", see below
lower_others_class = "below_normal"   # or "idle" (once mode only)
lower_others_exclude = ["Spotify.exe"]
lower_others_exclude_publishers = ["Spotify"]
```

**Try and undo by hand.** Task Manager > Details > right-click an app > Set priority > Below normal
(and back to Normal). Or PowerShell: `(Get-Process chrome).PriorityClass = 'BelowNormal'`, then
`'Normal'`. Priorities are forgotten when an app exits.

**Measure.** Task Manager > Details, "Base priority" column. Frame times in CapFrameX with a
background load (a browser playing video, a big download), with `lower_others` on and off.

## Dynamic mode

**What it is.** `lower_others_mode = "dynamic"` (with `lower_others = true`) replaces "lower
everything at boost" with Process Lasso's ProBalance idea: `boost` lowers nothing, and while
`gamebooster boost --watch` runs it restrains only background apps that actually keep the CPU busy,
and only for as long as they do. Code: `src/watch.rs`.

Once a second the watcher reads each candidate's CPU time (`GetProcessTimes`) and the machine's
(`GetSystemTimes`). A process's share = its CPU-time delta / (elapsed time x logical CPUs), so 100 %
means every logical CPU flat out. A candidate is exactly what once mode would lower (your own
Normal-priority processes, minus the built-in names, publishers, games, `close` and the exclude
lists), **minus the foreground window's process and all its descendants**.

- **Restrain** to Below normal when the machine is at least *machine* % busy and the process has
  stayed at or above *process* % for 900 ms (in practice: two 1-s samples in a row over it).
- **Release** back to Normal when it is under *release* % and has been restrained for at least
  4.2 s, or at once when it (or a parent) becomes the foreground window. No maximum time.
- Below normal only: `lower_others_class = "idle"` is ignored in this mode (the watcher prints a
  note). No affinity, EcoQoS or I/O priority changes.

Thresholds (`PROBALANCE` in `src/watch.rs`) are Process Lasso 19.0's own defaults by logical CPU
count, read from `ProcessGovernor.exe` (source:
[decomp/process-lasso/governor/notes.md](../../decomp/process-lasso/governor/notes.md#probalance-defaults-by-logical-cpu-count-normal-branch)):

| Logical CPUs | Machine busy >= | Process >= | Release under |
|---|---|---|---|
| 4 | 23 % | 15 % | 5 % |
| 8 | 11 % | 8 % | 4 % |
| 12 (i7-9750H) | 10 % | 7 % | 3 % |
| 16 (2019 MBP 16") | 10 % | 5 % | 2 % |
| 32 and more | 7 % | 2 % | 1 % |

Other counts are in the const. Process Lasso's separate hybrid-CPU (P/E-core) branch is not
reproduced; the Mac has no E-cores.

**Safety.** Each restraint is added to the state file's `lowered` list (pid, creation time, exe,
old class) before the priority changes, and removed on release; the file is written atomically.
Stopping the watcher (Ctrl+C, closing the window) releases everything it restrained. If it is
killed, `gamebooster restore` puts the recorded ones back, same pid and creation time only, as in
once mode. A release only resets a process that is still Below normal, so a priority you set by
hand meanwhile is kept.

**Verify.** Run `gamebooster boost --watch`, start a CPU-heavy background app (a video export, a
browser benchmark), keep another window in front: within about 3 s the watcher prints `restrained
...` and Task Manager > Details shows Below normal. Switch to that app: `released ...: now in the
foreground`. Stop the load: released about 4 s later at the earliest.

**Measure.** `gamebooster bench` (the config snapshot records `lower_others_mode`) with a
background load: `lower_others` off, `"once"`, `"dynamic"`. Keep dynamic only if 1 % lows improve
or stay equal; until then it is unmeasured.
