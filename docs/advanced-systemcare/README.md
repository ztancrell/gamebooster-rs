# Advanced SystemCare Free (IObit): what it really is

Advanced SystemCare (ASC) is IObit's all-in-one "PC optimizer": a one-click scan over 15 modules
(registry, junk, privacy, "spyware", Internet Boost, system optimization, …), plus **Turbo Boost**
(Work / Game / Economy modes), **AI Mode**, **Performance Monitor**, **Auto RAM Clean**, a browser
and privacy "shield", and a large upsell layer for Pro. It is the parent product of
[Smart Game Booster](../sgb/README.md): Turbo Boost is SGB's Boost under another name, with the
same data format, the same power-plan DLL and nearly the same service lists.

Version **20.0.1.256** (installer and `ASC.exe`), from the official
`https://www.iobit.com/downloadcenter.php?product=advanced-systemcare-free-new`, which redirects to
`https://cdn.iobit.com/dl/advanced-systemcare-setup.exe` (no account, no payment). Downloaded
2026-11-06, server `Last-Modified` 2026-10-16, SHA-256
`abea2697a4d50a3cc6a9054b6387c6be5b1774f901ace7c2ce13831d6e10a7f1`, 61,146,528 bytes.
**Nothing was run.** Static analysis only. Technical notes:
[decomp/iobit/advanced-systemcare/](../../decomp/iobit/advanced-systemcare/README.md).

## How it's built

- **Installer:** Inno Setup 5.5.7 (Unicode), unpacked with innoextract 1.9: 348 files, 204 MB.
  The real setup UI is a second program, `{tmp}\Installer\Setup.exe` (Delphi), which shows the
  offers (below).
- **Programs:** Delphi (VCL, `rtl120`/`vcl120` packages, madExcept), about 60 executables and
  DLLs: `ASC.exe` (main window, scan, Turbo Boost, Internet Boost), `ASCService.exe` (service),
  `ASCTray.exe` (tray), `Monitor.exe` (Performance Monitor), `AutoRamClean.exe`,
  `AutoCare.exe`, `diagnosis.exe`, `PowerConfig.dll` (power plans), `HardwareLib.dll` and
  `CPUIDInterface.dll`/`cpuidsdk.dll` (sensors), `RealTimeProtector.exe`, `PrivacyShield.exe`,
  `BrowserProtect.exe`, `DNSProtect.exe`, `IObitLiveUpdate.exe`, `ProductStat3.exe`.
- **Data:** `Database\*.dbd`, the same encrypted container as SGB's (HAVAL-256 key, AES-256-CBC,
  zlib; [format](../../decomp/iobit/smart-game-booster/SgbData/dbd/README.md)). All of the
  performance data was decoded: `TurboBoostGame.dbd` (password `asc5_TurboBoost`), `Opt.dbd`
  (`asc6_2012_6`), `Reg.dbd` and `PriTemp.dbd` (`asc4_2010`). Decoded output was deleted.

## Background services, drivers, telemetry, offers

**Red flags, in order of weight:**

