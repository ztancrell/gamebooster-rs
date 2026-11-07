# ProcessGovernor.exe (core engine) — notes

Process Lasso 19.0.0.94, `ProcessGovernor.exe` (sha256 `96d914a2…9003d`, see
[../README.md](../README.md)). Static only. Addresses are virtual addresses in this build (image base
0x140000000), given as evidence; the decompiled output was deleted.

The engine is one native C++ program. It reads `prolasso.ini` (INI sections named below), samples
every process's CPU use about once a second (the "core refresh rate", INI `UpdateSpeedCore`), and
applies rules. The GUI (`ProcessLasso.exe`) only displays and edits the config. Closing the GUI leaves
the engine running.

## Config loader and defaults

- The loader (FUN_1400598a0) reads every key with a built-in default; the saver (FUN_140053830)
  writes them back. Bitsum's INI reference lists several ProBalance defaults as "varies". The binary
  shows why: four of them come from **tables indexed by the number of logical CPUs** (1 to 32; more
  than 32 uses the 32 entry).
- Helpers that return those defaults: total-CPU threshold FUN_140067bb0, per-process threshold
  FUN_140067bf0, time over quota FUN_140067cc0, restore threshold FUN_140067c60, minimum restraint
  time FUN_140067d30. The tables are in `.rdata` at 0x140115230 (restore), 0x1401152b0 (per process),
  0x140115330 (total) and 0x1401153b0 (time over quota).
- Each helper has a second branch, selected by a CPU-topology flag in the engine's CPU-info object.
  When the flag is set, the per-process and restore defaults are computed for the logical-CPU count
  minus a counted subset of cores (a popcount over a mask built with
  `GetLogicalProcessorInformationEx`). The per-process default is then doubled, the time over quota
  tripled, and the minimum restraint time halved (2,100 ms). This is most likely the
  hybrid-CPU (P-core/E-core) case; that reading is not confirmed. A 2019 MacBook Pro (Coffee Lake, no
  E-cores) takes the normal branch.
- All CPU percentages are **percent of the whole machine** (all logical CPUs together), not of one
  core. `DivideCPUPercentThresholdsBySystemCPUCount` optionally lets you write them as
  "core-percent" instead (e.g. 3200 % on 64 threads = 50 %).

### ProBalance defaults by logical CPU count (normal branch)

Logical-CPU count → value (32 and above use the 32 entry):

| Setting | Values |
|---|---|
| `TotalProcessorUsageBeforeRestraint` (whole machine busy, %) | 1: 75, 2: 45, 3: 40, 4: 23, 5–6: 18, 7: 17, 8–9: 11, 10–16: 10, 17–21: 9, 22–26: 8, 27–32: 7 |
| `PerProcessUsageBeforeRestraint` (process share of machine, %) | 1: 64, 2: 32, 3: 20, 4: 15, 5–6: 10, 7: 9, 8–10: 8, 11–12: 7, 13–14: 6, 15–19: 5, 20–24: 4, 25–29: 3, 30–32: 2 |
| `PerProcessUsageForRestore` (%) | 1: 15, 2: 7, 3–7: 5, 8: 4, 9–12: 3, 13–16: 2, 17–32: 1 |

If the CPU count is unknown the fallbacks are 70 / 20 / 5 %. Note the per-process threshold stays
around one logical CPU's worth (64 % of 1 … about 0.6–0.8 of a thread on 8–32 threads): ProBalance
targets "one thread running flat out", not modest background work.

Constant defaults (all CPU counts):

| INI key (section `OutOfControlProcessRestraint`) | Default |
|---|---|
| `OocOn` (ProBalance on) | on |
| `TimeOverQuotaBeforeRestraint` | 900 ms (every table entry) |
| `MinimumTimeOfRestraint` | 4,200 ms |
| `MaximumTimeOfRestraint` | 0 (no limit) |
| `TameOnlyNormal` (only touch Normal-priority processes) | true |
| `LowerToIdleInsteadOfBelowNormal` | false (so: Below normal) |
| `DoNotLowerPriorityClass` | false |
| `LowerIOPriorityDuringRestraint` | false |
| `RestrainByAffinity`, `ProBalanceDropOneRandomCore`, `RestraintAffinity` | false, false, empty |
| `DoNotAdjustAffinityIfCustomized` | true |
| `UseEfficiencyMode` (EcoQoS during restraint) | false |
| `OocDisableCoreParkingWhileIn` | false |
| `ExcludeForegroundProcesses2` | true (inherits the old `ExcludeForegroundProcesses`, default true; forced true by one build check) |
| `ExcludeChildrenOfForeground` | true |
| `ExcludeServices` | true |
| `DisableProBalanceWhenSysIdle` / `…IfSysIdleThisManyMS` | false / 30,000 ms |
| `OocExclusions`, `OocHardCodedExclusionOverrides` | empty |
| `ChangeTrayIconOnRestraint` | true; sounds off |

