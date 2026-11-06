# Fortect: what it really is

Fortect (Fortect Ltd., formerly sold as Restoro) is a Windows "PC repair" product: a free scan that
reports "items" in seven categories (crashed programs, junk files, broken registry, browser cache,
cookies, Office traces, jump lists) plus malware/PUA, a PC "hardware summary", and a paid
**Repair**. The current build is also an antivirus: it ships Avira's scan engine (SAVAPI) and
Avira's real-time minifilter and ELAM drivers, a "NetShield" web filter driver, a driver updater
and a VPN upsell. It has **no game mode, no power-plan, priority, service or RAM tweak**: the only
"Boost Performance" button runs the junk and crashed-programs scans.

Version **7.3.5.4** (service, setup and updater), fetched 2026-11-06 from the official download
link `https://www.fortect.com/go/route-website.php?d=t&type=windows`, which redirects via
`util.fortect.com` to `https://cloud.fortect.com/app/installation/downloader/8011/Fortect.exe`
(server `Last-Modified` 2026-10-29). No account or payment was needed.

| File | Size | SHA-256 |
|---|---|---|
| `Fortect.exe` (online stub) | 846,416 | `10b5664330eb236b311adb2c9fadc9b8c183db891ca9c8786319df12d03edb0f` |
| `FortectSetup64.7z` (product) | 116,859,889 | `b299a5e44261a3ee1aff56a4944f151da261ab4853a80d93ecdd72aea1b12fc4` |
| `FortectProtection64.7z` (daemon) | 1,638,886 | `05a26a198db581f5d232efe894474e9ecf8d28f6f8002225501f9ab3971baf1a` |
| `MainService.exe` | 7,557,688 | `e7d476e63131ffccef0e73bc82a074dc55d3b02620982ab923fe3808e3d8ec03` |
| `FortectMain.exe` (Electron) | 199,443,000 | `62e8ce206d218ea028f6d6b1f191f00a31011de3a5ea5e3aae0a21845a7854d7` |
| `resources/app.asar` | | `b515c838b073fa22ca3f9f0f45dec89e1b4168d2d7d676ed63b00c9b415a9e41` |

**Nothing was run.** Static analysis only: strings, the Electron bundle and its shipped source map,
and a short look at `MainService.exe` in Ghidra. Decoded and decompiled output was deleted.

## How it's built

**Installer.** `Fortect.exe` is an NSIS stub with no payload (plugins `INetC`, `nsJSON`,
`nsis7z`, `ShellExecAsUser`, a custom `fUtil.dll`). It downloads
`https://app.fortect.com/events/version.php?data=json&sessionid=`, a JSON manifest (the server
also printed PHP "Undefined index" notices in front of the JSON) that names the two 7z packages
under `https://cloud.fortect.com/app/installation/engine/7354/` and `.../service/7354/` with their
SHA-256 (both matched). It sends install events to `ev-install-start.php`, `ev-install-end.php`
and `events.php` on `app.fortect.com` (event IDs such as `InstallJsonDownloadError`,
`MainAlreadyRunning`, `InstallerScanFirstRun`, and "Illegal silent run" when started silently
without permission). The stub then starts the first scan itself (`--firstRun`; also
`--driverScan`, `--withoutDriver`).

**Product** (`7z` package, 151 files, 424 MB unpacked):

- `FortectMain.exe`: Electron/Chromium front end. `resources/app.asar` holds an obfuscated
  `build/main.js`, plain-text main-process modules (`CategoryScan.js`, `CategoryParser.js`,
  `DriverUpdater.js`, `Vpn.js`, `Campaigner.js`), a native `ClientInterface.node` that talks to
  the service, and the React UI **with its full source map** (`main.323633fa.js.map`, 660 own
  source files). The scoring logic below was read from that map.
- `MainService.exe` (C++, Chilkat, PDB path `C:\projects\fortect.engine\engine\...`): the
  `FortectService` engine. Classes `EventLog`, `StabilityCalculator`, `TempFilesScanner`,
  `UnInstallAppsJunkScanner`, `BrowsersCookiesManager`, `AsyncJumpListRepair`,
  `AsyncOfficeRepair`, `RestorePoint`, `MSAcpiThermalZoneTemperature`, the driver updater and
  the SAVAPI malware scanner.
- `MainDaemon.exe` (`FortectDaemon` service, separate package), `MainProtection.exe`,
  `MainNetShield.exe` (`FortectNetShield` service), `AvProxy.exe`, `remediation.exe`,
  `bin\FortectTray.exe` (tray), `uninst.exe`.
