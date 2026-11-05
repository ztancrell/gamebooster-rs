# AutoDriver.exe + DrvInstall/ (driver updater)

Static strings and certificate tables only; nothing was executed. Copies: `AutoDriver.exe` = carved
`pe_023d9614_3772984.bin` (same size); `DrvInstall/*` read from the `-db` extraction (string reads
are unaffected by the Inno call filter). Version info: "Auto Driver for Smart Game Booster".

## What it is

IObit **Driver Booster** engine, rebranded. Evidence: source paths `E:\WorkSpace\IObit_common\...`,
`Driver Booster_is1`, `IObit\Driver Booster\Config.ini` (DbzInst.dll), Driver Booster forum URLs,
`idb.iobit.com/check.php`, `ascstats.iobit.com/other/db_driverinstall.php` (install stats).

| File | Role |
|---|---|
| `AutoDriver.exe` | UI and background "auto update while idle" (`/AutoDriver`, `/AutoDriverIdle`; idle minutes in `M_Some_DrvIdleTime`). |
| `DrvInstall/Scanner.dll` | Enumerates devices (SetupDi), uploads them, compares with the database. |
| `DrvInstall/Cmpnt.dll` | Component (VC++ runtime, PhysX, .NET, DirectX) "drivers": `APP\VEN_MS&VCRT&2015X64` etc. |
| `DrvInstall/DbzInst.dll` | "Dbz Installer" v5.0.0.36: unpacks a downloaded package (7z / zip / RAR / CAB-SFX), runs DpInst. |
| `DrvInstall/DpInstX32/X64.exe` | Microsoft's Driver Package Installer, **re-signed** by ORANGE VIEW LIMITED (HK), DigiCert EV, valid 2020-10..2023-10. |
| `DrvInstall/DIFxAPI32/64.dll` | Microsoft DIFx API (used by DpInst). |

## Where driver info comes from

- Scan upload: `https://s.pcgameboostdrivers.com/scan.php` (Scanner.dll); fallbacks
  `https://s1.driverboosterscan.com/worker.php`, `https://s2.driverboosterscan.com/worker.php`.
- Packages: `http://f.pcgameboostdrivers.com/drivers/` (plain HTTP base), Cmpnt fallbacks
  `http://www.cd4o.com/drivers/`, `http://www.cd4o.com/drivers/wlst/v.json`.
- Anti-block base-URL lists: `https://s3.amazonaws.com/myfilelist/list*`,
  `https://storage.googleapis.com/myfilelist/list*` (same mechanism as SgbTray).
- Country lookup: `http://ip-api.com/json/`, `http://ipinfo.io/`.
- Local cache: `ScanData\dev.dat`, `cache.dat`, `dev-pro.dat`, `cache-pro.dat` (encrypted).

Per-driver record fields (Scanner.dll): `drv_name drv_vendor drv_md5 drv_filesize drv_flag inf_provider
inf_version inf_date inf_whql dbxurl dbxmd5 dbxfilesize exturl extfilemd5 extdrvinfo extfilesize
extdectype`. `dbx` is IObit's own package container, i.e. drivers are **repackaged by the vendor**
and fetched from the vendor's CDN, with an `exturl` path for packages hosted elsewhere
("Extlink mode" in DbzInst).

## How it picks a driver

Scanner.dll log strings show a per-device rule set: skip if `LocalVersion >= CurrentVersion` or
`LocalDate > CurrentDate`; special case `LocalDate = 1970/01/01 | not Microsoft`; user ignore list
(`IgnoredByUser Key/Hid`); `IgnoreByDpInst` (a previous DpInst failure); `FLAG_DEFER`, `FLAG_RARE`,
`FLAG_RARE_HIDE`; a `ProUser` branch (Pro sees more). So "outdated" means *older version or date
than the database entry for that hardware ID*. No string mentions Apple or Boot Camp anywhere in
these binaries (case-insensitive grep: 0 hits), so Apple-supplied Boot Camp drivers get no special
treatment. `inf_whql` is stored, but whether non-WHQL packages are offered could not be determined
statically.

## Install, backup, restore

- Install: DbzInst extracts the package to a temp dir (needs 1 GB free), optionally deletes INFs
  listed in the package's `drvinst.ini`, then runs `DrvInstall\DpInstX64.exe` (one instance, mutex
  `DpInstX`) and waits. The DpInst command line is built at run time and was not recovered, so
  whether it passes `/lm` (legacy mode, accepts unsigned packages) is **unknown**. On 64-bit
  Windows a kernel driver still needs a Microsoft signature to load, whatever DpInst accepts.
- Backup: **no driver backup and no restore point** found. No `SRSetRestorePoint`, `srclient`,
  `SystemRestore` WMI or "backup driver" strings in AutoDriver.exe, Scanner.dll, Cmpnt.dll or
  DbzInst.dll. The UI only has a menu item `M_MenuSysReset=System Restore` (opens Windows' tool).
  Rollback relies on Windows keeping the old package in the DriverStore, which DriverClean.dll can
  delete (see `../DriverClean.dll/notes.md`).
- UI warning text: "You may encounter temporary black screens when you update graphic drivers".
