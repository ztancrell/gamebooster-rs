# gamebooster-rs: behaviour spec

What Smart Game Booster 6.0 ("SGB") does when you press **Boost** and **Restore**, written so it can
be reimplemented from scratch. This file describes behaviour only. The audit trail is in the
Evidence section at the end; implementers don't need it.

Status: PowerMgr.dll is fully covered. SgbAccelerate.dll was still being decompiled when this was
written (3831 functions present), but its strings and exports show it is the online network
accelerator, not part of local Boost (see "Out of scope"). The Boost orchestration itself lives in
`SgbMain.exe` (Delphi). Its Boost/Restore dispatcher, power-plan step and service-list selection were
read from targeted disassembly; the rest comes from its string table (log messages, config keys,
registry paths, imported API names). The exe is protected: its Win32 API calls go through stubs that
are encrypted on disk, so which API each step calls could not always be proven, and some values it
writes are still unknown. Run `capture/` before and after a Boost to fill those gaps (see
"Not specified / unknown").

---

## 1. Triggers

| Trigger | Behaviour |
|---|---|
| Manual **Boost** button | Runs the Boost sequence (section 3) right away. |
| "Boost and Launch" or auto-boost when a game from the game list starts | Same sequence, then the game is launched or detected. Auto-boost (SVIP only) is watched by the tray program `SgbTray.exe`, not SgbMain: it spots the game process and tells SgbMain to boost (window message or `/boost`). Superseded: this row implied SgbMain watched for games. |
| Manual **Restore** button | Runs the Restore sequence (section 4). |
| Option "Restore to Normal Mode when the game is closed" | Runs Restore when the boosted game's process exits. |

There is one boost state, "boosted" or "normal". Boosting while already boosted is a no-op.
Everything needed to undo a boost is written to disk **before** the first change, so a later run can
still restore after a crash or reboot.

## 2. Actions

The table follows SGB's own order. "Default" says whether SGB treats the action as part of a normal
Boost. Items marked "user list" act only on what the user ticked in the Boost config dialog.

