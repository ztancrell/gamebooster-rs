# ISLC 1.0.4.8: notes

ISLC is a .NET Framework 4.8 WPF program in two assemblies. The UI is
`Intelligent standby list cleaner ISLC.exe`; `ISLC.Core.dll` holds the Win32 wrappers, settings,
process monitor and update check. Names are not obfuscated and the PDBs ship with it. Both were
decompiled locally and the output deleted. The class and method names below are evidence, not code.
Nothing was run.

## Process model

- A single elevated GUI process (manifest `requireAdministrator`), with a tray icon. A second copy
  refuses to start ("only one instance").
- No service, no driver, no helper exe, no injection, no hooks into other processes. The only global
  input hook is an optional `RegisterHotKey` (see Hotkey).
- At start (`Main_Initialized`) it calls `ProcessInformationWrapper.SwitchToHighQoS` on **itself**.
  This sets `ProcessPowerThrottling` with the control bit `PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION`
  and the state bit cleared. That means "do **not** ignore my timer-resolution request", even while
  ISLC is minimised or hidden in the tray. Without it, Windows 11 would drop ISLC's request as soon
  as the window is hidden. The class name is misleading: it does not touch execution-speed (EcoQoS)
  throttling.
- It subscribes to power events. On suspend it stops monitoring (which releases the timer request),
  and on resume it restarts monitoring if it was running.

## How it purges the standby list

- `Win32_NtSetSystemInformation.ClearStandbyCache` first enables `SeProfileSingleProcessPrivilege`
  in its own token (`AdjustTokenPrivileges`). It then calls `NtSetSystemInformation` with information
  class **80 (`SystemMemoryListInformation`)** and the command **4 (`MemoryPurgeStandbyList`)**.
- This empties the **whole** standby list, all eight priorities. It does not use the gentler command
  5 (`MemoryPurgeLowPriorityStandbyList`, priority 0 only). It also does not use "empty working
  sets" (2) or "flush modified list" (3). It is the same operation as RAMMap's Empty > Empty Standby
  List.
- Minor bugs: the pinned buffer is not freed when the call fails, and the status is turned into an
  exception using `GetLastWin32Error`, which an NT status call does not set. Neither matters in use.
- The purge runs on a worker task. A counter tracks the purges and the "MB freed" (the standby size
  just before each purge) for the session display.

## Trigger logic and defaults

- Memory figures come from `NtQuerySystemInformation(SystemMemoryListInformation)`:
  - standby = the sum of the eight per-priority standby page counts × 4 KiB;
  - "free" = zeroed + free pages.

  Note that ISLC's "free" is **not** Task Manager's "Available", which includes standby. The UI also
  shows `GlobalMemoryStatusEx` available memory and page-file use (`SystemPageFileInformation`, class 18).
- A `System.Threading.Timer` fires every **polling interval** (`PollingRateSTR`, default **1000 ms**).
  Each tick reads the memory figures, re-checks the timer resolution (below) and, if monitoring is
  started, runs `PurgeLogic`.
- `PurgeLogic` purges when **all** of these hold:
  1. no process from the exclusion list is running;
  2. standby ≥ the "list size" threshold (`StandbyListValue`, default **1024 MB**);
  3. "free" < the "free memory" threshold (`FreeMemoryValue`, default **1024 MB**).
- With the defaults, ISLC purges about whenever free+zeroed memory drops under 1 GB while at least
  1 GB is cached. That is the normal steady state of any Windows machine that has been running for
  a while, so in practice it **purges repeatedly**: every time Windows has used free pages to cache.
  The community's usual advice ("free memory = half your RAM") makes it purge even more.
- Monitoring is **off** at launch unless "start minimized and auto-start" (`StartMinimized`, default
  off) or `-minimized` is used. The manual **Purge** button works at any time.
- **Exclusion list** (default empty): process names (`.exe` added). `AppMonitorWMI` counts running
  instances. It uses a Toolhelp snapshot at start, then WMI `Win32_ProcessStartTrace` /
  `Win32_ProcessStopTrace` subscriptions. While any listed process runs, **automatic purging is
  paused**; the timer request is unaffected. It does not open handles to those processes.

## Timer resolution

- Off by default (`CustomTimer` = false). The wanted resolution (`WantedResolution`) defaults to
  **1.0 ms** and is entered in ms with up to 4 digits. It is passed as 100-ns units.
- When monitoring starts with the option on, ISLC calls **`NtSetTimerResolution(value, TRUE)`**
  (ntdll, not `timeBeginPeriod`). On every poll tick it calls it again if the current resolution
  from `NtQueryTimerResolution` is coarser than wanted. On Stop, on exit and on suspend it calls
  `NtSetTimerResolution(value, FALSE)`. So the request is **held for as long as monitoring runs**.
  That is ISLC's own per-process request; the kernel also drops it if ISLC dies.
