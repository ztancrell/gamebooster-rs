# iolo System Mechanic 26.3: what it really is

System Mechanic (iolo technologies, now owned by RealDefense LLC) is a long-running Windows "PC
tune-up" suite. It has a free scan with a paid fix ("Upgrade to fix all items"), a Toolbox of about
25 tools (cleaners, registry repair, startup optimizer, defrag, NetBooster, Memory Mechanic, driver
updater, file shredder, recovery, bootable USB), **LiveBoost** ("real-time RAM, HDD and CPU
optimization": OptiCore, RAMJet, AcceleWrite, PowerSense), **ActiveCare** (automatic cleaning when the PC
is idle, "AI detects when your PC is active or idle"), and in the Ultimate Defense edition an
antivirus, Privacy Guardian, ByePass password manager and other add-ons.

**The main finding: LiveBoost is Process Lasso's engine.** `LBGovernor.exe` and `GvrMgr.dll` are
Bitsum's Process Governor rebuilt under iolo names. OptiCore is ProBalance, RAMJet is SmartTrim
under another name ("Memory Mechanic" in the API), PowerSense is Performance Mode with per-app
power plans, and "Energy Saver" is IdleSaver. Compare [Process Lasso](../process-lasso/README.md).

Version **26.3.0.123** (all editions share one installer; Standard, Pro and Ultimate Defense differ
only by licence). The download button on iolo.com (`https://secure1.iolo.com/downloads/IOLO_SMSTD_1Y`,
also `IOLO_SMFREE`) gives a 2.4 MB Delphi download manager (`smdm.exe`, "iolo Download Manager",
dated 2026-10-30). It asks `https://api.iolo.net/release/v3/releases/resolve?sku=…&channel=Stable`
for the current build; a plain request gets HTTP 401 ("Azure AD JWT not present"). Rather than imitate
the stub, the full installer was taken from iolo's public CDN path, which the Internet Archive has indexed
since 2024: `https://download.iolo.net/sm/26/11A12794-499E-4FA0-A281-A9A9AA8B2685/26.3.0.123/SystemMechanic.exe`
(`Last-Modified` 2026-03-24; `SystemMechanic_Ultimate_Defense.exe` at the same path is the same size).
No account or payment was needed. Whether the stub now resolves to a newer build than 26.3 was not
checked. Fetched 2026-11-07.

| File | Size | SHA-256 |
|---|---|---|
| `SystemMechanic.exe` (installer, `Bootstrap.exe` 26.3.0.123) | 62,338,248 | `ee33758a116bfa4fcc0dca078fa156902780e40866b21130419badde054754a2` |
| `smdm.exe` (download stub, SMSTD SKU) | 2,482,536 | `715a75e1bff0cf4724110b3cf8e9479d29d440231fb2a8b6eed68bbc81b193e1` |
| product zip (inner resource) | 51,380,179 | `5cf6be741c8b74085c5c73b55876e976b10f6ba1d1c8b9d43a18caccb336c7aa` |
| `iolo.exe` (main UI) | 4,360,392 | `6a22fe90bc2bfd99f297517f05566716833df2b442e8fe29aef254440531e9cd` |
| `x64/LBGovernor.exe` | 1,069,768 | `b044ede1d51bdc0f30a8b457eeb8f90a3514b90edc173da8ef23dc3611bed800` |
| `x64/GvrMgr.dll` | 517,320 | `0ed25558753344fe4c99517182a97b1e3c11063fc0936cff0c92a2eff8cdbf34` |
| `x64/ToolKit.dll` | 14,010,056 | `92fcab85a8bdd15757818847f54eddcea64f03bf4e0e73d4d2f37d18171c8677` |
| `x64/PDFsFilter.sys` | 94,152 | `a6274540d03b5f330fd7803e795c3cda323ee89f56279dc2c76e9818733ed4b2` |