| # | Action | Boost does | Restore does | Default |
|---|---|---|---|---|
| A1 | Monitor preset | Sets the external monitor's DDC/CI picture preset (VCP 0xDC display mode, 0x14 colour preset, 0x10 brightness, 0x12 contrast) to the user's "gaming" preset. Saves the originals to `Monitor.ini` (`<id>_original`) first. | Writes back the saved values from `Monitor.ini`. | Opt-in. Not on XP. |
| A2 | GPU overclock | Raises GPU core and memory clocks by a level (Basic, Max or Custom) through `OverDrive.dll` (exports OverClock, Reset, CanOverClock, GetGPUInfo, GetRange; AMD ADL / NVIDIA NVAPI). Never beyond the range the driver reports; likely unsupported on a Boot Camp Radeon Pro. | `Reset` writes the default clocks back. | Opt-in. |
| A3 | Stop services | Stops the selected services, dependent services first, with a timeout per service. Records which ones failed to stop. | Starts the services it stopped. | User list, split into "System Services" (Windows) and "Third Services" (non-Microsoft). SGB ships default suggestions per Windows version. |
| A4 | Close processes | Takes a fresh process list and terminates each selected process (needs SeDebugPrivilege). Saves the full image path of each one to a temp file first. Optionally also terminates `explorer.exe` ("KillExplorer"). | Reads the temp file and starts each process again from its path. Always makes sure `explorer.exe` runs again. Some known apps get a startup argument (see 2.4). | User list, with suggestions from a score database. |
| A5 | Free RAM | Trims the working set of every process it can open (SetProcessWorkingSetSize(-1, -1); Superseded: earlier said EmptyWorkingSet, see 2.4). Reports the MB freed. | Nothing (not reversible). | On (SGB's default; gamebooster-rs ships it off, see docs/tweaks/ram-trim.md). |
| A6 | Clear clipboard | Empties the clipboard (plain VCL `Clipboard.Clear`; nothing saved). | Nothing. | On. |
| A7 | Power plan | Switches to a dedicated high-performance plan (section 2.1). | Re-activates the plan that was active before and deletes the game plan. | On. |
| A8 | Windows automatic updates | Saves the current automatic-update notification level, then sets it to "disabled" (2.5). | Writes back the saved level. | On. |
| A9 | Windows Game Mode | Sets `HKCU\Software\Microsoft\GameBar` value `AutoGameModeEnabled` (REG_DWORD) to 1. Saves the old value or notes that it was absent. | Writes back the old value, or deletes the value if it was absent. | On. |
| A10 | Game process priority | When the boosted game runs, sets its priority class to the user's choice: Realtime, High, Above normal, Normal, Below normal or Low. Never changes `steam.exe`, `explorer.exe` or SGB's own processes. | Sets the game back to Normal, or leaves it alone if it has exited. | Per game, opt-in. |
| A11 | Tidy tray icons | **No-op in SGB 6.0**: only logs "Remove Dead Tray Icons" (SgbMain.exe 0x7c7a2e, `decomp/SgbMain.exe/features.md`). Do not implement. Superseded: said it removed dead icons after A4. | Nothing. | Cosmetic. |

Not part of Boost (separate SGB features, do not implement as Boost): the TCP/IP registry tweaks
(TcpTimedWaitDelay, TcpMaxDupAcks, TcpWindowSize, Nagle, MaxCPS), the SSD/defrag and Windows Search
"system tweaks", "Game Desktop" (a separate virtual desktop), "Unlock Roblox FPS", the FPS/temperature
HUD, driver updates and the browser "Internet Booster".

### 2.1 Power plan (A7), the one fully specified action

Uses the Windows power API in `powrprof.dll` (PowerGetActiveScheme, PowerEnumerate,
PowerReadFriendlyName, PowerDuplicateScheme, PowerWriteFriendlyName, PowerWriteDescription,
PowerWriteACValueIndex, PowerWriteDCValueIndex, PowerReadACValueIndex, PowerReadDCValueIndex,
PowerSetActiveScheme, PowerDeleteScheme). Requires Windows Vista or later; the XP path is not
needed.

**Boost:**
1. Read the active scheme GUID and save it to the restore state as the "base scheme".
2. Read the active scheme's display brightness (subgroup SUB_VIDEO, setting VIDEO_BRIGHTNESS), both
   the AC and the DC value. They are carried into the new plan so the screen doesn't jump.
3. Delete every existing scheme named exactly "Smart Game Boost Power Plan" (case-insensitive; at
   most 1024). This cleans up leftovers from an earlier run. If one of them is active, first
   activate Balanced `{381B4222-F694-41F0-9685-FF5BB260DF2E}`.
4. Duplicate **High performance** `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c` into a new scheme. Name it
   "Smart Game Boost Power Plan", with description "Activate it when you switch to Gaming Mode for
   boosting." (Our implementation may pick its own name.) If High performance is missing, as on many
   Modern Standby systems, SGB's plan creation fails. Ours should fall back to duplicating Balanced
   and still write all the values below.
5. Write these values to the new scheme, AC and DC both, unless the table says otherwise:

| Subgroup | Setting | Value | Meaning |
|---|---|---|---|
| SUB_PROCESSOR `54533251-82be-4824-96c1-47b60b740d00` | PROCTHROTTLEMIN `893dee8e-2bef-41e0-89c6-b55d0929964c` | 100 | Minimum processor state 100 %. |
| SUB_PROCESSOR | PROCTHROTTLEMAX `bc5038f7-23e0-4960-96da-33abaf5935ec` | 100 | Maximum processor state 100 %. |
| SUB_PROCESSOR | SYSCOOLPOL `94d3a615-a899-4ac5-ae2b-e4d8f634367f` | 1 | Cooling policy active (fans before throttling). |
| SUB_PCIEXPRESS `501a4d13-42af-4429-9fd1-a8218c268e20` | ASPM `ee12f906-d277-404b-b6da-e5fa1a576df5` | 0 | PCIe link state power management off. |
| ATI graphics power `f693fb01-e858-4f00-b20f-f30e12ac06d6` | ATI PowerPlay `191f65b5-d45c-4a4f-8aae-1ab8bfd980e6` | 1 | AMD PowerPlay "maximize performance". Exists only with AMD drivers that publish it. |
| SUB_DISK `0012ee47-9041-4b5d-9b77-535fba8b1442` | DISKIDLE `6738e2c4-e8a5-4a42-b16a-e040e769756e` | 0 | Never turn off the disk. |
| Wireless adapter `19cbb8fa-5279-450e-9fac-8a3d5fedd0c1` | Power saving mode `12bbebe6-58d6-4636-95bb-3217ef867c1a` | 0 | Wi-Fi Maximum Performance. |
| SUB_SLEEP `238c9fa8-0aad-41ed-83f4-97be242c8f20` | STANDBYIDLE `29f6c1db-86da-48c5-9fdb-f2b67b1f44da` | S | Sleep after S seconds. |
| SUB_SLEEP | HIBERNATEIDLE `9d7815a6-7ee4-497e-8888-515a05f02364` | S | Hibernate after S seconds. SGB writes only the DC value (an apparent slip); write both. |
| SUB_BUTTONS `4f971e89-eebd-4455-a8de-9e59040e7347` | LIDACTION `5ca83367-6e45-459f-a27b-476b1d01c936` | 0 | Closing the lid does nothing. |
| SUB_NONE `fea3413e-7e05-4911-9a71-700331f1c294` | CONSOLELOCK `0e796bdb-100d-47d6-a2d5-f7d2daa51f51` | 0 | No password on wake. |
| SUB_VIDEO `7516b95f-f776-4464-8c53-06167f40cc99` | VIDEO_BRIGHTNESS `aded5e82-b909-4619-9949-f5d71dac0bcb` | AC: saved AC value; DC: saved DC value | Keep the current brightness. |
| SUB_VIDEO | VIDEOIDLE `3c0bc021-c8a8-4e07-a973-6b14cbcb2b7e` | 0 | Never turn off the display. |
| SUB_VIDEO | VIDEODIM `17aaa29b-8b43-4b94-aafe-35f64daaf1ee` | 0 | Never dim the display. |
| SUB_VIDEO | ADAPTBRIGHT `fbd9aa66-9553-4097-ba44-ed6e9d65eab8` | 0 | Adaptive brightness off. |
| SUB_VIDEO | `90959d22-d6a1-49b9-af93-bce885ad335b` | 0 | Unidentified hidden display setting. Skip it if the system doesn't have it. |
| Multimedia `9596fb26-9850-41fd-ac3e-f7c3c00afd4b` | When sharing media `03680956-93bc-4294-bba6-4e0f09bb717f` | 1 | Prevent idle sleep while sharing media. |
| Multimedia | When playing video `34c7b99f-9a6d-4b3c-8dc7-b6693b78cef4` | 0 | Optimize video quality. |

   gamebooster-rs note: our plan does not take the LIDACTION row; it copies the user's lid, power
   button and sleep button actions instead (see [tweaks/power-plan.md](tweaks/power-plan.md)).

   S is a sleep timeout the main app passes in, in seconds. SGB passes **0** (never). It also passes
   0 as the display-off timeout, which matches VIDEOIDLE = 0 above.
   Superseded: earlier versions of this spec said S was unknown and recommended 0 as an assumption.

   A setting that doesn't exist on this machine (for example the ATI one on Intel or NVIDIA
   graphics) is skipped, not treated as an error.
