# decomp/

## Policy

Vendor code is decompiled only locally, under `/home/zpt/re/`, and only to research. The findings
are written here as prose (`notes.md`, `README.md`, `features.md`), with function addresses as
evidence; the decompiled or disassembled output and decoded vendor data are deleted afterwards and
never committed. Earlier commits that held such output were purged from the history on 2026-11-05.

Static analysis notes for Smart Game Booster 6.0 (vendor binaries). Only the spec writer reads
this; implementers work from `docs/spec.md` and never from here. Nothing was executed.
Plain-language overview of every part: [docs/sgb/README.md](../docs/sgb/README.md).

## Source and extracted copies

The vendor installer is an **Inno Setup 6.4.2** setup (`smart_game_booster_enseo_hp_setup.exe`). It
was unpacked (not run) twice:

| Where | What | Trust |
|---|---|---|
| `/home/zpt/re/sgb-extract/carved/pe_<offset>_<size>.bin` (113 PEs) | Carved by `inno_carve.py` from the solid LZMA chunk, with Inno's call-instruction filter undone. No file names (the Inno header isn't parsed); each `notes.md` names its `pe_…` file. `sgb-extract/app/cmAppVer/SgbMain.exe` is a named copy of `pe_00444dac_9008696.bin`. | **Correct**: use these for code and call targets. |
| `/home/zpt/re/sgb-extract-db/` (`data/{app}`, `{tmp}`, `{sys}`…, `embedded/script.ps`, `meta/`) | Named files and installer metadata from binary-refinery (`emit setup`, `xtinno`). | File names, strings, data files and resources are fine; **E8/E9 call targets may still be Inno-filtered** (wrong). |

Superseded: this section used to describe a single `sgb-extract/` with `tmp/`, `sys/`, `app/` dirs
(238 files, 111 PEs). That copy had the Inno call filter not undone, which produced the false
"protected exe" finding in `SgbMain.exe/notes.md`.

PowerMgr.dll is `pe_077713fe_74240.bin` (export name PowerConfig.dll).

## Helper scripts

| Script | What |
|---|---|
| `inno_carve.py` | Decompresses the Inno solid LZMA chunk, undoes the call filter, carves each PE. Python, `-I`. |
| `gdis.py` | Generic PE32/PE32+ annotated disassembly of a VA range (`objdump` + strings/imports), and `--refs "text"` to find code referencing a string. |
| `dfm_timers.py` | Lists the timers in Delphi form (DFM) resources. |
| `SgbMain.exe/dis.py`, `SgbMain.exe/refs.py` | The SgbMain-specific versions of the same two jobs (output not kept). |
| `SgbData/winsvc_lists.py` | Decodes the `WinSvc.ini` service lists. |
| `pgware-gameboost/tweaks.py` | Lists PGWare GameBoost's registry tweaks. |

## Components

Each row links to its notes. Status words: **Complete** / **Fully analysed** (full decompile, read locally and deleted), **Targeted** (the parts
that matter read in code), **Strings** (strings, imports/exports and resources, little or no code).
Plain-language pages: [main app](../docs/sgb/main-app.md),
[GPU/HUD/recording](../docs/sgb/gpu-hud-recording.md),
[telemetry/licensing/updates](../docs/sgb/telemetry-licensing-updates.md),
[drivers/tray/accelerator](../docs/sgb/drivers-tray-accelerator.md).