- Every set and release is wrapped in `ImpersonateUser.Taketoken` / `ReleaseToken` (below). That
  has no effect on the request, which the kernel tracks per process, not per thread token.
- **Windows 11 "GlobalTimerResolutionRequests"** checkbox (marked `**`, "needs reboot"). Checking it
  writes DWORD `GlobalTimerResolutionRequests` = 1 under
  `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel`. Unchecking **deletes** the value
  (even if it was set before ISLC). At start the box mirrors the current registry value. With the
  value set, Windows 11 applies the highest requested resolution system-wide again, as before
  Windows 10 2004.
- The UI shows the min/max/current resolution and guesses the QPC source from
  `QueryPerformanceFrequency`: 10 MHz is labelled TSC, 14.318 MHz HPET, and so on. This is display only.

## SYSTEM-token impersonation (`ImpersonateUser.Taketoken`)

- It enables `SeDebugPrivilege` and asks for Security, Backup, Restore, TakeOwnership, **Tcb** and
  **CreateToken** privileges. An admin token has no Tcb or CreateToken; `AdjustTokenPrivileges` still
  "succeeds" and simply does not assign them. These privileges stay enabled in ISLC's token; they are
  never disabled.
- It walks **every process** (`Process.GetProcesses`) except a short skip list (idle, system,
  registry, smss, csrss, wininit, services, searchfilterhost, nissrv). For each one it reads
  `Process.Handle`, which in .NET means `OpenProcess` with **full access**. It opens the token and
  stops at the first process whose token is SYSTEM, then impersonates it on the calling thread.
  `ReleaseToken` reverts. The process handles are left to the garbage collector.
- So every timer set and release (and only those; the purge path does not do this) may try a
  full-access open of whatever processes come before the first SYSTEM process in enumeration
  order. **A running game can be one of them.** A protected or anti-cheat-guarded game simply refuses.
  This only happens when the timer option is on.
- The changelog for 1.0.3.0 says "Removed a process from the system ImpersonateLoggedOnUser"; the
  mechanism is still present in 1.0.4.8.

## Autostart

- An "auto start" checkbox creates a **Task Scheduler** task named `Intelligent StandbyList Cleaner`
  in the root folder (COM `ITaskService`). The task:
  - has a logon trigger for any user (group logon), with a 30-second delay;
  - runs with **highest privileges**, so there is no UAC prompt;
  - has no time limit and ignores new instances;
  - runs the exe without arguments.

  Unchecking deletes it. If the exe has moved, the task's path is updated when the box is checked.
  Off by default.
- The task passes no arguments, so automatic monitoring after logon depends on the saved
  "start minimized" setting.

## Other features

- **Hotkey** (off by default): `RegisterHotKey`, default Ctrl+Alt+P, for a manual purge in game.
- **Command line** (`ReadMe.txt` and `SetConfig`):
  - `-minimized` starts minimised and starts monitoring.
  - `-polling`, `-listsize` and `-freememory` override the settings.
  - `-runonce` polls once and exits.
  - `-purge` purges once and exits before any UI, even when another copy is running.
- **Settings:** `config.json` next to the exe, written atomically (temp file and replace) whenever a
  value changes. Portable; no registry settings of its own.
- **Performance counters:** the ReadMe says ISLC re-enables disabled performance counters
  (`Perflib\Disable Performance Counters`). In 1.0.4.8 the method that does it, `CheckCounterAvail`,
  exists but nothing calls it, and since 1.0.4.0 counters are no longer used.
- **"Prevent G-SYNC for the app"** (changelog 1.0.0.1): the app turns off its own WPF hardware
  rendering. It does not touch the driver.
- **Always on top, dark theme, minimize on close, 18 UI languages, Discord/PayPal/website buttons.**
  The buttons open the browser with `Process.Start`.

## Network and telemetry

- **Update check at start**: `UpdateCheck.GetVersionInfoAsync` makes an HTTPS GET of
  `https://www.wagnardsoft.com/api/islc/version.json` with a 5-second timeout. The user agent is
  `ISLC/<version> (Intelligent Standby List Cleaner)`. The reply is the latest version, date,
  download and notes URLs and the minimum supported version. A newer version only shows an "update
  available" link. Nothing is downloaded or installed automatically.
- An older `CheckUpdate.VersionUpdateAsync` (GET `https://www.wagnardsoft.com/ISLC/currentversion.txt`)
  is still in the DLL; no caller was found in the UI.
- **No telemetry, analytics, crash upload, account or ads.** No other URLs are fetched; the
  Discord, PayPal and website links open only on click.
