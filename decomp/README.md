# decomp/

Static analysis output for Smart Game Booster 6.0 (vendor binaries). Only the spec writer reads
this; implementers work from `docs/spec.md` and never from here. Nothing was executed.

## Source

The vendor installer is an **Inno Setup** setup. It was unpacked (not run) into a scratch directory
`sgb-extract/` with three parts:

| Dir | Files | What |
|---|---|---|
| `tmp/` | 31 | Setup-time files: `Inno_*.lng` (29 setup languages), `Icon_0.bmp`, `Icon_1.bmp`. |
| `sys/` | 5 | DirectX runtime DLLs installed to System32 (`d3dx9/10/11_43`, `D3DCompiler_43`, `D3DX8Wrapper`). |
| `app/` | 202 | `app/SgbMain.exe` (502,328-byte launcher stub) and `app/cmAppVer/` (the program). |

111 of the 238 files are PE binaries (.exe/.dll/.sys/.bpl). Main ones in `app/cmAppVer/`:

| Binary | What it is | Decompile status |
|---|---|---|
| `SgbMain.exe` (9.0 MB, Delphi) | Main UI and Boost/Restore orchestration. | **Targeted Boost/Restore**: 10 functions as annotated disassembly in `SgbMain.exe/` (see its `notes.md`): dispatcher, power plan, services, priority, RAM, GUID. Ghidra via REA now analyses the whole exe (183 s). Superseded: earlier this said the exe was protected and Ghidra couldn't finish. Both came from a bad extraction (Inno call filter not undone). |
| `PowerMgr.dll` | Power-plan helper (powrprof wrapper) called by Boost. | **Complete**: 510 functions in `PowerMgr.dll/functions/`. |
| `SgbAccelerate.dll` | Online network accelerator (WinDivert/KCP tunnel to vendor servers). | **Partial**, stopped deliberately (out of scope): 4011 functions in `SgbAccelerate.dll/functions/`. |
| `WinDivert.dll`, `nDrivers/itvwd64.sys` | Rebranded WinDivert packet driver, used only by the accelerator. | Strings only. |
| `OverDrive.dll`, `GPUlib.dll` | GPU overclock helpers (A2, out of scope). | Not analysed. |
| `SgbTray.exe`, `SgbMini.exe`, `SgbHUD32.exe`, `HUD64/`, `FPS*.dll`, `VideoRecord*.dll` | Tray, mini window, FPS HUD and recording. | Not analysed (not Boost). |
| `SgbUpdater.exe`, `AUpdate.exe`, `DDownloader.exe`, `ProductStat3.exe`, `Register.dll`, `FeedBack.exe`, `BugReport.exe`, `UninstallPromote.exe` | Update, telemetry, licensing, feedback, promotion. | Not analysed (out of scope). |
| `AutoDriver.exe`, `DrvInstall/`, `DriverClean.dll` | Driver updater. | Not analysed (not Boost). |
| other DLLs (`madExcept_`, `rtl120`/`vcl120`/`vclx120` .bpl, OpenSSL, SQLite3, 7z, LibAV, D3DX) | Runtime and third-party libraries. | Not analysed. |

Data files: the default Boost service lists are the plain-text `WinSvc.ini` the installer drops in
`%APPDATA%\PCGameBoost\Smart Game Booster\` (decoded in `SgbData/`); `Database/*.dbd` and `*.sgdt`
are encrypted, not decoded, and hold no Boost lists; `Database/startupService.db` belongs to the
startup manager. Superseded: this line used to name the `.dbd` files as the likely list source.

Re-extraction (2026-11-05): innoextract isn't installed, so `decomp/inno_carve.py` (Python, nothing
is executed) decompresses the solid LZMA chunk and carves each PE, undoing Inno's call-instruction
filter. Output: `/home/zpt/re/sgb-extract/carved/pe_<offset>_<size>.bin` (no file names, since the
Inno header isn't parsed) and `/home/zpt/re/sgb-extract/app/cmAppVer/SgbMain.exe` (copied from
`pe_00444dac_9008696.bin`). PowerMgr.dll is `pe_077713fe_74240.bin` (export name PowerConfig.dll).

## Layout

- `PowerMgr.dll/`, `SgbAccelerate.dll/`: REA `decompile_all` output (`decompile-index.json`,
  `decompile-journal.jsonl`, `functions/<address>.c`).
- `SgbMain.exe/`: `<address>_<descriptive name>.asm`, `notes.md`, and the two helper scripts
  (`refs.py`, `dis.py`) that produced them.
