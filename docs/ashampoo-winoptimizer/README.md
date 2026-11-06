# Ashampoo WinOptimizer Pro 29: what it really is

Ashampoo WinOptimizer is a German Windows "clean and tune" suite (Ashampoo GmbH & Co. KG) with
about 30 modules: four cleaners behind a **One-Click Optimizer**, a Tweaking module (several
hundred registry switches), an **Internet Tuner**, **Live-Tuner** (per-app priority/CPU-mask rules
and a "launch accelerator"), **Game Booster** (stops services and ends processes while you play),
Process Manager, Service Manager, a power-plan editor, a defragmenter with an **SSD optimization
wizard**, a benchmark, privacy settings, a firewall front end and more. The product page says
"Speeds up your PC instantly", "More speed for your daily PC needs" and "Fast. Clean. Safe.",
and its FAQ says it "fixes problems, and improves computer speed and stability". No FPS numbers.

Version **29.0.2** (Pro, the trial build; the Free edition is a separate product). Downloaded
2026-11-06 from the product page's Download button
(`https://www.ashampoo.com/en-us/dld/7306/winoptimizer-pro-29` → `https://www.ashampoo.com/ashampoo_winoptimizer_pro_29.exe`,
which redirects to Ashampoo's own CDN,
`https://cdn1.ashampoo.net/ashampoo/7306/ashampoo_winoptimizer_pro_29_29.0.2.exe`). No account, no
payment, a full offline installer (not a stub). `Last-Modified` 2026-09-24, SHA-256
`4b35e6ced9616f8c436d86e2630579e0edead1ea3ab1497d6ac6412c034d5dd6`, 36,837,976 bytes.
**Nothing was run.**

## How it's built

