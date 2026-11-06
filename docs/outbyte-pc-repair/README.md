# Outbyte PC Repair: what it really is

Outbyte PC Repair is a Windows "repair and speed up" suite sold by Outbyte Computing Pty Ltd
(Australia), at [outbyte.com/software/pc-repair/](https://outbyte.com/software/pc-repair/). The
product page is no longer linked from the Outbyte home page or the `/software/` list (only Driver
Updater and AVarmor are), but it is still live and still serves downloads. It is built by
**Auslogics**: every file in the installer was compiled from
`D:\Auslogics_Sources\Outbyte Products\Outbyte PCRepair\_Tags\2.0.0.X\Binaries\`, and the module
names (Rescue Center, `TweakManagerHelper.dll`, `RegistryCleaner`, `DiskWipeHelper.dll`,
`BrowserCare`) are the Auslogics BoostSpeed family.

The vendor's own page sells it as: fixes "more than a hundred of the most common PC issues",
"Optimization" tweaks that remove "unneeded background processes", Disk Space cleanup, Privacy
Protection (tracking cookies), Vulnerabilities (Windows settings and missing updates, plus a PUA and
malware scanner), **Real-Time Boost** ("switches priorities as you switch programs"), Real-Time
Privacy (telemetry off, auto-clear history) and a File Shredder. Trial: "free by-category trial for
at least 2 days"; full version "from $29.95 (for a 3-month subscription)".

Version **2.0.01.54517** (installer `FileVersion`; the stub's info endpoint returns the same).
Fetched 2026-11-06, no account or payment:

- Online stub `https://outbyte.com/software/pc-repair/download/?_sid=…&fs`, redirecting to
  `https://dynamicdownloads.outbyte.com/en/pc-repair/stub/outbyte-pc-repair?clientId=…`, served as
  `outbyte-pc-repair.exe` (`Last-Modified` 2026-10-09), 335,640 bytes, SHA-256
  `5262c0b710af816e7dd0ef46e8cc7559fba5b3afe02aa415b0ceb911e6f1dfd1`.
- Full installer `https://downloads.outbyte.com/en/pc-repair/outbyte-pc-repair-setup.exe` (the
  stub's `build-link`), `Last-Modified` 2026-09-10, 40,456,344 bytes, SHA-256
  `de82a7f2aceb820f44dacd50f0d9b2b922a2ffb41f4d4ae8891c8737036a190b`, signed by Outbyte Computing
  Pty Ltd.

**Nothing was executed.** Static analysis only.

**Limit of this analysis.** The installer is a modified Inno Setup, and the program files could
not be unpacked (see below). So this page is built from the installer's file list, its setup
script strings, the first part of the English UI language file, the vendor's pages and the EULA.
**The scan code itself (how each "issue" is counted) was not read.** Where a verdict rests on the
UI strings or the vendor's own text rather than the code, it says so.

## How it's built

**Download stub.** `outbyte-pc-repair.exe` is a small Delphi program ("Outbyte PC Repair Setup",
internal name `StubLoader.exe`, WinHTTP). It asks
`https://outbyte.com/tools/software/info/?softwareId=pc-repair&softwareCode=pc-repair&registered=false&sender=stub`
for a JSON record (`version`, `build-link`, `buyurl`, `sid`), downloads the `build-link`, looks up
your IP at `https://outbyte.com/tools/ipinfo` and `https://outbyte.com/sid/ip/pc-repair`, and posts
Google Analytics 4 events to `https://www.google-analytics.com/mp/collect` (measurement ID
`G-924XWBQ2KM`, with an embedded API secret). So the download is tracked before anything is
installed.

**Installer.** A modified Inno Setup. The loader signature is `rDlPtS8v7b4n` (stock Inno uses
`rDlPtS02…`/`rDlPtS04…`) and it lives in an `RCDATA/RES_DATA` resource, not Inno's usual
`RCDATA/11111`; the header blocks say `Setup Data 1.0.0` instead of `Inno Setup Setup Data`. So
innoextract 1.9 and binary-refinery `xtinno` both refuse it. The header blocks are LZMA (props
`0x5D`, 64 MiB dictionary) cut into 4,096-byte pieces with 4 bytes between each piece; decoded that
way they give the setup engine (3.3 MB Delphi, with `SetupHelper.dll`, export `SHPrepareSetup`),
the language list, the installer messages and the compiled setup script. The file data is one
`zlb\x1a` chunk at file offset `0x88A00`: 6 unknown bytes, then a `0x5D` props byte and LZMA data.
Only its first ~4.7 KB decode (the start of `Lang\enu.lng`, 23 KB of UI strings); after that the
stream is altered in a way that neither plain LZMA nor the 4,096+4 layout explains. Reversing the
reader in the setup engine was out of scope.

**Files installed** (from the setup script; `{app}` = `C:\Program Files\Outbyte\PC Repair\` or
similar under `{pf}\Outbyte`):

- `PCRepair.exe` (main program, Delphi with Embarcadero 10.x `rtl250`/`vcl250` packages and the
  Auslogics `AxComponents*`/`OxComponentsRTL`/`DrvRTL` packages), `Downloader.exe`,
  `SendDebugLog.exe`, `HWHelper.dll` and `HWHelper64.exe`.
- One helper DLL per module: `SystemCleanerHelper`, `TweakManagerHelper`, `ServiceManagerHelper`,
  `StartupManagerHelper`, `UninstallManagerHelper`, `PerformanceMonitorHelper`,
  `SpywareCheckerHelper`, `MalwareDetectionHelper`, `MalwareHeuristicHelper`,
  `SecurityHolesHelper`, `ActivityTracesHelper`, `PrivacyGuardHelper`, `BrowserHelper`,
  `BrowserPluginsHelper` (+ `.Agent.x32`/`.Agent.x64`), `DriverUpdaterHelper`, `DiskWipeHelper`,
  `VolumesHelper`, `PopupManagerHelper`, `DataStorageHelper`, `SystemInformationHelper`,
  `PCInfoCollector`, `RescueCenterHelper`/`RescueCenterForm`, `TaskSchedulerHelper`,
  `GoogleAnalyticsHelper` and `GoogleAnalyticsHelperIV`, `Chat.dll`, OpenSSL
  `libeay32`/`ssleay32`, `sqlite3.dll`.
- `ShellExtension.dll` and `ShellExtension.ContextMenu.x32/x64.dll` (Explorer context menu).
- **Three kernel drivers, each x86 and x64:** `DrvMonX86/X64.sys` and `DrvHWX86/X64.sys` in
  `{app}`, and `DrvMatX86/X64.sys` in `%AppData%`-style `{p:AppDataDir}Outbyte\PC Repair\2.x\Drivers\`.
  Their purpose could not be read (not unpacked). Inference from the names and the `HWHelper`
  files next to them: a hardware/sensor driver and a monitoring driver.
- Data: `Data\main.ini`, `Database.dsdh`, `database.dat`, `Applications.dat`,
  `compromised_passwords.txt`, `hbwlists.mdict`, `hmthinfo.mdict`, `hsysfiles.mdict`,
  `hwscheme.mdict`. The uninstaller also removes `{app}\Engine`, `{app}\idx`, `{app}\servers.json`
  and `vwcore.log`/`vwcore_ss2.log`/`vwcore_wg.log`, which points to a separately downloaded scan
  engine (inference).
- Temporary at install: `Installer.exe`, `InstallerUtils.dll`, `CFAHelper.dll` (inference from the
  name: Windows Defender Controlled Folder Access handling) and **`LhSvc_Setup.exe`**, a second
  setup run from `{tmp}`. Its contents are unknown; with `Chat.dll` and the vendor's "PC Help Desk"
  it is probably a live-help service (inference).
- Registry: `HKLM\Software\Outbyte\PC Repair\2.x`; Rescue Center backups in
  `{p:AppDataDir}Outbyte\PC Repair\Rescue`; shortcuts in the Start menu and on the common desktop.

## Background components, telemetry, offers (red flags)

1. **Three kernel drivers** in a "PC repair" tool. Nothing on the product page explains why a
   cleaner needs ring 0. On a gaming PC, any extra driver is a risk with anti-cheat and a source
   of bluescreens.
2. **Tracked from the first click.** The stub sends GA4 events and IP lookups before install; the
   installer has a `TSetupAnalytics` class (client ID, events, error reports with OS and .NET
   version), and the program ships two Google Analytics helper DLLs.
3. **A second setup bundled in the installer** (`LhSvc_Setup.exe`, copied to `{tmp}`). Whether
   the installer asks before running it is not visible in the strings that were decoded.
4. **Cloud rules and uploads.** The UI language file lists the server endpoints:
   `https://pcr.outbyte.com/base2/quickScan.zip` (57 MB, AES-encrypted ZIP holding one 295 MB file
   `data`), `.../base2/programs.zip`, `.../base2/filters/<name>.zip`,
   `https://pcr.outbyte.com/api/uptodate/?f=…`, and two upload endpoints
   `https://pcr.outbyte.com/api/add/?type=1` and `?type=2` (key names `URL_POST_MONITORING_RESULTS`
   and `URL_POST_CHANGE_RULES`). So the scan rules come from the server and "monitoring results"
   are posted back. The archives were not decrypted.
5. **Auto-renewing subscription.** EULA section 13 "Automatic renewals": Outbyte "will
   automatically renew the subscription at the end of each subscription term … by directly charging
   your account the current renewal price"; cancellation is "provided together with your license
   key". EULA 6.2: if you enable auto-update, Outbyte "will not be required to offer you the option
   to decline or delay update installation".
6. **Pressure UI.** Strings include "Only %1 out of %2 days of free trial remaining; please purchase
   the program…", "The trial period has expired", "Your subscription has expired. Please renew your
   subscription now to keep your computer optimized…", "This tool is available in the registered
   program version only", and the "(recommended)" tag.

## What the scan checks, and how counts are made

What is known from the decoded strings and the vendor page:

- The scan is grouped into categories that match the license blocks: **Computer Speed**, **System
  Stability**, **Disk Space**, **Privacy Protection**, **Browser Care**, **System Advisor**. The
  trial is "by category": per the vendor page it "lets you fix all detected items within each
  category" for "at least 2 days". So the free scan finds everything, and fixing beyond that is paid.
- "Quick scan" data is a 295 MB server-side rule set (`quickScan.zip`), refreshed through
  `api/uptodate`. With the rule set encrypted and the main program not unpacked, **how the counts
  are computed (one per file, per registry value, per cookie, or per check) was not verified.** The
  vendor's FAQ itself says junk files "do not generally affect your computer performance" and that
  the crash log is "for your reference only" and cannot be repaired automatically, so at least
  those categories add to the problem total without being speed or stability faults.
- The modules behind the categories (from the helper names): junk/temp cleanup, registry cleaner,
  activity traces and cookies, browser add-ons, startup items, services, scheduled tasks, Windows
  setting "tweaks", missing updates and "security holes", a PUA/malware scanner, compromised
  passwords (a local `compromised_passwords.txt` list), driver updates and an uninstaller.

## Claims vs reality

| Claim (vendor page or UI) | What's there | Verdict |
|---|---|---|
| "Solve more than a hundred of the most common PC issues … invalid redirects, system malfunctions, shared dll files" | Registry cleaner and system-file checks driven by server rules; not read in code | **Misleading** (inferred from the categories): "invalid redirects" and "shared dll" entries are registry leftovers that cost nothing on Windows 10/11; counting them as "issues" inflates the total |
| "Up to 45% faster startup and shutdown" / "PC boots up and powers off in 45% less time" (registered-version benefits) | No benchmark, method or test machine given anywhere in the strings or on the page | **Misleading**: an unexplained marketing number. Startup gets faster only if you disable heavy startup items, which Task Manager does for free |
| "Things like file copying or application opening will go faster" | Same benefit list | **Misleading**: no code that changes disk, cache or scheduler behaviour was found in the file list |
| **Real-Time Boost**: "switches priorities as you switch programs … prioritize certain apps for CPU processor time" | Foreground priority switching; the module was not unpacked | **Placebo for games**: Windows already boosts the foreground window's threads; raising a game's priority class rarely changes FPS (see our [priority](../tweaks/priority.md) notes). Real but small |
| "Optimization … get rid of unneeded background processes" | `TweakManagerHelper`, `ServiceManagerHelper`, `StartupManagerHelper` | **Real but persistent**: these are permanent setting, service and startup changes, backed up by Rescue Center, not per-session |
| Junk/Disk Space cleanup | `SystemCleanerHelper` | **Real** for disk space; the vendor's own FAQ says it does not speed the PC up |
| Privacy: "tracking cookies … the real danger … can be used with malicious intent by hackers" | Cookie and activity-trace deletion | **Scare framing**: deleting cookies logs you out of sites; it is not a security fix |
| "Lightweight scanner … PUA's … known malware" | `MalwareDetectionHelper`, `MalwareHeuristicHelper`, separate engine files | **Unverified**; the vendor itself says it does not replace an antivirus. Defender already does this |
| Driver updates (`DriverUpdaterHelper`) | Third-party driver feed | **Harmful on Boot Camp**: Apple's driver set is the safe one |
| Free scan, "fix all detected items within each category" in trial | Trial is time- and category-limited; renewal is automatic | **Dark pattern**: the free scan's job is to show a large number; fixing it is a subscription that renews unless cancelled |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing here makes BF6 faster. Real-Time Boost is a foreground priority switch, which Windows
already does and gamebooster-rs does per session with `restore`. The tweaks are persistent. Three
kernel drivers, a live-help service, analytics and an always-updating scan engine are extra load
and risk on a heat-limited 4-core CPU with an anti-cheat game. The Driver Updater can replace
Boot Camp drivers. Don't install it on the target Mac.

## Adoption items for gamebooster-rs

1. **Nothing to adopt.** Its only gaming-related feature (Real-Time Boost, a foreground priority
   switch) is already covered by gamebooster-rs's game priority and
   `lower_others_mode`, which are session-only and undone by `restore`.
2. **Registry cleaner, cleanup, privacy, driver updater, malware scan**: **skip** (placebo for
   games, or harmful on Boot Camp).
