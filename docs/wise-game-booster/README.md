# Wise Game Booster (WiseCleaner): what it really is

Wise Game Booster is a free Windows game booster from WiseCleaner (Lespeed Technology Co., Ltd),
written in Delphi. One window, three "optimizers" behind one **Optimize All** button:
a **System Optimizer** (permanent registry tweaks), a **Process Optimizer** (ends programs) and a
**Service Optimizer** (stops services), plus a game library ("Game Box") that launches games.
There is no Pro edition, no driver, no power plan and no network accelerator.

Everything here comes from reading the shipped files (static reverse engineering) of
`WGBSetup_1.5.7.81.exe` (version 1.5.7, build 81), downloaded from the official link on
wisecleaner.com (`downloads.wisecleaner.com/soft/`) on 2026-11-05, sha256
`0fbba3eb236372389f8dfcf8002ba0e0124a61401fb8f2e1c06d5669c9ab0c7f`. **Nothing was run.**
The installer is Inno Setup 5.5.7 and unpacks cleanly with binary-refinery
(`emit | xtinno`, no CALL/JMP filter problems). The tweak and service lists are plain text inside
the program's form resources; the service and process logic was read in Ghidra (through REA) and
the output deleted afterwards (policy: [decomp/README.md](../../decomp/README.md#policy)).

## Short version

- **Optimize All** writes up to ~40 permanent registry tweaks (mostly Windows XP/7-era folklore),
  ends "high memory" programs and stops "irrelevant" services. Nothing touches the CPU, GPU,
  power plan, timer resolution or game priority.
- **Restore**: stopped services are recorded and can be started again from a **Resume Service**
  window. Ended programs are not restarted. The registry tweaks have no backup of your old values;
  the tweak definitions only carry Windows default values to fall back to.
- "Free up memory" means closing programs. The "MB saved" number is the memory of the programs it
  ended. There is no working-set trim or standby purge in the Boost.
- **Uploads your game and program list** (paths, file names, version info, MD5, and service
  names/publishers) to `info.wisecleaner.com` over plain HTTP, to grow its game database. The
  shipped database already holds paths from other people's PCs, Windows user names included.
- No kernel driver, no anti-cheat risk from drivers. Signed with an EV certificate from SSL.com.

## What Optimize All changes, item by item

### System Optimizer (registry, permanent)

Grouped as the program groups them. All are plain registry values; each is applied once and stays.

| Group | Tweak (value set) | Reality on Windows 10/11 |
|---|---|---|
| Boot/shutdown | `BootOptimizeFunction\Enable = y`; `AutoChkTimeOut = 5`; `ClearPageFileAtShutdown = 0`; `WaitToKillAppTimeout = 10000`, `HungAppTimeout = 3000`; `EnablePrefetcher = 2` | No game effect. `ClearPageFileAtShutdown = 0` is already the default. |
| Stability | `AutoEndTasks = 1` (kill hung apps at logoff/shutdown); `AutoRestartShell = 1`; `DesktopProcess = 1` (separate Explorer process); `remoteregaccess = 1`; `CrashControl\AutoReboot = 0` (stay on the blue screen) | `AutoEndTasks` can lose unsaved work. None affects games. |
| System speedup | `PCHealth\ErrorReporting` off (XP key); `AeDebug\Auto = 0`; `IoPageLockLimit = 128 MB`; list-view alpha and taskbar animations off; `NtfsDisableLastAccessUpdate = 1`; `NoLowDiskSpaceChecks = 1`; `NoDriveTypeAutoRun = 221`; `EnableAutoLayout = 1`; `MinAnimate = 0`; **`Win32PrioritySeparation = 38`**; `DisablePagingExecutive = 1`; Aero Peek/Snap hover delays 0; `ForegroundLockTimeout = 0`; `Max Cached Icons = 4096`; `MenuShowDelay = 0`; `MouseHoverTime = 100`; `NoSimpleNetIDList = 1` | `IoPageLockLimit` and the PCHealth key do nothing since Vista. `Win32PrioritySeparation = 38` (0x26: short, variable quanta, 3:1 foreground boost) is the only scheduling change; it is close to the client default (2) in practice. `DisablePagingExecutive` is folklore with no measured gain. The rest is UI feel. |
| Network speedup | `DefaultTTL = 64`; `EnablePMTUBHDetect = 1`, `EnablePMTUDiscovery = 1`; `NameSrvQueryTimeout = 3000` (WINS); LanmanWorkstation `MaxCmds`/`MaxThreads = 30`, `MaxCollectionCount = 32` (file sharing); `TcpMaxDupAcks = 2`, `SackOpts = 1`, `Tcp1323Opts = 1`; Tcpip `MaxConnectionsPerServer = 0`; IE max connections 10; DNS client negative caching 0, max TTL 3 h; `NoNetCrawling = 1` | Most are Windows defaults already, or apply to WINS, SMB file sharing or Internet Explorer. None touches game UDP traffic or latency. |
| SSD (only shown when an SSD is found) | `DisableDeleteNotification = 0` (TRIM on, the default); `EnablePrefetcher = 0` and `EnableSuperfetch = 0`; boot and idle defrag off | Prefetch/SysMain off does not extend SSD life measurably and can slow app and game launches. Disabling idle "defrag" also stops Windows' scheduled TRIM retrim. |

Each tweak counts as an "issue" while the value differs from the target; the start screen's
"Found %d issues" and the health score come from this count, so a clean Windows 11 install always
shows "issues".

### Process Optimizer (ends programs)

Lists "High Memory Usage Non-system Programs" with a suggestion (End / Keep) per program; Optimize
All ends the suggested ones with `TerminateProcess` (no polite close, so unsaved work is lost).
Protected names: its own process, `audiodg`, `dwm`, `svchost`, `taskhost`, `taskeng`, `wuauclt`,
`System`, `wininit`, `winlogon`, `lsm`, `lsass`, `csrss`, `smss`, `spoolsv`, `services`,
`explorer`, `iexplore`, `bds` (the Delphi IDE, a developer leftover), `QQ` and WiseCleaner's
`WiseTray`; plus a user keep list (`Ub.lst`). Ended programs are not restarted by Restore.
The exact memory threshold could not be read with certainty.

### Service Optimizer (stops services)

A built-in table of 281 service names, each with a suggestion class (roughly: can stop / keep /
neutral / never touch). Optimize All stops the running services it suggests stopping (Stop
control, 5 s wait each); it does **not** change their start type. A separate routine can change a
service's start type, used from the manual right-click actions, not by Optimize All. The
stoppable class includes things like Remote Registry, Volume Shadow Copy, Windows Installer,
Remote Desktop, Font Cache and the disk optimiser service. The list is mostly Windows 7-era (Media
Center, HomeGroup, NetDDE, Messenger); Windows Defender, Windows Update, Search, BITS, MMCSS and
Power sit in the middle class.

Each stopped service is written to a restore list (name, display name, description, time) shown
in **Resume Service**, where "Resume All" starts them again. Because the start type is untouched,
a reboot also brings back any automatic-start service.

## What Restore undoes

| Change | Undone by the program | Undone by a reboot |
|---|---|---|
| Stopped services | Yes (Resume Service) | Yes, for automatic-start services |
| Ended programs | No | No (you restart them) |
| Registry tweaks | Only back to Windows defaults where a default is known; your previous value is not saved | No |

## Network and telemetry

- Game/app database: downloads `info.wisecleaner.com/wgbdata/gamebooster.db` and submits local
  records to `info.wisecleaner.com/wgbdata/index.php?to=submit` (plain HTTP). The record formats in
  the binary hold the folder path, file name, CompanyName, FileVersion, LegalCopyright and MD5 of
  games and programs, and display name, publisher and start type of services. The bundled
  `gamebooster.db` (2,033 games, 5,418 apps/services) contains folder paths from other users'
  machines, Windows user names and a WiseCleaner test account included, which shows the paths are
  uploaded as-is.
- Update check `wisecleaner.com/software_update/getinfo.php?p_id=15`, news/messages
  `info.wisecleaner.com/messages/`, install statistics `wisecleaner.net/install_statistics/`,
  feedback pages, share buttons (Twitter, Facebook, Weibo, QQ, Baidu).
- Installer: admin install to `Program Files (x86)\Wise\Wise Game Booster`, writes only
  `HKLM\SOFTWARE\WiseCleaner\Wise Game Booster` (path, product name), Start-menu and desktop
  shortcuts, opens a thank-you page. No service, task, Run key or driver. The uninstaller opens a
  feedback page.

## Claims vs reality

| Claim (wisecleaner.com) | Reality |
|---|---|
| "Boost your PC performance immediately by optimizing system settings, ending unnecessary programs and stopping irrelevant services with one single click" | Accurate description of the three steps. The "system settings" are permanent registry tweaks with no game effect on Windows 10/11. |
| "Close startups, free up memory and tuneup network" | No startup manager in this build. "Free up memory" = kill programs. "Tune-up network" = TCP/WINS/SMB/IE registry values that are defaults already or unrelated to games. |
| "Stabilize & speed up your system ... getting your PC into game mode" | Does not touch Windows Game Mode, the power plan or game priority. "Stability" tweaks include auto-killing hung apps at shutdown and staying on a blue screen. |
| "Your PC will allocate more system resources to the game" | True only to the extent that closed programs and stopped services used CPU or RAM. On a clean PC with enough RAM that is close to nothing. |
| "The operation of the service [is backed up and] can be restored to the state before use" | True for services. Not true for registry tweaks or ended programs. |
| "Doesn't perform miracles" | Agreed. |

No fake scan or fake progress found: the issue count is the number of tweaks not yet applied,
services suggested and processes over the threshold. No FPS figures are claimed.

## On a 2019 MacBook Pro under Boot Camp playing BF6

Harmless but almost useless. No driver, so no anti-cheat problem; no power-plan change, so nothing
that adds heat. The registry tweaks do not touch what limits the Mac (thermals, BD PROCHOT, the
power plan's processor state). The SSD group turns off SysMain/prefetch and scheduled retrim,
which is the opposite of what an SSD wants. Killing programs with `TerminateProcess` can lose work.

## Worth adopting into gamebooster-rs?

Only measurable and undoable things on the Mac:

- **Service stop without start-type change, with a restore list**: gamebooster-rs already does
  this (`services`, recorded and restarted by `restore`).
- **Process ending**: not worth adopting. Ending programs is not undoable, and gamebooster-rs's
  `lower_others` (priority, restored) is the reversible version.
- **`Win32PrioritySeparation`**: undoable (save and put back the old value) and measurable only as
  frame-time differences, which are within noise in published tests. Not worth adding.
- Not worth adopting: every other registry tweak (no effect on games, several are defaults
  already), SSD prefetch/defrag off, TCP/DNS tweaks (gamebooster-rs's `ping` and `fastest_dns`
  cover network), the upload of game/program lists.

## Compared with the others

- Same model as [SGB](../sgb/README.md) and [PC Game Booster](../smartpcutilities-gb/README.md)
  (stop services, close programs) minus the power plan, Game Mode, drivers and paid tiers.
- Closest to [PGWARE GameBoost](../pgware/README.md) in its registry part: permanent, folklore,
  mostly placebo.