6. Activate the new scheme.

**Restore:** activate the saved base scheme, then delete the game plan. If the base scheme no
longer exists, activate Balanced.

### 2.2 Services (A3)

- Stop each selected service through the Service Control Manager: OpenSCManager, then
  ControlService with SERVICE_CONTROL_STOP. Stop the services that depend on it first. Wait for each
  with a timeout.
- Only services that were **running** when Boost started are stopped and remembered. Restore starts
  exactly those, in reverse order.
- The startup type is never changed. All three ChangeServiceConfigW calls in SGB are outside the
  service boost: one sets the Task Scheduler service's start type, and two set the firewall
  (SharedAccess/MpsSvc) and BFE to Automatic before starting them, which is for the network
  accelerator. Ours must not change start types either.
  Superseded: earlier versions said the caller could not be traced, so a start-type change couldn't
  be ruled out.
- SGB never offers services in its own safe list or protected security-product services. Our
  implementation needs an explicit allow-list, and services not on it are never touched.
- SGB picks a default list by Windows version, from INI sections named `Services_2000`,
  `Services_XP`, `Services_VISTA`, `Services_WIN7`, `Services_WIN8` and `Services_WIN10` (Windows 11
  reports itself as 10 and gets the WIN10 list). The lists are a plain-text INI the installer
  drops as `%APPDATA%\PCGameBoost\Smart Game Booster\WinSvc.ini`; `SgbInit.exe` copies it to
  `UserWinSvc.ini`, which the service boost reads. Value 1 = ticked (selected by default), 0 =
  listed unticked. There is no `Services_2000` section. Win10/11 ticks 61 services (e.g.
  `RemoteRegistry`, `Fax`, `WerSvc`, `WinRM`, `W32Time`, `FontCache`, `StiSvc`, `PrintNotify`,
  the `vmic*` Hyper-V guest services); many are legacy and absent on Windows 11. Full lists per
  version: `decomp/SgbData/README.md`. Ours ships its own explicit list.
  Superseded: earlier versions said the lists were encrypted or downloaded and the names unknown.

