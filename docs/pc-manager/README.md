# Microsoft PC Manager: what it really does

Microsoft PC Manager (pcmanager.microsoft.com) is Microsoft's free "PC cleaner" app with a big **Boost**
button, a process list, startup management, "Health check" and "Deep cleanup". This page comes from
reading the official Microsoft Store package `Microsoft.MicrosoftPCManager_3.23.0.0` (x64 part of the
msixbundle) with static reverse engineering of its .NET assemblies, 2026-11-07. **Nothing was run.**

Where the package came from: the website's download button only links to the Store. The bundle was
fetched from Microsoft's own Store delivery servers (`displaycatalog.mp.microsoft.com`,
`fe3.delivery.mp.microsoft.com`, `tlu.dl.delivery.mp.microsoft.com`); its SHA-1 matches the digest the
Store's delivery service lists for 3.23.0.0. The Store also offered a newer 3.23.2.0 bundle, not
read. The package's Authenticode signature was not checked separately.

What was verified vs inferred: everything below marked "verified" was read in the assemblies or
bundled config. Several defaults are **server-controlled**: PC Manager downloads signed, AES-encrypted
config files (`mspcm_*.dat`) and polls for updates; the bundled copies were not decrypted, so what
Microsoft's servers currently switch on is not known. Those places are marked.

## Short version

- **Boost** (verified) trims memory, nothing else that touches performance. With the bundled config
  (memory mask 0) it calls `SetProcessWorkingSetSize(-1, -1)` on every process it can open, which
  is `EmptyWorkingSet` for all of them, Razer/SGB's RAM trim. A server config can switch it to a
  Mem Reduct-style path in PC Manager's elevated core process (`NtSetSystemInformation`): purge the standby
  list and low-priority standby list, flush the modified list, empty working sets, combine identical
  pages, shrink the file cache, flush registry hives. It runs at most once per 30 s and reports the
  "memory freed". There is also a "PcBoost" temp-file clean scene (that the home Boost button runs it too is inferred from its name; the web UI that calls it was not read) (Windows logs, temp, web cache,
  Windows Update download cache, error reports, crash dumps). No power plan, no priority, no Game
  Mode, no service changes: **PC Manager 3.23 has no game mode at all** (no game, power or priority
  code; the only "game" strings are a cleanup category for leftover game installers).
- **Smart Boost** (verified, off by default): an hourly timer that runs the memory trim when memory
  use is at or above 80 %, and a temp-file clean when at least 1 GB is found and the CPU is under 80 %.
  It **skips whenever Windows says the user is busy** (`SHQueryUserNotificationState` is anything but
  "accepts notifications", e.g. a full-screen D3D game or presentation mode), so it is built not to
  fire during a game.
- **Process management** (verified): lists the user's apps with their resource use; "End" is a plain
  `Process.Kill`, no save prompt.
- **Startup management** (verified): uses the same `Explorer\StartupApproved` enabled/disabled bytes
  as Task Manager's Startup tab (HKCU or HKLM), switches services between **Automatic and Manual**
  (never Disabled, via WMI `ChangeStartMode`), and enables/disables scheduled tasks. A bundled list of
  about 200 rules colours entries with "suggest turn off" / "keep" hints, aimed mostly at Chinese
  software (WeChat, WPS, Baidu Netdisk, Thunder, 360) plus Razer, Discord, Logitech and similar.
  All of it is persistent but undoable from PC Manager or Task Manager.
- **Health check** (verified names, server-controlled rollout): temp/log/web-cache/recent-files
  clean, Smart Boost suggestion, startup suggestions, plus "reclaim" items that put Microsoft defaults
  back: Edge as default browser, Edge settings, lock screen back to Spotlight, taskbar items,
  Office and PDF default-app "upsell". The percentage of users opted in by default to each reclaim
  item comes from server config.
- **Deep cleanup** (verified): Windows' own Disk Cleanup handlers (the `VolumeCaches` COM plug-ins)
  plus browser caches. Preselected items include the **DirectX Shader Cache**, Delivery Optimization
  files, **device driver packages**, memory dumps, Defender files, thumbnails and Windows upgrade logs.
  Deleting is permanent.
