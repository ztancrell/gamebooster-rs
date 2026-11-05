# SgbMain.exe notes (targeted Boost/Restore only)

Target: `sgb-extract/app/cmAppVer/SgbMain.exe` (9,008,696 bytes, Delphi 2009 / RTL120, i386). The
`sgb-extract/app/SgbMain.exe` copy (502,328 bytes) is a small Delphi launcher stub, not the app.

## Method and limits
- Ghidra via REA did not finish import + auto-analysis of this exe within its 30-minute startup
  deadline (this is what stalled the previous agent). Rizin is not installed. So there is no Ghidra
  pseudocode: the functions below were read as `objdump -M intel` disassembly, annotated by
  `dis.py` (string literals, import names). `refs.py` finds code that references a string (VA =
  file offset + 0x400C00 in .text). Both scripts are kept here so the output can be regenerated
  locally; the disassembly itself is not kept (see `../README.md`, 'Policy').
- The exe is **protected**: many helper calls and every Win32 API call site observed go to stubs at
  VA 0xb4xxxx–0xb8xxxx, a region inside the `.rsrc` section range whose bytes are encrypted on disk
  (no `E8` call reaches the normal `FF 25` import thunks of TerminateProcess, ControlService,
  ChangeServiceConfigW, SetPriorityClass, SetProcessWorkingSetSize, EmptyClipboard). Which API a
  given stub calls therefore can't be proven statically; a stub's identity is inferred from its
  arguments and the surrounding log strings.

- Delphi class layout (from `features.md`): the VMT self pointer is at **VMT−0x58** (class name at
  VMT−0x38, parent at VMT−0x30). Scanning with −0x58 finds 1,466 classes. Superseded: an earlier
  pass used −0x4C, which finds only 113.
- Superseded: this file covers Boost/Restore only. Every other SgbMain feature (monitor DDC, overclock,
  clipboard, tray no-op, game detection, CYRI, startup manager, ...) is in `features.md`.

## Functions read
| Function (VA) | What |
|---|---|
| 0x7c6f0c | Boost/Restore dispatcher. |
| 0x7b81f4 | Power-plan Boost (steps "TPowerSch.Boost 1–11"). |
| 0x78754c | Picks the INI section of default services by Windows version. |
| 0x788a58 | ServiceBoost.ini / "StopServiceFail" bookkeeping. |
| 0x473324 | Diagnostics: reads PriorityControl, Memory Management (PagingFiles), SubSystems; logs them. |
| 0x5458a0 | 0x5458a0 itself is the debug-log function (UnicodeString → `OutputDebugStringW`, ends ≈0x5458f0); the PSAPI loader (`LoadLibraryW("PSAPI.dll")`, GetProcAddress of EnumProcesses, EmptyWorkingSet, ...) is the next function, ≈0x5458f4. Superseded: this row called 0x5458a0 the PSAPI loader. Not the RAM boost itself. |

## Findings
- **Order** (0x7c6f0c): Boost = Monitor, OverClock, Service, Process, RAM, Clipboard, Power Schemes,
  AutoUpdate, GameMode, each a virtual Boost method run only if its sub-object exists. Restore =
  the same order (virtual Restore method). A temp file is created first; on failure it logs
  "CreateTempFile Err:" and **continues** (no abort). After Restore it logs "osRestore Over", then
  computes a before/after RAM/CPU chart for the report.
- **Power plan** (0x7b81f4): 1 log; 2 read active scheme, save as config `Power/BasePwrSchID`;
  3 DeletePwrCFGByName(plan name); 4 one-arg PowerMgr setter called with **0** (SetDisplayTimeValue);
  5 one-arg setter called with **0** (SetSleepTimeValue); 6 read AC/DC brightness
  (GetCurrentDisplayBrightness, two out-params); 7 SetGamePwrCFG(name, description,
  "{58231417-2B7B-4575-8231-96CB9CF58B36}", AC brightness, DC brightness); 8–10 write config
  `Power/Custom`=1 and a "Power Scheme" flag. The two zero calls match the import order and the
  PowerMgr exports (both are single-argument setters storing a global) → sleep timeout S = 0 and
  display timeout = 0, high confidence. The GUID is passed to PowerMgr as a parameter of SetGamePwrCFG.
