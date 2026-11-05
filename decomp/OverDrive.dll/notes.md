# OverDrive.dll notes (GPU overclock, Boost action A2)

Carved copy: `carved/pe_031f9d95_253136.bin` (253,136 bytes, i386 DLL, ImageBase 0x10000000,
PDB `OverDrive.pdb`, version "Overclock for Smart Game Booster" 5.2.0.2). Static only; analysed with
`objdump` + `decomp/gdis.py` (REA was shared with other agents and not used for this file).

## Exports
`CanOverClock`, `GetGPUInfo`, `GetRange`, `OverClock(int *gpuOut, int *memOut, int level)`,
`OverClockFixNoIni`, `Reset`. Imports only KERNEL32/USER32/SHELL32: every vendor API is loaded at
run time with LoadLibrary/GetProcAddress. No network code.

## How it overclocks
- **AMD**: loads `atiadlxx.dll` (64-bit name) / `atiadlxy.dll` and resolves ADL. Three code paths by
  driver generation: Overdrive5 (`ADL_Overdrive5_ODPerformanceLevels_Set`, logs "ATL5"), Overdrive6
  (`ADL_Overdrive6_State_Set`, "ATL6") and OverdriveN / Overdrive8 (`ADL2_OverdriveN_SystemClocksX2_Set`,
  `..._MemoryClocksX2_Set`, `ADL2_Overdrive8_Setting_Set` for `OD8_GFXCLK_FMAX`/`OD8_UCLK_FMAX`).
  Fan, power-limit and voltage APIs are resolved and logged, but the setters seen in the log strings
  are engine clock and memory clock only ("Set OD8_GFXCLK_FMAX/FMIN/UCLK_FMAX parameters failed").
  One OD8 path resets "Optimized Power mode to Manual" first.
- **NVIDIA**: loads `nvapi.dll` and resolves functions by ID through `nvapi_QueryInterface`
  (undocumented interface IDs; only names in log strings: `NvAPI_Initialize`, `EnumPhysicalGPUs`,
  `GPU_GetAllClockFrequencies`, `GPU_GetPstates20`). The setter is presumably `SetPstates20` (core/mem
  clock offset), not provable from strings.
- `GPUDB.dll` (+ `GPUInfo.db`) supplies a per-model reference clock ("dbGPUinfo.GPUClock").

## Limits / safety (0x10005058 AMD5 offset by level, 0x100022a0 ReadBoostIni)
- Level comes from `%APPDATA%\PCGameBoost\Smart Game Booster\SmartGameBooster.ini`, `[Boost]`
  `OCLevel`; custom MHz from `OC_GPU`, `OC_RAM`; `Support` flag.
- Offset = driver-reported headroom, capped by the GPUDB reference clock when known; **level 1 halves
  it**, level 3 uses the user's `OC_GPU`/`OC_RAM`; other levels use the full driver headroom. Values
  are checked against the driver range ("range should be in Min/Max"). No voltage is raised.
  Nothing tests stability (no stress test, no temperature watchdog).
- "Already OverClocking" guard: if current clock != default it does not stack another offset.
- `Reset` writes the default clocks back ("EngineClock/MemoryClock Rstrore Succeed"); presumably
  called by the OverClock sub-object's Restore in SgbMain (not traced). The change lives in the
  driver's OverDrive state; whether it survives a reboot depends on driver generation (not checked).
- Log: `...\Logs\OverClock.log` (+ `.bak`).

## Boot Camp Radeon Pro 5300M/5500M
Navi 14 mobile. Everything is gated by `ADL2_Overdrive_Caps` / `CanOverClock`. AMD does not expose
OverDrive on mobile Radeon Pro parts and Apple's Boot Camp driver package has no Radeon Software
tuning page, so `CanOverClock` most likely returns false and A2 is a no-op. Not verified on hardware.
