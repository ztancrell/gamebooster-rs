# SgbMain.exe notes (targeted Boost/Restore only)

Target: `sgb-extract/app/cmAppVer/SgbMain.exe` (9,008,696 bytes, Delphi 2009 / RTL120, i386). The
`sgb-extract/app/SgbMain.exe` copy (502,328 bytes) is a small Delphi launcher stub, not the app.

## Method and limits
- Ghidra via REA did not finish import + auto-analysis of this exe within its 30-minute startup
  deadline (this is what stalled the previous agent). Rizin is not installed. So there is no Ghidra
  pseudocode: the `.asm` files here are `objdump -M intel` disassembly of single functions,
  annotated by `dis.py` (string literals, import names). `refs.py` finds code that references a
  string (VA = file offset + 0x400C00 in .text). Both scripts are kept here so the output can be
  regenerated.
- The exe is **protected**: many helper calls and every Win32 API call site observed go to stubs at
  VA 0xb4xxxx–0xb8xxxx, a region inside the `.rsrc` section range whose bytes are encrypted on disk
  (no `E8` call reaches the normal `FF 25` import thunks of TerminateProcess, ControlService,
  ChangeServiceConfigW, SetPriorityClass, SetProcessWorkingSetSize, EmptyClipboard). Which API a
  given stub calls therefore can't be proven statically; a stub's identity is inferred from its
  arguments and the surrounding log strings.

## Files
| File | What |
|---|---|
| 0x7c6f0c_TBoost_Execute.asm | Boost/Restore dispatcher. |
| 0x7b81f4_PowerSch_Boost.asm | Power-plan Boost (steps "TPowerSch.Boost 1–11"). |
| 0x78754c_ServiceList_PerWinVersion.asm | Picks the INI section of default services by Windows version. |
| 0x788a58_ServiceBoost_ini_Load_Save.asm | ServiceBoost.ini / "StopServiceFail" bookkeeping. |
| 0x473324_SysInfo_Win32PrioritySeparation_log.asm | Diagnostics: reads PriorityControl, Memory Management (PagingFiles), SubSystems; logs them. |
| 0x5458a0_PsAPI_dynamic_loader.asm | Generic PSAPI loader (GetProcAddress of EnumProcesses, EmptyWorkingSet, ...). Not the RAM boost itself. |

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
  10 → `Services_WIN10` (Windows 11 reports as 10). The INI text is in no extracted file in plain
  text (only `Database/startupService.db`, the startup manager's DB, names services). The encrypted
  `Database/*.dbd` / `*.sgdt` files or a download are the likely source; not decoded.
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