### 2.3 Processes (A4)

- Before killing anything, write each target's full image path, plus any startup argument, to the
  restore state.
- Kill with TerminateProcess (OpenProcess with PROCESS_TERMINATE, after enabling SeDebugPrivilege).
- Never kill: the game, its launcher (e.g. the EA app, Steam), SGB itself, Windows core processes,
  or security software. SGB has a hard-coded list of Symantec/Norton components it protects, among
  them `ccSvcHst.exe`, `Smc.exe`, `SmcGui.exe` and `SepLiveUpdate.exe`.
- Adobe Reader (`AcroRd32.exe`) is handled specially: its "renderer" child processes are part of the
  same app.
- `explorer.exe` is killed only when the user opts in. Restore always relaunches it if it isn't
  running.
- On restore, some known apps are started with an argument: `SmartDefrag.exe /STARTUP`,
  `ASCTray.exe /auto`, `IOTransfer.exe /autorun`. Everything else is started with no arguments.
- SGB keeps user-editable lists `UserProcWL.txt` and `UserProcBL.txt` (white- and blacklist) next to
  `UserWinSvc.ini`. The installer ships no default for either, so SGB has no built-in default
  kill list from a data file (see `decomp/SgbData/README.md`).

### 2.4 RAM (A5)

For every process in a Toolhelp snapshot, call SetProcessWorkingSetSize(handle, -1, -1), which
trims its working set like EmptyWorkingSet, on a handle opened with PROCESS_SET_QUOTA. Skip a
process that can't be opened, silently. SGB skips only its hard-coded list of 31 Symantec Endpoint
Protection processes (the same list as in 2.3, compared case-insensitively); ours also skips the
game. Measure free physical memory (GlobalMemoryStatusEx) before and after, for the report. There is
nothing to restore.
Superseded: earlier versions said EmptyWorkingSet on processes opened with PROCESS_SET_QUOTA |
PROCESS_QUERY_INFORMATION. SGB loads EmptyWorkingSet but never calls it.

### 2.4a Other processes' priority (part of A10)

