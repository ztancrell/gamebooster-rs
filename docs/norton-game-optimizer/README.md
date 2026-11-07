# Norton Game Optimizer: what it really is

Norton Game Optimizer ("Norton GO") is the gaming feature of **Norton 360 for Gamers** (Gen Digital,
[norton.com](https://us.norton.com/products/norton-360-for-gamers), $44.99 first year, then
$119.99/year; the 30-day trial needs a payment method). Norton sells it as a "patented technology
for multi-core CPU PCs" that "dedicates CPU cores to the game" by isolating "non-essential apps to a
single CPU core", plus Full-Screen Detection that silences alerts.

**What it is in the current build:** Norton 360 is now built on the **Avast engine** (Gen Digital
bought Avast in 2022). Every binary is an Avast module (`ashServ.dll`, `aswAv.dll`, `asw*`, driver
prefix `nll`), and Game Optimizer is Avast's **Gaming Mode / Do Not Disturb** feature: the UI calls
the page `doNotDisturb`, the engine namespace is `asw::gaming_mode` (built from
`features\gaming_mode\src\gaming_mode\GamingModeModule.cpp`). The "core isolation" is one
`SetProcessAffinityMask` call per background process with a fixed mask of **logical CPUs 0 and 1**
(the first physical core on a hyper-threaded CPU). The game itself is never touched by that rule;
it is set to High priority by a separate rule, and the power plan is switched to High performance
with core parking off.

Version **26.10.11201.2729** (product `norton-suite`, build time 2026-10-06, Icarus
26.10.11639.0), fetched 2026-11-07 from Gen Digital's own CDN `honzik.avcdn.net`, the same
content-addressed store as [AVG TuneUp](../avg-tuneup/README.md). No account, payment or stub was
needed: the setup manifest is public at `https://honzik.avcdn.net/defs/norton-suite/release.xml`
(product ID found by trying names; the online stub `norton_360_online_setup.exe` is an `icarus_sfx`
build, ProductId `norton-icarus`, per public file-analysis listings). Every file was checked against
the SHA-256 in the manifest.

| File | Size | SHA-256 |
|---|---|---|
| `release.xml` (setup manifest) | 10,633 | `ea76edfd2d47d4a94f38f042ca741975a8c4e603821f1698bfea83412f114d4a` |
| `product-def.xml` (5,276 files) | 2,573,289 | `55399adafbd805fa76a33e6094b2bbd0536c19b8ea6e9df093fd8fadf8845609` |
| `aswAv.dll` x64 (Gaming Mode engine) | 27,804,600 | `8d5f81f2b20e29100786386ecc76dfb11a8f2319d910c811b8f40b7ebf5ff8c8` |
| `ashServ.dll` x64 (service core) | 28,782,520 | `d6576a5a140b4820a84577badbb082ecf32e310b34a186c8ba708383162f0a52` |
| `gaming_mode/dndrules.dat` (rule set) | 1,801 | `9a41d1fd21a0856918071852fc2d6ccb319e824932b144debc134312af1f693d` |
| `gaming_mode/dnddetection.dat` (game list) | 555,642 | `99ba57bef2cefca310594e853769baa572c016d7de5245cc80fcb3bba711624f` |
| `dnd_helper.dll` x64 | 1,642,936 | `be820aaa8256dddad9aa2da5b5f1ed66d0235caa0de24d3c9981f8971c90680e` |
| `gaming_hook.exe` x64 | 598,456 | `4d6e21908f4a1b247cbea3023a76437e7e5b4e22a48c98972ae6c25c0c99966d` |
| `gui_resources/default_av/doNotDisturb.js` | | `17aa362fbf0f4cb73b9a5baeb5fedc07bfebfd1a661442b1e2f164ed2cfd9785` |
| `gui_resources/default_av/locale-en.json` | 674,155 | `670390e23758caeb1e9fea180c600d8fc06a1ef13bf4526b064bad81989236b9` |

**Nothing was run.** Static analysis only: the manifest, the UI strings and settings bundle, the two
gaming data files (LZMA-packed JSON, unpacked), imports and strings of the modules above, and
objdump disassembly of the rule code in `aswAv.dll` (Ghidra timed out on these 28 MB modules).
Disassembly output was deleted.

**Limits.** The default value of each Game Optimizer toggle comes from the remote config
(`config.def` is downloaded from Shepherd at install, not shipped), so the defaults below are from
Norton's own support text and the shipped rule set, not from a config file. The x64 binaries were
read; x86/arm64 builds were not. The same binaries serve every Norton 360 tier; which tier unlocks
Game Optimizer is a licence check that was not traced (inferred).

## How it's built

**Installer (Icarus).** Identical to AVG TuneUp: `release.xml` names `icarus.exe`, `icarus_ui.exe`,
`product-def.xml` and `setupui.cont`; every file comes from
`https://honzik.avcdn.net/universe/<h[0:4]>/<h[4:8]>/<h[8:12]>/<sha256>.lzma` and carries an
`ASWSig2A` signature. Branding variables: product prefix `nll`, `Norton 360`, brand ID `6_19`,
service module `NortonSvc.exe` (service name "Norton Antivirus"), UI `NortonUI.exe` (CEF).

**Product.** One suite with optional groups: antivirus (`ais_*`), firewall, sandbox, VPN
(`vpn_*`: OpenVPN, WireGuard, `nllWintun`), Norton Family, backup, password manager, a "tu_suite"
(Software Updater, `su_*`), media scan with ONNX/OpenVINO models, and migration agents that remove
the old Symantec-era Norton (`av_migration/norton-av.dll`). Game Optimizer is group
`ais_cmp_gamingmode` (feature `Features.GamingMode`): only `gaming_hook.exe`, `dnd_helper.dll` and
the two data files. The logic lives in the shared engine `aswAv.dll`; the service API in
`ashServ.dll` registers `app.gameMode.*` RPC calls (`GetGames`, `SetGame`, `RunGame`,
`RemoveGame`, `IsRuleRunning`).

**Compared with AVG TuneUp** (same company, same installer and CDN): TuneUp is the CCleaner/Avast
Cleanup engine and has no gaming mode at all. Norton 360's Game Optimizer is the Avast antivirus
Gaming Mode, so Avast and AVG antivirus products with "Do Not Disturb / Game mode" very likely run
the same code (inferred from the shared `asw::gaming_mode` namespace; not checked in those
products).

## Background services, drivers, telemetry, offers

- **Kernel drivers (x64, `Inf\x64\`):** `nllArPot`, `nllAsz`, `nllbidsdriver`, `nllbidsh`,
  `nllbuniv`, `nllDrvBrg`, `nllElam` (early-launch anti-malware), `nllKbd`, `nllMonFlt`,
  `nllNetHub`, `nllNetNd6`, `nllRdr2`, `nllRvrt`, `nllSnx` (sandbox), `nllSP` (self-protection),
  `nllStm`, `nllVmm` (virtualization), `nllArDisk`, plus VPN drivers (`nllWireguard`, `nllWintun`,
  `nll-ovpn-dco`). That is roughly 18 resident filter, network and process drivers before the VPN:
  the full Avast stack. Game Optimizer itself adds no driver; `dnd_helper.dll` talks to a minifilter
  port (`FilterConnectCommunicationPort`).
- **Services:** "Norton Antivirus" (`NortonSvc.exe`, LocalSystem), the firewall and tools services,
  `NortonVpn`, `FamilySvc`, and the update/scheduler helpers (`sched.exe`, `AvEmUpdate.exe`).
- **Telemetry:** Avast "Burger" analytics (`burger_client.dll`, `event_manager_burger.dll`,
  `event_manager_ga.dll`). Gaming Mode has its own Burger translator and protobuf messages
  (`asw.gamingmode.proto.SubmitGame`, `Application`, `FileInfo`, `DisabledRule`), so the names,
  paths and versions of detected games are reported (inferred from message names). A default rule
  `MeasurePerformance` (the only rule in the rule set's `default` group) samples every process's
  `cpuAverage`, `cpuPeak`, `workingSet`, `pageFile` and `threads`, with an `isGameProcess` flag,
  while a game runs (strings next to `PerformanceMonitorRule::ActivateImpl`; where the samples go
  was not traced).
- **Remote control:** `config.def` comes from Shepherd at install (`%ICARUS_SHEPHERD_URL%`), and
  `aswAv.dll` refreshes the game white/black lists from the server
  (`avdef://config/GamingMode/RefreshWhiteListSeconds`, "forcing update of data file").
- **Offers:** the UI has Norton Utilities ("Revitalise your PC using our cleaner and optimiser
  tool"), Dark Web Monitoring (gamer tags: in scope only as a mention), VPN, LifeLock tiers.

## What Game Optimizer actually changes

**When it activates.** A game is detected by the downloaded whitelist (`dnddetection.dat`: 31,105
whitelist entries, 848 blacklist, 37 presentation apps), by launch from one of ten "game clients"
(`steam.exe`, `origin.exe`, `battle.net.exe`, `ubisoftgamelauncher(64).exe`, `launcher.exe`,
`idtechlauncher.exe`, `bethesdanetlauncher.exe`, `epicgameslauncher.exe`, `galaxyclient.exe`), by
"fullscreen decision" (Full-Screen Detection with high CPU, per Norton's footnote), or by manual
add. Detected games are added to the list and the **Game** profile starts while the game runs; a
full-screen non-game gets the **Presentation** profile (notifications only). The affinity rule
refuses to run below `avdef://config/GamingMode/MinLogicalProcessorsCount` logical processors
("system has few logical processors (...), not activating the rule"); Norton documents "four or
more core processors".

**The rule set** (`dndrules.dat`, LZMA-packed JSON, server-updatable) defines the rules. The Game
profile uses: `DisableDrawOverWindow`, `DisableWindowsNotifications`, `KeepGameInForeground`,
`PauseAvBackgroundTasks`, `DisableAvNotifications`, `PauseSystemBackgroundTasks`,
`PauseWindowsUpdate`, `DisableWinUpdateAutoReboot`, `SetHighPerformanceMode`, `SetHighPriority`,
`SetProcessAffinity` and `EnableAutoDetections`. Two rules of the "MaximumPerformance" group,
`SetCpuLimit` and `SuspendNongamingUserProcesses`, exist in the engine but are **not** in the Game
profile. Each Game-profile rule has a UI toggle under `avcfg://settings/GamingMode/GameRule_<Rule>_Enabled`.

| Rule (UI label) | What the code does | API / place | Revert |
|---|---|---|---|
| `SetProcessAffinity` ("Restrict resource usage for user processes": "Isolates non-essential apps to a single CPU core") | Every process except the game is pinned to an affinity mask read from `GamingMode`/`AffinityMask` with a **default of 0x3** (logical CPUs 0 and 1). Process selection flags 0x28320 plus 0x10000: *IncludeAllProcesses*, *ExcludeSetProcess* (the game exe; no child or process-tree exclusion flag is set, so its helpers are not excluded, inferred), *ExcludeAvastPath* (Norton itself), *ExcludeWindowsProcesses*, *ExcludeSystemSessionProcesses* (session 0, i.e. services) and *ExcludeProcessesMatchingWildcard* (the `ignoredProcesses` list below). A process already at the mask is skipped | `aswAv.dll` rule at 0x180132c40 (mask default 0x180132d7c, flags 0x180132ddc); `Process::setAffinityMask` does `OpenProcess`, `GetProcessAffinityMask`, `SetProcessAffinityMask` (0x180b965d1) | Old mask is stored per process and put back when the game exits ("reverted other session processes affinity mask") |
| `SetAffinityForSystemProcesses` ("Restrict resource usage for system processes": "Isolates non-essential system processes to a single CPU core") | Same rule, with the Windows-process and session-0 exclusions cleared (bits 0x8000 and 0x20000), so services are pinned too, except the ignore list. Only selectable when the user-process toggle is on | `avcfg://settings/GamingMode/SetAffinityForSystemProcesses` (0x180132fd1) | Same |
| `SetHighPriority` ("Prioritize the game's processes") | Game process to **High** priority class; other user processes to **very low I/O priority** | `SetPriorityClass`, I/O priority via `NtSetInformationProcess` (rule at 0x18012d707) | "priority class of process ... restored", "IO priority of other user processes restored" |
| `SetHighPerformanceMode` ("Automatically set power plan to Max Performance") | **Only on AC power** (`GetSystemPowerStatus`): activates Windows' High performance plan `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c` (skips if already on) and writes its AC value for processor *core parking min cores* (`54533251-…` / `0cc5b647-c1df-4637-891a-dec35c318583`) to **100 %**, i.e. core parking off. The previous plan GUID is saved to `prev_power_scheme.txt` | `PowerSetActiveScheme`, `PowerReadACValueIndex`/`PowerWriteACValueIndex` (0x18012c722, battery check 0x18012c9b4) | Old plan reactivated and the old core-parking value written back ("Power scheme successfully restored") |
| `PauseWindowsUpdate` ("Pause Windows Updates") | Stops the Windows Update service and postpones the `\Microsoft\Windows\WindowsUpdate` scheduled-task trigger | Service control, Task Scheduler COM (`PauseWindowsUpdateRule`) | Trigger restored, service relaunched if it was running |
| `DisableWinUpdateAutoReboot` | Disables the `\Microsoft\Windows\UpdateOrchestrator` Reboot task | Task Scheduler | Re-enabled |
| `PauseSystemBackgroundTasks` | Disables a set of scheduled tasks; depending on a stop mode it can also stop the Task Scheduler service (task list not traced) | Task Scheduler COM, service control | Tasks re-enabled, service started only if it was running before |
| `PauseAvBackgroundTasks` ("Pause Norton background tasks") | Norton's own scheduled scans and updates are postponed ("Task suspended due to gaming mode is activated", "Gaming mode: update postponed") | internal | automatic |
| `DisableAvNotifications`, `DisableWindowsNotifications`, `DisableDrawOverWindow`, `KeepGameInForeground` ("Suppress notifications …", Full-Screen Detection) | Norton's and Windows' notifications are suppressed; `dnd_helper.dll`/`gaming_hook.exe` hook window creation so pop-ups cannot draw over the game, and keep the game in front (`LockSetForegroundWindow`, `SetForegroundWindow` imports) | window hooks in user processes | automatic |
| `SetCpuLimit` (not in the Game profile) | Puts other session processes in a job object with a CPU limit ("other session processes cpu limit changed to …") | `CreateJobObjectW`, `SetInformationJobObject`, `AssignProcessToJobObject` (inferred from imports) | "reverted other session processes cpu limit" |
| `SuspendNongamingUserProcesses` (not in the Game profile; setting `GamingMode/SuspendProcesses` has no UI label) | Suspends non-game user processes while playing | `NtSuspendProcess`/`NtResumeProcess` imports (link inferred) | "resumed non-game processes" |

**The affinity ignore list** (`processAffinity.ignoredProcesses` in `dndrules.dat`) keeps these on
all cores: `svchost`, `services`, `explorer`, the search indexer, `sppsvc`, `wininit`, `smss`,
`csrss`, `lsass`, `dwm`, `winlogon`, `conhost`, `MsMpEng`, `audiodg`, `wmiprvse`, `dllhost`,
`spoolsv`, NVIDIA containers and GeForce Experience, PunkBuster (`pnkbstra/b`), BattlEye
(`beservice`), `steamservice`, `gameoverlayui`, SteamVR, Vive and Oculus runtimes, OBS, Streamlabs,
XSplit, vMix, Wirecast, Twitch Studio, NVIDIA Share, Discord, `javaw`, `3dmark`, Process Explorer.
Not on it: the EA app (`EADesktop.exe`), Battle.net, Epic, Ubisoft and Steam client UIs, browsers,
EasyAntiCheat, Vanguard and EA Javelin.

**Nothing persistent.** No registry tweak, no service start-type change, no file is left behind
except `prev_power_scheme.txt`. Every change is reverted when the game exits, and the power plan
survives a crash through the saved GUID. Affinity and priority are in memory only, so a crash of
the Norton service leaves background processes pinned to CPU 0 and 1 until they restart
(inferred: no persistent journal for affinity was found).

## Claims vs reality

Norton's product page and support pages (2026-11-07): "Maximize game performance by dedicating
CPU cores to the game", "Norton GO directs more power to your game, maximizing speed", "Helps
eliminate FPS lags and slowdowns from your other apps", "feed them the maximum power", "This may
improve your frame rate and load times depending on what your system is doing at the time",
"patented technology". The only number is from a 2021 press release on a Norton-commissioned
PassMark test against five unnamed competitors: "Norton GO increases FPS up to 30% while
downloading and playing games simultaneously". Games, hardware and frame-time data were not
published in that release.

| Claim | What the code does | Verdict |
|---|---|---|
| "Dedicating CPU cores to the game" | The game keeps its default affinity (all CPUs). Every *other* non-Windows process is pinned to CPUs 0 and 1. The game gets cores 1 to N to itself only in the sense that other apps can no longer run there | **Misleading wording, real mechanism.** On a 16-thread CPU, 14 threads are freed from background apps. It only matters while background apps actually use CPU; on a quiet PC it changes nothing |
| "Isolates non-essential apps to a single CPU core" | Fixed mask 0x3 = logical CPUs 0 and 1 = the first physical core with its hyper-thread sibling | **True**, but CPU 0 is also where Windows sends most interrupts and DPCs by default, so the pinned apps share the busiest core |
| "Up to 30 % more FPS while downloading and playing" (PassMark, 2021) | A download client (Steam, EA app, browser) pinned to one core cannot take CPU from the game | **Plausible in that one scenario** (a CPU-heavy download next to a CPU-bound game). Vendor-commissioned, no frame-time data, and it says nothing about a PC without background load. Throttling the downloader also slows the download |
| "Feed them the maximum power" / "Max Performance" power plan | High performance plan plus core parking off, on AC only | **Real but small**, and Windows' own High performance plan does the same. It costs heat: on thermally limited laptops it can lower sustained clocks |
| "Prioritize the game's processes" | `SetPriorityClass` High on the game exe | **Real**, the same as Task Manager > Set priority. Anti-cheats that block outside handles (EA Javelin) refuse it (inferred from our BF6 test of the same call) |
| "Free your PC from power-hungry programs running in the background" | Pausing Windows Update, Norton's own scans and some scheduled tasks; pinning, not closing, other apps. Suspending apps exists in the engine but is not in the Game profile | **Partly true**: nothing is freed from RAM, background apps keep running on one core |
| Full-Screen Detection / "silences all but critical alerts" | Notification suppression, window hooks against pop-ups, Norton's own scans postponed | **Real**, and it is mostly about Norton not interrupting its own user |
| "Patented technology for multi-core CPU PCs" | Affinity and priority changes through documented Windows APIs, the same as Process Lasso's CPU Limiter or Task Manager > Set affinity | **Marketing**: whatever the patent covers, the mechanism is a stock API call |
| "Game launchers currently monitored: Bethesda, Blizzard, Epic, ID, Origin, Rockstar, Steam, and Uplay" | The ten `game_clients` match this (plus GOG Galaxy) | **True but dated**: EA's current EA app is not on the list (Origin was retired in 2022), so EA games are found only by full-screen detection, the whitelist or a manual add |

## On a 2019 MacBook Pro under Boot Camp playing BF6

The target is an i9-9880H (8 cores / 16 threads), CPU-bound in BF6, with EA Javelin anti-cheat.

- **Detection.** BF6 is not in the downloaded whitelist (Battlefield 2, Bad Company 2 and older
  are), and the EA app is not a "game client". BF6 would be picked up by full-screen detection with
  high CPU, or after a manual add.
- **Affinity.** BF6 is not touched. Background user apps (EA app, browser, Spotify, launchers,
  anything not on the ignore list) are pinned to CPUs 0 and 1. Services and Windows processes are
  left alone unless "system processes" is turned on. With 16 threads and BF6 using many of them,
  freeing 14 threads from background noise is a small gain, only while something in the background
  is busy. The cost: the EA app (overlay, friends, party voice) and any browser or voice client not
  on the list run on one core that also takes the interrupts, and the game's own helper processes
  are pinned too, because only the detected exe is excluded (inferred from the flags).
- **Anti-cheat.** Norton never opens BF6 for affinity and does not read game memory. The High
  priority call on `bf6.exe` will very likely fail with access denied, as our own process method
  did on this Mac ([priority.md](../tweaks/priority.md#registry-method)). Javelin's service runs in
  session 0, which the default affinity rule skips. Low ban risk, but the "system processes" option
  would try to pin it (and fail on a protected process, inferred).
- **Power.** High performance plus core parking off keeps all cores awake. On this heat-limited
  Mac that works against the low minimum processor state used for BD PROCHOT
  ([power-plan.md](../tweaks/power-plan.md#optional-low-minimum-state)), the same objection as
  Process Lasso's Bitsum Highest Performance plan.
- **The rest.** About 18 kernel drivers and an always-on service with cloud telemetry is a large
  install for the gaming part, which is a few hundred lines of `SetProcessAffinityMask`,
  `SetPriorityClass` and `PowerSetActiveScheme`. Not worth installing for BF6. If you already run
  Norton 360, turn off "Restrict resource usage for system processes" and the Max Performance power
  plan, and keep the notification part.

## Compared with gamebooster-rs

| | Norton Game Optimizer | gamebooster-rs |
|---|---|---|
| Game priority | High via `SetPriorityClass` while the game runs | Same, plus the IFEO `PerfOptions\CpuPriorityClass` method that works under Javelin ([priority.md](../tweaks/priority.md)) |
| Background apps | **Affinity to CPUs 0–1** for all non-Windows processes, plus very low I/O priority for other user processes | `lower_others`: Below normal priority (once, or `lower_others_mode = "dynamic"` like ProBalance) and I/O priority Low; no affinity |
| Exclusions | Wildcard list (Discord, OBS, NVIDIA, SteamVR, BattlEye, PunkBuster…), Windows and session-0 processes | Name and publisher lists, games, `close` list, SYSTEM/services |
| Power | Stock High performance + core parking off, AC only | Own plan copied from High performance with explicit settings, optional low minimum state for the Mac |
| Windows Update | Service stopped and trigger postponed while playing | Not touched (the [services](../tweaks/services.md) step stops SysMain, DiagTrack, Spooler and third-party updaters, not Windows Update) |
| Notifications | Suppressed in full screen, pop-ups hooked | Not done (Windows Focus Assist does this) |
| Undo | In memory, at game exit; power plan via saved GUID | Journal on disk (pid + creation time), `restore` |
| Footprint | Antivirus suite, ~18 drivers, telemetry | One CLI, no driver, nothing resident (unless `--watch`) |

Priority lowering and affinity differ in kind: lowering only gives the game first pick when both
are ready, while affinity caps what a background app can ever use, even when the game leaves cores
idle. Process Lasso uses affinity only for its opt-in CPU Limiter and calls it risky for voice and
streaming apps ([process-lasso](../process-lasso/README.md)).

## Adoption items for gamebooster-rs

1. **Affinity restriction of background apps: skip for now.** It is reversible and documented
   (`SetProcessAffinityMask`), but on the Mac's 16 threads the gain is small, the cost (EA app,
   voice, browser and the game's helpers squeezed onto the interrupt core) is real, and
   `lower_others` already covers the "background spike" case without capping anything. If a
   measurement ever shows `lower_others` is not enough on the 4-core Macs, a dynamic, per-process
   version (pin only a process that keeps hogging, like Process Lasso's CPU Limiter) fits the
   `--watch` design better than Norton's pin-everything-at-start.
2. **Norton's affinity ignore list: worth borrowing as data.** It names the processes a
   background-restraint feature must not touch. `lower_others` already skips most of them (Discord,
   `obs64.exe` and the OBS publisher, NVIDIA and Valve by publisher, which covers SteamVR,
   `audiodg`, `dwm`, BattlEye, `MsMpEng`; see [priority.md](../tweaks/priority.md)). Missing and
   cheap to add to `NEVER_LOWER`: Streamlabs OBS, XSplit, vMix, Wirecast, Twitch Studio, the Oculus
   and Vive runtimes, and PunkBuster (`pnkbstra/b`).
3. **Power plan on battery: already better.** Norton skips the power rule on battery; our plan has
   explicit AC/DC values and a Mac low-power option. No change.
4. **Pause Windows Update while playing: optional, needs a decision.** Norton stops the Windows
   Update service and postpones its scheduled trigger, then restores both; the `UpdateOrchestrator`
   Reboot task is disabled too. gamebooster-rs does not touch Windows Update today. It is
   reversible, but it touches update state, so it would need its own opt-in key and journal entry;
   not worth it unless updates are seen interrupting games on the Mac.
5. **Notification suppression: skip.** Windows Focus Assist / Do Not Disturb does it natively, and
   hooking other processes' windows is anti-cheat surface.