1. **A WinRing0-derived kernel driver.** `drivers\Monitor_win10_x64.sys` ("IObit Temperature
   Monitor" 15.0.0.3; also `Monitor_win7_x64.sys`, `Monitor_x86.sys`) creates
   `\Device\IOBIT2021_WinRing0_1_3_0`; its PDB path says it was built from WinRing0. It is loaded
   by `HardwareLib.dll` (used by `ASCInit.exe` and `CPUIDInterface.dll`) to read CPU
   temperatures. WinRing0 is the driver family that gives any program raw MSR and port access;
   Defender flags the original, and [DESIGN.md](../../DESIGN.md) already rules it out for us.
2. **Four more kernel drivers** (win7/win10, x86/x64/ia64 builds):
   - `NetDataFlow.sys` (20.0.0.20): a WFP callout driver (`FwpmCalloutAdd0`, `FwpmFilterAdd0`),
     loaded by `Monitor.exe` for per-program network traffic.
   - `AscFileFilter.sys` and `AscFileControl.sys`: file-system minifilters (FltMgr) behind the
     privacy and "safe folder" protection, loaded through `rgfpctl.dll`/`rgfpctlextend.dll`.
   - `AscRegistryFilter.sys`: a registry callback driver (`CmRegisterCallbackEx`) for the
     homepage and registry protection.
   - `RegistryDefragBootTime.exe` per Windows version: a native boot-time program for Registry
     Defrag.

   A minifilter, a registry callback and a WFP callout on a gaming PC are exactly what kernel
   anti-cheats look at.
3. **Always-on processes.** The LocalSystem service `AdvancedSystemCareService20` (`ASCInit.exe`
   also references the keys of services 12 to 19 from older versions, presumably to clean them up), the `ASCTray.exe` tray, and `Monitor.exe`
   ("Enable Performance Monitor and load it at Windows startup"). The installer form has *Load
   ASCTray* and *Load ASCService* checkboxes, both ticked. Scheduled tasks: `ASC_SkipUac_…` /
   `ASC14_SkipUac_` (start ASC elevated without a UAC prompt), `ASCTaskASC`,
   `QuickSettingTaskASC`, and AutoCare / AutoClean / AutoUpdate tasks (`ASC15_*`).
4. **Telemetry with no off switch found:** `stats.iobit.com` and `ascstats.iobit.com` (in about 25
   binaries, the RAM cleaner, Turbo Boost and the installer among them), `aegis.iobit.com` (plus
   `aegis-test`), `*.cd4o.com` (`InfoHelp.dll`), and IP geolocation through `ipinfo.io` and
   `ip-api.com` (`ASC.exe`, `ASCTray.exe`, `Monitor.exe`, `Setup.exe`). `SendBugReportNew.exe`
   uploads crash reports.
5. **Bundled offers in the installer:** an **iTop Screen Recorder** page ("By clicking
   "Accept (%s)" to agree and install iTop Screen Recorder…", with Accept / Decline / Skip buttons;
   the `(%s)` looks like a countdown) and an **iTop VPN** recommendation (`itopvpn.com` EULA and
   privacy links). There is also an *Add Screenshot tool* checkbox, which installs `ScreenShot.exe`
   as "iTop Screenshot". The app itself advertises Driver Booster, IObit Uninstaller and iTop VPN.
6. **Upsell and scare copy:** "85% OFF Special Offer", "Speed up your PC up to 200% faster",
   "Boost Internet up to 300% faster", "Guaranteed to make your PC run faster … 100% money back
   guarantee", "Turbo Boost: OFF" counted as a problem, "outdated drivers may slow down &
   destabilize your PC", and tracking cookies reported as "Spyware threats" (see below).

## Performance features: what they change

Revert column: what undoes it. "Driver" means it needs one of the kernel drivers above.

| Feature | Exact change | Revert | Driver? | Risk |
|---|---|---|---|---|
| **Turbo Boost: services** | Stops the ticked services in the mode's list (`TurboBoostGame.dbd`: `Service_Game.ini` for Game, `Service_Economy.ini` for Economy, presumably `services.ini` for Work). Game mode on Win10/11 has 77 entries, 61 ticked; unticked by default: `BITS`, `BDESVC`, `bthserv`, `DPS`, `WdiServiceHost`, `WdiSystemHost`, `hidserv`, `SharedAccess`, `MSiSCSI`, `RemoteRegistry`, `SSDPSRV`, `Wcmsvc`, `DiagTrack`, `MapsBroker`, `PhoneSvc`, `WSearch` | Turning Turbo Boost off (not traced; SGB restarts what it stopped) | no | Same as SGB's list. `Wcmsvc` (Windows Connection Manager) is new here: ticking it can drop Wi-Fi |
| Turbo Boost: protected list | `Filter_SWL.txt`: about 150 service names never stopped (antivirus, VMware, `Spooler`, and IObit's own `AdvancedSystemCareService11…20`, `LiveUpdateSvc`, `iTopDataRecoveryService*`, `IObitUnSvr`, `sgbupt`, "game assistant by-pass UAC") | n/a | no | none; it shields IObit's own services |
| Turbo Boost: background apps | Per-app `HKCU\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications\<package family>` `Disabled` / `DisabledByUser`, and `GlobalUserDisabled`, for Store apps not in `Filter_BackTaskApp.txt` (which keeps Edge, Store, Windows Security, Start, Xbox Game Bar/TCUI/Identity and a few more) | Turning off (assumed; it keeps a `BackApps` list in its config) | no | low; HKCU |
| Turbo Boost: close programs | Work mode closes the user's ticked programs (`CheckedApps`, `CloseCount`), skipping a list of core system processes | Not restarted | no | unsaved work |
| Turbo Boost: power plan | Optional *Activate Advanced SystemCare Power Plan*: `PowerConfig.dll` `SetGamePwrCFG` duplicates High performance as "Advanced SystemCare Power Plan" ("Activate it when you switch to Turbo Boost Mode to boost performance."). The DLL has the same exports, imports and all 29 power GUIDs as SGB's `PowerMgr.dll` (a 2021 build of the 2014 one), so presumably the same values: min CPU 100 %, PCIe ASPM off, never sleep, lid does nothing, … ([spec 2.1](../spec.md)). Economy mode uses an "Advanced SystemCare Economy Mode" plan | Previous plan reactivated; plan deleted by name (`DeletePwrCFGByName`) | no | **heat**: min CPU 100 % on a heat-limited laptop |
| Auto RAM Clean (Pro) | `ASCService.exe` `TProcessMemoryOptimize` on a timer: scans processes and their CPU use, skips `Database\ActiveBoost.db` (241 antivirus/IObit executables), trims working sets (`EmptyWorkingSet` / `SetProcessWorkingSetSize`) | none needed (pages fault back in) | no | stutter after a trim; no standby-list purge (no `NtSetSystemInformation`) |
| Performance Monitor "Clean RAM" | `Monitor.exe` `EmptyWorkingSet` across processes, "Memory released: %s" | none | no | as above |
| Performance Monitor | CPU/RAM/disk/network/temperature overlay window and tray, "Turbo Boost: ON/OFF" switch, kill buttons | n/a | **yes** (WinRing0 fork for temperatures, `NetDataFlow.sys` for network) | anti-cheat, vulnerable driver |
| **Internet Boost** | `netsh int tcp set global`: `autotuninglevel=normal`, heuristics `disabled`, `congestionprovider=ctcp`, `chimney`, `rss=enabled`, `dca`, `ecncapability`, `netdma`; registry `Tcpip\Parameters` `EnablePMTUDiscovery`, `EnablePMTUBHDetect`, `SackOpts`, `DefaultTTL`, `TcpMaxDupAcks`, `Tcp1323Opts`, `MaxUserPort`, `TcpTimedWaitDelay`; `Policies\Microsoft\Windows\Psched` `NonBestEffortLimit`; per-adapter MTU; IE `MaxConnectionsPerServer` / `FEATURE_MAXCONNECTIONSPERSERVER`; DNS-cache times; `DisabledComponents` (IPv6); Edge/Firefox/Opera prefs (prefetch, Do Not Track, Opera "turbo") | "Rescue Internet Boost" from backups `ASCNETSHBak.dat` / `ASCInternetBak.dat` | no | none to low; most values are already the Win10 default or ignored |
| System Optimization (scan module) | `Opt.dbd` profiles: **identical** to SGB's except one `TrkWks` entry and some `OS=` fields (see [SGB's table](../../decomp/iobit/smart-game-booster/SgbData/dbd/README.md#tweaks-win10-relevant-deduplicated-across-sysspeedgb3opt-win10-profiles)): permanent registry values, service `Start` = 4, old TCP values | Rescue Center restore point, or by hand (`DefVal`) | no | **permanent** service disables; some break things |
| AI Mode | A flag (`IsAIMode` in `Main.ini`) that runs the scan with a stored module selection (`LastSelectModules`, `AIFix1`/`AIFix2`) and auto-fixes | n/a | no | as for the modules it runs |

## What's shared with Smart Game Booster

- **Data container** (`.dbd`): same format, salt, IV and cipher. `Opt.dbd` (15 of 31 files
  byte-identical, the others differ in one `TrkWks` entry and `OS=` fields), `PriTemp.dbd`,
  `Def.dbd` (the same 13 scanner lists, `TrackingCookie.db` included) and their passwords are
  shared. `startupBlack.db`, `startupService.db` and all `InBoxDriverFeature\*.ini` are
  byte-identical to SGB's.
- **Turbo Boost = SGB Boost.** `Service_Game.ini` has SGB's `WinSvc.ini` layout and sections
  (`[Services_XP]` … `[Services_WIN10]`, `[Services_3rd]`), with nearly the same Win10 list: ASC adds
  `BITS`, `Wcmsvc`, `WSearch`, `PhoneSvc`, `NcdAutoSetup`, `PNRPsvc`, `p2psvc`, `p2pimsvc`,
  `wlidsvc`, `WdNisSvc` and `DeviceAssociationService`. SGB has `Themes`, `Fax`, `Netlogon`,
  `TrustedInstaller`, `DeviceInstall`, `AllUserInstallAgent` and `CscService`, which ASC lacks.
  ASC ticks `TrkWks`, `RemoteRegistry`… differently in places.
- **Power plan:** `PowerConfig.dll` is SGB's `PowerMgr.dll` API (eight identical exports,
  identical imports, every power-setting GUID from [spec 2.1](../spec.md) present). Only the plan
  name and description differ.
- **RAM trim:** the same working-set trim with an antivirus skip list.
- **Not in SGB:** the WinRing0-derived temperature driver (SGB used none), the WFP, minifilter and
  registry drivers, Internet Boost's `netsh` layer, background-app switching, and AI Mode.

## Claims vs reality

| Claim (UI text) | What the code does | Verdict |
|---|---|---|
| "Speed up your PC up to 200% faster" | No benchmark, measurement or formula behind the number was found. The speed-ups are SGB's service stops, a High-performance clone, working-set trims and old registry values | **misleading** |
| "Boost Internet up to 300% faster" (Internet Boost) | Writes TCP globals that are already Windows 10's defaults (`autotuninglevel=normal`, heuristics disabled, RSS on). It also writes some that Windows 10 no longer has (Chimney, NetDMA, DCA; `congestionprovider` is no longer a `set global` option), plus server-only knobs (`MaxUserPort`, `TcpTimedWaitDelay`) and the QoS "reserved bandwidth" myth (`NonBestEffortLimit`). Nothing touches latency to a game server | **placebo** (misleading number) |
| "AI Mode … based on the status of your PC and the AI technology" | No model, ML library or inference code in the package. It is a saved module selection plus auto-fix | **misleading** |
| Turbo Boost: "intensifying processor performance", Game mode "terminating unnecessary System and Non-Windows services" | Real, same as SGB: stops idle services, optional min-CPU-100 % plan, background Store apps off. Small and situational, and the power plan adds heat on a laptop | **real** (small); power plan **harmful** on a hot laptop |
| Auto RAM Clean: "intelligently managing system resources … improve PC performance" | Timed working-set trim of other processes | **placebo** to **harmful** (page faults afterwards) |
| Performance Monitor | Real monitoring, but it needs a WinRing0-family kernel driver and a WFP callout | **real**, at a **harmful** price (vulnerable driver, anti-cheat) |
| "Guaranteed to make your PC run faster … You WILL see a significant improvement" | Upsell text only | **misleading** |

**How the scan counts "problems".** The total is the sum of `FModuleInfo[...].Problems` over 15
modules: `wmRegistryFix`, `wmJunkFilesClean`, `wmPrivacySweep`, `wmMalwareRemoval`, `wmShortcutFix`,
`wmInternetBoost`, `wmSystemOptimization`, `wmDiskCheck`, `wmDiskDefrag`, `wmRegistryDefrag`,
`wmSystemSecurity`, `wmInternetSecurity`, `wmReinforce`, `wmVulnerabilityFix`, `wmSoftwareHealth`.
Each unit counts as one problem:

- every orphan registry value (`Reg.dbd` rules);
- every junk file;
- every **privacy trace**: each history entry, MRU item and cookie (`PriTemp.dbd`, 165 junk and
  365 privacy templates);
- every cookie whose domain is in `TrackingCookie.db`, reported as a **"Spyware threat"** (as in
  SGB);
- every TCP setting that differs from Internet Boost's choice ("[Internet improvements]: %d
  Detected"), even when Windows' default is the better one;
- every `Opt.dbd` value that differs from `OptVal`;
- outdated drivers;
- Turbo Boost being off ("We recommend turning on Turbo Boost").

A clean, healthy Windows install therefore always shows dozens to thousands of "problems".

## On a 2019 MacBook Pro under Boot Camp playing BF6

Don't install it on the gaming install. Four kernel drivers plus a WinRing0 fork is worse
anti-cheat exposure than anything SGB shipped. The "System Optimization" fixes make permanent
service changes. The useful part, Turbo Boost, is SGB's Boost, and gamebooster-rs already does
that with a strict allow-list and a real `restore`. The Turbo Boost power plan is exactly the
min-CPU-100 % plan that heats this CPU.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **Background Store apps off while boosted**: `HKCU\Software\Microsoft\Windows\CurrentVersion\
   BackgroundAccessApplications` `GlobalUserDisabled` = 1 (old value or absence recorded, restore
   puts it back). Never touch per-app keys, and keep Xbox Game Bar's own behaviour. **Try by
   feel**, low expected gain. HKCU, no admin, harmless. It only stops Store apps' background
   tasks, which are few on a gaming install.
2. **Service list**: nothing to add. ASC's extra Win10 entries (`WSearch`, `DiagTrack`,
   `MapsBroker`, `BITS`, `PhoneSvc`) are already in, or covered by, our opt-in `services` list
   ([docs/tweaks/services.md](../tweaks/services.md)). **Skip `Wcmsvc`**: stopping it can drop
   Wi-Fi mid-match. **Skip** a protected-service list like `Filter_SWL.txt`: our allow-list already
   means we never touch an unlisted service.
3. **Power plan, RAM trim, Game Mode**: already in gamebooster-rs (same source: SGB). **Skip**
   anything more.
4. **Internet Boost (`netsh` TCP globals, `MaxUserPort`, `TcpTimedWaitDelay`, QoS reserve, MTU)**:
   **skip**. These are defaults, removed settings or server knobs. Our `ping` / `ping --route` and
   the optional DNS step cover the network side ([docs/tweaks/network.md](../tweaks/network.md)).
5. **Temperature readout via a ring-0 driver, WFP traffic monitor**: **skip** (no kernel driver,
   ever).
6. **AI Mode, scan-and-fix modules, Auto RAM Clean on a timer**: **skip**: placebo or harmful.