- **Avira engine**: `savapi.dll`, `apcfile.dll`/`apchash.dll` (Avira Protection Cloud),
  `programdata\av\avupdate.exe`, `HBEDV.KEY` (Avira licence key file), definitions from
  `https://cloud.fortect.com/updates/av/`.
- **Config**: `programdata\conf\conf.json` (357 KB), `settings.json`, `sil.json`, `vp.json`,
  `netShield.json`, `cron_MainService.json`, `cron_MainNetShield.json`. All are encrypted
  (16-byte-block cipher with a fixed key/IV: three files share the same first block). The key was
  not recovered statically, so the engine-side thresholds (event-log windows, junk folder lists,
  registry rules) are **not** read; their key names are known from `MainService.exe`
  (`eventLog.daysBackToReadEventsFrom`, `eventLog.frequentErrorApps`, `eventLog.windowsEffecting`,
  `eventLog.windowsInstallation.Factor`, `junk.folders`, `junk.exclude`, `privacy.office.apps`,
  `jumpLists.appIds.`, `malware.savapi.DetectPUA`, `malware.savapi.HeurLevel`, …).

## Background services, drivers, telemetry, offers

**Red flags, in order of weight:**

1. **Up to eight kernel drivers in the package**, more than any product researched here so far:
   - Avira `avgntflt.sys` (Avira Minifilter Driver 15.0.2110.297, `Class = AntiVirus`,
     `StartType = 2`, auto start), `avipbb.sys`, `avkmgr.sys` (`on_access\win64\win7|win8`).
   - Avira `amselam.sys` (Anti-Malware SDK ELAM driver) plus `AMSProtectedService.exe` /
     `AMSAgent.exe`, which run as a protected (PPL) anti-malware service
     (`SecurityProductInformation.ini`: "Fortect Security Suite").
   - `fns.sys` "Fortect NetShield Driver" 1.7.0.7, a WFP callout driver
     (`Class = WFPCALLOUTS`), with NetFilter SDK-style `nfapi.dll`/`ProtocolFilters.dll` for web
     filtering.
   - Callback Technologies CBFS Filter drivers `cbfilter`, `cbprocess`, `cbregistry` (cabinets
     plus `CBFilterInst.dll` etc.; file, process and registry filter drivers, used here for
     self-protection or on-access scanning: **inference**, the call site was not traced).
   Which of these the free scan installs, and which only come with "real-time protection"
   (`AvInstallFlow`, `ams_setup.exe`), was not traced.
2. **Several always-on services and a tray**: `FortectService`, `FortectDaemon`,
   `FortectNetShield`, `AMSProtectedService`, plus `FortectTray.exe`. The installer starts the
   services itself and retries ("installed, but service not starting - reinstall").