| Binary | What it is | Status |
|---|---|---|
| [`SgbMain.exe`](SgbMain.exe/notes.md) (9.0 MB, Delphi) | Main UI; Boost/Restore orchestration; game detection, CYRI database, startup manager, cleaners, accelerator UI. | **Targeted**: Boost/Restore in `notes.md` (disassembly read locally, deleted); every other feature surveyed in [`features.md`](SgbMain.exe/features.md). Ghidra via REA analyses the whole exe (183 s). |
| PowerMgr.dll | Power-plan helper (powrprof wrapper) called by Boost. | **Analysed** (decompile deleted); findings in [`docs/spec.md`](../docs/spec.md) 2.1. |
| [`OverDrive.dll`](OverDrive.dll/notes.md) | GPU overclock (A2): ADL OverDrive 5/6/8 and NVAPI, core and memory clock only. | **Targeted** (`gdis.py`). |
| [`GPUlib.dll`](GPUlib.dll/notes.md) | Per-process GPU usage (Process Hacker's ExtendedTools method). Read-only. | **Strings** + exports. |
| [`SMGBFilter64.dll`](SMGBFilter64.dll/notes.md) (+ `SMGBFilter.dll`) | HUD injector: global `WH_CBT` hook, loads FPS64.dll into matching games; `WH_KEYBOARD_LL` hotkeys. | **Targeted**. |
| [`FPS64.dll`](FPS64.dll/notes.md) (+ `FPS.dll`, `FPSD1264.dll`, `FPSD12.dll`) | In-game overlay: Detours on D3D9/DXGI/OpenGL/D3D12 `Present`, Dear ImGui; frame capture. No Vulkan. | **Targeted**. |
| [`SgbHUD64.exe`](SgbHUD64.exe/notes.md) (+ `SgbHUD32.exe`) | HUD controller: publishes game and exclusion lists, calls `SetHook`. | **Strings**. |
| [`VideoRecord64.dll`](VideoRecord64.dll/notes.md) (+ `VideoRecord.dll`) | Frame/audio hand-off from the game to the encoder via shared memory. | **Strings** + exports. |
| [`sgbGameRecord.exe`](sgbGameRecord.exe/notes.md) | Encoder: FFmpeg 4.x via FFVCL, software MPEG-4. | **Strings**. |
| [`RobloxFPS.dll`](RobloxFPS.dll/notes.md) | Copy of rbxfpsunlocker (writes Roblox memory). | **Strings**. |
| [`AutoDriver.exe`, `DrvInstall/`](AutoDriver.exe/notes.md) | Driver updater (IObit Driver Booster engine, re-signed DpInst). | **Strings** + certificates. |
| [`DriverClean.dll`](DriverClean.dll/notes.md) | Deletes old DriverStore packages. | **Strings** (from `-db`; no carved copy of that size). |
| [`DDownloader.exe`](DDownloader.exe/notes.md) | Resumable HTTP downloader for driver packages. | **Strings**. |
| [`SgbTray.exe`](SgbTray.exe/notes.md) | Tray resident: auto-boost game watching (SVIP), temperature/RAM/driver nags, updater, licence, stats. | **Strings**. |
| [`SgbMini.exe`](SgbMini.exe/notes.md) | Small Boost/Restore window without the main UI. | **Strings**. |
| [`SgbAccelerate.dll`](SgbAccelerate.dll/notes.md) (+ `WinDivert.dll`, `itvwd*.sys`) | Network accelerator (iTop VPN core): WinDivert redirect, KCP/ChaCha20 tunnel. | **Fully analysed** (decompiled locally, deleted); notes. |
| [`ProductStat3.exe`](ProductStat3.exe/notes.md) | Feature-usage statistics sender. | **Strings**. |
| [`Register.dll`](Register.dll/notes.md) | Licensing (online check, hosts-file edits). Key algorithm deliberately not studied. | **Strings** + one function. |
| [`SgbUpdater.exe`](SgbUpdater.exe/notes.md) | Program updater (HTTPS manifest, MD5 + Authenticode allow-list). | **Strings**. |
| [`AUpdate.exe`](AUpdate.exe/notes.md) | Daily/monthly active pings, cross-product detection. | **Strings**. |
| [`FeedBack.exe`](FeedBack.exe/notes.md) | Feedback form. | **Strings**. |
| [`BugReport.exe`](BugReport.exe/notes.md) | Crash/debug-log upload (can be silent). | **Strings**. |
| [`UninstallPromote.exe`](UninstallPromote.exe/notes.md) | Install/uninstall/upgrade beacon; shows nothing in 6.0. | **Strings**. |
| [`SgbInit.exe`](SgbInit.exe/notes.md) | First-run init: scheduled tasks, list-file copies, attribution ping. | **Strings**. |
| [Setup](Setup/notes.md) | Inno Setup script: tasks, shortcuts, driver, China IP block. | IFPS script read. |
| [SgbData](SgbData/README.md) | Data files: `WinSvc.ini` service lists, `.dbd` container format and tweak summary. | **Decoded** (decoded output not kept). |

Superseded: the earlier table marked OverDrive/GPUlib, tray/mini/HUD/FPS/recording, the
update/telemetry/licensing helpers and the driver updater as "Not analysed"; all now have notes.

**Still not analysed**: runtime and third-party libraries (`madExcept_`, `rtl120`/`vcl120`/`vclx120`
.bpl, OpenSSL, SQLite3, 7z, LibAV, D3DX); `StatUser.exe` and `datastate.dll` (only seen referenced
from SgbMain); SgbMain's junk/registry scanner and malware scanner internals; the DpInst command line
(built at run time); SgbTray's poll intervals; who calls `TPriorityController.Boost` (spec section 8).

Data files: the default Boost service lists are the plain-text `WinSvc.ini` the installer drops in
`%APPDATA%\PCGameBoost\Smart Game Booster\` (described in `SgbData/`). `Database/*.dbd` are encrypted
containers (format in `SgbData/dbd/`) for the optimizer tweaks, cleaner templates and the "Can You
Run It" database (`CYRI.dbd`), not Boost lists; `*.sgdt` and `Def.dbd` feed the malware scanner (see
`SgbMain.exe/features.md`); `Database/startupService.db` belongs to the startup manager. Superseded: this line used to name the `.dbd` files as the likely Boost list source.

## Layout

- `SgbMain.exe/`: `notes.md` (Boost/Restore), `features.md` (everything else), and `refs.py`, `dis.py`.
- `SgbData/`: `README.md`, `winsvc_lists.py`, `dbd/README.md`, `scanner/README.md`.
- Superseded: `PowerMgr.dll/` and `SgbAccelerate.dll/functions/` held REA `decompile_all` output and
  `SgbMain.exe/` held per-function `.asm` listings; all deleted under the Policy above.
- Every other binary: one `notes.md`.
