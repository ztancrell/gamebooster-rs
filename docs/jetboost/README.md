# JetBoost (BlueSprig): what it really is

JetBoost is a free Windows "one-click" booster from BlueSprig, Inc. (San Francisco), released
2011–2013. You pick a mode (Gaming, Working or Custom) and press **Boost**. It stops a list of
services, kills background programs, empties every process's working set, clears the clipboard,
lowers other programs' priority, pauses Automatic Updates, and switches to its own power plan.
**Restore** (button, tray menu or a hotkey) restarts the services and programs and switches the plan
back. It is a rebranded IObit product: the program manifests say `IObit`, the code has IObit's
`IObit_Routine_New` classes, and the installer script is IObit's template (Chinese comments listing
IObit product codes such as ASC3/ASC4/GB/IS360/SD).

**The product is dead.** `bluesprig.com` now redirects to an unrelated company (BlueSprig Autism),
and the download URL returns 404. The last version, **2.0.0** (released 2012-11-28, main program
2.0.0.67), was taken from the Internet Archive's byte-exact capture of the official
`http://www.bluesprig.com/dl/jetboost-setup.exe` (capture 2013-06-03; its SHA-1 matches the archive
record). Installer SHA-256
`3c0223046521fca8c37a6816af5b4b3cae3eaae8600a2e7f131b20f85e507c4b`, 3,511,744 bytes, Inno Setup
5.5.0, signed "BlueSprig, Inc." (VeriSign, 2012). It was unpacked statically with binary-refinery
(`emit | xtinno`). The main program is Delphi 2009. Its default lists are plain INI files, its power
plan is a small C DLL, and the program code was read with Ghidra headless and objdump.
**Nothing was run.** The decompiled output was deleted afterwards (see
[decomp/README.md](../../decomp/README.md#policy)).

## Short version

- Boost is real work, and Restore undoes most of it. Services that were stopped are started again,
  and killed programs are relaunched from a record in `Temp.ini`. The old power plan comes back.
  Automatic Updates gets its old setting back.
- **It is built for Windows XP–8.** It identifies Windows with `GetVersionEx`, and its manifest only
  declares Vista and 7. So Windows 8.1, 10 and 11 report themselves as 6.2, and JetBoost uses its
  **Windows 8 service list**. That list was written for 2012-era Windows. It includes `BITS`,
  `Spooler`, `W32Time`, `WerSvc`, `FontCache`, `TermService`, `iphlpsvc`, `SharedAccess`, `DPS` and
  `WinHttpAutoProxySvc`, among about 60 others.
- Programs are killed with `TerminateProcess`, which is a hard kill with no chance to save work.
  The app shows "Please save your work before boost" for this reason.
- The "Gaming Mode" power plan forces **minimum CPU 100 % on AC and battery**, turns off sleep, the
  display timeout and the disk timeout, and turns off PCIe link power saving.
- The RAM boost empties every process's working set. That is the classic placebo: the memory is
  paged back in, often during the game.
- Network: the installer opens `bluesprig.com/jbinstall.php` in a hidden browser (an install ping).
  The app downloads a `free.dat` news file. A scheduled task runs an updater that fetches
  `jb-update.ini` over **plain HTTP**. The domain has changed hands, so this updater is now a real
  risk.

## What Boost changes, item by item

Defaults come from the shipped `cfgFiles\*.ini`. Gaming mode (`Process_Game.ini`) has every
`[Boost]` switch on. Working mode turns off the clipboard, Windows Update and priority switches.

| Item | What it really does | Undone by Restore? |
|---|---|---|
| Services | Takes the section for your Windows from `Service_<mode>.ini`. The sections are `Services_2000`, `_XP`, `_VISTA`, `_WIN7` and `_WIN8`, plus a user `_3rd` section that is empty by default; Windows 8.1/10/11 get `_WIN8`. For each running service marked `=1` it saves the name to `Temp.ini`, then sends **Stop** (`ControlService`) and waits up to about 0.5 s. It also stops services that depend on it. The Win8 list (about 60 entries) includes `AxInstSV`, `SensrSvc`, `AeLookupSvc`, `ALG`, `AppMgmt`, `BITS`, `BDESVC`, `PeerDistSvc`, `CertPropSvc`, `Browser`, `VaultSvc`, `DPS`, `WdiServiceHost`/`WdiSystemHost`, `TrkWks`, `EFS`, `Fax`, `fdPHost`, `FDResPub`, `hkmsvc`, `UI0Detect`, `SharedAccess`, `iphlpsvc`, `KtmRm`, `lltdsvc`, `swprv`, `MSiSCSI`, `NetTcpPortSharing`, `Netlogon`, `napagent`, `CscService`, the `PNRP`/`p2p` services, `pla`, `WPDBusEnum`, `wercplsupport`, `PcaSvc`, `QWAVE`, `SessionEnv`, `TermService`, `UmRdpService`, `RpcLocator`, `RemoteRegistry`, `RemoteAccess`, `SCPolicySvc`, `SNMPTRAP`, `SSDPSRV`, `StorSvc`, `TabletInputService`, `WebClient`, `WbioSrvc`, `WcsPlugInService`, `wcncsvc`, `WerSvc`, `Wecsvc`, `FontCache`, `StiSvc`, `WMPNetworkSvc`, `WinRM`, `W32Time`, `WinHttpAutoProxySvc` and `Spooler`. It spares `bthserv`, `hidserv`, `SCardSvr`, `WSearch` and `Themes`. The boost path does not change start types. The binary has a `ChangeServiceConfig` routine, but it belongs to the service-list editor as far as was read. | Yes: `StartService` for each recorded name. Services that started again on their own in the meantime are harmless. |
| Background programs | Kills running processes marked `=1` in the mode's `[Process0]` list. The Gaming defaults are only `taskmgr.exe`, plus anything you tick in the UI. The Working list has about 80 2012-era apps (Steam, ImgBurn, Handbrake, cpuz, etc.), mostly unticked. The kill is `TerminateProcess` with exit code 1, and the path is recorded under `[Process]` in `Temp.ini`. An optional "Close Explorer.exe" switch (`KillExplorer`, off by default) kills the shell. | Yes: each recorded path is relaunched. Unsaved work in a killed program is lost. |
| RAM | `EmptyWorkingSet` on every process (PSAPI), shown as "RAM cleaned". | Nothing to undo; the pages fault back in. |
| Clipboard | Clears the clipboard (from the UI and log strings; the routine was not traced). | No (the contents are gone). |
| Priority | "Lower process priority": lowers the priority class of the other running programs (from the UI and log strings; the exact class was not traced). | Yes (the "Restore Process" step). |
| Automatic Updates | Uses the legacy Windows Update Agent COM API (`IAutomaticUpdates` → `Settings`). If Automatic Updates is enabled and the setting is neither read-only nor policy-required, it writes a new `NotificationLevel` and `Save`s it. Before that it reads the old level. | Yes (it writes the old level back). On Windows 10/11 this API is effectively ignored. |
| Power plan (`PowerConfig.dll`) | Creates or reuses a plan with the fixed GUID `{D68F881F-75D5-4673-BCBD-34076A3A08E9}`, named "JetBoost Gaming Mode Power Plan", and activates it. Values, all on **both AC and DC (battery)** unless noted: max **and min** processor state 100 %; system cooling policy active; sleep never (AC; a bug writes the DC sleep value where DC hibernate should go); hibernate never (AC); turn off display never; turn off disk never; PCIe link state power management off; wireless adapter maximum performance; **require a password on wake off**; "When sharing media" prevent idle; "When playing video" optimise quality; and on AMD drivers the ATI PowerPlay setting at maximum performance. | Yes: it activates the saved previous plan, falling back to Balanced. The JetBoost plan stays installed. |
| Countdown, transparency, tray | A 10-second countdown before boost, then the window minimises to the tray. Restore hotkey default Ctrl+Alt+R (`Restore=49218`). | n/a |

`AutoRecover=1` by default. Judging by the name and the leftover `Temp.ini` record, the restore runs
automatically, for example after a crash; the trigger was not traced.

## Install, autostart, network and telemetry

- Installer: Inno Setup, admin. It installs to `Program Files (x86)\BlueSprig\JetBoost`, and adds a
  desktop shortcut plus a "Quick Boost" shortcut (`JetBoost.exe -game`). It writes no registry
  values itself. It has no driver, no service and no bundled third-party offers. Its template has an
  unused "FreeDownloader recommended product" hook.
- `JetBoostInstallBackWork.exe` is run from `{tmp}` at the end of setup without waiting. It loads
  `http://www.bluesprig.com/jbinstall.php` in an embedded browser control and records the version in
  `JetBoostInstallBackWork.ini`. This is an install ping (what the page collected is server-side and
  unknown).
- On every start, the main program creates the scheduled task **`JetBoost_AutoUpdate`** if it is
  missing (it uses its `TaskSchedule.exe`/`taskMgr.dll`). The task runs
  `AutoUpdate.exe /AUTORUN`. The updater fetches `http://download.bluesprig.com/iniupdate/jb-update.ini`
  over **plain HTTP** and offers to download a new version. There is also a `JetBoost_Startup` task
  for "start with Windows" (off by default).
- `http://download.bluesprig.com/news/jetboost/free.dat` is a news/promo feed shown in the UI.
  There are Facebook/Twitter links and a mail address for beta feedback. Bug reports use madExcept,
  which sends only when you choose to.
- There is no hardware ID or analytics in the strings read, beyond the install ping and update check.
- **Today:** `bluesprig.com` belongs to someone else. Anyone who controls `download.bluesprig.com`
  could answer the HTTP update check of an old install. Uninstall it if it is still around.

## Claims vs reality

| Claim (product page, PAD file) | Reality |
|---|---|
| "Shuts down all unnecessary processes and services … with one single click" | It stops a fixed 2012 service list and kills the processes you ticked. On Windows 10/11 the list is the Windows 8 one, so it stops things such as `BITS`, `Spooler`, `W32Time`, `FontCache`, `iphlpsvc` and `WinHttpAutoProxySvc`. Most of these are idle or trigger-started and cost nothing. "Unnecessary" is a guess. |
| "Superb Gaming Experience / get the maximum capacity out of your PC" | The measurable parts are the power plan (min CPU 100 %, PCIe ASPM off) and fewer background programs. Both are small on a clean PC. The power plan hurts on a hot laptop. The RAM trim can cause stutter. |
| "Cleaning RAM … increase available physical memory" | `EmptyWorkingSet` on everything. The "free" number goes up, then the memory is paged back in. Placebo, slightly harmful. |
| "Intensifying processor performance" | The power plan: min/max processor state 100 % and active cooling, also on battery. Real, but it means more heat and battery drain, and on a throttling laptop it can mean lower clocks. |
| "Speedup your PC to its Top performance" | No measurement in the product; it reports counts (services stopped, processes closed, RAM cleaned), not speed. |
| "Clean and Safe" | No malware found. But it hard-kills programs (data loss), turns off the password on wake in its plan, and leaves an HTTP updater on a domain that has changed owners. |
| Supports "Windows 8, 7, Vista, XP and 2000" | True for those. On 8.1/10/11 it runs with the Windows 8 lists, by accident of `GetVersionEx`. |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Don't use it. Its power plan pins minimum CPU at 100 % on battery as well as AC, with active
cooling. That is the opposite of what helps this Mac's heat and its 800 MHz BD PROCHOT lock (see
[prochot](../tweaks/prochot.md)). The RAM trim and clipboard clear do nothing useful. The service
list is aimed at Windows 8. Stopping `BITS`/`Spooler`/`W32Time` gains nothing measurable for BF6,
and the old updater is a security risk. It has no driver and does not touch the game process, so BF6's
anti-cheat is not the issue; the issue is that nothing in it helps.

## Worth adopting into gamebooster-rs?

Only items that are measurable and undoable on the target Mac count:

- **Nothing new.** gamebooster-rs already sets PCIe ASPM off and Wi-Fi maximum performance in its
  boost plan ([power plan](../tweaks/power-plan.md)), already stops an allow-listed set of services
  and restarts them ([services](../tweaks/services.md)), already closes and relaunches chosen apps
  ([close apps](../tweaks/close-apps.md)), and already lowers other processes reversibly
  ([priority](../tweaks/priority.md)).
- One idea confirmed, not new: record what was changed *before* changing it (JetBoost's `Temp.ini`),
  so a crash mid-boost can still be restored. gamebooster-rs's saved state already does this.
- Not worth adopting: working-set emptying, clipboard clearing, pausing Automatic Updates (ignored
  on Windows 10/11), minimum CPU 100 % on battery, the no-password-on-wake setting, hard-killing
  programs, and an OS-version-keyed service list.