- **Installed parts** (verified from the manifest): an auto-start **LocalSystem service** ("PCManager
  Service Store"), a logon startup task (`PCMAutoRun`, "start with Windows" on by default), a widget,
  and an elevated core process (`MSPCManagerCore`) that does the system calls over a local RPC channel. The package also
  ships **three kernel drivers**: `ahflt.sys` (minifilter, "AntiHack" browser/default-app
  protection), `ClawFlt.sys` (minifilter) and `ClawNet.sys` (WFP callout), the last two for a
  "ClawSecurity" feature that watches an OpenClaw AI agent PC Manager can deploy into WSL. The service
  installs them only when server config enables them (`IsAhfltEnabled`, and a sampling rate for Claw),
  and only after checking they carry a Microsoft signature. Both defaults in code are off; what the
  servers set was not readable. The Claw INFs declare start type 2 (auto start) although their
  comments say demand start.
- **Telemetry** (verified): Application Insights (US, EU, China endpoints), "experience improvement"
  on by default. Events include boost results, memory before/after, names of processes you end,
  startup changes, and a **daily report of how long selected competitor processes have been running**
  (Chrome, 360, QQ/Tencent, Lenovo PC Manager, Huorong, 2345, Kingsoft, and Edge itself). Whether
  every event obeys the switch was not traced.

## Every part, in one table

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky**
(can break things, cost privacy or security, or trip anti-cheat), **bloat**.

| Part | Default | What it does | Reversible | Verdict |
|---|---|---|---|---|
| Boost: memory (bundled config) | button | Empties every accessible process's working set | n/a (pages come back by faulting in) | placebo / slightly harmful (stutter while pages fault back in) |
| Boost: memory (server-enabled mask) | server | Purges standby lists, flushes modified list, empties working sets, page combining, file cache shrink, hive flush | n/a | placebo / harmful (throws away Windows' file cache; same as ISLC) |
| Boost: temp files | button | Deletes temp, logs, web cache, WU download cache, error reports, crash dumps | no | placebo for speed; frees disk space |
| Smart Boost | off | Hourly memory trim at 80 % use and temp clean at 1 GB; skips while full-screen/busy | n/a | placebo, harmless during games (it skips them) |
| Process list "End" | user | `Process.Kill` | no (unsaved work lost) | **risky** if pointed at the wrong thing; same as Task Manager |
| Startup apps | user, with hints | Task Manager's `StartupApproved` flag | yes | useful (same as Task Manager) |
| Startup services | user | Automatic to Manual via WMI | yes | mostly useful, never Disabled, so nothing breaks hard |
| Scheduled tasks | user | Enable / disable | yes | fine |
| Health check: clean | button | As Boost temp files plus recent-files list | no | placebo for speed |
| Health check: "reclaim" items | server rollout | Edge default browser, Edge settings, lock screen, taskbar, Office/PDF defaults | yes (change it back in Settings) | bloat / dark pattern; not about performance |
| Deep cleanup | manual, many preselected | Disk Cleanup handlers incl. **DirectX shader cache** and driver packages | no | frees space; shader cache wipe is **harmful** for games (recompile stutter); driver packages wipe blocks driver rollback |
| LocalSystem service + logon task | on | Elevated back end, starts with Windows | uninstall | bloat |
| Kernel drivers (ahflt, ClawFlt, ClawNet) | server-gated | File-system minifilters and WFP callout | removed by the service / uninstall (not traced) | **risky** in principle (kernel code that anti-cheat sees); not loaded unless switched on server-side |
| OpenClaw deployment (WSL) | manual | Installs an AI agent into WSL from openclaw.ai | uninstall in WSL | out of scope, bloat for gaming |
| Telemetry | on | Usage, boost results, competitor process uptime | switch in settings | **risky** (privacy), mild |

## For a 2019 MacBook Pro under Boot Camp playing BF6

- **Nothing in PC Manager raises frame rate.** BF6 on this Mac is GPU-bound under a shared CPU+GPU
  power budget; PC Manager touches neither the power plan, CPU limits, priorities nor Game Mode.
- **Boost is the classic RAM trim.** Running it before a match makes Windows page BF6's launcher and
  helper processes back in; running it mid-game (it can't touch BF6 itself if the anti-cheat
  protects the process handle, inferred, not tested) only causes page faults elsewhere. Placebo.
- **Deep cleanup's DirectX shader cache item** (preselected) is the one that can hurt. It is
  Windows' own Disk Cleanup handler of that name (which folders it clears was not read; the rule
  data is encrypted). After a wipe the first matches stutter while shaders compile again. Untick it.
- **Startup management is fine** and equals Task Manager's Startup tab.
- **Anti-cheat:** no injection, no ETW tracing found for boosting. The kernel drivers are the only
  thing an anti-cheat might object to, and they stay unloaded unless Microsoft's config turns them on;
  `sc query ahflt`, `sc query ClawFlt` and `sc query ClawNet` on the target Mac would show whether they
  are present.
- **Overall:** a disk cleaner and Task Manager front-end with a placebo button, a LocalSystem service,
  server-controlled features and telemetry. For gaming it adds nothing gamebooster-rs lacks.

## Compared with other boosters and gamebooster-rs

- The memory "boost" is the same idea as Razer Cortex's and SGB's RAM trim, and with the server mask
  the same list purges as ISLC and Mem Reduct ([ISLC](../islc/README.md)): placebo or harmful.
- Unlike most cleaners it is honest about startup items (Task Manager's own mechanism, services
  only to Manual) and has no registry "tweaks" at all.
- Smart Boost's "don't act while the user is busy" check is the reverse of what a game booster wants,
  but it is a neat, driver-free signal: Windows itself reports when a full-screen D3D app runs.

## Adoption items for gamebooster-rs

1. **Docs advice only:** tell users not to clear the DirectX shader cache before playing (PC Manager's
   Deep cleanup preselects it, as do Windows' Disk Cleanup and other cleaners), since BF6 then
   recompiles shaders and stutters. Nothing for `boost` or `restore` to do.

Everything else is placebo, not about performance, or already covered (gamebooster-rs's services
list, `lower_others`, power plan and `status` checks go further and are undone by `restore`).
