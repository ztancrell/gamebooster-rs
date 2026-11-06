# GameShift: what it really does

GameShift is an open-source (GPL-3.0) Windows game optimizer in C#/.NET 9 by lhceist41
(<https://github.com/lhceist41/gameshift>). It detects a game starting, applies a list of
"session optimizations", and reverts them in reverse order when the game exits. Every change is
recorded first in an atomic journal, and a separate watchdog process replays the journal after a
crash. It also has an always-on "Background Mode", one-time "persistent tweaks", and a DPC/kernel
tuning page.

**Source read:** commit `ed7d72a7be2248d2ce1de673551ee9968707fefc` (2026-10-30, release 3.9.0),
shallow clone, read 2026-11-06. Only the source was read; nothing was built or run. Quotes are
from its README and code comments.

## Short version

- The architecture is good and close to ours: snapshot the old value before writing, revert in
  LIFO order, crash recovery from a journal, IFEO `CpuPriorityClass` fallback when an anti-cheat
  blocks `SetPriorityClass`. It confirms our design rather than adding to it.
- The tweak list is the usual folklore set, pushed hard: 0.5 ms timer resolution, C-states limited
  to C1, Ultimate Performance plan, Nagle off, NIC interrupt moderation off, MMCSS "Games" task at
  High, AMD ULPS and deep sleep off, MPO off, standby-list purging.
- Several session tweaks cannot work per session: driver class keys (AMD `EnableUlps`,
  `FlipQueueSize`, TDR), NIC advanced properties and MPO are read at driver/adapter/DWM start, and
  GameShift neither restarts the driver nor reboots. Its own log says "Some GPU driver settings may
  require a driver restart to take full effect".
- It contradicts itself in places: the session MMCSS step writes `NetworkThrottlingIndex =
  0xFFFFFFFF`, `SystemResponsiveness = 10`; the persistent MMCSS tweak writes 20 and 0. The
  session priority step writes `Win32PrioritySeparation = 0x2A`; the persistent one writes `0x28`.
- Its built-in game tips claim "0.5ms timer resolution ... produces 20-30% improvement" for CS2
  with no source. Elsewhere its own tips admit "Nagle's algorithm tweaks (TcpNoDelay,
  TcpAckFrequency) are PLACEBO for Fortnite -- gameplay uses UDP".
- No Battlefield / EA Javelin profile. `EAAntiCheat.GameService` appears only in ProBalance's
  do-not-touch list.
- For a heat-limited i7-9750H laptop, most of the "competitive" items (C-state limit, idle off,
  timer 0.5 ms, ULPS off, deep sleep off, interrupt moderation off) add heat and power, which is the
  wrong direction on the target Mac.

## Session optimizations, item by item

"Reversible" means GameShift records and puts back the old value. "Driver-free" means no kernel
driver of its own (all items are; it loads none for these). Anti-cheat and heat columns are for
BF6 (EA Javelin) on the target Mac.

