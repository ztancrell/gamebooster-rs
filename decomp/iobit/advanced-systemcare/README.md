# decomp/iobit/advanced-systemcare/

Policy: see [decomp/README.md](../../README.md).

Static analysis notes for IObit Advanced SystemCare Free 20.0.1.256. Nothing was executed.
Plain-language page: [docs/advanced-systemcare/README.md](../../../docs/advanced-systemcare/README.md).

## Source and extracted copies

| Where | What |
|---|---|
| `/home/zpt/re/iobit-asc/dl/advanced-systemcare-setup.exe` | Installer, SHA-256 `abea2697…10a7f1`, Inno Setup 5.5.7 (u) |
| `/home/zpt/re/iobit-asc/x/` (`app/`, `tmp/`) | innoextract 1.9 output, 348 files. Inno 5.5 is fully supported by innoextract, so the call filter is undone correctly (no `inno_carve.py` needed, and no "protector" artefact) |

binary-refinery's `xtinno` was tried for the `[Run]` / `[Registry]` sections and was stopped after
two minutes without output. Services and scheduled tasks were therefore taken from `ASCInit.exe`,
`ASCService.exe` and `ASCTray.exe` strings instead.

## Method

- **Strings** (ASCII and UTF-16) of `ASC.exe`, `ASCService.exe`, `ASCTray.exe`, `ASCInit.exe`,
  `Monitor.exe`, `AutoRamClean.exe`, `PowerConfig.dll`, `HardwareLib.dll`, the drivers and
  `{tmp}\Installer\Setup.exe`. Delphi string literals sit next to their users, so the order of
  `ASC.exe`'s strings gives the Internet Boost table (setting → recommended value → UI name) and the
  Turbo Boost file and section names.
- **`Language\English.lng`** (UTF-16 INI) for every claim quoted.
- **Delphi forms** in `Setup.exe`: `TfrmASCInstaller` (checkboxes `cbxLoadASCTray`,
  `cbxLoadASCService`, `cbxCreateIcon`, `cbxLaunch`, `cbxTaskbar` with `Checked`, `cbxAddScreenshot`,
  `Image_ERecomiTop`), `TFrame_SingleRec2`/`3` (iTop Screen Recorder offer). Timers listed with
  `../../tools/dfm_timers.py`.
- **`.dbd` decoding** with a fresh local decoder for the format in
  [`../smart-game-booster/SgbData/dbd/README.md`](../smart-game-booster/SgbData/dbd/README.md).
  Its HAVAL-256/5 was checked against PHP's `haval256,5` on three vectors, then switched to
  DCPcrypt's `0x80` padding. Every entry's CRC-32 matched. Passwords, all literals in `ASC.exe`:
  `asc5_TurboBoost` (`TurboBoostGame.dbd`), `asc6_2012_6` (`Opt.dbd`), `asc4_2010` (`Reg.dbd`,
  `PriTemp.dbd`, `Def.dbd`). `SysSpeed.dbd`'s one entry has compression byte 0 and failed its CRC
  with `asc9_speedup` (not pursued). `Cus.dbd` did not open with `asc4_2010`. The decoder and
  decoded output stay out of the repo.
- **Comparison with SGB** by SHA-256 over every extracted file, by section names and Win10
  entries of the service lists, by `diff` of the decoded `Opt.dbd` files, and for
  `PowerConfig.dll` vs SGB's `PowerMgr.dll` (`pe_077713fe_74240.bin`) by exports, imports and
  the presence of each power GUID from `docs/spec.md` 2.1 (all 29 present in both).

## Not traced in code

Whether Turbo Boost changes service start types (`ChangeServiceConfigW` is imported by `ASC.exe`,
but the "Fix ChangeServiceConfig fail" log line belongs to the optimization fixes). Also not traced:
the exact numbers Internet Boost writes for `MaxUserPort`, `TcpTimedWaitDelay`, `DefaultTTL` and
MTU; the Auto RAM Clean timer interval and thresholds; the Economy plan's values; and whether the
iTop offer page auto-accepts on its countdown.