**Nothing was run.** Static analysis only: PE resources and strings, the embedded JSON in
`GvrMgr.dll`, a short objdump of its power-plan code, and .NET metadata read with ILSpy for
research. Most .NET string literals are encrypted by an obfuscator, so names below come from
native string tables, resources, the UI text (`ACResources.dll`) and unencrypted code. Decompiled
and disassembled output was deleted afterwards (see [decomp/README.md](../../decomp/README.md#policy)).

## How it's built

**Installer.** `SystemMechanic.exe` is a .NET bootstrapper (`Bootstrap.exe`, RealDefense LLC). Its
managed resources carry the installer DLLs (`Installer`, `InstallerUI`, `InstallerCommon`),
Newtonsoft.Json, Application Insights, the .NET 4.6.2 web installer, and an `Installer.dll` whose only
resource is the product as a zip (167 files, 124 MB unpacked). On reinstall the installer
removes older iolo products it finds (System Checkup, `Phoenix360.exe`, an old `sstray.exe` /
`WscRmd.exe` antivirus) and can schedule locked files for deletion through
`PendingFileRenameOperations`.

**Product** (the zip):

- `iolo.exe`: the WPF main app (.NET, obfuscated: renamed classes, encrypted strings), with Prism
  modules for each area (`SM.LiveBoost`, `SM.ActiveCare`, `SM.Tools`, `SM.PrivacyGuardian`,
  `SM.DriveScrubber`, `SM.ByePass`, `SM.SearchNRecovery`, …), `SMInfrastructure.dll`,
  `SMCommon.dll`, `SDKModels.dll`, `UIResources.dll`, `ACResources.dll` (all UI text, nine
  languages).
- `x64/ToolKit.dll` (14 MB, Delphi): the native engine behind the scan and the Toolbox (registry
  scanner, junk cleaners, Startup Optimizer and Startup Manager (`StartupApproved`, MSConfig
  keys, services, tasks, IE extensions), the Security Optimizer (Remote Registry, Windows Update,
  IE zones), NetBooster (`ApplyOptimalMTU`, `ioloICMPPing`, `DefaultTTL` under
  `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters`), Memory Mechanic (`EmptyWorkingSet`,
  `NtSetSystemInformation`), privacy settings (`CapabilityAccessManager\ConsentStore\*`,
  `DataCollection` policies, `Software\Microsoft\GameBar` with `AllowAutoGameMode`)).
  `x64/gpp.exe` is a Delphi helper for the same engine.
- **LiveBoost**: `x64/LBGovernor.exe` + `x64/GvrMgr.dll`, PDB paths
  `C:\Jenkins-Slave\workspace\sm\26.0\ioloProcessGovernor\…`. They contain Bitsum's own text
  ("Windows OS compatibility failure. Exiting for safety. Notify Bitsum Technologies at
  support@bitsum.com", "Process Lasso", `Software\ProcessLasso`, `Global\ProcessLasso_ProBalanceEvent`,
  `processgovernor.exe`, `AggressivelyTrimProcessLassoWorkingSet`) next to iolo's
  (`Software\iolo technologies\Governor`, RAMJet log lines). The GUI talks to it through
  `GvrMgr.dll` exports: `InstallGovernor`, `Set/IsProBalanceEnabled`, `SetProBalanceAggressiveness`,
  `Set/IsMemMechanicEnabled`, `Set/IsEnergySaverEnabled`, `Engage/DisengageEnergyBoosterProfile`,
  `SetPowerProfileByGuid`, `SetDisplayOffTimeOut`, `SetComputerSleepTimeOut` and counter getters
  (`get_GvrCounters_ProBalance_TotalRestraints`, `…_MemMech_TotalMegabytes`, …).
- **Raxco PerfectDisk parts**: `x64/defrag.dll` ("Copyright (C) 2018 Raxco Software Inc.") for
  Program Accelerator and drive defrag, and `x64/PDFsFilter.sys`, "PerfectDisk OptiWrite Filter
  Driver" (PDB `c:\perfectdisk_v14.0_sp1\writeright\pdfsfilter\objfre_wnet_amd64\…`), which is AcceleWrite.
- **Privacy Guardian**: NetFilter SDK (`WPF_Driver\…\pgfilter.sys` "NetFilter SDK WFP Driver", the
  older TDI `netfilter2.sys`, `nfapi.dll`, `ProtocolFilters.dll` with OpenSSL 1.1), tracker lists
  `pg_trackers.dat` and `pg_patterns.json`.
- **Antivirus (Ultimate Defense)**: an Avira Endpoint Protection client (`EndpointProtectionClient.Net.dll`,
  `EndpointProtectionInterfaces.dll`), engine and config from `api.iolo.net/redirect/v1/antivirus/{update,config}`.
- **Other**: `ads.dll` (driver updater, `iuupdateservice.iolo.com/driverupdateservicesIolodu/updateservice.asmx`),
  `Incinerator` (shredder), `smartscan.exe`, `netscan.dll`/`portscan.exe`/`btscan.dll` (network
  and Bluetooth scanners), `syslinux6.exe` (bootable USB), `SmartActiveCare.dll` (ML.NET),
  `ioloTrayApp.exe`, `ActiveBridge.exe`, `mpns.exe`, `WSC.exe`, MSAL (Microsoft sign-in) and
  WebView2.

## Background processes, drivers, telemetry, offers

**Red flags, in order of weight:**

1. **Kernel drivers, by feature.** Not installed by the setup itself: each is installed when its
   feature is switched on (inferred from where the files and install scripts sit; the call sites are behind
   encrypted strings).
   - `PDFsFilter.sys` (AcceleWrite): a file-system **minifilter** at altitude 186000, class `HSM`,
     installed by `RUNDLL32 SETUPAPI.DLL,InstallHinfSection DefaultInstall 132 PDFsFilter.inf` and
     `NET START PDFsFilter`. Its source tree is PerfectDisk 14.0 SP1, built with the Server 2003
     (`wnet`) DDK, so the code is about ten years old. The UI warns "you may experience increased boot-time
     the first time you restart your machine after enabling AcceleWrite".
   - `pgfilter.sys` (Privacy Guardian): a NetFilter SDK WFP callout driver copied into
     `system32\drivers` and registered with `nfregdrv.exe pgfilter` (service
     `HKLM\SYSTEM\CurrentControlSet\Services\pgfilter`). `ProtocolFilters.dll` exports
     `pfc_setRootSSLCertSubject` / `pfc_getRootSSLCertFileName`, so the SDK can
     intercept HTTPS through its own root certificate. Whether Privacy Guardian turns HTTPS
     filtering on was not traced.
   - iolo System Shield once shipped `amp.sys` (CVE-2018-5701, a local privilege escalation). This build has
     no such driver.
2. **A resident Process Lasso engine** (`LBGovernor.exe`) once LiveBoost is on. It runs elevated
   (it enables `SeDebugPrivilege`, `SeIncreaseQuotaPrivilege`, `SeProfileSingleProcessPrivilege`),
   reads other processes (`ZwQueryInformationProcess`, `ZwReadVirtualMemory`; **inferred** to be for
   command lines, as in Process Lasso) and sets priority, affinity, priority boost, working-set size and the active
   power plan. It starts at logon (**inferred**: `GvrMgr.dll` carries the `Software\Microsoft\Windows\CurrentVersion\Run`
   key and Bitsum's "start at login with elevated rights" code path). It excludes iolo's own processes
   and `ioloSystemService` from its rules.
3. **A global keyboard and mouse hook for "AI" ActiveCare.** `SmartActiveCare.dll` uses
   `Gma.System.MouseKeyHook` (`Hook.GlobalEvents()`, `KeyDown`, `MouseClick`). The handlers only
   **count** key presses and clicks (not which keys). The counts go per time window to
   `raw.dat` / `*.d1` / `*.d2` files in iolo's AI folder, and an ML.NET multiclass classifier
   (`LbfgsMaximumEntropy` on the timestamp) predicts the busiest hour of the day
   (`PredictDailyHighestActivityHour`) so maintenance runs at other times. It is not a keylogger,
   but every key press in every app passes through a low-level hook in an iolo process.
4. **Upsell notifications that ignore your opt-out once the licence expires.** `mpns.exe` runs from
   the scheduled task "Multi Product Notification Service" and shows "toasters" from a JSON list.
   Its only exit check is: if the user turned toaster notifications off **and** the service is not
   expired, quit. So after expiry the toasters show whatever the setting. Tray texts include
   "You're Missing out on Critical Features", "Upgrade your System Mechanic now to boost your PC's
   performance", "You have {0} of junk files, which may be slowing down your reboot speeds".
5. **Cross-selling installer.** `ActiveBridge.exe` has a messenger window (WebView2), event
   toasters, a driver-scan result page and a `ProductDownloadWindow` / `ProductInstaller` that
   downloads and installs other iolo/RealDefense products.
6. **Telemetry.** Microsoft Application Insights (`Microsoft.ApplicationInsights.dll`,
   `ServerTelemetryChannel`) in the installer and the app, a `DeviceId` assembly, MSAL account
   sign-in, the driver updater's web service, and the release API. The Application Insights
   key and the event names are in encrypted strings and were not read.
7. **Scare and speed framing**: "Repairing registry issues … restoring your PC's performance",
   "clearing RAM cache, closing unneeded background applications", "Unleash the full potential of
   your internet speed", "Industry-leading PC Performance Software".

## What the scan counts and what each boost actually changes

**The scan** ("Issues Found") has six categories (`SCU_*` in the UI text): Security
Vulnerability, Internet Connection, Registry, Internet Junk, Windows Junk, Memory. Item counts and
per-tool "health scores" come from the native engine as JSON (`BaseProblemItem.HealthScore`) and
were not traced further. One rule is readable in `OverviewViewModel`: **Memory counts as an issue
whenever RAM load is above 50 %**, and its health score is `100 − memory load`. On a gaming PC with
a browser open that is nearly always true, so the scan almost always has at least one "issue" whose
fix is a working-set flush. The free edition scans; repairs need a licence ("Upgrade to fix").

**Performance features.** Revert column: what the product offers to undo it.

| Feature (UI) | Exact change | Revert | Risk (heat / anti-cheat / stability) |
|---|---|---|---|
| **OptiCore** (LiveBoost) "prioritizing CPU power for the program being currently used" | Bitsum **ProBalance**: lowers a background process that hogs the CPU to a lower priority for a few seconds; foreground excluded. iolo can set the aggressiveness (`SetProBalanceAggressiveness`); its default was not read, so the thresholds are **inferred** to be Bitsum's ([table](../process-lasso/README.md#probalance-exactly)) | turn off; restraints are temporary | low; same as Process Lasso |
| **RAMJet** (LiveBoost) "reclaims wasted RAM in real time" | SmartTrim under another name: periodic working-set trims (`SetProcessWorkingSetSize`; log "Full trim complete. Memory load was %u%%, is now %u%%, %u MB trimmed", tiers 1–3, `TrimAllProcessesAtThisIntervalInMs`, `/trimnow`) | turn off; trims undo themselves as pages fault back | **placebo to harmful** (stutter as pages come back) |
| **Memory Mechanic** (Toolbox, scan "Memory" fix) | One-shot flush: `EmptyWorkingSet` on processes plus `NtSetSystemInformation` (**inferred**: system working-set or standby-list purge); shows "RAM Before / RAM After" | none needed | placebo; brief stutter |
| **PowerSense / On-Demand Boost** (Bitsum Performance Mode) | When a trigger process runs, switch the power plan; optional per-mode lists of services and processes to stop (`AutoStopServices`, `AutoStopProcesses`). Built-in modes (embedded JSON in `GvrMgr.dll`): **Gaming** → Ultra Performance, triggered by `steam.exe`, `steamwebhelper.exe`, `ubisoftgamelauncher.exe`, `uplay.exe`, `battle.net.exe` or `origin.exe` children and a few named games (SC2, HotS, LoL, Smite, FFXIV); Music/Video Production, Graphics/3D/CAD Design and Programming → Ultra Performance; Office Productivity and Web Browsing → Balanced; Maximize Battery → Endurance. **All seven built-in modes have empty service and process stop lists.** The EA app (`EADesktop.exe`) is not a trigger | plan switched back when the trigger exits | heat (see below) |
| **"System Mechanic – Ultra Performance"** plan `C54D89C6-D8F0-4BC6-BD6F-B8D4F1EDAA68` | Duplicated from High performance, then (disassembly of `GvrMgr.dll` around 0x1800069e9–0x180006bbd, `PowerWriteACValueIndex` / `PowerWriteDCValueIndex`): minimum processor state **100 %** AC and DC, maximum 100 % AC and DC, core-parking minimum cores **100 %** AC and DC (parking off). UI: "Engages all processor cores at the highest clock speed, allocates RAM to critical programs, and turns off non-essential processes"; only the first part is in the plan | switch plan / uninstall | **heat**: same as Bitsum Highest Performance |
| **"System Mechanic – Endurance"** plan `22901212-B918-4C66-A01C-F09DA066464D` | Battery (DC) values only: brightness **60 %**, core-parking minimum **25 %**, minimum processor state **5 %**, maximum **25 %** | switch plan | none; very slow on battery |
| **Energy Saver** (IdleSaver) | Switch to a power-saving plan after an idle timeout (`SetEnergySaverIdleTimeoutMS`); options to skip while a game or media runs (`DisableEnergySaverWhenGameRunning`) | turn off | can kick in during gamepad play (controller input isn't idle input to Windows), as in Process Lasso |
| **AcceleWrite** "minimizing random data writes to hard drives & SSDs" | PerfectDisk **OptiWrite** minifilter (`PDFsFilter.sys`), which steers writes so files fragment less | uninstall the driver (`UninstallDriver.bat` / disable) | adds a 2016-era kernel filter on every file write; useful only on spinning disks. On NTFS and SSDs it does nothing useful for games |
| **Program Accelerator / defrag** | Raxco `defrag.dll` file moves (`Move_File`) to defragment and "re-align program data". The UI refuses SSDs: "Disk defragmentation and/or program data re-alignment are unnecessary for solid-state drives" | none (data layout) | none on SSD (refused); slow on HDD |
| **NetBooster** "fine-tuning over a dozen hidden TCP/IP settings" | Visible in `ToolKit.dll`: an ICMP MTU probe (`FindBestMTUValue`, `ApplyOptimalMTU`) and `DefaultTTL` under `Tcpip\Parameters`; the other value names were not found in plain text | the tool's undo (not traced) | placebo for UDP games; a wrong MTU can break connections |
| **Startup Optimizer / Manager** | Disables startup entries through `Explorer\StartupApproved` and the MSConfig `startupreg` / `startupfolder` / `services` keys; "This startup item is a Windows Service, which can be stopped immediately. This usually provides an instant speed boost" | re-enable | as Task Manager's Startup tab |
| **Privacy settings** | Per-app capability consent (`ConsentStore\webcam`, `microphone`, `location`, …), telemetry policies, `GameBar` `AllowAutoGameMode` | toggle | none |
| **Registry / junk / internet cleanup, ActiveCare** | Deletes registry leftovers, temp files, caches, cookies, run automatically at idle by ActiveCare | backups / restore point (Backup and Restore tool; per-cleaner behaviour not traced) | placebo for speed |

## Claims vs reality

Product UI and feature-compare text, 26.3.

| Claim | What the code does | Verdict |
|---|---|---|
| "LiveBoost – real-time RAM, HDD and CPU optimization" | Licensed Bitsum Process Governor (ProBalance, SmartTrim, Performance Mode, IdleSaver) plus a PerfectDisk write filter | **Real, but not iolo's**: the CPU part is Process Lasso's ProBalance, which Process Lasso itself gives away free |
| "OptiCore – dynamically adjusts system resources to prioritize in-use applications" | ProBalance lowers hogging background processes | **Useful**, mildly, on a CPU-bound PC |
| "RAMjet – reclaims wasted RAM in real time", Memory Mechanic "reclaims trapped memory" | Working-set trims and flushes | **Placebo to harmful**: Windows already manages RAM; trimmed pages fault back |
| Scan "Memory" issue | Raised whenever RAM load > 50 % | **Misleading**: a normal state is reported as a problem |
| "Ultra Performance … allocates RAM to critical programs, and turns off non-essential processes" | A plan with 100 % minimum state and no parking; built-in stop lists are empty | **Overstated**: only the power-plan part exists unless the user fills the lists |
| PowerSense "Gaming" mode | Plan switch when a Steam/Ubisoft/Battle.net/Origin child starts | **Useful** on desktops (same as Bitsum Performance Mode); **harmful** on heat-limited laptops |
| "AcceleWrite – optimizes file writes for faster HDD performance" (also "& SSDs") | OptiWrite fragmentation-prevention minifilter | **Placebo** on SSDs and on modern NTFS; adds kernel code |
| "NetBooster – increases internet speed by fine-tuning over a dozen hidden TCP/IP settings" | MTU probe, `DefaultTTL`, other values not visible | **Placebo** for games (most such values are ignored since Vista or don't affect UDP) |
| "AI detects when your PC is active or idle" | Counts keys and clicks with a global hook, ML.NET predicts the busiest hour | **Real but trivial**: an hour-of-day guess; Windows' own idle detection and maintenance window do this without a hook |
| Registry repair "restoring your PC's performance" | Deletes registry leftovers | **Placebo** |
| "Upgrade now to boost your PC's performance" toasters, "Missing out on Critical Features" | Scheduled-task notifier that ignores the opt-out after expiry | **Dark pattern** |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing here makes BF6 faster than gamebooster-rs already does, and several parts work against
the Mac:

- **PowerSense Gaming / Ultra Performance**: on a Steam copy of BF6 it switches to a plan with
  100 % minimum processor state and parking off, on AC **and** battery. That keeps the i7/i9 hot
  and makes BD PROCHOT throttling worse (see [prochot](../tweaks/prochot.md)). The EA app copy
  doesn't trigger it at all.
- **OptiCore** (ProBalance) is the one feature that fits the Mac's problem; the same is free in
  Process Lasso and already in gamebooster-rs as `lower_others_mode = "dynamic"`. Under EA Javelin
  the governor can't touch `bf6.exe` anyway; it only lowers other programs.
- **RAMJet / Memory Mechanic**: trims cause stutter when pages come back. Skip.
- **AcceleWrite** and **Privacy Guardian** add kernel drivers (a file-system minifilter and a WFP
  callout) for no frame-rate gain: extra anti-cheat surface and boot-time cost.
- **Driver updater**: generic drivers can replace Apple's Boot Camp drivers.
- ActiveCare's global input hook and the resident governor are small but constant background load.

Don't install it for gaming. If you want ProBalance, use Process Lasso or `gamebooster watch`.

## Adoption items for gamebooster-rs

1. **ProBalance-style restraint**: **already done** (`lower_others_mode = "dynamic"`,
   [watch.md](../watch.md)). System Mechanic's OptiCore is the same Bitsum algorithm, which confirms
   it is the CPU tuning commercial optimizers actually rely on.
2. **Per-launcher triggers (start the boost when a child of `steam.exe` / `battle.net.exe` starts)**:
   **skip**. `gamebooster auto` matches game executables directly, which also covers the EA app;
   a launcher-child rule fires for every Steam helper process.
3. **"Ultra Performance" plan values**: **already done** (100 % minimum processor state in our game
   plan, [power-plan.md](../tweaks/power-plan.md)). Do **not** copy the core-parking-off and DC
   values: our plan keeps the user's battery behaviour and offers a maximum-state cap for PROCHOT
   Macs instead.
4. **RAM trims, write filter, TCP tweaks, registry cleaning**: **skip**. Placebo or harmful, as
   already decided for SGB and Ashampoo ([ram-trim.md](../tweaks/ram-trim.md),
   [network.md](../tweaks/network.md)).
5. **Report only real numbers**: **already done**. `status` shows measured values; System
   Mechanic's "RAM above 50 % is an issue" is the counter-example.