| Item | Exact change | Reversible? | Anti-cheat risk | Heat risk | Evidence |
|---|---|---|---|---|---|
| Process priority | Game to High via `SetPriorityClass`; falls back to IFEO `PerfOptions\CpuPriorityClass` when blocked. Sets `HKLM\SYSTEM\CurrentControlSet\Control\PriorityControl\Win32PrioritySeparation = 0x2A` ("fixed quantum (0x20) \| short length (0x08) \| max foreground boost (0x02)"); deletes it on revert if it was absent. | Yes | IFEO: low (same as ours). | None | Priority: real when CPU-bound. 0x2A: no measurement; PC-Tuning shows the kernel still uses the low bits as a foreground wake boost with a fixed quantum. Takes effect live. |
| Service suppression | Stops 20+ services in three tiers (telemetry, `WSearch`, Windows Update, Delivery Optimization ...). | Yes (restarted) | None | Slightly less | Same as our allow-list. |
| Scheduled task suppression | Disables heavy scheduled tasks (defrag, telemetry, update scans) for the session, optional Defender scan suppression. Not traced in detail. | Yes | None | Less | Plausible against a scan starting mid-match; no measurement. |
| Memory | Threshold-based standby list purge; `EmptyWorkingSet` on background processes; a hard minimum working set on the game (`SetProcessWorkingSetSizeEx` with a hard-min flag). | Purge/trim: nothing to undo. Hard min: released. | Hard min opens the game process with quota rights: likely blocked by Javelin like `SetPriorityClass`. | None | Purge/trim: placebo or harmful on Windows 10/11 (see [ISLC](../islc/README.md)). |
| Timer resolution | `NtSetTimerResolution(5000)` (0.5 ms, Competitive) or 10000 (1 ms), plus `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel\GlobalTimerResolutionRequests = 1` so it applies system-wide on Windows 11. | Yes (request released, value restored) | None | More: fewer deep idle periods on every core | PC-Tuning: the registry value exists only on Windows 11 / Server 2022+ and brings back the global overhead; prefer per-process. Placebo for games that already request 1 ms. |
| Power plan | Activates Ultimate Performance (`e9a42b02-d5df-448d-aa00-03f14749eb61`, created if missing), then overrides AC and DC: EPP `36687f9e-...6863` (and `...6864`) = 0; processor boost policy `45bcc044-d885-43e2-8605-ee0ec6e96b59` = 100; USB selective suspend `2a737441-.../48e6b7a6-50f5-4782-a5d4-53bb8f07e226` = 0; USB 3 link power `d4e98f31-5ffe-4ce1-be31-1b38b384c009` = 0; NVMe primary idle timeout `0012ee47-.../d639518a-e56d-4345-8af2-b9f32fb26109` = 0; Wi-Fi power saving `19cbb8fa-.../12bbebe6-...` = 0. | Yes (old plan reactivated; overrides restored from AC/DC snapshot) | None | EPP 0 and boost 100: more. USB/NVMe/Wi-Fi: negligible on AC. | Plan switch: real. USB selective suspend off is the common fix for a mouse lagging after idle. |
| PCIe ASPM | `powercfg /setacvalueindex SCHEME_CURRENT 501a4d13-... ee12f906-... 0` | Yes | None | Small | We already set it. |
| Core parking | In the active plan, AC and DC: `CPMINCORES` (`0cc5b647-...`) = 100 (50 on AMD dual-CCD X3D), `CPMAXCORES` = 100, minimum processor state = 100, concurrency threshold set. Competitive: idle state max `9943e905-...` = 1 (C1 only), promote/demote 100, idle scaling 0, latency hints 100. | Yes | None | High for the C-state limit and min 100 % | Parking: High performance (which our plan copies) already keeps cores unparked. C-state limit: PC-Tuning advises keeping idle states on for most users and not with Turbo Boost. |
| Hybrid CPU scheduling | P-core soft affinity for the game, E-cores and EcoQoS for background. | Yes | Affinity on the game: same risk as priority. | None | Not applicable: the i7-9750H has no E-cores. |
| Core isolation | `ReservedCpuSets` (opt-in, reboot). | Yes, with a reboot | Low | None | PC-Tuning: soft policy, measure first. Reboot rules it out for us. |
| MPO off | `HKLM\SOFTWARE\Microsoft\Windows\Dwm\OverlayTestMode = 5`; on 24H2+ also `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\DisableOverlays = 1`. Competitive only. | Yes (value restored or deleted) | None | None | Fixes flicker/stutter on some multi-monitor and driver combinations. PC-Tuning: "run the command below ... and reboot", so a per-session toggle without a reboot likely does nothing. |
| Visual effects | `HKCU\...\Themes\Personalize\EnableTransparency = 0`, animations off via `SystemParametersInfo`. | Yes | None | None | Negligible in a fullscreen game. |
| Efficiency mode | EcoQoS on background processes, rescanned every 30 s. | Yes | None | Less | Little effect on a CPU without per-core clocks or E-cores. |
| I/O priority | `NtSetInformationProcess(ProcessIoPriority = 33)` lowers the I/O priority of background processes; original read first, PID reuse checked. | Yes | None (never the game) | None | Plausible when something hits the disk while the game streams assets; no measurement. |
| Network | Per interface under `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{id}`: `TcpAckFrequency = 1`, `TCPNoDelay = 1`. Stops `DoSvc`. NIC class keys `{4d36e972-...}\NNNN`: `*InterruptModeration`, `*LsoV2IPv4`, `*LsoV2IPv6`, `*RscIPv4`, `*RscIPv6` = `"0"`; `netsh int tcp set global rsc=disabled`. | Yes | None | Interrupt moderation off: more interrupts, more CPU wake-ups | Nagle: placebo for UDP games (GameShift's own tips say so). NIC keys need an adapter restart, which it does not do. |
| MMCSS | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`: `NetworkThrottlingIndex = 0xFFFFFFFF`, `SystemResponsiveness = 10`; `...\Tasks\Games`: `Affinity = 0`, `Background Only = "False"`, `Clock Rate = 10000`, `GPU Priority = 8`, `Priority = 6`, `Scheduling Category = "High"`, `SFIO Priority = "High"`. | Yes | None | None | Only threads that register with MMCSS as "Games" (`AvSetMmThreadCharacteristics`) are affected; most games don't. Several values are already the Windows defaults. PC-Tuning does not recommend any of it. |
| USB HID suspend | Under the HID class key `HKLM\SYSTEM\CurrentControlSet\Control\Class\{745A17A0-74D3-11D0-B6FE-00A0C90F57DA}\NNNN`, for devices that already have the value: `SelectiveSuspendEnabled = 0`, `EnhancedPowerMgmtEnabled = 0`. | Yes | None | None | Read at device start: needs a replug. The plan setting above is the live equivalent. |
| GPU driver (AMD) | In the display class key: `EnableUlps = 0`, `PP_SclkDeepSleepDisable = 1`; in its `UMD` subkey `FlipQueueSize = 31 00` (binary "1"), `ShaderCache = "1"`; ADLX Anti-Lag attempt (the 3.9.0 changelog says the ADLX library is no longer loaded). | Yes (values restored) | None | ULPS/deep sleep off: higher idle GPU power and heat | Read at driver load; no restart is done, so within a session they likely do nothing. |
| TDR | `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers`: `TdrDelay = 8`, `TdrDdiDelay = 10`. | Yes | None | None | A stability workaround for false "GPU hung" resets, not performance; read at boot. |
| ProBalance | Demotes background processes above 15 % CPU to Below normal, restores when calm. | Yes | None | None | We already have it (`lower_others_mode = "dynamic"`). |
| Competitive mode | Suspends Discord, Steam, NVIDIA overlays with `NtSuspendProcess`; kills Widgets, Edge WebView. | Suspended: resumed. Killed: no. | Suspending Steam/EA overlay processes can upset a launcher. | Less | Close-apps does this more safely. |

## Background Mode and persistent tweaks (not per session)

- **Background Mode**: always-on standby purge, timer resolution lock (0.5 ms), a "GameShift
  Performance" plan cloned from Ultimate Performance with 50+ overrides, and persistent priority
  rules. All permanent while enabled; against our "restore undoes everything".
- **Persistent tweaks** (applied once, revertible by hand): Game DVR off
  (`HKCU\...\GameDVR\AppCaptureEnabled = 0`, `HistoricalCaptureEnabled = 0`, policy
  `AllowGameDVR = 0`), HAGS toggle (`HwSchMode`), VBS/HVCI off, NTFS last-access and 8.3 off,
  `DisablePagingExecutive`, `LargeSystemCache = 0`, page file, memory compression off, large pages,
  `Win32PrioritySeparation = 0x28`, global `PowerThrottlingOff = 1`, MMCSS (20 / 0 values).
- **DPC Doctor / kernel tuning**: interrupt affinity, MSI mode, `bcdedit` `disabledynamictick`,
  `useplatformtick`, `tscsyncpolicy`, `x2apicpolicy`, `hypervisorlaunchtype`. All need a reboot.

## Anti-cheat and the target Mac

GameShift's table lists Vanguard, EAC, BattlEye, Ricochet, ACE, VAC and FACEIT, not EA Javelin.
Its approach to kernel anti-cheats is the same as ours: IFEO priority instead of touching the
process. Nothing it does is a driver, so Javelin should not mind the system-level items; the hard
minimum working set and competitive-mode suspends touch processes and are the risky ones. VBS off
is a security trade with no Javelin requirement either way (BF6 needs Secure Boot, which GameShift
does not touch).

## Adoption items for gamebooster-rs

Ranked by expected value on the target Mac. "Adopt" = cheap, reversible, no heat cost, low risk.

1. **Adopt: USB selective suspend off in our plan** (`2a737441-1930-4402-8d77-b2bebba308a3` /
   `48e6b7a6-50f5-4782-a5d4-53bb8f07e226` = 0, AC). Plan setting, gone when restore deletes our
   plan; stops an external mouse or headset waking late. Check first that High performance doesn't
   already have it at 0 on this Mac.
2. **Try-by-feel: lower background I/O priority** with `lower_others` (`NtSetInformationProcess`
   `ProcessIoPriority`, old value journalled, never the game). No heat; uncertain gain, undocumented
   API.
3. **Try-by-feel: `Win32PrioritySeparation = 0x2A` while boosted**, old value or absence recorded
   and put back. Live, global, cheap; no measurement anywhere shows a gain.
4. **Try-by-feel: EPP 0 and boost policy 100 in our plan** (`36687f9e-e3a5-4dbf-b1dc-15eb381c6863`,
   `45bcc044-d885-43e2-8605-ee0ec6e96b59`), only if `powercfg /qh` shows our copied plan has them
   lower. Heat risk on this CPU: more time at peak clocks.
5. **Try-by-feel: USB 3 link power off and NVMe idle timeout 0 in our plan.** Plan-scoped; the Mac's
   Apple SSD may ignore the NVMe setting.
6. **Skip: MPO off** as a boost step (needs a reboot to take effect). Document as a manual fix for
   flicker/stutter instead (see [WinUtil](../winutil/README.md#adoption-items-for-gamebooster-rs)).
7. **Skip: timer resolution, C-state limit, idle off, core parking, AMD ULPS / deep sleep,
   interrupt moderation off.** Placebo or more heat on a heat-limited laptop; ULPS/driver keys need
   a driver restart anyway.
8. **Skip: Nagle, `NetworkThrottlingIndex`, MMCSS Games values, NIC LSO/RSC.** BF6 plays over UDP;
   MMCSS only affects registered threads; NIC keys need an adapter restart.
9. **Skip: standby purge, working-set trim, hard minimum working set, competitive-mode suspends,
   TDR, VBS off, BCD edits.** Placebo, anti-cheat exposure, or permanent / reboot-only.

Confirmed, not new: snapshot before write, LIFO revert, crash journal, IFEO fallback, ProBalance,
EcoQoS opt-out for the game (we already do this, see [game priority](../tweaks/priority.md#power-throttling-off)).
