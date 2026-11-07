# Glary Utilities 6.49: what it really is

Glary Utilities (Glarysoft Ltd) is a long-running Windows "clean and tune" suite: about 30 small
MFC tools (registry cleaner, disk cleaner, tracks eraser, startup manager, uninstaller, process
manager, disk defrag, registry defrag, memory optimizer, file shredder, undelete, duplicate finder
and so on) behind one launcher, plus **1-Click Maintenance**, a "Windows Boot Time" ranking, and a
Software Update module that points at Glarysoft's own download site. Free and Pro share one binary
set; Pro unlocks scheduling, deep cleaning and auto-update. It has **no game booster, game mode,
power-plan or priority-boost feature**. The language file still contains "Turbo mode", "Speed up
internet" and "System Optimization", but no shipped binary implements them (see below). Glarysoft
publishes no separate game booster with this package; the installer only offers a desktop icon for
`maiagames.com` (unchecked).

Version **6.49.0.53** (`Integrator.exe`, FileDescription "Glary Utilities 6"), fetched 2026-11-07
from `https://download.glarysoft.com/gusetup.exe` (server `Last-Modified` 2026-10-19). No account
or payment. A full offline installer, not a stub.

| File | Size | SHA-256 |
|---|---|---|
| `gusetup.exe` (installer) | 28,500,912 | `3ac0088a4918a4700d53efa45df43b6d4038d8159617b82b01b9d41650064a06` |
| `Integrator.exe` (main window) | 938,896 | `2aa85f5c07009e8340c2b2567568d43e30516bb964243d3e2fd7ce189c3827a2` |
| `OneClickMaintenance.exe` | 241,552 | `6f341809dc3db9ac453ecda74bda4d17398101e479d38a8e49371081920cd1cf` |
| `memdefrag.exe` (Memory Optimizer) | 318,344 | `55fb3ee306daca197eea19dd175ca70de7a32bd198b1469b987f2f1b64fe2eb6` |
| `InfoCollect.dll` (data upload) | 639,888 | `52b4b4da574bb589fbb659a96f97c905d3806f6515a17de65877bc341bdf38ee` |
| `GUPMService.exe` (priority lock) | 76,688 | `6833299982716bdea210c0d50d21c2017d5107993137ca1f75978684049d7e4f` |
| `Native/win10_x64/BootDefragDriver.sys` (packed) | 10,868 | `29c17d1cca1386d7f3f33bcfee46cc81ca1cdabb07e305b337605a42fcb8d612` |