3. **Telemetry by design**: `app.fortect.com/events/` endpoints `events.php`, `evt_scan.php`,
   `scan.php`, `getScanData.php`, `getSettings.php`, `getLicenseInfo.php`,
   `getSubscriptionInfo.php`, `upsale.php`, `netShield.php`, `vpn.php`, `cloud-scan.php`,
   `ev-service-uninstall.php`. Suspicious files are uploaded to Avira Protection Cloud
   (`malware.cloud.*`, `APCUploadRiskRatingLevel`; UI: "flagged as suspicious and is being scanned
   in the cloud"). There is a log-upload feature (`log-upload-start`).
4. **Remote-controlled scoring**: the UI asks the main process for a severity table
   (`get-severity-score`) and stores it in `localStorage.scoreJson`, which then **replaces** the
   built-in thresholds below. The vendor can change what counts as "minor" without an update.
5. **Upsells everywhere**: pricing modal, VPN upsell with "Upgrade Now for {{price}}" — "will
   charge the discounted price right away" (inference: a one-click charge to the payment method on file),
   "Exclusive premium discount", "Grab a special discount to keep peak PC performance", an
   exit-intent screen ("Before you close, discover more features for Free"), driver-updater "10x
   Faster Download Speed". A "One Time Repair" licence is unregistered automatically after one
   repair (`unregister-license` when `licenseType === 'onetime'`). Renewal text: "Renew to
   restore peak performance". No auto-renew wording was found in the app; billing is on the
   website (Paddle).
6. **Scare framing**: "Crashing applications can cause data loss", "Viruses can invade the Windows
   registry … the registry is left corrupt", "Unnecessary and temporary files can slow down your
   PC", stop-scan warnings that items "potentially affecting your PC's performance".

## What the scan checks and how each count is computed

The engine reports each category to `CategoryParser.js`, which builds the numbers the UI shows.
The headline "Repair N items" is the sum of the category totals.

| Category | Source (engine) | What is counted | Notes |
|---|---|---|---|
| **Crashed Programs** | Windows Event Log (`EventLog::loadEvents`, "application error"), config `eventLog.*` | The **sum of crash events**, not programs: each program's `count` is added to the total. Split into "Windows" and "3rd party". Crashes of any path matching `avg`, `avast` or `avira` are **dropped** from the list | One program that crashed 30 times over "the past several months" shows as 30 items. The fix for this category is the Stability Repair (below), which does not touch the crashing program |
| **Stability grade** | `StabilityCalculator` from the same events (formula in the encrypted config) | A 0–100 grade | Weighted 40 % in the overall score |
| **Junk Files** | `TempFilesScanner`, `UnInstallAppsJunkScanner` (`junk.folders`) | Number of files, plus total size | The size, not the count, feeds the score |
| **Broken Registry** | Registry scanner | Number of entries | "missing, outdated or other registry entries that may impact your PC's stability" |
| **Browser cache** | Per browser | Number of cache files and size | Shown under "Privacy traces" |
| **Cookies** | Per browser | Number of cookies | Each cookie is an item |
| **Office apps** | `privacy.office.apps` | Recent-file traces per Office app | |
| **Jump Lists** | `AsyncJumpListRepair` | Number of jump-list entries per app | Earlier builds called this "Redundant Windows Files"; that category is commented out and re-labelled |
| **Malware / PUA** | Avira SAVAPI (`DetectPUA`, heuristics, cloud) | Detections. Anything whose name matches `pup`, `pua`, `adware`, `reputation`, `not malicious`, `generic suspicious`, `not-a-virus` … goes in "PUA", the rest in "Malware"; an empty name becomes "Mal/Win32.Generic PUA" | Real AV detections from a licensed commercial engine. PUA and "reputation" hits are counted as "threats" too |
| **Drivers** | Driver updater (`driver-scan-result`) | Outdated drivers | 2 or more always block the "minor" result |

**Overall severity** (`pages/FullScan/severityCalculator.js` and `ScannerFooter.jsx`):

- Any malware (more than one) gives the worst result; exactly one malware hit **or any PUA**
  gives the second-worst. Two or more outdated drivers also block "minor".
- Otherwise each category becomes a 0–100 grade by fixed thresholds, then
  `0.4 × stability + 0.25 × registry + 0.25 × junk + 0.1 × privacy`.
  Registry: **10 entries** already drops it to 90, 100 to 80, 1,000 to 0. Privacy: 500 traces
  to 90 … 2,700 to 0. Junk: below 10 MiB counts as nothing; otherwise junk bytes divided by free
  disk bytes is compared with thresholds 1 … 80, so on a normal disk it stays at 100 — but if the
  free-space value is 0 (not reported) the ratio is forced to 100 and junk scores 0.
- The result is "minor" only above **95**; 85–95 and 30–85 are the two middle bands, 30 or less
  is the worst. One registry band down (for example 100–149 entries, grade 80) caps the total at
  95, so a PC with a few hundred ordinary registry leftovers can never reach "minor", even with no
  crashes.
- When the result is "minor", the app does say so honestly: "Minor Irregularities Detected …
  Don't be alarmed! … junk files and cache … are typical and aren't harmful".

**Hardware summary** (`pages/FullScan/partials/HardwareSummary.jsx`): your PC is compared with
"Fortect users" ("comparing the hardware specification of your PC with other PCs in our client
network"), but the comparison values are **constants in the UI**: 16 GB RAM, 1,200 MBps disk,
8 cores, 4 GHz, and a CPU temperature of **18 °C**. The scales are fixed at 64 GB, 6,000 MBps,
16 cores, 5 GHz and 100 °C. The CPU temperature comes from WMI `MSAcpi_ThermalZoneTemperature`,
which many PCs don't expose; when the value is not an integer, the UI shows a **random number
between 60 and 65 °C**.

## What "repair" changes

- **Free**: "View & Fix" lets you fix one section at a time for free (with a 24-hour trial for
  "all scan results" in some flows); one-click "Repair all", "Auto Repair" and deep malware scan
  are premium (`isPremium` = an `annual` licence that hasn't expired).
- **Junk, cache, cookies, Office traces, jump lists**: deleted. The UI warns "this action cannot
  be reversed".
- **Registry**: entries deleted ("Once repaired, the changes can't be reverted"). A system restore
  point is created for driver installs (`SRSetRestorePointW`, `autoCreateSystemRestore`); a
  general registry backup was not found.
- **Stability Repair** ("Verifying and Repairing Core OS Components"): the engine runs Windows'
  own **System File Checker**: `sfc.exe /SCANNOW` and `sfc.exe /SCANFILE=` (strings in
  `MainService.exe`, `handleSFCRepair*`). The UI says "It's normal to see the 406 components in
  every repair … we've verified the health of these 406 key components", so the "repaired
  components" count is shown on every repair whether or not anything was wrong. The Restoro-era
  repair (replacing system files from the vendor's own file database) was not found in this build.
- **Crashed programs**: no change to the program itself; the crashes are "solved" by the SFC pass
  above, and the list then only counts crashes "since you last ran Fortect repair ({{date}})", so
  they drop out of the next scan. Afterwards, the UI tells the user "It's common to
  see new items … if you're still experiencing PC malfunctions, we suggest a malware scan".
- **Malware/PUA**: quarantined by the Avira engine.
- **Drivers**: downloaded and installed from the vendor's driver feed, with a restore point.

## Claims vs reality

Product UI and fortect.com (2026-11-06): "Repair N items to get a cleaner and faster PC",
"Repair N items to optimize your PC performance today", "Boost Performance", "Upgrade to restore
peak performance", crash fixes that "prevent application crashes", hardware compared with "our
client network".

| Claim | What the code does | Verdict |
|---|---|---|
| "Repair N items" (scan total) | Sum of crash *events*, junk files, registry entries, cache files, cookies, Office traces, jump-list entries, threats | **Misleading**: almost all of the number is individual cookies, cache files and registry leftovers that cost nothing. One crashing program counts once per crash |
| PC health / severity ("issues detected" vs "minor") | Fixed weighted thresholds, remotely replaceable. 10 registry entries already lower the grade; "minor" needs > 95 | **Misleading**: tuned so that ordinary leftovers push most PCs out of "minor". It is not hard-coded to always find problems, and a clean result is shown honestly |
| "Crashed Programs … may impact your PC's performance", fix "prevents application crashes" | Reads the event log; fix runs `sfc /scannow` | **Misleading**: SFC is a free Windows tool (`sfc /scannow` in an admin prompt). It only helps if Windows system files are actually corrupt; it does nothing for a crashing third-party program. Crashes of AVG, Avast and Avira are hidden from the list |
| "Verifying 406 core Windows components" in every repair | SFC run; the count shows every time | **Placebo framing**: a fixed reassurance number, not a finding |
| Hardware compared with "Fortect users" | UI constants (16 GB, 1,200 MBps, 8 cores, 4 GHz, **18 °C**) | **Misleading**: no network data; 18 °C is below room temperature, so every real CPU looks hot by comparison |
| CPU temperature | WMI ACPI thermal zone, else a **random 60–65 °C** | **Misleading**: a made-up reading on PCs without that sensor |
| "Unnecessary and temporary files can slow down your PC" | Deletes temp files, caches, cookies | **Placebo** for speed (frees disk space only; caches rebuild, sign-ins are lost) |
| Registry "left corrupt by viruses", repair keeps it "smooth" | Deletes orphaned registry entries, irreversible | **Placebo / scare framing** |
| "Boost Performance" | Junk + crashed-programs scan | **Placebo**: no power, CPU, GPU, memory or scheduler change exists |
| Malware/PUA detection | Licensed Avira SAVAPI engine, cloud lookups | **Real** (third-party engine). PUA/"reputation" hits are also counted as "threats" |
| Driver updates for "peak performance" | Vendor driver feed | **Harmful on Boot Camp**: generic drivers can replace Apple's |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing here makes BF6 faster. Fortect adds four services (start types not traced), a tray app, up to
eight kernel drivers (file system, process, registry and WFP filters plus an ELAM driver), cloud
file uploads and event telemetry: background load and anti-cheat surface on a
heat-limited 4-core CPU, the opposite of the goal. Its driver updater can replace Boot Camp
drivers. The one real repair, `sfc /scannow`, is built into Windows. Don't install it.

## Adoption items for gamebooster-rs

1. **Nothing to adopt.** Fortect has no game, power, priority, service or memory tweak.
2. **`sfc /scannow` as a hint**: **skip**. It is free, built in, slow, needs admin and is
   unrelated to frame rate. If Windows itself is crashing, the user can run it by hand.
3. **Show real numbers only**: **already done**. gamebooster-rs `status` reports measured values
   and changes nothing; Fortect's constant "our users" bars and random temperature are a reminder
   to never show a placeholder as a reading.
