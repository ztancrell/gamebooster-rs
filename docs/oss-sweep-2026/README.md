# 2026 sweep: newer game boosters, and Optimizer / Atlas OS from source

A sweep for game boosters and optimizers that are popular or new in 2024–2026 and were not yet
covered in [research](../research.md), plus a source reading of the two big open-source ones:
hellzerg's **Optimizer** (and its successor **OptimizerNXT**) and the gaming/performance parts
of the **Atlas OS** playbook. Microsoft PC Manager is covered separately.

**Source read (2026-11-07), shallow clones, nothing run:**

- Optimizer <https://github.com/hellzerg/optimizer> (GPL-3.0), commit
  `cc88a0e79bde103dd30dcb9fa1f166ac9ad5d924` (2026-01-20), version 16.7: `Optimizer/OptimizeHelper.cs`
  (performance, network throttling, Superfetch, Gaming Mode, Xbox Live, Game Bar), `SilentOps.cs`,
  `Utilities.cs`. The README now says it is deprecated in favour of OptimizerNXT.
- OptimizerNXT <https://github.com/hellzerg/optimizerNXT>, commit
  `964d988580be2de1014afc850de8ca422eaeaf2c` (2026-01-22), version 1.0.1: `optimizerNXT/Engine.cs`,
  `yaml/enable-game-mode.yaml`, `yaml/enable-performance-tweaks.yaml`, `yaml/disable-vbs.yaml`.
- Atlas <https://github.com/Atlas-OS/Atlas> (GPL-3.0), commit
  `9e6c2066c3acdb45332e9b0527a4ef92929b7b3e` (2026-11-03): `playbook/Executables/AtlasModules/Scripts/Tweaks/performance/*`,
  `.../Tweaks/networking/atlas-network-settings.psd1`, the toggles `General/TimerResolution`,
  `PowerSaving`, `FSOGameBar`, `CpuIdle`, `Security/Mitigations`, the power-plan code
  `Scripts/Modules/Atlas.Hardware/Domain/Power.ps1`, and `docs/power-saving-policy.md`.

"Verified" below means read in that source; "claimed" means from the project's or vendor's own
docs or web pages, not checked.

## Short version

- **Nothing new worth building from Optimizer or Atlas.** Their gaming-relevant entries are the
  usual registry set (MMCSS, `NetworkThrottlingIndex`, Game Bar/DVR off, HAGS on, Superfetch off,
  global timer resolution, idle off, mitigations off), all permanent, most placebo on Windows 11,
  and the ones that do something add heat or remove security. gamebooster already covers the
  useful part per session (power plan, Game Mode, priority, `Win32PrioritySeparation`, a Game DVR
  warning in `status`).
- **Optimizer's undo is "write a fixed value", sometimes not the default** (verified): Gaming Mode
  off forces HAGS *off* (`HwSchMode = 1`) whatever you had; Game Bar on writes a Game DVR *policy*
  (`AllowGameDVR = 1`) that did not exist before; network throttling on creates a QoS policy
  (`NonBestEffortLimit = 80`) and deletes two DNS cache values that "off" never wrote.
  **OptimizerNXT has no undo at all**: its YAML jobs only apply (no revert/undo code in the
  engine at this commit); jobs are signed `.yaml.sig` files.