- **Default services** (0x78754c): one INI section per OS name: Windows 2000 → `Services_2000`,
  XP → `Services_XP`, Vista → `Services_VISTA`, 7 → `Services_WIN7`, 8 → `Services_WIN8`,
  10 → `Services_WIN10` (Windows 11 reports as 10). The INI is `UserWinSvc.ini`, read at VA
  0x7885d0. Superseded: this note said the INI text was in no extracted file; it is the plain-text
  `WinSvc.ini` installed to `%APPDATA%` (the earlier extraction missed that user-file entry), see
  `../SgbData/README.md`.
- **Service start type**: ChangeServiceConfigW is imported, but its callers are behind the
  protector stubs; no log string mentions a start-type change. Still unknown.
- **Win32PrioritySeparation**: only read (diagnostic log together with PagingFiles and the
  SubSystems Windows string), never written in that function.
- **Not present at all** (no import, no string): timeBeginPeriod / NtSetTimerResolution,
  SetProcessAffinityMask, MMCSS `Multimedia\SystemProfile`, NetworkThrottlingIndex,
  SystemResponsiveness, SetSystemFileCacheSize, NtSetSystemInformation.
- **RAM**: EmptyWorkingSet (via dynamically loaded PSAPI) and SetProcessWorkingSetSize are both
  available; which one the RAM boost calls and on which processes is behind the protector. Unknown.
- **Other-process priority**: SetPriorityClass callers are behind the protector. Unknown.

## Second pass (2026-11-05): correct re-extraction; priority, RAM, services, GUID answered

