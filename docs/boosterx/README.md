# BoosterX: what it really is

BoosterX is a free-plus-Pro Windows "optimizer" from a solo Russian developer
([boosterx.org](https://boosterx.org), wiki at [wiki.boosterx.org](https://wiki.boosterx.org)). It is
a large settings manager rather than a one-click booster. It has hundreds of registry, service,
task, privacy and debloat switches, grouped into quick methods (*Optimal*, *Advanced*, *Hardcore*)
and display levels (*Safe*, *Advanced*, *Expert*). Recovery profiles are taken before applying.
Around it sit tools: **ProcessX** (per-exe priority/affinity profiles), **GameModeX** (the older
in-game session manager), the **IFEO Manager**, the **MSI Mode Tool**, *Interrupt Routing Lock*,
the **PowerX** power plans, an AMD PBO tuner, a hardware monitor, network tests, a "PC latency" test
and an AI helper ("TweaX"). Pro is a per-PC key bound to a hardware ID.

The homepage advertises **"6.6× less CPU load"**, **"5.8× fewer context switches"** and **"30 %
lower share of frame-time differences in CS2"**, linking to the developer's own write-up
([BoosterX: Windows background activity and FPS in CS2](https://wiki.boosterx.org/en/windows-research/services/boosterx-windows-cs2/)).

Version **2.4.2.4**, the official `https://dl.boosterx.org/BoosterX.exe` (the homepage's download
link; no account, no payment), downloaded 2026-11-06, `Last-Modified` 2026-10-25, SHA-256
`c1c17604c0331df5fc3a9fc5220b2db2bad11d1ff25f31c5faf7e2240808b0c2`, 51,955,096 bytes. It is not an
installer: the download *is* the program, one .NET Framework 4.8 WPF executable (x86/AnyCPU),
Authenticode-signed with a GlobalSign code-signing certificate issued to the developer's Russian
sole-proprietor business (`pro@boosterx.ru`). **Nothing was run.**

## How it's built, and why this doc leans on the vendor wiki

The executable is protected with **Eazfuscator.NET**. Type and method names are hashed (`#=q…`),
the user-string heap holds only a handful of plain strings (every registry path, value and URL is
encrypted), most P/Invoke declarations are hidden, and all 20 managed resources are encrypted
blobs with entropy 8.0. One of them is 23 MB, presumably the embedded dependency assemblies
(LibreHardwareMonitorLib, NvNative, Microsoft.Dism, TaskScheduler, LiveCharts, SkiaSharp and so
on are referenced but not shipped beside it). The tweak catalogue could therefore **not** be read
out of the binary without writing a deobfuscator, and that was not done.

What the binary does show in the clear: the assembly references, the WPF property names (kept for
data binding) and a few namespaces. Those confirm the feature set and how the scheduling tools are
modelled. ProcessX/IFEO have `CpuPriority`, `IoPriority`, `PagePriority`, `AffinityPreset`,
`AffinityCores` and `AffinityDelaySeconds`. The devices page has `SupportsMsi`, `SupportsMsiX` and
`MsiSupportedDevices`. There are also `HWID`, `LicenseStatus`, `Telemetry*` and `Upload*` members,
a `RemoteRepairContract` namespace (server-delivered repair rules), an `AmdPboTuner` and a
`MemoryMsrReader`, and an embedded **PawnIO** notice. PawnIO is the signed kernel driver that
current LibreHardwareMonitor uses for MSR/SMU/sensor access, so the sensor and PBO features load a
**kernel driver**. The exact registry values below come from the developer's own wiki, which
lists hive, path, name, type and value per tweak and is unusually candid about what doesn't help.
Where the wiki gives no value, the table says so. Only metadata was dumped (names, resource sizes),
no IL was decompiled, and those dumps were deleted afterwards (see
[decomp/README.md](../../decomp/README.md#policy)).

No sign of malware: the network use is licensing, telemetry with an upload consent, cloud backups,
the AI helper, the network tests and the repair-rule feed. That is still a lot of phone-home for a
settings tool, and with the strings encrypted it cannot be audited statically.

## What the CS2 claim actually is

It is **not** a scheduling, affinity or MSI result. The numbers come from one PC (Ryzen 7 7800X3D,
RTX 5070, Windows 11 26H2), a Debug build of 2.4.2.4, and four stages applied cumulatively on one
install: Clean, then Safe *Advanced*, then *Advanced* Advanced, then Expert *Hardcore* (which also
switches to PowerX v2.1). Each stage was measured with WPR (a 5-minute window after sign-in, then a
5-minute **idle desktop** window) and two 105-second CapFrameX runs of a CS2 benchmark map at
about 900 FPS.

- **6.6× less CPU load** is the idle-desktop CPU ratio (1.238 % → 0.183 %; the article now says
  5.3× after sign-in and 6.8× at idle). **5.8× fewer context switches** is idle-desktop context
  switches per second (1,945 → 337). Both describe Windows doing less **while nothing runs**. The
  companion VM study ([quiet idle](https://wiki.boosterx.org/en/windows-research/services/quiet-idle/))
  attributes this to the profile disabling **about 50 services and 58 scheduled tasks** and, in
  part, **Microsoft Defender components**. Processes at idle dropped 153 → 64.
- In game, average FPS rose **2.2–3.0 %** and P1 by up to **5.2 %**. The "30 %" is the share of
  adjacent-frame differences of 2 ms or more (5.88 % → 4.09 % of frame pairs), at ~1.1 ms frame
  times. Two runs per stage, no restore between stages and no per-tweak attribution, as the
  article itself says.

So the claim is honest about its scope, but it measures background debloat (including turning off
antivirus), not anything a game-session booster can do. It says nothing about the Expert scheduling
tools.

## Catalogue: what matters for gaming

Exact changes are from the vendor wiki unless marked. "Driver" means a kernel driver is needed.

| Tweak (BoosterX name) | Exact change | Revert | Driver? | Risk (heat, anti-cheat) |
|---|---|---|---|---|
| ProcessX priorities (Free) | Per-exe CPU, I/O and memory-page priority applied "by Windows mechanisms, without a running monitor": IFEO `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<exe>\PerfOptions` `CpuPriorityClass`, `IoPriority`, `PagePriority` (inferred from the wiki plus the bound property names; the values are encrypted) | Delete the values/entry | no | Low. Same mechanism as our `game_priority_method = "registry"`. The wiki itself advises leaving games at Default |
| ProcessX affinity (Free; "Actions" Pro) | Presets: all logical CPUs, one thread per physical core, skip CPU 0, P-only/E-only, manual. Applied after an `AffinityDelaySeconds` delay, so something must watch for the process (mechanism not visible) | Exit restores | no | Moderate. Setting affinity on a protected game is the kind of handle access anti-cheats block. The wiki: "without measurements, leave All cores" |
| ProcessX launch/exit actions (Pro) | Start/close/suspend/resume other processes, efficiency mode, priority/affinity on others, power plan switch, restore on exit | "Restore saved state" on exit | no | Suspending processes is risky. The wiki warns against suspending anti-cheat or security software |
| IFEO Manager (Pro, Tweaker Mode) | Edits `Debugger` (with a "Block launch" preset), `GlobalFlag`, `MitigationOptions`, `SilentProcessExit` monitoring/dumps, priority and affinity under the exe's IFEO key | "Delete configuration" per exe, or rollback | no | High for misuse: a debugger or mitigation entry on a game exe can block it or trip anti-cheat |
| MSI Mode Tool / Devices (Free toggle) | Per device: line-based vs MSI vs MSI-X, plus interrupt affinity and priority in the advanced view. Standard keys under `HKLM\SYSTEM\CurrentControlSet\Enum\PCI\<dev>\<inst>\Device Parameters\Interrupt Management\` (`MessageSignaledInterruptProperties\MSISupported`, `Affinity Policy`); exact values not readable | "Rollback" from a saved copy, reboot | no | Moderate: a wrong mode can break audio, network or boot. Modern drivers already pick MSI. The wiki says don't start with GPU, NIC or system drive |
| Interrupt Routing Lock (Pro, Win 11) | Stops Windows rebalancing device interrupts between cores (fixes the current spread). Key not given in the wiki | Toggle off, reboot | no | Moderate. The wiki: "leave default" |
| Unneeded devices (Free) | Disables device groups/devices (`Default` re-enables all) | Default button | no | Moderate |
| PowerX v2.1 (Expert) | Imported plan "BoosterX Performance": AC minimum processor state 100 %, core parking off (min cores 100 %), aggressive turbo, prefer P-cores, USB selective suspend off, PCIe link power saving off. DC values untouched | Pick the old plan; PowerX stays installed | no | **High heat**: the wiki itself says not for laptops |
| PowerX v3.1 (Expert; Intel non-K only, hidden on AMD/K) | Turbo kept, low AC minimum state, some idle parking, long loads steered to P-cores | As above | no | Low heat. The same idea as our `min_processor_state` |
| Active window priority (Free) | `HKLM\SYSTEM\CurrentControlSet\Control\PriorityControl` `Win32PrioritySeparation`: Windows default `0x02` (recommended) or `0x1A` (long fixed quanta, max foreground boost) | Set back | no | Low. Their own test found no general winner; default had slightly lower click-to-photon latency |
| Reduce Raw Input background event rate (Free, Win 11) | `HKCU\Control Panel\Mouse` `RawMouseThrottleEnabled` = 1 (REG_DWORD), `RawMouseThrottleDuration` = 20 (ms) | Delete both values | no | Low. Background apps get mouse events at ~50 Hz instead of Windows' ~125 Hz; the foreground game still gets full rate. Breaks background mouse tools/overlays that need full rate |
| CPU reserve (moved to "Legacy") | `...\Multimedia\SystemProfile` `SystemResponsiveness`, recommends **20** (the default); 10 showed nothing, 100 disables MMCSS | Set 20 | no | None. The "Legacy" page has a **Default** button that undoes old-guide tweaks (`NetworkThrottlingIndex`, `LargeSystemCache`, mouse/keyboard data queue size, power throttling, TSX, WPBT) |
| Service grouping | Selectively regroups verified services into shared `svchost.exe` processes (fewer processes, less isolation); reboot | Toggle, reboot | no | Low/moderate |
| Services / tasks / Defender (quick methods, Hardcore) | ~50 services and 58 scheduled tasks disabled, plus Defender components in Hardcore (per the quiet-idle study; lists encrypted) | Recovery profile | no | Security trade-off. Start-type changes outlive the session |
| GameModeX (legacy, Win 10) | In-game session: stop Explorer/DWM-related components, throttle detected background raw-input listeners, Snap Tap, post-game power plan | Best-effort on normal game exit | no | High on Win 11 (the wiki says don't). Snap Tap is banned in CS2 |
| Sensors, AMD PBO tuner, MSR reader | LibreHardwareMonitor through PawnIO | n/a | **yes (PawnIO)** | A kernel driver; anti-cheats vary on it |

Most of the rest (privacy, telemetry, customization, debloat, Game Bar, Game Mode, HAGS,
fullscreen optimizations, mouse acceleration, Nagle-style network items) overlaps entries already
judged in [Tier1Suite](../tier1suite/README.md#adoption-items-for-gamebooster-rs) and
[docs/tweaks/](../tweaks/README.md), and is not repeated here. The wiki's own research pages
([placebo knobs](https://wiki.boosterx.org/en/windows-research/registry/placebo-knobs/), scheduler
timers, MMCSS, GPU presentation, USB/NVMe power) all end in "leave the default", which matches our
findings.

## On a 2019 MacBook Pro under Boot Camp playing BF6

The headline gains come from debloating Windows at idle, which a session booster doesn't do and
`restore` can't cleanly undo (start types, scheduled tasks, Defender). The Expert scheduling tools
are honest power tools, and their own docs say to leave the defaults alone. PowerX v2.1 is the
classic heat mistake on this CPU. The PawnIO driver, the hardware-ID licence and the encrypted
phone-home features are more than this machine needs.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **Background raw-input throttle while boosted**: `HKCU\Control Panel\Mouse`
   `RawMouseThrottleEnabled` = 1, `RawMouseThrottleDuration` = 20, old values or absence recorded
   and restored. **Try by feel**, Windows 11 24H2+ only. HKCU, no admin, no driver, invisible to
   anti-cheat, and the game in front keeps full mouse rate. It only matters with a high-polling mouse
   and background listeners (Discord, the EA app overlay, browser). Check first whether it applies
   without sign-out.
2. **IFEO `IoPriority` / `PagePriority` beside our `CpuPriorityClass`**: **skip**. Normal I/O and
   page priority are already the top values a user process gets (High I/O is reserved), so for a
   game they are no-ops. They only help as *lowering* tools, and `lower_others` covers that.
3. **Affinity presets (one thread per core, skip CPU 0)**: **skip**. Frostbite spreads across all 16
   threads of the i9-9880H. Touching a protected game's affinity is what EA Javelin blocks. The
   vendor itself says leave All cores.
4. **MSI mode / interrupt affinity / Interrupt Routing Lock**: **skip**. Reboot-only, persistent,
   not undoable by `restore`, and the Radeon and Apple Boot Camp drivers already choose their
   interrupt mode.
5. **PowerX v2.1**: **skip**. 100 % min state and no parking add idle heat on a heat-limited CPU.
   **PowerX v3.1**: already covered by `min_processor_state` / `max_cpu_mhz`.
6. **`Win32PrioritySeparation`**: **skip** here; the vendor's own test found no general winner.
   Judged in the [GameShift](../gameshift/README.md) and [pc-tuning](../pc-tuning/README.md)
   docs.
7. **Mass service/task disabling, Defender off (the CS2 "6.6×")**: **skip**. It is idle-desktop
   debloat, worth ~2–3 % average FPS on a 900 FPS CPU-bound CS2 run, with permanent start-type
   changes and a security cost. Our session-only `services` stop list is the reversible version.
8. **GameModeX shell/DWM stop, Snap Tap, `SystemResponsiveness`, service grouping**: **skip**.
   Harmful on Windows 11, banned in some games, default already, or needs a reboot.
