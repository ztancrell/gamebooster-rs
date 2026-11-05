# Game priority

**What it is.** Sets the CPU priority class of each running `[[game]]` process (default High).
Restore sets running games back to Normal; nothing is needed if the game has exited. The game has
to be running when this step runs: start BF6, then run `gamebooster boost`. If you boosted
before starting the game, run `gamebooster boost` again: while boosted, it only re-applies
priority.

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
lower_others_class = "below_normal"   # or "idle"
lower_others_exclude = ["Spotify.exe"]
lower_others_exclude_publishers = ["Spotify"]
```

**Try and undo by hand.** Task Manager > Details > right-click an app > Set priority > Below normal
(and back to Normal). Or PowerShell: `(Get-Process chrome).PriorityClass = 'BelowNormal'`, then
`'Normal'`. Priorities are forgotten when an app exits.

**Measure.** Task Manager > Details, "Base priority" column. Frame times in CapFrameX with a
background load (a browser playing video, a big download), with `lower_others` on and off.