**Nothing was run.** Static analysis only: the unpacked file tree, the English language file, the
module data files (zlib-packed text, unpacked for reading), string tables read in context, and two
functions looked at in Ghidra (`memdefrag.exe` 0x404c60, `InfoCollect.dll` 0x10008cf0 and
0x1000cf80). Decompiled output and the unpacked data files were deleted afterwards (see
[decomp/README.md](../../decomp/README.md#policy)).

## How it's built

**Installer.** NSIS 3.06.1, `requireAdministrator`, unpacked with 7-Zip (the NSIS script itself is
not exposed by 7-Zip, so the exact install steps, services created and Run keys written by setup
were not read). The options page (`$PLUGINSDIR\GlaryUtilities.ini`) has desktop and Quick Launch
icons, **"Join the Customer Experience Improvement Program", pre-ticked** (`State=1`, link
`www.glarysoft.com/inf/ceip.php`), and "Create a desktop icon for Maiagames" (unticked, a
`maiagames.com` link with `utm_source=GU`). Setup plugins include `KillProcDLL`/`KillProcPath` and
a `statisticsinfo.exe` helper (inferred: install statistics, its endpoint was not found in clear
text).

**Program** (927 files, 91 MB, mostly 32-bit MSVC 2008/MFC 9, with an `x64\` folder of helpers):

- `Integrator.exe`: the main window (five tabs: Clean Up & Repair, Optimize & Improve, Privacy &
  Security, Files & Folders, System Tools). Each tool is its own exe or dll:
  `RegistryCleaner`, `DiskCleaner`, `TracksEraser`, `StartupManager`, `ShortcutFixer`
  (`FileScanFilter.dll`), `CheckDisk`, `procmgr.exe`, `memdefrag.exe`, `regdefrag.exe`,
  `DiskDefrag.exe`, `Uninstaler.exe`, `SoftwareUpdate.exe`, `DriverBackup.exe`, `QuickSearch.exe`,
  `FileUndelete.exe`, `shredder.exe` and others.
- `Initialize.exe`: the autostart helper (Run value `GlaryInitialize`; settings default
  `AutoLoadGU=1`), which starts `CheckUpdate.exe`, `Integrator.exe` and `autoupdate.exe` as needed.
- Services (binaries with service code; which ones setup registers was not determined):
  `GUPMService.exe` "Glary Utilities Process Manager Service" (priority lock, below),
  `GUAssistComSvc.exe` (an ATL COM service around `GUShellLink`), `MemfilesService.exe` (raw
  `\\.\C:` NTFS reader with a named pipe `\\.\pipe\{1AF636C4-…}_2`; inferred: the index behind
  Quick Search / Undelete), and the boot-time defrag pieces below.
- Data: `data\*.dat` are a 2-byte header plus a zlib stream of plain text or XML:
  `settings.dat` (all defaults), `registry.dat` (registry-cleaner rules), `startup.dat` (10,645
  startup entries with a 0–100 rating), `process.dat` (7,402 process ratings), `xdata.dat`
  (adware names), `junkcleaner.dat` (a zip of icons), `apps.dat` (privacy-trace definitions).
- `data\ModuleInfo.ini` lists 119 module names, including `TurboMode.exe`, `InternetBooster.exe`,
  `PCBooster.exe`, `NetworkOptimize.dll`, `NetOpt.dll`, `MemClean.dll`, `SpyRemover.exe`,
  `MalwareHunter.exe`. **None of those files is in the package.**

**"Turbo mode" and "Speed up internet" are dead UI text.** They are in the English language file
(IDs 1004017/1004018 under Advanced Tools) but the Advanced Tools table compiled into
`Integrator.exe` (`ADT_DS_CLEANUP|ADT_WR_REPAIR|…|ADT_SS_UNDOC`) has no entry for them, and no
shipped binary contains "turbo", "InternetBooster" or "NetOpt" strings. Inferred: leftovers from the
Glary Utilities 5 era, when Turbo Mode and Internet Booster existed.

## Background services, drivers, telemetry, offers

**Red flags, in order of weight:**

1. **Opt-out data sharing, pre-ticked.** The CEIP box is ticked in setup and `HelpPlan` defaults to
   1 in `settings.dat`. `AppMetrics.dll` (`EnableShareData`, `SendInstallInfo`,
   `SendUninstallInfo`) posts to `http://analytics.glarysoft.com/api/v1` `/install` and
   `/uninstall`, with a machine code from `MachineCode.dll`. `InfoCollect.dll` builds
   `data\UploadData.dat` from the record layout in `data\UpLoadStruct.ini` (per startup entry:
   command, company, file description, file version, original filename, product name and version,
   file size, **file path**, hash, platform), zips it to `UploadDataZip.dat` on an interval
   (`LastTime` check) and uploads it multipart to `http://infocs.glarysoft.com/file.php`
   (also `query.php`, `query_file.php`, `token_get.php`, `token_query.php`). The same DLL can read
   the disk model and serial number (`IOCTL_STORAGE_QUERY_PROPERTY`, "Hard Drive Serial Number").
   `StartupManager.dll` uploads startup-program icons (`image/png`) to a raw EC2 host,
   `http://ec2-52-39-105-18.us-west-2.compute.amazonaws.com/api/v1/file`. `StartupManager.dll`,
   `AppMetrics.dll`, `DriverBackup.exe` and `settings.dll` all read `HelpPlan`; that the uploads
   stop when it is 0 is **inferred** from that, the branch was not traced.
2. **Everything over plain HTTP**: analytics, info upload, crash reports
   (`crashreport.glarysoft.com/api/v1/report`), boot-time ranking
   (`gu.glarysoft.com/boottime/service.php`), comments on startup items and processes
   (`startups.glarysoft.com`, `processes.glarysoft.com`), licence calls
   (`myaccount.glarysoft.com/openapi.php/...`), updates (`www.glarysoft.com/update/...`), and the
   driver feed (`drivers.glarysoft.com`, plus a hard-coded `54.225.106.66/server.php`). Only the
   web links and `version3.ini` use HTTPS.
3. **Autostart and scheduled automatic repair by default.** `AutoLoadGU=1` (start with Windows),
   `Auto1Click=1` with `ScanFix=1` (Automatic 1-Click Maintenance, "Scan and repair issues", not
   only scan) and `AutoUpdate=1`. The schedule is created through the Task Scheduler COM API
   (`ITaskService`; Integrator even offers to "repair" a stopped `Schedule` service).
4. **One small kernel driver, only for boot-time defrag**: `BootDefragDriver.sys` (ten builds,
   XP to Windows 10, x86 and x64; zlib-packed in the package, unpacked size 20 KB, WDK Win7 build,
   signed by "Microsoft Windows Hardware Compatibility Publisher"). It imports only device,
   registry, file and `IoRegisterBootDriverReinitialization` calls and reads the `BootDefrag`
   service key and `Session Manager\BootExecute`. It works with the native-mode `BootDefrag.exe`
   that runs from `BootExecute` before Windows starts. No memory, port or MSR access. Inferred:
   installed only when the user schedules a boot-time defrag (`BottDefrag.dll`).
5. **Upsells**: "Upgrade Now" in the main window, per-tool `goto.php?a=upgradetopro&s=<tool>`
   links, `go.glarysoft.com/g/t/news` (a news feed shown by the updater), Software Update served by
   `update.filepuma.com` (Glarysoft's download portal), File Undelete pointing to "File Recovery
   Pro".
6. **Social proof from the vendor's servers**: Startup Manager shows "%s of users recommend
   disabling or delaying this startup program" and the boot-time panel says "Your Windows boot time
   is Slower, %s of users are slower" (reference histogram shipped in `data\BootTime.ini`, plus the
   online service).

## What 1-Click Maintenance and the scan count and how

The 1-Click table in `Integrator.exe` has six items (`ID|module|default|order`):

| Item | Module | Default | What it counts |
|---|---|---|---|
| Registry Cleaner | `RegistryCleaner.dll` | on | Each invalid entry. 23 sections, all on by default: Registry Integrity, Adware and Spyware, Obsolete Software, Custom Controls, File Extensions, File Associations, Start Menu, Startup Programs, Shared Programs, Software Locations, Add/Remove Programs, Windows Fonts, Help and Resources, Sounds and AppEvents, **MRU and History Lists**, Device Drivers, Windows Services, Virtual Devices, **ARP Cache**, **MUI Cache**, Application Settings, Firewall Settings, Invalid Shortcut. "Deep clean" is off (Pro) |
| Shortcuts Fixer | `FileScanFilter.dll` | on | Each shortcut whose target is missing |
| Startup Manager | `StartupManager.dll` | on | Startup entries it calls "dangerous" ("Cleans up dangerous startup entries"): inferred, entries whose file is missing ("Invalid entry. It can be deleted.") or rated bad in `startup.dat` / online |
| Temporary Files Cleaner | `DiskCleaner.dll` | on | Number of files and bytes. On by default: system and user temp, thumbnail cache, memory dumps, error reports, **Recycle Bin**, Windows Update leftovers, "Windows Installer" (`SysInstaller`); off: logs, prefetch, font cache, IIS logs and app caches |
| Tracks Eraser | `TracksEraser.dll` | off | Each trace. 587 definitions; 38 on by default, all of them browser caches; cookies are off by default |
| Disk Repair | `CheckDisk.dll` | off | Disk errors (Windows' own check-disk) |

The result screen shows "%s issues found" per item and "%s issues fixed" afterwards. No severity
score or health grade was found: the number is a plain sum. MRU lists, the MUI cache and the ARP
cache are ordinary records Windows rebuilds, so a normal PC always "has issues". Fixes are backed
up through Restore Center (`RestoreCenter.dll`, backups under `%APPDATA%\GlarySoft\`), and the
Undo Changes page manages System Restore points. The language file still lists "Defrag Registry",
"Malware Immunization", "System Optimization", "Defrag Hard Disks" and "Spyware Remover" as
1-Click items, but they are not in the compiled table.

## What each optimizer actually changes

- **Memory Optimizer** (`memdefrag.exe`, "Monitor and optimize free memory in the background"):
  verified in Ghidra at 0x404c60. "Optimize" commits the chosen number of **1 MiB blocks** with
  `VirtualAlloc` (read/write, no-cache), writes zeros over every block, then frees them all with
  `VirtualFree`. That makes Windows take physical pages from every other process (their pages go to
  standby or the page file) and hand them to this one, which then returns them as "free". This is
  the old "RAM booster" trick. No `EmptyWorkingSet` or `NtSetSystemInformation` is used. The
  process is 32-bit without the large-address flag, so one run can take at most about 2 GB.
  Auto mode runs it "when free memory at" a threshold and "ONLY optimize if CPU-Usage is below",
  and can "Load automatically on Windows startup" (Run key). It also has "Clear clipboard"
  (`EmptyClipboard`).
- **Process Manager** (`procmgr.exe`): end process, set priority, "Always on top", and
  **"Replace Task Manager"** through `Image File Execution Options\taskmgr.exe` (inferred: a
  `Debugger` value pointing at procmgr). Ratings from `process.dat` and
  `processes.glarysoft.com`.
- **Priority lock** (`GUPMService.exe`, LocalSystem service): "keep the current setting priority
  for the next startup". Rules live in `data\guprocessm.dat` (path, priority class); the service
  calls `SetPriorityClass` on matching processes (log line "Path:%s … PriorityClass:%d"). Started
  only when the user enables it (procmgr calls `StartServiceW`). Inferred: it polls the process
  list; no process-notify driver or ETW consumer is present.
- **Startup Manager**: enable, disable, delete, and **delayed start** ("Advanced Boost" / Delay):
  the entry is moved to `HKCU\Software\GlarySoft\Glary Utilities\startup\DelayItems` and launched
  later by Glary's own boot helper (`StartupManager.exe -boot`, `GUDelayStartup`). It also lists
  services and scheduled tasks; a built-in list of Windows services (`SysMain`, `wuauserv`,
  `Spooler`, `Themes`, `TermService`, …) is shown with ratings. Disabling is persistent.
- **Disk Defrag "SSD Optimize"** (`DiskDefrag.exe`), each a persistent switch, "Optimize All"
  applies them: disable Windows Search indexing (`WSearch` service), enable drive optimization
  (`defragsvc`), disable Prefetch (`SysMain` service), disable boot-file defrag
  (`Software\Microsoft\Dfrg\BootOptimizeFunction` `Enable`), "disable Windows event logging"
  (value `EnableEvents` under `Software\Microsoft\Wbem\CIMOM`; inferred from string order, so this
  switch touches WMI's legacy event setting, not the Event Log service), disable hibernate
  (`Control\Power` `HibernateEnabled`), disable last-access timestamps and 8.3 names
  (`Control\FileSystem` `NtfsDisableLastAccessUpdate`, `NtfsDisable8dot3NameCreation`). AHCI is
  advice only. Regular defrag refuses SSDs ("there is no need to defragment it").
- **Registry Defrag** (`regdefrag.exe`): saves each hive from `Control\hivelist` with
  `RegSaveKeyExW` (which writes a compacted copy) and schedules a native `RegBootDefrag.exe`
  through `BootExecute` to swap them in at reboot. The `RegBootDefrag.exe` paths it names
  (`Native\wxp_x86|x64\`) are **not in this package**; whether it now uses another path was not
  traced.
- **Boot Time**: measures boot time (`BootTime.dll`: BIOS time, system load time) and ranks it
  against other users. Changes nothing.
- **No power, CPU, GPU, network or game tweak** exists in any shipped binary: no power-plan,
  `PowerThrottling`, Game Mode, Game DVR, MMCSS, Nagle/TCP, `Win32PrioritySeparation` or timer
  string was found.

## Claims vs reality

Product UI text (6.49): "Clean up your registry to improve your system's performance", "Defrag
the Windows registry to speed up your computer", "Disable unnecessary startup programs to make
Windows boot faster", "Defrag hard disks to speed up the PC performance", "Monitor and optimize
free memory in the background", "keep your computer running at top speed".

| Claim | What the code does | Verdict |
|---|---|---|
| Registry cleaning "to improve your system's performance" | Deletes orphaned entries in 23 sections incl. MRU lists, MUI cache and ARP cache; backs them up | **Placebo**: unused keys cost nothing; the "issues" count is mostly normal history. The backup is real |
| 1-Click "%s issues found" | Plain sum of registry entries, dead shortcuts, startup entries, temp files and traces | **Misleading**: a count of harmless items. No fake score, though |
| Memory Optimizer "optimize free memory" | Allocates and zeroes up to ~2 GB, then frees it, pushing other apps' pages out | **Harmful**: Windows already keeps free and standby memory well; this throws away the standby cache and makes apps fault their pages back in. Auto mode repeats it in the background |
| Startup Manager "make Windows boot faster", delayed start | Disables or delays Run entries | **Real**: fewer startup apps is a real gain; delaying spreads the load. Windows' own Task Manager can disable entries too |
| Registry Defrag "speed up your computer" | Compacts hives via saved copies at reboot | **Placebo** on SSD-era Windows (hives are small and cached); the native helper it names is missing from the package |
| Disk defrag "speed up the PC" | Real defrag for HDDs; refuses SSDs | **Real for HDDs**, irrelevant for SSDs |
| SSD Optimize "reduces the lifetime of your SSD" | Turns off indexing, SysMain, hibernate, timestamps, 8.3 names, boot-file defrag | **Outdated / mildly harmful**: wear from these is negligible; no SysMain makes launches slower, no indexing makes search slow |
| Boot time "%s of users are slower" | Measured boot time against a shipped histogram and an online service | **Real measurement**, the ranking is social proof |
| "Turbo mode", "Speed up internet" | No implementation shipped | **Absent** (dead UI text) |
| CEIP "improvement program" | Pre-ticked; uploads install/uninstall events and startup-entry metadata incl. file paths, over HTTP | **Opt-out telemetry** |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing in Glary Utilities makes BF6 faster. The Memory Optimizer is the one feature aimed at
"performance", and it is counterproductive: on a machine where BF6 needs most of the RAM, flushing
everyone's pages just before or during a match causes paging and stutter. The SSD switches turn
off `SysMain` permanently, which slows launches. Startup Manager is useful once, to stop tray
apps from starting with Windows. Leave Automatic 1-Click and Memory Optimizer autostart off, and
untick CEIP during setup if you install it at all.

## Adoption items for gamebooster-rs

1. **Nothing new to adopt.** No game, power, priority-boost, service or network tweak exists.
2. **Allocate-and-free RAM "optimizer"**: **skip**. Our [RAM trim](../tweaks/ram-trim.md)
   (`EmptyWorkingSet`, off by default) is the gentler form of the same idea; Glary's method is
   worse (it also evicts the standby cache and needs a large allocation). Nothing to change.
3. **Persistent priority lock service**: **skip**. A LocalSystem service polling for processes
   is what our IFEO `PerfOptions` method already avoids ([priority](../tweaks/priority.md)), and
   outside priority changes don't reach BF6 under Javelin.
4. **SSD Optimize switches**: **skip**. Persistent and default-worse; `services` can already
   stop `SysMain` / `WSearch` per session if someone wants that.
