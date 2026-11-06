# AVG TuneUp: what it really is

AVG TuneUp is Gen Digital's Windows "speed up and clean up" suite, sold at
[avg.com](https://www.avg.com/en-us/avg-pctuneup) (7-day free trial, then about $70/year list).
It is the same engine as Avast Cleanup Premium: the installer migrates an old `CleanupPSvc` service,
and the Sleep Mode code is shared with CCleaner (its database is called
`CCleanerProgramDeactivator.db` and the UI has strings for programs "put to sleep by CCleaner").

The current product is a cleaner, not a game booster. It has **Sleep Mode** (formerly *Program
Deactivator*), **Automatic Maintenance**, a registry cleaner, disk/browser cleanup, a duplicate
finder, a disk defrag/SSD optimize page, a Software Updater, a Driver Updater and **Fix Issues**
(security and privacy settings checks). The old TuneUp Utilities modes (**Turbo**, **Economy**,
**Gaming mode**, Live Optimization) **no longer exist**: none of these words are in the service, the
UI bundle or the English strings, and the only "Turbo" strings are names in its crapware uninstall
list (MyTurboPC, PC Turbo Cleaner). There is no power plan, priority, RAM trim or timer code.

Version **26.9.19073.13346** (product-info build time 2026-10-29), fetched 2026-11-06 from the
official trial link `https://honzik.avcdn.net/setup/avg-tu/release/avg_tuneup_online_setup.exe`
(served 2026-11-02). Online stub SHA-256
`92ca6729d0d1483fc86d713f0d59447095735210081ee687e9b209bc958ac102` (1 901 024 bytes, product name
"AVG Installer", ProductId `avg-icarus`, Sectigo-signed). x64 `TuneupSvc.exe`
`b20e762ed74989d71165da239d2953591669e1bf76b6c1998a9f9e4b467cf6bb`, x64 `pdfix.exe`
`5d7946dd66a435757f41dfbe15e0e061cca90b4de6086c442aeb5a805a7b23a7`, x64 `TuneupUI.exe`
`8e98e5084a7893691caa779c03f02706ae1c48e4ef0fb1b0e5d5b49f8e2897ef`. No account or payment was
needed to get the files. Nothing was executed. Static analysis only.

## How it's built

**Installer (Icarus).** The 1.9 MB stub (`icarus_sfx`, needs admin) has no payload. It reads its
embedded `sfx-info.xml` (product `avg-tu`, channel `release`, repository `honzik.avcdn.net`, remote
config `https://shepherd.avcdn.net/`, branding `branding.avg.com/PROD`) and then downloads, in order:

1. `https://honzik.avcdn.net/defs/avg-tu/release.xml.lzma` (also served unpacked as `.xml`): the
   signed setup manifest (`icarus.exe`, `icarus_product.dll`, `icarus_ui.exe`, `icarus_rvrt.exe`,
   `bug_report.exe`, `dump_process.exe`, `product-def.xml`, `setupui.cont`; x86, x64 and arm64).
2. Every file from a content-addressed store:
   `https://honzik.avcdn.net/universe/<h[0:4]>/<h[4:8]>/<h[8:12]>/<h>.lzma`, where `h` is the
   file's SHA-256 (not its `src-id`). The files are LZMA-alone and carry an `ASWSig2A` DSA signature
   that is checked before use.
3. `product-def.xml` (519 files) lists the product. We downloaded only the x64 binaries we needed
   plus the config and UI strings, all from the same store, with the hash checked.
   The stub also posts install analytics to `https://analytics.avcdn.net/v4/receive/json/25`.

**Product.** `C:\Program Files\AVG\TuneUp\` (`%PRODUCT_INST%`), with data in `%PRODUCT_DATA%`:

- `TuneupSvc.exe` (23 MB, C++): the `AVGTuneUpSvc` service, LocalSystem, OwnProcess, depends on
  `RpcSS`. It is **Auto start** when `usercfg.ini` `[General] KeepProcsRunningAfterClose` = 1,
  which is the default. Otherwise it is Demand start. It holds every module as a JSON-RPC
  namespace: `app.ProgramDeactivator`, `app.AutoMaintenance`, `app.BackgroundScan`,
  `app.RegistryCleaner`, `app.DiskCleanup`, `app.BrowserDataCleanup`, `app.DiskDefrag`,
  `app.DriverUpdater`, `app.SoftwareUpdater`, `app.DuplicateFinder`, `app.FileRecovery`,
  `app.dataShredder`, `app.RescueCenter`, `app.BurgerClient`, `app.NPS`, `app.IPM`,
  `app.CardSystem`, `app.lic` and others.
- `TuneupUI.exe` (CEF/Chromium front end; `Resources/dist/app-bundle.js`, `en.json`), tray class
  `AVGTuneUpTrayWndClass`, also started after close (`KeepUIRunningAfterClose=1`).
- `pdfix.exe` ("AVG PD Fix"): the Sleep Mode engine and repair tool. Install runs `/fixifeo`,
  uninstall runs `/revertall`.
- `autoreactivator.exe`: the "wake up" program that Sleep Mode puts in front of sleeping programs
  (see below).
- `tucontextmenu.dll` (registered with `regsvr32`; Explorer context menu), `gf2hlp.exe`,
  `su_worker.exe`/`su_*.dll` (Software Updater), `wa_3rd_party_host_*.exe` (OPSWAT-style
  `libwa*` third-party app detection), `AvBugReport.exe`, `avDump.exe`, `SupportTool.exe`.

**No kernel driver.** The product manifest has no `.sys` file and no driver install step.

## Background activity and telemetry (red flags)

- **Always-on service plus tray UI** by default (above). Background scan starts 1800 s after boot
  and repeats every 168 h. Install and browser monitors run daily. The Driver Updater background
  scan runs every 7 days (`BackgroundScanCPULimit=50`). The service also syncs time with
  `pool.ntp.org`.
- **Telemetry on by default.** "Burger" analytics to `analytics.avcdn.net` (install, heartbeat,
  update, activity and uninstall categories all enabled). `config.def` also has a Google Analytics
  ID `UA-58120669-51`. Crash dumps and feedback reporting are on (`DumpReporting=1`,
  `SendCrashReports=1`), and there is a Qualtrics NPS survey for the first 60 days.
- **Opt-out "GDPR" defaults**: `usercfg.ini [GDPR]` ships with `salesOnlineContentEnabled=1`,
  `thirdPartyOfferEnabled=1` and `dataAnalysisIn3rdPartyToolsEnabled=1`.
- **Bundled offers.** In-product messaging (IPM cards, `IpmBuyElms` for VPN and Driver Updater).
  `config.def [LIS]` holds direct installer URLs for AVG AntiTrack, BreachGuard, Driver Updater,
  Secure VPN, Battery Saver and Internet Security.
- **Remote configuration** from `shepherd.avcdn.net`. Modules register "Shepherd update config"
  handlers, so behaviour can change server side.
- **Scare framing.** The registry cleaner says broken entries "may cause crashes and performance
  issues". The dashboard cards count "issues waiting for your action" (see Claims).

## Feature catalog (gaming-relevant)

| Feature | Exact change | Revert | Driver | Risk |
|---|---|---|---|---|
| **Sleep Mode** (Program Deactivator) | For each program the user puts to sleep, it disables the program's startup and background entries. That is Run entries (through `Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\...`, plus the MSConfig `startupreg`/`startupfolder` keys on old Windows), Start-menu Startup shortcuts, AppX startup tasks, scheduled tasks (Task Scheduler COM) and services. A service is stopped (`ControlService` stop) and its start type changed, with the original start type kept as `origStartType`. Then it takes over launching: it writes `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<exe>` `Debugger` = `autoreactivator.exe`, scoped to the exact file with the Windows 10 IFEO filter subkeys (`UseFilter`, `FilterFullPath`). It records every hijack in a SQLite table `DebuggerRegistration (Exe, TUID)`. When you start a sleeping program, Windows runs `autoreactivator.exe` instead, which "wakes" it: it restores the entries and runs the program, and it is "put back to sleep after you close it and your PC is idle, or after you restart". | Per program, or "Wake all programs". `pdfix.exe /revertall` at uninstall, `/fixifeo` repairs leftover IFEO keys. Backups in Rescue Center (`pdBackupEnabled=1`, kept 30 days) | no | **Medium.** Persistent across reboots. It hijacks program launch through IFEO, the same mechanism malware uses, and it breaks if TuneUp is removed badly. Its never-sleep list covers security suites, Steam (`steamservice.exe`), Riot Games/Vanguard and NVIDIA, but **not** EA (EA app, `EABackgroundService.exe`, Javelin), EasyAntiCheat, BattlEye or AMD. Those programs are not protected, so a user can put them to sleep |
| **Automatic Maintenance** | Scheduled (`[AutoMaintenance]` day/hour/interval). It runs the chosen cleaners: registry cleaner (`regIssuesLimit`), shortcut cleaner (`scIssuesLimit`), browser data cleanup (`bdcuCookiesLimit`) and trash (`trashSizeLimit`). It skips when CPU is busy, the UI is open or nobody is logged in | Registry/shortcut fixes are undoable from the Undo Center / Rescue Center (30-day backups) | no | Low for performance. It deletes caches and cookies, which can make the next game or browser start *slower* |
| Registry Cleaner | Deletes "broken" keys in categories: SharedDLLs, Open With, obsolete software, MUICache, installer folders, help files, fonts, App Paths, autorun, recent files | Rescue Center backup | no | Low, no speed effect |
| Disk Cleanup / Browser data / System junk | Deletes temp, cache, log, dump and setup files (`TUBrowserCleanupTC.tudb` cleaning rules) | no | no | Low; clears shader/browser caches that get rebuilt |
| Disk Defrag / SSD optimize | `app.DiskDefrag` with a schedule. "Optimize your SDD to prevent excessive wear" on SSD (Windows' defrag engine; it can open the Windows UI) | n/a | no | None worth having: Windows already does scheduled optimize/TRIM |
| Fix Issues | Checks UAC on, network/registry/share access off, certificate warning on, antivirus/firewall warnings on, Windows Update on, and **AutoEndTasks off** / **shutdown timeouts long enough** (from UI text; exact value names not traced) | Toggle back | no | None. This is security hygiene, and it *undoes* classic booster tweaks (AutoEndTasks, short WaitToKill timeouts) |
| Driver Updater / Software Updater | Downloads drivers from `driver-updater.svc.avast.com` / `drup.avcdn.net` and third-party updates | Windows driver rollback | no (installs vendor drivers) | **High on Boot Camp**: generic driver updates can replace Apple's Boot Camp drivers (keyboard, trackpad, audio, the AMD GPU driver) |

## Claims vs reality

Product page (avg.com, 2026-11-06): "The results speak for themselves … **79% quicker startup**
(89 → 18 s), **30% faster work performance** (7 990 → 10 421 points), **71 GB cleaned up**
(80 → 151 GB free)", "patented Sleep Mode technology to help your programs run faster, your PC start
quicker", "AVG TuneUp is proven to improve your PC's performance". No test method, PC or footnote is
given on the page.

| Claim | What the code does | Verdict |
|---|---|---|
| "79% quicker startup" | Sleep Mode disables Run entries, startup tasks and auto-start services, so fewer programs start at logon | **Misleading**: real only on a PC that starts dozens of programs. 89 s to 18 s is an extreme, unspecified test machine. Disabling startup apps in Task Manager does the same thing without an IFEO hijack. Zero effect on game FPS |
| "30% faster work performance" (benchmark points) | Nothing in the code changes CPU, GPU, memory or disk behaviour. The only runtime effect is fewer background processes | **Misleading**: a benchmark score only rises that much if the "before" PC was swamped by background apps. There is no power, priority, scheduler or memory code to make a clean PC faster |
| "Sleep Mode … programs run faster … wakes them up when you need them — and not a moment sooner" (patented) | IFEO `Debugger` = `autoreactivator.exe` with filters, plus disabled startup entries, tasks and services | **Real but persistent**. It removes background CPU and RAM use from apps you don't use. The "patent" is a launch hijack that Windows supports for debuggers. It changes system state across reboots, and the protected list misses EA and the major anti-cheats |
| "Automatic maintenance … better performance every time you turn it on" | Weekly registry, shortcut, cookie and trash cleanup | **Placebo** for speed. Registry cleaning does nothing measurable on Windows 10/11. Cache cleaning frees disk space only |
| Registry cleaner "fixes registry errors that can slow your PC" / "may cause crashes" | Deletes orphaned MUICache, recent-files and App Paths entries | **Placebo / scare framing** |
| Scan "issues" count | Smart Scan runs the cleaner, registry, shortcut, Sleep Mode, Software Updater and Fix Issues scans and reports one `SmartScanIssuesCount`. The cards say "N issues waiting for your action", and the registry cleaner shows "N broken registry items found", counting each key. The exact sum was not traced in the UI bundle | **Misleading**: the "issues" are mostly ordinary leftovers (MUICache, recent files, cookies, temp files) that cost nothing. Only the Fix Issues security checks are real findings, and none of those concern speed |
| Defrag/optimize "for more speed" | Windows' own defrag/TRIM engine on a schedule | **Placebo** on SSDs (Windows already does it); real only on a fragmented hard disk |
| Driver Updater keeps drivers current "to improve PC performance … graphics quality" | Third-party driver feed | **Harmful on Boot Camp**: Apple's driver set is the safe one; a generic AMD or audio driver can break brightness, audio or sleep |
| (Older marketing) Turbo / Economy / Gaming mode | **Not present** in this version | n/a |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing here makes BF6 faster. Sleep Mode is a persistent startup manager. It overlaps with
gamebooster-rs's `close` and `services` (which are session-only and undone by `restore`), but
TuneUp does it with IFEO launch hijacks, which also survive reboots. Putting the EA app or its
background service to sleep would place `autoreactivator.exe` in BF6's launch chain. That is a
risk with EA Javelin, which is not on TuneUp's never-sleep list. The Driver Updater can replace
Boot Camp drivers. The always-on service, tray UI, scheduled scans, telemetry and offers add
background load on a heat-limited 4-core CPU, which is the opposite of the goal. Don't install it on
the target Mac.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **Nothing to adopt.** TuneUp has no session tweak that gamebooster-rs lacks. Its only
   performance feature (Sleep Mode) is a persistent version of our session-only
   [close apps](../tweaks/close-apps.md) and [services](../tweaks/services.md).
2. **Stop only what is actually heavy (memory-ranked suggestions for `close`)**: **try by feel,
   low priority**. Sleep Mode logs each service's private working set before stopping it and ranks
   programs by "performance impact". A read-only listing in `status` of the user's background
   processes by private working set could help pick `close = [...]` entries. It changes nothing, so
   there is nothing to restore. Only worth building if choosing the `close` list by hand turns out
   to be hard.
3. **IFEO `Debugger` hijack to keep apps asleep across launches**: **skip**. It is persistent
   (survives a crash or reinstall, so `restore` can't promise to undo it), it is the same mechanism
   malware uses, and anti-cheats look at what launches the game.
4. **Never-sleep list as a source for `NEVER_LOWER_PUBLISHERS`**: **skip**. Our built-in list in
   `src/priority.rs` already covers TuneUp's entries (security suites, Valve, Riot, NVIDIA) and
   more (Electronic Arts, BattlEye, AMD, Apple).
5. **Registry cleaner, Automatic Maintenance, defrag/SSD optimize, Driver Updater**: **skip**.
   They are placebo for games or harmful on Boot Camp.
6. **Fix Issues' AutoEndTasks / shutdown-timeout checks**: **skip** (not gaming). They confirm that
   gamebooster-rs is right not to touch `AutoEndTasks` or `WaitToKillServiceTimeout`.