**Superseded: the "protector" finding above was wrong.** The first copy of SgbMain.exe had wrong
`E8`/`E9` call targets, most likely because Inno Setup's call-instruction filter was not undone (or
was undone wrongly) when it was extracted. The supposed stubs at 0xb4xxxx–0xb8xxxx were bogus call
targets (old `0x7b81f4` listing: `call 0xb6f5df`; correct copy: `call 0x7b7eb4` →
GetCurrentDisplayBrightness). The copy used now was carved by `decomp/inno_carve.py` (raw LZMA1 solid
chunk, then the inverse Inno call filter in 64 KiB blocks). It is
`/home/zpt/re/sgb-extract/app/cmAppVer/SgbMain.exe`, 9,008,696 bytes, sha256 `060f6621…d16f`. In it,
every API call goes straight to its `FF 25` import thunk, and only 2 `E8` calls in the whole of
.text land in 0xb4xxxx–0xb8xxxx. The six earlier disassembly listings were regenerated from this copy, so
their call targets are now right. With this copy, Ghidra via REA finished auto-analysis in 183 s
(startup limit raised to 90 min, but it wasn't needed).

Direct callers per import (E8 → thunk): SetPriorityClass 1 (0x7c34d5), SetProcessWorkingSetSize 1
(0x7b781f), ChangeServiceConfigW 3 (0x5bf498, 0x81c667, 0x9c63b2), ControlService 1 (0x7874ab),
TerminateProcess 12, OpenSCManagerW 11, StartServiceW 4, EmptyClipboard 4.

Functions read in this pass: 0x7c4278 (TPriorityController Boost/Restore, IDLE), 0x7b7768 (RAM clear,
SetProcessWorkingSetSize), 0x5bf498 (ChangeServiceConfigW callers, not Boost), 0x7b8349 (game scheme
GUID, PowerMgr lookup).

- **Service start types (high confidence): Boost never changes them.** There are only three
  ChangeServiceConfigW calls. 0x5bf498 is a generic set-start-type helper (fn ~0x5bf360, dwServiceType
  0x110), called only for the Task Scheduler service `"Schedule"` (0x5bef80, 0x5befb9). 0x81c667
  (SharedAccess/MpsSvc, i.e. the firewall) and 0x9c63b2 (BFE) both push start type 2
  (SERVICE_AUTO_START) followed by StartServiceW. Those are "make sure the firewall is running"
  helpers, for the accelerator. The service Boost unit (OpenSCManagerW at 0x787250…0x787af2,
  ControlService 0x7874ab, StartServiceW 0x78739d) has no ChangeServiceConfigW call. Nothing passes
  "ChangeServiceConfigW" to GetProcAddress either: the only runtime-resolved name is
  ChangeServiceConfig2W, used for descriptions.
- **Priority (high confidence): Boost lowers other processes to IDLE.** TPriorityController
  (VMT 0x7c4454) is a subclass of TPsPriorityBoost (VMT 0x7c33cc) with the same virtual methods.
  TBoost.Create stores it at TBoost+0xa0 (0x7c676f). Boost 0x7c4278 builds the list (0x7c3a9c), saves
  it (0x7c38b4, category "Priority"), then for each entry calls `SetPriorityClass(OpenProcess(0x1F0FFF, pid), 0x40)`
  (`mov ecx,0x40` at 0x7c42e1 → 0x7c34b4). That is IDLE_PRIORITY_CLASS. Restore 0x7c4358 reloads the
  list (0x7c37a8) and writes each saved class back (entry+8, read with GetPriorityClass at
  collection time, 0x7c3484). Which processes (0x7c3a9c, Toolhelp snapshot): those whose owner equals
  the current user (0x7c34ec opens with 0x400 and gets the owner, compared against GetUserNameW at
  0x7c368c). It skips the 31 Symantec names (0x786eac), `explorer.exe`, a name taken from the
  application object (most likely SGB's own exe), `steam.exe`, `SgbTray.exe`, `SgbMain.exe`,
  `SgbMini.exe`, and every child of steam.exe (parent PID == steam PID, 0x7c3a90/0x7c3984). The game
  itself is not explicitly excluded in this function. Unknown: whether the game is filtered out
  before this point (for example by A10 running before the game starts), and who triggers the
  virtual Boost (TBoost.Execute does not touch +0xa0; only settings load/save do, at 0x7c8acc–0x7c8b1d).
  Superseded: answered in `features.md` (Cleaners section): nothing calls it. Boost/Restore of
  TPriorityController are dead code; the object only stores `[Boost] Priority`/`PriorityClass`.
- **RAM (high confidence): SetProcessWorkingSetSize(h, -1, -1), not EmptyWorkingSet.** TRAMClear Boost
  0x7b78a4 → 0x7b7768. It walks every process in a Toolhelp snapshot, skips the 31 Symantec names
  (table 0xa4b7b0: BHCA.exe, ccSvcHst.exe, Checksum.exe, ControlAP.exe, DevViewer.exe, DoScan.exe,
  dot1xtray.exe, DWHWizrd.exe, EFAInst.exe, FixExtend.exe, installTeefer.exe, Luawrap.exe,
  MigrateUserScans.exe, nlnhook.exe, PatchWrap.exe, ProtectionUtilSurrogate.exe, RtvStart.exe,
  SavUI.exe, SepLiveUpdate.exe, SepStub.exe, Sevinst.exe, sisnat.exe, Smc.exe, SmcGui.exe,
  smcinst.exe, snac.exe, SRTSP_CA.exe, SylinkDrop.exe, SymCorpUI.exe, WFPUnins.exe,
  WSCSAvNotifier.exe; case-insensitive), and calls OpenProcess(PROCESS_SET_QUOTA 0x100) and
  SetProcessWorkingSetSize(h, -1, -1) on each one. A process that won't open is skipped silently.
  EmptyWorkingSet is resolved into 0xa5e6a4 by the PSAPI loader at ≈0x5458f4 (Superseded: said 0x5458a0,
  which is the debug-log function), but nothing ever reads that pointer.
- **GUID {58231417-…} (medium-high confidence): a lookup key, not the new plan's ID.** SgbMain
  0x7b8349 passes it as the 3rd argument of SetGamePwrCFG(name, description, guid, AC brightness,
  DC brightness). In PowerMgr (0x10001050 → 0x100016f0) it is parsed with CLSIDFromString. In the
  PowerEnumerate loop, a scheme whose GUID equals it, or whose friendly name equals the plan name, is
  reused. Otherwise High performance is duplicated with a NULL destination GUID (0x10001950), so
  Windows assigns a new random GUID. Because Boost deletes plans by name first, normally a new plan
  with a random GUID is created.

## Network, licence and promotion calls (2026-11-05)

Not part of Boost. SgbMain's endpoints (stats, licence check, accelerator API, hardware upload,
promo links with installed-product list `idata`) are listed with evidence in
`../../docs/sgb/telemetry-licensing-updates.md`; the helper programs each have a `notes.md` in
`../<program>/`.