- **Atlas is the most careful tweak set read so far** (verified): data-only tweak files, a health
  check that reads values back, a power-plan toggle that records and restores the previous plan
  and never runs `powercfg /restoredefaultschemes`, and honest "not a performance guarantee" notes.
  Its CPU Idle toggle refuses to disable idle on SMT CPUs (the i9-9880H has SMT). Its own
  `Win32PrioritySeparation` entry pins `0x26` (= Windows' default behaviour) and warns against
  "server-style" values; gamebooster writes `0x2A` while boosted (see the table).
- **One small new idea: a Fault Tolerant Heap check.** Atlas turns FTH off globally because FTH
  silently adds heap mitigations to an exe that has crashed several times, at a performance cost.
  A read-only `status` line saying whether FTH has flagged a configured game is cheap and fits
  the "status explains, changes nothing" pattern.
- **Of the closed-source 2024–2026 tools, only Lossless Scaling claims something that could matter
  on a GPU-bound machine** (frame generation and scaling outside the game). The rest are network
  relays (LagoFast, GearUP), renamed or sibling products of tools already read (Game Fire =
  Smart PC Utilities' PC Game Booster; CCleaner Performance Optimizer = Gen Digital's AVG TuneUp
  engine; Auslogics BoostSpeed = Outbyte's maker), or CPU power-plan front ends (QuickCPU).

## Shortlist of tools not yet researched

Ranked by relevance to the target Mac (GPU held to 30–35 W under Apple's shared CPU+GPU budget,
BF6 with Javelin kernel anti-cheat, AMD tuning hidden by the driver). All from vendor pages,
store listings and reviews (claimed), not read.

| # | Tool | URL | Source | Maintained? | What it claims | Relevance here |
|---|---|---|---|---|---|---|
| 1 | Lossless Scaling (THS) | <https://store.steampowered.com/app/993090/> | Closed (Steam, ~$7) | Yes, frequent 2025–26 updates (LSFG 3.x) | Frame generation (2×–20×) and spatial upscaling applied to any windowed game, outside the engine | **High but uncertain.** The only claim aimed at a GPU-bound PC. Costs GPU time itself (on a 30–35 W 5500M that can eat the gain), adds input latency, and needs the game in a window. No EA statement on Javelin (an EA forum thread asks and gets no staff answer). Not adoptable by gamebooster (it is a capture-and-present app); a docs question. |
| 2 | LagoFast | <https://www.lagofast.com/> | Closed | Yes | Relay "ping reducer" plus an "FPS boost" / system optimiser mode | **Low-medium.** The relay part is ExitLag again (helps only on a bad ISP route). The FPS mode is the unknown: probably SGB-style service/process tweaks, but unverified. |
| 3 | QuickCPU (CoderBag) | <https://coderbag.com/product/quickcpu> | Closed, free | Yes (2024–25 releases) | Front end for core parking, frequency scaling, turbo boost, C-states and power-plan values | **Medium.** On a shared CPU+GPU power budget, *less* CPU power can mean more GPU watts. gamebooster already has `max_processor_state` / `max_cpu_mhz`; QuickCPU may expose other plan settings worth knowing (e.g. turbo boost mode). |
| 4 | GearUP Booster | <https://www.gearupbooster.com/> | Closed | Yes | Patented multi-path relay routing, 1000+ games | Low: network only, same class as ExitLag. |
| 5 | MSI Afterburner + RivaTuner Statistics Server | <https://www.guru3d.com/download/rtss-rivatuner-statistics-server-download/> | Closed, free | Yes | Overlay, frame limiter, GPU tuning | Low-medium: the frame limiter is the docs advice we already give (cap a bit below what the hot Mac sustains); BF6 has its own limiter. GPU tuning is hidden on this card. Hooks into the game (anti-cheat question). |
| 6 | CCleaner Performance Optimizer | <https://www.ccleaner.com/> | Closed | Yes | "Sleep mode" for background apps | Low: Gen Digital, same idea as AVG TuneUp's Sleep Mode (already read: persistent startup disabling). |
| 7 | Auslogics BoostSpeed 14 | <https://www.auslogics.com/en/software/boost-speed/> | Closed | Yes | Real-time CPU/RAM optimisation, "speed up internet" | Low: same maker as Outbyte PC Repair (already read). |
| 8 | Win11Debloat (Raphire) | <https://github.com/Raphire/Win11Debloat> | Open (PowerShell) | Yes | Debloat, telemetry off, UI clean-up, `.reg` undo files | Low: no gaming tweaks beyond Game Bar/DVR off (claimed). |
| 9 | Sophia Script for Windows | <https://github.com/farag2/Sophia-Script-for-Windows> | Open (PowerShell) | Yes | 150+ documented-API settings functions | Low: has HAGS and Game DVR functions (claimed); both already reported by `status`. |
| 10 | ReviOS playbook / FR33THY "Ultimate" | <https://github.com/meetrevision/playbook>, <https://github.com/FR33THYFR33THY/Ultimate> | Open | Yes | Atlas-like playbook / aggressive optimisation scripts | Low: same family as Atlas and PC-Tuning; expect no new session-scoped items. |

Not listed: Game Fire (now Smart PC Utilities' "PC Game Booster 8", already in
[smartpcutilities-gb](../smartpcutilities-gb/README.md)); W11Boost (felikcat, Rust) was in search
results but its GitHub repository returned "not found" on 2026-11-07; AMD Software's own HYPR-RX
/ Chill / RSR is the driver, covered by the SPPT/Overdrive investigation.

## Optimizer (hellzerg) and OptimizerNXT, per tweak

All verified from source. "Permanent" means it stays until the matching "off" button.
OptimizerNXT's `enable-game-mode.yaml` and `enable-performance-tweaks.yaml` carry the same
values as Optimizer's methods below, without any revert path.

| Tweak (Optimizer name) | What it changes | Undo | Verdict | gamebooster |
|---|---|---|---|---|
| Optimize Performance: MMCSS profile | `SystemResponsiveness = 1`, `NoLazyMode = 1`, `AlwaysOn = 1`; `Tasks\Games` GPU Priority 8, Priority 6, Scheduling High, SFIO High; `Tasks\Low Latency` values | Writes `SystemResponsiveness = 14` (not the default 20) and deletes the rest | **Placebo.** Microsoft's MMCSS docs (as Atlas quotes them) treat values below 10 as 20, so 1 is at best 10; `NoLazyMode`/`AlwaysOn` are not documented; the `Games` task only applies to threads that register with MMCSS as "Games", which BF6 is not known to do. | Not done (rejected in Hone, BoosterX, eclean research). |
| Optimize Performance: desktop and shutdown | `AutoEndTasks`, `HungAppTimeout 1000`, `WaitToKillAppTimeout 2000`, `LowLevelHooksTimeout 1000`, `WaitToKillServiceTimeout 2000`, Explorer link-resolve policies, smaller crash dump | Deletes values / fixed values | Placebo for games; the hook timeout can make input hooks (overlays, macro tools) drop out sooner. | Not done. |
| Optimize Performance: telemetry services | Stops and **disables** `DiagTrack`, `diagsvc`, `diagnosticshub.standardcollector.service`, `dmwappushservice`; RemoteRegistry disabled | Sets three back to Automatic and starts them; `diagsvc` and RemoteRegistry stay disabled | Small effect, but changes start types (against our rule) and the undo is incomplete. | Services are stopped per session from an allow-list; `dmwappushservice` already suggested by the eclean research. |
| Disable Network Throttling | `NetworkThrottlingIndex = 0xFFFFFFFF`; QoS policy `NonBestEffortLimit = 0` | Creates `NonBestEffortLimit = 80` policy, deletes `NetworkThrottlingIndex` and two `Dnscache` values it never set | **Placebo** (the "20 % reserved bandwidth" is a myth; the index throttles only non-multimedia traffic during MMCSS playback). The undo leaves a policy that was not there. | Not done. |
| Disable Superfetch | Stops SysMain, sets it Disabled, `EnablePrefetcher = 0`, `EnableSuperfetch = 0` | Re-enables | Placebo to slightly harmful on an SSD with 16–32 GB (slower app launches). Permanent. | `SysMain` can be stopped per session via `services = [...]`. |
| Enable Gaming Mode | `HwSchMode = 2` (HAGS on), `AllowAutoGameMode = 1`, `AutoGameModeEnabled = 1`, `GameDVR_FSEBehaviorMode = 2` | Writes `HwSchMode = 1` (HAGS **off**), Game Mode **off**, `FSEBehaviorMode = 0` | Game Mode part useful (default on anyway); HAGS needs a reboot and driver support (whether the Boot Camp 5500M driver offers it is unchecked; `status` shows it); the undo forces settings off rather than restoring. **Risky undo.** | Game Mode per session with the old value restored; HAGS read-only in `status`. |
| Disable Xbox Live | Stops and disables five Xbox services, disables XblGameSave tasks | Sets them Automatic (default is Manual) and re-enables tasks | Placebo (manual-start services that rarely run); breaks Game Pass/Xbox sign-in. | Not done; possible per session via `services`. |
| Disable Game Bar | `AppCaptureEnabled`, `AudioCaptureEnabled`, `CursorCaptureEnabled = 0`, `GameDVR_Enabled = 0`, `AllowGameDVR = 0` policy, startup panel off | Writes all to 1, including a new `AllowGameDVR = 1` policy | Useful only if background recording was on (GPU encoder plus capture cost). Permanent; undo adds a policy. | `status` warns when background recording is on and names the Settings switch. |
| Disable VBS (NXT `disable-vbs.yaml`) | `DeviceGuard\EnableVirtualizationBasedSecurity = 0` | None in NXT | **Risky** (security off; reboot). Matters more in CPU-bound games than on this GPU-bound Mac. | Not done; never. |
| svchost split threshold | `SvcHostSplitThresholdInKB` = RAM | Fixed value 380000 | Placebo for frames; reboot. | Not done. |

## Atlas OS playbook, gaming and performance parts

All verified from source unless marked. Atlas applies these once at install (playbook) or via
desktop "toggle" scripts; nothing is per game session.

| Tweak / toggle | What it changes | Undo | Verdict | gamebooster |
|---|---|---|---|---|
| Configure MMCSS | `SystemResponsiveness = 10` (comment: 10 is the lowest honoured value) | Not in the tweak (install-time) | Placebo for BF6 (see Optimizer row); harmless. | Not done. |
| Prioritize foreground applications | `Win32PrioritySeparation = 0x26` (short, variable, 3× foreground), documented as "changes nothing on a stock install", with a warning not to use `0x28`/`0x18`-style values (claimed "server semantics", no source given) | Install-time | Normaliser, not a boost. A counterpoint to our `0x2A`: fixed quantum (`0x08`) turns off the foreground *quantum* stretch that variable mode gives; the priority boost on wake stays. Neither choice has frame-time data. | We write `0x2A` per session (`priority_separation`, default on, try by feel). See adoption item 2. |
| Disable Fault Tolerant Heap | `HKLM\SOFTWARE\Microsoft\FTH\Enabled = 0`, and runs FTH's own reset entry point to clear what it has already applied | Install-time; a Mitigations toggle can re-enable | Useful in a narrow case: FTH applies heap mitigations to an exe after repeated crashes (Microsoft docs, claimed), with a performance cost. Turning it off globally is a permanent safety-net removal. | Not done (global off). Read-only check done: adoption item 1. |
| Disable Game Bar / FSO and Game Bar toggle | Game DVR/Game Bar values and policies off, Game Bar presence writer off, Game Bar package removed; FSO off machine-wide via `__COMPAT_LAYER = ~ DISABLEDXMAXIMIZEDWINDOWEDMODE` in the system environment, plus `GameDVR_FSEBehavior` values | Toggle back writes defaults and deletes the policy key | Game DVR part: as Optimizer. The global compat-layer string applies to every process and is **risky** (it overrides per-app compatibility layers); FSO off only matters for DX9–11 exclusive fullscreen, not DX12 BF6. | `status` warns on background recording; FSO not done. |
| Disable Background Apps | Per-user `GlobalUserDisabled = 1`, `BackgroundAppGlobalToggle = 0`; deliberately not the policy | Install-time; per app in Settings | Small; Store apps only. | Already an adoption item ("try background Store apps off", Advanced SystemCare research). |
| Disable Service Host splitting | Raises the split threshold (keeps Xbox services split) | Install-time; reboot | Placebo for frames; Atlas itself notes the isolation trade-off. | Not done. |
| Network settings | Sets `AutoDisableGigabit`, `ApCompatMode`, `SipsEnabled`, `ReduceSpeedOnPowerDown`, `DMACoalescing` to 0 on PCI adapters that expose them | Atlas notes it has no exact per-adapter rollback | Placebo here: the Mac's Wi-Fi does not expose these Ethernet keywords; Atlas's own docs say no exact rollback. | Not done. |
| Power-saving toggle (Atlas plan) | Copy of **Balanced** (fixed GUID), AC only: NVMe NOPPME off, *Allow Throttle States* off, display never off, *Processor performance time check interval* 200 ms; records and restores the previous plan, never resets all plans | Yes, exact (plan restored, Atlas plan deleted) | Placebo for BF6 on this CPU: with Speed Shift (HWP) the CPU picks P-states itself, so the check interval and throttle states hardly matter; Atlas's doc says as much ("not a performance or DPC-latency guarantee"). | Our High-performance-based plan per session already does the useful part, the same record-and-restore way. |
| Timer Resolution toggle | `kernel\GlobalTimerResolutionRequests = 1` and a logon task that holds 0.506 ms system-wide (bundled `SetTimerResolution.exe`, source shipped as a zip) | Toggle deletes both | **Harmful here**: more wake-ups and power on a machine whose GPU shares one power budget with the CPU; already rejected (ISLC/Hone research, DESIGN "No timer-resolution flag"). Atlas's own warning says the benefit depends on the workload. | Not done; never. |
| CPU Idle toggle | Plan setting *processor idle disable* = 1 | Toggle back to 0; refused on SMT CPUs | **Harmful** on a heat-limited laptop (all cores at full clock power); refused on this i9 anyway. | Not done; never. |
| Mitigations toggle | Spectre/Meltdown `FeatureSettingsOverride = 3`, SEHOP off, `ProtectionMode = 0`, CFG/DEP masks | Toggle to Windows default | **Risky** (security off, reboot). Gains are CPU-side; the Mac is GPU-bound. | Not done; never. |
| Automatic maintenance / search indexing | Maintenance may not wake the PC; indexing respects power mode | Install-time | Harmless; nothing during play. | `WSearch` per session via `services`. |

## On the target Mac

- Do not run Optimizer's or OptimizerNXT's "Gaming Mode" / "Optimize Performance" on the Mac:
  they add nothing gamebooster lacks, and Optimizer's undo turns HAGS and Game Mode off instead of
  restoring them; NXT has no undo.
- Atlas is an OS replacement playbook, not something to run on the existing Boot Camp install;
  its value here is as a well-documented second opinion (its power-plan and FTH notes).
- If BF6 has crashed several times and then feels worse than before, run `gamebooster status`: it says whether FTH has flagged it (see
  adoption item 1). Clearing it is still a manual step (the warning gives it).

## Adoption items for gamebooster-rs

Only items that `restore` can undo (or that change nothing), need no driver, and could matter on
the GPU-bound Mac.

1. **Done (`status`, read-only): report Fault Tolerant Heap for configured games.** `status`
   lists the value names under `HKLM\SOFTWARE\Microsoft\FTH\State` (one per flagged exe, named
   by full path per Microsoft's docs), matches their file names against the `[[game]]` exes, and
   prints `Fault Tolerant Heap: no configured game flagged`, `Fault Tolerant Heap: off`
   (`Enabled = 0`), or a warning with the fix (delete the value, or `rundll32.exe
   fthsvc.dll,FthSysprepSpecialize`, Microsoft's documented reset that clears every FTH entry).
   Changes nothing. On the target Mac the key exists with `Enabled = 1` and an empty `State`; a
   populated one hasn't been seen yet. See `fth_flagged` in `src/main.rs`.
2. **Docs only: note Atlas's counterpoint on `Win32PrioritySeparation`.** In
   [priority](../tweaks/priority.md#win32priorityseparation), one line: Atlas pins `0x26` (the
   default behaviour) and advises against fixed-quantum values; `0x2A` drops the variable-quantum
   foreground stretch. Keep `priority_separation` as try-by-feel; no code change.
3. **Docs only (later, after a research pass): Lossless Scaling on the 5500M.** If it turns out
   not to trip Javelin, a short note on when frame generation can help a GPU-bound Mac (a frame cap
   at a stable base rate first) and its latency and GPU cost. Not a boost step.
4. **Skip: everything else above.** MMCSS, network throttling, Superfetch off, Xbox services off,
   VBS/mitigations off, timer resolution, idle off, FSO compat layer, NIC power keywords, svchost
   split: placebo, permanent, reboot-only, security-reducing, or already covered.