Worked example, 2019 MacBook Pro 16" with 16 threads: restrain a process when the whole machine is at
or above 10 % and the process has been at or above 5 % of the machine (0.8 of one thread) for
900 ms. It is restored once it is below 2 % and has been restrained for at least 4.2 s. With 12
threads (i7-9750H, or the target i9-9880H while `numproc 12` was set) the thresholds are 10 / 7 / 3 %; with 8 threads (13" models) 11 / 8 / 4 %.

## ProBalance: the algorithm

Read from the disassembly of the main loop (one huge function at 0x14001d010; the Ghidra decompiler
gave up on it, "response buffer size exceeded"), around 0x140024c90–0x1400266d7. The engine keeps
its config inside that function's frame, so the INI fields appear there at fixed frame offsets.

Per sampling tick, per process:

1. **Gate.** Skip if ProBalance is off, or the process is flagged as excluded. Exclusions are: the
   foreground process and its children, services (when `ExcludeServices`), processes not at Normal
   priority (`TameOnlyNormal`), the user list `OocExclusions` (names, wildcards, regex; optionally
   full paths), and a built-in list of critical processes. The binary names `csrss`, `smss`,
   `wininit`, `winlogon`, `lsass`, `services`, `dwm`, `audiodg` and its own processes; the GUI says
   "Critical system processes can not be throttled". `OocHardCodedExclusionOverrides` removes
   entries from that built-in list. Performance Mode's target process is also kept out of ProBalance
   (Bitsum's BHP page). Optionally the whole of ProBalance pauses when the user has been idle for
   30 s.
2. **System load.** Compare the machine's total CPU use for the tick with the total threshold
   (0x140024ccf). Below it, nothing new is restrained, and restrained processes go to the restore
   check.
3. **Process load.** Compare the process's share of the machine with the per-process threshold
   (0x140024ced). Below it, the process goes to the restore check.
4. **Time over quota.** The engine accumulates how long the process has stayed above the per-process
   threshold. Restraint starts only when that time reaches `TimeOverQuotaBeforeRestraint` plus a
   small configured extra, compared in 100-ns units (0x140025d66–0x140025d7e). One short spike never
   triggers it; a sustained 0.9 s does.
5. **Restrain** (0x140025e7c onward):
   - Priority class → **Below normal** (0x4000), or **Idle** (0x40) with
     `LowerToIdleInsteadOfBelowNormal`, unless `DoNotLowerPriorityClass`. The current class is read
     first; a process already at Idle is left alone. The old class is saved.
   - Optionally **I/O priority → Very low** (`NtSetInformationProcess`, class 0x21 ProcessIoPriority,
     value 0), saving the old value (0x140026017–0x140026059; setter FUN_14004dfd0).
   - Optionally **Efficiency Mode** (EcoQoS), **affinity** (a configured mask, or "drop one random
     core"; skipped if the user customised the affinity), and **core parking off** while anything is
     restrained.
   - It logs "ProBalance begin" and turns the tray icon red. The start time is recorded with
     `GetTickCount`.
6. **Restore** (0x140026588–0x1400266d7). A restrained process is put back when:
   - its CPU use drops below the restore threshold **and** it has been restrained for at least
     `MinimumTimeOfRestraint` (4.2 s);
   - or `MaximumTimeOfRestraint` has passed, if set (default 0 = never). It may be restrained again
     on the next ticks if it still qualifies;
   - or it becomes the foreground process (log: "ProBalance ended because process is now in
     foreground");
   - or it exits.

   Restore puts back the saved priority class, and also the I/O priority, affinity and parking if
   those were changed. Each path has its own log action (`PL_ACTION_TOOK_PROBALANCE_END_*`, the name
   table built in FUN_140083bd0).

Only documented process APIs are used: `OpenProcess`, `SetPriorityClass`, `SetProcessAffinityMask`,
`NtSetInformationProcess` for I/O priority, and `SetProcessInformation` for EcoQoS. The governor
imports no `WriteProcessMemory`, `CreateRemoteThread`, `VirtualAllocEx` or hook APIs.

Bitsum's own claims match the code: on by default, Below normal by default, the foreground app is
never touched, temporary. Bitsum gives no numbers; the numbers above are from the binary.

## Performance Mode and "Bitsum Highest Performance" (BHP)

- INI section `GamingMode`: `GamingModeEnabled` (default false), `GamingChangePowerPlan` (true),
  `TargetPowerPlan` (BHP), `GamingModeEngageForSteam` (false), `AutomaticGamingModeProcessPaths`.
- What it does: while a chosen program runs (right-click "Induce Performance Mode", or Steam
  detection), it switches the active power plan to BHP and switches back when the last one exits. It
  changes **no process priorities** (Bitsum's BHP page). By default it also pauses IdleSaver
  (`DisableEnergySaverDuringGamingMode` default true).
- **Steam game detection** (FUN_140069ad0) treats executables under `steamapps\common\` as games. It
  skips helpers whose names contain `overlay`, `webhelper`, `steamservice`, `errorreporter`,
  `wrapper`, `vulkandriverquery`, `gldriverquery`, `dsx`, `epicservice`, plus SteamVR and
  Mixed-Reality driver folders and `boinc.exe`.
- **BHP creation** (FUN_140049600). If the BHP plan is missing, the engine duplicates **Ultimate
  Performance** (`e9a42b02-…-03f14749eb61`) or, if that is absent, **High performance**
  (`8c5e7fda-…`). It names the copy "Bitsum Highest Performance" ("Provides Bitsum optimized CPU
  performance.") and stores its GUID in `HKLM\Software\ProcessLasso\CustomHighPerfGUID`.
- **BHP contents** (FUN_140049920), processor subgroup `54533251-…`, AC and DC both:
  - Minimum processor state 100 % (`893dee8e-…`), so the CPU never clocks below base.
  - Core parking minimum cores 100 % (`0cc5b647-…583`, and `…584` for the second efficiency class),
    so parking is off.
  - Parking timing: performance check interval 15 ms (`4d2b0152`), core-parking increase time 1
    (`2ddd5a84`), increase policy 2 (`c7be0679`, all cores at once), decrease policy 0 (`71021b41`),
    decrease time 20 (`dfd10d17`). This is "unpark at once, park slowly", as in the ParkControl 6.0
    changelog.
  - Three more processor settings from the same table, with values 20, 1 and 100 (`7f2492b6`,
    `4009efa7`, `fddc842b`). Not identified.
  - On hybrid CPUs it also touches the heterogeneous thread-scheduling policy settings (`93b8b6dc`,
    `bae08b81`).
- `HKLM\Software\ProcessLasso` also keeps recovery GUIDs (`BitsumHighPerfRecoveryGUID`,
  `IdleSaverRecoveryGUID`, `AppPowerProfileRecoveryGUID`), so the previous plan is restored after a
  crash.

## IdleSaver, power plans per app, load-based switcher

- IdleSaver (INI section `PowerSaver`; GUI "IdleSaver"): after `PowerSaver_IdleTime` without user
  input it switches to a power-saving plan, and back on input. The loader's default idle time is 0
  (off); the installer's initial config may set it. Optionally only below a CPU threshold
  (`EnergySaverUseCPUThreshold`). Bitsum notes that Windows does not count controller input as
  activity, so IdleSaver can engage while you play with a gamepad.
- Application power profiles (`DefaultPowerSchemesEx`) switch the plan while a given exe runs.
- A load-based power switcher (`LoadBasedPowerSwitcher*`: low/high CPU thresholds, minimum time in
  plan, averaging window) is off by default.

## SmartTrim (memory)

INI section `MemoryManagement`: off by default (`SmartTrimIsEnabled` false). When on, at most every 15
minutes, and only at ≥ 65 % memory load, it:

- trims the working sets of processes using more than 196 MB (`SetProcessWorkingSetSize` to
  "empty", needs `SeIncreaseQuotaPrivilege`);
- optionally purges the standby list and the system file cache (`NtSetSystemInformation`, needs
  `SeProfileSingleProcessPrivilege`). By default the purge only runs during Performance Mode and when
  free RAM is below 1,024 MB.

It has an exclusion list.

## CPU Limiter, persistent rules, other process tools

- **CPU Limiter** (`CPULimitRules`). Not a duty-cycle throttle. Once a process stays over a CPU
  threshold for a configured time, it **reduces the process's affinity to fewer cores** for a
  configured period (resource text: "limiting processes to fewer CPU cores for a while"). It
  measures each process on its own or matching processes combined.
- **Persistent ("default") rules**: priority class (`DefaultPriorities`), affinity
  (`DefaultAffinitiesEx`), CPU Sets, I/O priority, memory priority, GPU priority, Windows GPU
  preference, Efficiency Mode, thread-priority-boost off. These are reapplied whenever a matching
  process starts.
- **Registry enforcement**: the engine can also write the documented IFEO `PerfOptions` values
  (`CpuPriorityClass`, `IoPriority`, `PagePriority`, `AffinityMask`). Windows itself then applies
  them at process start, which is how Bitsum suggests handling anti-cheat-protected games.
- Also present, all user-configured and off by default: instance limits, disallowed processes
  (terminate), watchdog rules (act at thresholds: restart, terminate, trim, priority, affinity),
  keep-running, foreground boost (`BoostForegroundProcess` off; Above normal when on), processor
  group extender, and "prevent sleep while X runs".
- Timer resolution: `SetTimerResolutionAtStartup`. InstallHelper also knows Windows 11's
  `GlobalTimerResolutionRequests` kernel value.