An **Inno Setup 6.7** installer (too new for innoextract 1.9; unpacked with binary-refinery's
`xtinno`). Inside: `WO29.exe` (the main app, Delphi, 18 MB), `WO29eureka.exe` (the same app built
with EurekaLog crash reporting), `LiveTuner3.exe` (the tray client for Live-Tuner, Game Booster
and Auto-Clean, Delphi), `LiveTunerService.exe` (a Windows service), **`LiveTuner32.sys` /
`LiveTuner64.sys` (kernel drivers)**, `WinOptimizerFW.exe` (firewall), `exit.exe` (uninstall
helper that stops and deletes the driver), `undelete.dll`, shell context-menu handlers,
`ash_inet2.dll` / `ash_libcurl.dll` (Ashampoo's licence, update and news library), `sqlite3.dll`
and a `data/` folder with plain-text profiles. The English UI text comes from
`Translation/en-US.nlang3` (UTF-16 key=value). Registry paths, value names and service names come
from the binaries' string tables, read in context. No disassembly was done, so the exact values
written by most tweaks were not recovered; where this README names a value, it is the
documented meaning of that value name, not a read immediate. The strings dumps were deleted
afterwards (see [decomp/README.md](../../decomp/README.md#policy)).

## Short version

- **Kernel driver: yes.** `LiveTuner64.sys` (and a 32-bit twin), service **`LiveTuner2PM`**,
  installed by the app (not the setup script) with `sc.exe create LiveTuner2PM type= kernel
  start= auto`, display name "Ashampoo LiveTuner 2 Driver". It is small and old: built with the
  Windows 7 WDK from a `wo19\treiber` source tree (PDB name `LiveTunerProcessMonitor.pdb`),
  signed by Microsoft's hardware-compatibility publisher. It imports only
  `PsSetCreateProcessNotifyRoutine` plus device and memory basics, and exposes
  `\\.\LiveTuner2PM`: it tells the service when a process starts. No memory, port or MSR access.
  Harmless in itself, but an **auto-start kernel driver just to get process-start events**, which
  Windows gives without a driver (WMI process traces, ETW, or polling as `gamebooster auto` does).
  Whether the driver is installed at first run or only when Live-Tuner / Game Booster is switched
  on was not determined.
- **A LocalSystem broker service**: `WO_LiveService2` ("Ashampoo LiveTuner 2 Service",
  `LiveTunerService.exe`, `start= auto`, delayed). It applies Live-Tuner rules to new processes
  (`SetPriorityClass`, `SetProcessAffinityMask`, `TerminateProcess`) and also carries out
  registry operations and file copy / move / delete / remove-directory requests for the GUI over an
  IPC queue (`IPC_WO_LiveTuner2_Service`). For browser-trace cleaning it **terminates running
  `msedge.exe` processes** so it can delete their files. The queue's access control was not
  examined.
- **Autostart:** the tray client `LiveTuner3.exe` via `HKLM\...\CurrentVersion\Run`
  ("Ashampoo WinOptimizer Live-Tuner3"; option "Run automatically after Windows start"), plus an
  optional weekly One-Click task (the app offers "Fridays at 4 pm"; Task Scheduler XML with
  `HighestAvailable`, not on battery).
- **Phones home:** the installer pings `et.ashampoo.com` on setup start, abort, install, update and
  uninstall and asks `swc.ashampoo.com/geoip`; the app talks to `act.ashcdn.net` (activation),
  `licenses.ashampoo.com/updates.php` (updates), `myapi.ashampoo.com` (product/news images for the
  in-app news channel), `reporting.ashampoo.com/error.php` (EurekaLog crash reports), and, on
  user actions, `benchmarks.ashampoo.com/{wo7,services,hdd}/?encoded=` ("Compare online"),
  `uninstaller.ashampoo.com/rate*` (community ratings) and `cdn1.ashampoo.net/web/benchmarks/services.dat`
  (service ratings). Usage-and-licence telemetry, nothing that looks like exfiltration.
- **Offers:** an in-app news channel, "My Software Deals" and "More Downloads" menu entries, a
  trial "extend time" page and buy-now links (`linktarget.ashampoo.com`). No bundled third-party
  software in the installer.
- **Tweaks are persistent** registry or service changes with a per-module "Restore standard
  settings" or backup file; only Game Booster is per session.

## Performance features

Revert column: what the app offers to undo it. No row except Live-Tuner / Game Booster's
automatic trigger involves the driver.

| Feature (UI) | Exact change | Revert | Risk (heat / anti-cheat / stability) |
|---|---|---|---|
| **Game Booster** (manual, tray, or auto when a listed exe starts) | Stops the services and ends the processes in its profile. Default game profile (`data/PB_Game_default.ini`): services `VSS`, `QWAVE`, `BITS`, `SNMPTRAP`, `WMPNetworkSvc`, `SDRSVC`, `DiagTrack`, `SessionEnv`, `TermService`, `TeamViewer`, `TapiSrv`, `IpOverUsbSvc`; processes `vlc.exe`, `wmplayer.exe`, `Video.UI.exe`. User-editable by category (browser, communication, media, office, printer, updaters, Windows features). A second "Office" profile and a custom one exist ("Profile Booster") | Services it stopped are started again when Game Booster is turned off ("Only disabled services will be re-enabled"), or at reboot. Ended processes are not restarted unless the user picks that action type. No start-type change seen | none; ending a media player loses its state. No power plan, priority, RAM or timer change |
| **Live-Tuner: rules** | Per exe name: priority class and/or CPU mask (affinity), or "End process", applied by the service when the process starts (and optionally to running ones) | delete the rule; the process keeps its setting until it restarts | affinity masks can cost performance; outside priority changes are blocked by EA Javelin for BF6 |
| **Live-Tuner: "Accelerate launch of new applications"** | Raises a newly started process's priority for a set number of seconds, optionally only when CPU use is above a set percentage; an exception list | turn off | none; mostly no effect |
| **Process Manager** | Set priority (Low … High), terminate, details | manual | as Task Manager |
| **Service Manager** | Start / stop / start type, with community ratings from `services.dat`; a fixed list it won't touch (`services-fixed.ini`: RPC, DCOM, audio, DHCP, Plug and Play, Task Scheduler, `WO_LiveService` itself and others) | manual | depends on what you disable |
| **Internet Tuner** | Per adapter: MTU (with an "automatic" ICMP probe), RWIN (`TcpWindowSize`, `GlobalMaxTcpWindowSize`), `DefaultTTL`, `EnablePMTUDiscovery`, `EnablePMTUBHDetect`, `SackOpts`, `Tcp1323Opts` under `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters`; QoS `HKLM\SOFTWARE\Policies\Microsoft\Windows\Psched` `NonBestEffortLimit`; **Nagle**: `TcpAckFrequency` and `TCPNoDelay` under `...\Tcpip\Parameters\Interfaces\{id}` (and `HKLM\SOFTWARE\Microsoft\MSMQ\Parameters` `TCPNoDelay`). "Optimize automatically" sets these from the probe | "Restore standard settings"; backup file | MTU mistakes can break connections. Reboot or reconnect needed |
| **Tweaking: Windows 11 → Performance** | `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling` `PowerThrottlingOff` ("Disable system-wide throttling of background processes"); `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\Maintenance` `MaintenanceDisabled` | untick | **heat**: every background process runs at full speed; maintenance never runs |
| **Tweaking: RAM / caches / NTFS** | `...\Session Manager\Memory Management` `LargeSystemCache`, `IoPageLockLimit`, `ClearPageFileAtShutdown`; `...\PrefetchParameters` `EnablePrefetcher` (Superfetch radio: off / apps / boot / both); `HKLM\System\CurrentControlSet\Control\FileSystem` `FileNameCache`, `PathCache`, `NtfsDisable8dot3NameCreation`, `NtfsDisableLastAccessUpdate`, `NtfsMftZoneReservation`; `HKLM\SOFTWARE\Microsoft\Dfrg\BootOptimizeFunction` `Enable`; `AutoEndTasks`, `WaitToKillAppTimeout` | untick / backup | `IoPageLockLimit`, `PathCache`, `FileNameCache` are pre-Vista values with no effect today; `LargeSystemCache` is a server setting; `ClearPageFileAtShutdown` slows shutdown |
| **Tweaking: network, USB, visuals** | QoS reserve, admin shares, network crawling, USB polling (the text says it can raise power use), animations, menu and window effects | untick | none for games |
| **SSD optimization wizard** | "Apply recommended settings" turns off boot-time defrag, scheduled defrag (`defragsvc`), the index service (`WSearch`), Prefetch (`EnablePrefetcher`), Superfetch (`SysMain`) and hibernation ("Standby mode … unnecessary"); checks TRIM | per item | **slower app and game launches** (SysMain/Prefetch off), no Optimize Drives retrim schedule, no hibernation |
| **Defrag** | Own defragmenter: analyse, defragment, merge free space, "retrim (only useful with SSDs)" | n/a | wear on SSDs if defragmented |
| **Power** | Lists, copies, edits, deletes and activates power plans; "Restore Windows defaults" (`PowerDuplicateScheme`, `PowerWriteACValueIndex`, `PowerSetActiveScheme`, `powercfg`) | "Reset power scheme" | none in itself |

## One-Click Optimizer: how it counts

One Click runs four cleaners: **Drive Cleaner** (junk files), **Registry Optimizer** (orphaned
entries), **Internet Cleaner** (browser cache, cookies, history, session files) and **Privacy
Traces Cleaner** (MRU lists, jump lists, app traces). The headline is "%1 objects with %2 selected
for deletion", where every file, registry value, cookie and history entry is one "object". So a
day of browsing alone produces hundreds or thousands of "objects". It does not count speed
problems, and no performance setting is checked. The website's FAQ says One Click also "optimizes
autostart"; the One Click module in 29.0.2 has only the four cleaners.

## Claims vs reality

| Claim (ashampoo.com / in-app text) | What the code does | Verdict |
|---|---|---|
| "Speeds up your PC instantly" / One-Click "Optimized with a single click" | Deletes junk files, registry entries, browser data and privacy traces; counts each as an object | **Misleading**: cleaning frees disk space and protects privacy, but deleting caches and registry entries does not speed up Windows, and emptying browser caches slows the next page loads |
| One-Click "removes junk data, cleans the registry and optimizes autostart" | Four cleaners; no autostart step in the One Click module | **Partly false**: autostart entries are managed in a separate module |
| Game Booster "automatically shuts down services and processes non-essential to gaming … reactivated once Game Booster is disabled" | Stops a 12-service list and ends three media players; restarts only the services it stopped | **Real but small**: an honest session tool, the same mechanism as our `services` list. Most of its services are idle on a home PC, so stopping them frees almost nothing. `BITS` is the only one that can matter (background downloads) |
| Live-Tuner "speeds up programs at startup" / "Accelerate launches" | Raises a new process's priority for a few seconds when the CPU is busy | **Mostly placebo**: launches wait on disk and initialisation, not on CPU priority, and only matter while another program saturates the CPU |
| Live-Tuner needs "the required Driver" | An auto-start kernel driver whose only job is to report process starts | **Unnecessary driver**: the same events are available without one (WMI/ETW, or polling) |
| Internet Tuner: "Deactivate Nagle (might accelerate online games)" | `TcpAckFrequency` / `TCPNoDelay` per interface | **Placebo for most games**: Nagle only affects TCP; BF6 and most shooters play over UDP. Real only for a few old TCP games (the text names World of Warcraft) |
| Internet Tuner: RWIN "(MTU-40)*4 - this is usually the best choice", TTL, MTU "the more bandwidth … the higher" | Writes `TcpWindowSize` / `GlobalMaxTcpWindowSize`, `DefaultTTL`, MTU | **Placebo / harmful**: the TCP window values are ignored since Vista (receive-window autotuning), TTL has no effect on speed, and a wrong MTU causes fragmentation or dropped connections |
| QoS: "reserve bandwidth … may slow down" | `NonBestEffortLimit` | **Placebo**: the old "Windows reserves 20 %" myth; nothing is reserved unless an app uses QoS |
| Tweaking: cache sizes "can increase performance" | `IoPageLockLimit`, `PathCache`, `FileNameCache`, `LargeSystemCache` | **Placebo**: ignored or server-only on Windows 10/11 |
| Win 11 Performance: disable background throttling | Global `PowerThrottlingOff` | **Harmful on a laptop**: turns off EcoQoS for every process, so background work runs at full clocks and adds heat. (Windows 10 21H2 reads the key too; the UI hides it there.) |
| SSD wizard: settings "that can affect the lifespan or performance of an SSD" | SysMain, Prefetch, indexing, scheduled optimisation and hibernation off | **Harmful / outdated**: Windows already treats SSDs correctly (retrim instead of defrag); turning off SysMain and Prefetch makes launches slower; SSD wear from these is negligible |
| "Automatic optimization in the background … keeps your system consistently clean, fast, and stable" | Auto-Clean deletes temp files and browser traces on a schedule or when a browser closes (killing `msedge.exe` to do it); Live-Tuner applies rules | **Misleading**: tidiness, not speed, with a resident service and kernel driver |
| "Many optimizations offer a preview or undo option … backups" | Per-module backup files and "Restore standard settings"; registry cleaner backs up before deleting | **Real** |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Game Booster is harmless and per session, but adds nothing to what gamebooster-rs already does
with `services`. Live-Tuner's priority rules would hit EA Javelin's block on outside priority
changes, and its rules need an auto-start kernel driver and a LocalSystem service that stay loaded
all the time. Avoid the SSD wizard (slower launches) and the Windows 11 throttling switch
(heat on a heat-limited CPU). The Internet Tuner does nothing for BF6's UDP traffic and its MTU
changes can break the connection.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **`BITS` as a suggested `services` entry**: **try by feel**. Game Booster's default list stops
   Background Intelligent Transfer Service, which carries Windows Update and other background
   downloads; stopping it for the session (and starting it again on restore, as our `services`
   already does) can cut background disk and network use during a match. It is a trigger-start
   service, so Windows may start it again; harmless if it does. Mention it in
   [services](../tweaks/services.md) only if the suggested list is ever extended. Default stays
   `services = []`.
2. **`VSS`, `SDRSVC`, `TermService`, `TapiSrv`, `IpOverUsbSvc`**: **skip**. Idle on a home PC
   (backup, restore points, Remote Desktop, telephony, a Windows Phone USB bridge), so stopping
   them frees nothing measurable. The rest of Game Booster's list is already in our suggested list.
3. **Process-start detection without polling**: **skip**. Ashampoo uses a kernel driver for it;
   `auto` polls, which is enough and keeps us driver-free (DESIGN.md).
4. **Launch priority boost, affinity rules**: **skip**. Placebo for launch time, and outside
   priority changes don't reach BF6 under Javelin; our IFEO method covers the game.
5. **Global `PowerThrottlingOff`, `MaintenanceDisabled`**: **skip**. We already opt only the game
   out of power throttling (`game_power_throttling_off`); doing it system-wide adds heat.
6. **Internet Tuner (Nagle, RWIN, TTL, MTU, QoS) and the Tweaking cache values**: **skip**.
   Placebo for UDP games or ignored since Vista; MTU changes can break the connection.
7. **SSD wizard (SysMain, Prefetch, indexing, defrag, hibernation off)**: **skip**. Persistent
   and makes launches slower; `services` can already stop `SysMain` / `WSearch` per session if
   someone wants that.