SGB 6.0 contains code (TPriorityController) that would lower **other** processes to
IDLE_PRIORITY_CLASS (0x40), saving each one's class first so Restore can write it back. Its targets
would be processes owned by the current user, except `explorer.exe`, `steam.exe` and every child of
steam.exe, `SgbTray.exe`, `SgbMain.exe`, `SgbMini.exe`, SGB's own exe and the 31 Symantec processes;
the game is **not** excluded. But **nothing in SgbMain ever calls it**: Boost dispatch skips the
object, and the object is only used by the Boost settings dialog to store `[Boost] Priority` (on/off)
and `[Boost] PriorityClass` (default 3), the "Set the priority of game when running" option (evidence:
SgbMain.exe 0x7c8acc, `decomp/SgbMain.exe/features.md`). The game-priority setting is
probably applied by `SgbTray.exe`, which imports SetPriorityClass and reads the same two keys (strings
only). So lowering other processes is **not** something SGB does; ours is an extra.
Ours: opt-in, a configurable class (default BELOW_NORMAL, IDLE starves voice chat and similar), and it
excludes the game, its launcher and their children.
Implemented as `lower_others` (opt-in, only processes at NORMAL, PID-reuse-safe restore); see
[docs/tweaks/priority.md](tweaks/priority.md#lower-other-processes).
Superseded: this section used to say SGB lowers other processes to IDLE while boosted. The code
exists but has no caller (decomp/SgbMain.exe/notes.md, second pass, left that open).

### 2.5 Windows automatic updates (A8)

SGB goes through the Automatic Updates COM object (CLSID `bfe18e9c-6d87-4450-b37c-e02f0b373803`,
IAutomaticUpdates / IAutomaticUpdatesSettings):

1. Read the settings.
2. Stop without changing anything if the service is disabled, the settings are read-only (policy
   managed, which is the norm on Windows 10/11) or required.
3. Otherwise save the current NotificationLevel to the restore state (key "PreviousStaus" under
   "WindowsUpdate") and set it to "disabled".

Restore writes back the saved level. On Windows 10/11 these settings are usually read-only, so the
action normally does nothing there.

---

## 3. Boost order

1. Refuse if already boosted.
2. Create the restore-state file. SGB only logs a failure here and carries on. Ours should abort
   before changing anything (safer). Superseded: an earlier version of this spec said SGB aborts.
3. A1 Monitor, A2 Overclock, A3 Services, A4 Processes, A5 RAM, A6 Clipboard, A7 Power plan,
   A8 Windows Update, A9 Game Mode.
4. A11 Tidy tray icons (log line only, no-op in 6.0).
5. Write a boost report (items changed, MB freed), mark the state "boosted" and save a timestamp.
6. A10 Priority runs when the game process appears or is launched.

Each action is independent and runs only if it is enabled. One failing is logged and the rest
continue.

Power-plan step order inside A7, as SGB does it: save the active scheme as the base scheme, delete
old plans with the game-plan name, set display-off and sleep timeouts to 0, read the current AC and DC
brightness, then create and activate the game plan with those values, then record in its config that
a custom plan is active.

SGB writes no timer-resolution, process-affinity, MMCSS (`Multimedia\SystemProfile`), network
throttling, `SystemResponsiveness` or file-cache settings: none of those APIs or registry names occur
in it. It only reads `Win32PrioritySeparation` for a diagnostics log.

## 4. Restore order

SGB undoes the actions in the **same** order as Boost: Monitor, Overclock, Services, Processes, RAM
(no-op), Clipboard (no-op), Power plan, Windows Update, Game Mode (confirmed from its dispatcher, not
just its logs). Then it marks the state "normal", deletes the restore state and shows a before/after
RAM and CPU chart.

Our implementation should use reverse order (A9 to A1) instead. Same result, and safer when actions
depend on each other.

---

## 5. Kernel drivers in the installer

`nDrivers/itvwd64.sys` and the zipped `Update/*.dat` files (itvwd32/64/arm64.sys) contain a rebranded
**WinDivert** packet-capture driver (device `\Device\itvwd`, built from `windivert.c`). Only the
network accelerator (`SgbAccelerate.dll`, `WinDivert.dll`) uses it to redirect game traffic. Boost
does not use it, and **no kernel driver is needed** for anything in this spec.

## 6. Out of scope

- Network acceleration (`SgbAccelerate.dll`): a UDP/TCP tunnel to the vendor's proxy servers, built
  on WinDivert, libhv and KCP, with filters for DNS, HTTP, HTTPS, P2P and BitTorrent and a kill
  switch. It needs the vendor's servers.
- Licensing, activation and registration; ads and promotions; telemetry and statistics uploads;
  update checks and the updater; the online process score database. These exist; nothing more is
  said about them.
- GPU overclock (A2): vendor-specific and risky. Leave it out unless asked for.
- Monitor DDC/CI (A1): the built-in display of a Boot Camp Mac does not support DDC/CI.

## 7. Ranking by likely impact (CPU/GPU-bound game, Boot Camp Mac, BF6)

1. **A7 Power plan.** Boot Camp installs default to Balanced, where the CPU parks and downclocks
   between frames. Minimum processor state 100 %, active cooling and ASPM off remove those latency
   spikes. On Macs with AMD Radeon graphics, the ATI PowerPlay "maximize performance" setting also
   keeps the GPU from downclocking. This alone is the most likely reason Boost made BF6 playable.
2. **A4 Close processes and A3 Stop services.** Frees CPU time and RAM taken by background apps
   (updaters, cloud sync, Apple Boot Camp helpers the user picks). The gain depends on what is
   running; it is large on a 4-core Mac.
3. **A10 Game priority (High).** Helps when background load competes for CPU. Avoid Realtime: it
   can starve input and audio.
4. **A9 Game Mode.** Already on by default in Windows 10/11, so usually a no-op. Cheap to make sure.
5. **A8 Windows Update.** Prevents download and install spikes in theory, but is usually read-only on
   Windows 10/11 and then does nothing.
6. **A5 Free RAM.** Helps only if RAM is nearly full. It can briefly hurt, since trimmed pages fault
   back in.
7. **A6 Clipboard, A11 Tray icons, A1 Monitor.** No measurable effect.

## 8. Not specified / unknown

Fill these from a `capture/` diff (snapshot before and after Boost on the real machine):

- Superseded (answered by static analysis, see decomp/SgbMain.exe/notes.md, second pass): services
  are never set to Disabled or Manual (2.2); A10's "lower other processes to IDLE" code is never called (2.4a, superseded); A5 uses
  SetProcessWorkingSetSize(-1, -1) and skips only the Symantec list (2.4); the GUID
  `{58231417-…}` is only a lookup key. The "protector" that seemed to hide API calls did not exist:
  it was an extraction error, and every API call site can now be traced.
- Superseded (answered): auto-boost game watching is in `SgbTray.exe` (SVIP), which tells SgbMain to
  boost; A11 is a no-op; A1 saves VCP 0xDC/0x14/0x10/0x12 to `Monitor.ini`; A2 goes through
  `OverDrive.dll` within the driver's range; A6 is a plain clear. See `decomp/SgbMain.exe/features.md`
  and `docs/sgb/`.
- Superseded (answered): nothing calls TPriorityController's Boost; it is dead code, and the game was
  never excluded from its list (2.4a). Still open: where the game's own class (A10, `[Boost]
  PriorityClass`) is applied. SgbMain's only SetPriorityClass call is in the dead code, so most likely
  in `SgbTray.exe` (imports SetPriorityClass, reads the same keys; not read in code).
- Superseded: the default service list per Windows version
  is now known (plain-text `WinSvc.ini`, see 2.2 and `decomp/SgbData/README.md`).
- The default process list. The installer ships no default `ProcWL.txt`/`ProcBL.txt` (SgbInit would
  copy them to `UserProcWL.txt`/`UserProcBL.txt`), so it comes from the user's choices and possibly
  a download or online score database.
- Registry values written beyond `AutoGameModeEnabled`: none found (the registry writers were
  not audited one by one). Timer resolution, affinity, MMCSS and network throttling are ruled out
  (no API or string for them).
- Superseded: the sleep timeout S is now known (0), see 2.1; `Win32PrioritySeparation` is confirmed
  read-only.
- The exact identity of power setting `90959d22-d6a1-49b9-af93-bce885ad335b`.
- Superseded (answered): GUID `{58231417-2B7B-4575-8231-96CB9CF58B36}` is passed to PowerMgr's
  SetGamePwrCFG, which uses it only to find an existing plan with that GUID (or the plan name) and
  reuse it. A new plan is duplicated from High performance with a Windows-assigned random GUID. Ours
  can ignore it.
- SgbAccelerate.dll is only partly decompiled, on purpose; it is out of scope (section 6).
- Superseded: "`SgbMain.exe` was only partly analysed". Every feature is now surveyed
  (`decomp/SgbMain.exe/features.md`), but only Boost/Restore was read in detail; error handling and
  timeouts of A1, A2, A8 and A9 are inferred from log strings only.

---

## Evidence

Paths under `decomp/` are relative to the repo root. "SgbMain strings" means the static UTF-16 string
table of `sgb-extract/app/cmAppVer/SgbMain.exe`. Nothing was executed.

| Item | Source |
|---|---|
| PowerMgr exports (GetCurrentPwrCFG, SetPwrCFG, SetGamePwrCFG, DeleteGamePwrCFG, SetSleepTimeValue, SetDisplayTimeValue, GetCurrentDisplayBrightness, DeletePwrCFGByName) | PowerMgr.dll exports 0x10001010 … 0x10001100 |
| powrprof functions resolved at runtime; which pointer is the AC writer and which the DC writer | PowerMgr.dll strings (file offsets 0xac44–0xad58); disassembly of the resolver at VA 0x100012e0–0x10001457 (DC writer → 0x1000ddec, AC writer → 0x1000ddf0) |
| Power GUID table (VA 0x1000b9b4–0x1000bb14, 0x1000bd98–0x1000be18) | PowerMgr.dll .rdata, file offset = VA − 0x10001200 |
| Plan values written (processor, ASPM, ATI, disk, Wi-Fi, sleep, lid, console lock, video, multimedia) | PowerMgr.dll 0x10001a30 (full set); PowerMgr.dll 0x10001b5a (overlapping variant; Ghidra split the code at the wrong boundaries) |
| Sleep timeout value supplied by the caller | PowerMgr.dll 0x10001090 |
| Duplicate from High performance, then write name and description | disassembly VA 0x10001956–0x10001a1a (DuplicateScheme on 0x1000b9b4, then WriteFriendlyName and WriteDescription) |
| Find scheme by name, then activate | PowerMgr.dll 0x100016f0; disassembly VA 0x100017e1–0x100018fb |
| Delete scheme(s) by name; Balanced fallback | PowerMgr.dll 0x10001100 (Balanced GUID string at 0x1000bd48); PowerMgr.dll 0x10002100 |
| Read brightness of the active scheme | disassembly VA 0x100021a0–0x1000222e |
| Plan name and description; base scheme saved and restored | SgbMain strings "Smart Game Boost Power Plan", "Activate it when you switch…", "BasePwrSchID", "TPowerSch.Boost 1–11", "TPowerSch.Restore 1–7" |
| Boost and Restore order | SgbMain strings "Boost Monitor … Boost GameMode", "Restore Monitor … Restore GameMode", "TBoost.Execute 4 osBoost / 5 osRestore" |
| Processes: temp file, kill, restart, explorer, special arguments, protected list | SgbMain strings "ProcessBoost.Boost [Begin KillProcess %s]", "ReadProcessFromTempFile", "StartProcessByPath", "KillExplorer", "UserProcBL.txt", "AcroRd32.exe", "renderer", the restart-argument list, the Symantec process list; imports TerminateProcess |
| Services | SgbMain strings "Stop Service(Dependent)", "Start Service", "SetServiceStop Time Out", "Services_WIN10", "System Services", "Third Services", "ServiceBoost.ini", "StopServiceFail"; imports OpenSCManagerW, ControlService, StartServiceW, ChangeServiceConfigW |
| RAM | SgbMain strings "EmptyWorkingSet", "GlobalMemoryStatusEx", "Released RAM"; class TRAMClear |
| Clipboard | SgbMain strings "TClipbrd.Boost", "Clear unused clipboard"; import EmptyClipboard; code 0x7b7bc4 (VCL Clipboard.Clear) |
| Tray icons no-op | SgbMain 0x7c7ae9: "Remove Dead Tray Icons" only passed to the debug log, TBoost.Execute 0x7c7a2e |
| Auto-boost watcher | SgbTray strings `OnProcessStart`, `TGameMonitorInfo`, `FGameMonitorList`, "Upgrade to SVIP"; SgbMain `WMBoost`, `/boost` |
| Game Mode | SgbMain strings "Software\Microsoft\GameBar", "AutoGameModeEnabled", "GameMode" |
| Windows Update | SgbMain strings "TUpdateBoost.Boost", "save windows update state to file", "PreviousStaus", "ServiceEnabled Exit", "ReadOnly Exit", "Required Exit"; CLSID_AutomaticUpdates at file offset 0x64aa6c, IID_IAutomaticUpdates at 0x64aa5c, IID_IAutomaticUpdatesSettings at 0x3c17b0 |
| Priority | SgbMain strings "Process Priority", "PriorityClass", "Realtime/High/Above normal/Normal/Below normal/Low", exclusions "steam.exe", "explorer.exe", "SgbTray.exe", "SgbMain.exe", "SgbMini.exe"; imports SetPriorityClass, GetPriorityClass |
| Monitor DDC/CI | SgbMain strings "TFrameMonitorDDC.Boost", "VCP_DC", "VCP_14", "Monitor.ini"; Boost 0x7c07bc, Restore 0x7c0f7c, VCP get/set 0x7bd5e0 |
| Overclock | SgbMain strings "TOverClockBoost", "OverDrive.dll", "OC_GPU", "OC_RAM"; loader 0x7b8da4 (GetProcAddress OverClock/Reset/CanOverClock/GetGPUInfo/GetRange); `decomp/OverDrive.dll/notes.md` |
| Network accelerator, WinDivert | SgbAccelerate.dll strings ("[CClientP2S]…", "vpnenv_firewall", WinDivertOpen/Recv/Send); decomp/SgbAccelerate.dll/notes.md (full decompile, deleted); itvwd64.sys strings `\Device\itvwd`, `windivert.c` |
| SgbMain targeted disassembly (method, limits; the "protector" was an extraction error) | decomp/SgbMain.exe/notes.md; decomp/README.md |
| Boost/Restore dispatch order; temp-file failure only logged | SgbMain.exe 0x7c6f0c |
| Power step order; display and sleep timeouts 0; GUID {58231417-…} passed to SetGamePwrCFG | SgbMain.exe 0x7b81f4 (VA 0x7b82f8 and 0x7b830b push 0); PowerMgr.dll 0x10001090, 0x100010a0 |
| Default service INI sections per Windows version | SgbMain.exe 0x78754c |
| Default service lists (plain-text WinSvc.ini, copied to UserWinSvc.ini by SgbInit.exe VA 0x6575e0); no default ProcWL/ProcBL in the installer; .dbd files are not Boost data | decomp/SgbData/README.md, decomp/SgbData/winsvc_lists.py |
| Win32PrioritySeparation is read only (diagnostics) | SgbMain.exe 0x473324 |
| No timer-resolution, affinity, MMCSS, network-throttling APIs or strings | SgbMain import table and string tables (searched for timeBeginPeriod, NtSetTimerResolution, SetProcessAffinityMask, SystemProfile, NetworkThrottlingIndex, SystemResponsiveness, SetSystemFileCacheSize) |
| Other processes lowered to IDLE (0x40), saved class restored; exclusions | SgbMain.exe 0x7c4278 (0x7c42e1 mov ecx,0x40 → SetPriorityClass at 0x7c34d5; list built at 0x7c3a9c) |
| ...but that Boost has no caller (dead code); the object only stores `[Boost] Priority`/`PriorityClass` | SgbMain.exe 0x7c8acc (VMT refs only; TBoost+0xa0 never dispatched by 0x7c6f0c) |
| RAM: SetProcessWorkingSetSize(-1,-1), Symantec skip list; EmptyWorkingSet never called | SgbMain.exe 0x7b7768 (0x7b781f; table 0xa4b7b0; pointer 0xa5e6a4 written only) |
| No start-type change in service boost | SgbMain.exe 0x5bf498 (all 3 call sites: 0x5bf498, 0x81c667, 0x9c63b2) |
| GUID {58231417-…} is a lookup key | SgbMain.exe 0x7b8349 (CLSIDFromString 0x100017ab, compare 0x10002250, duplicate with NULL GUID 0x10001950) |
| Win32PrioritySeparation is only logged | SgbMain strings: it appears next to "PagingFiles=" and "MB(free/total)" diagnostic text |
