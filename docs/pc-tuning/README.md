# valleyofdoom PC-Tuning: what it says about the usual tweaks

PC-Tuning (<https://github.com/valleyofdoom/PC-Tuning>, by valleyofdoom) is a long written guide,
not a booster: BIOS, Windows install, drivers, interrupts, scheduling, timers. It applies nothing
by itself apart from optional helper scripts (`bin/`). It is the most cited "do it properly" guide
in the tuning community, and its rule is to measure every change: almost every section opens with
"**Do NOT** blindly follow the recommendations in this section. **Do** benchmark the specified
changes" and points to PresentMon, FrameView and xperf.

**Source read:** commit `d991648532e44fe80175c3ec4916e92fdd661250` (2026-09-29), shallow clone,
read 2026-11-06: `README.md`, `docs/research.md`, `docs/registry-opts.md`,
`docs/configure-amd.md`. Nothing was run. Quotes are attributed to the guide.

The guide is written for a desktop the reader controls end to end (often a fresh, stripped
Windows install, overclocked, with a known-good cooling setup). It says little about laptops, and
its anti-cheat notes are about Vanguard / FACEIT needing HVCI, Secure Boot or the vulnerable
driver blocklist.

## Short version

- **What it endorses** (with "benchmark first"): power saving off for real-time work (ASPM, link
  power, device power saving), MSI mode, interrupt and DPC affinity, Reserved CPU Sets, VBS/HVCI
  off (security trade), RTSS or the in-game limiter for frame pacing, registering the game in
  Game Bar's config store, QoS DSCP policies.
- **What it warns against**: disabling CPU idle states for most readers ("I would recommend keeping
  idle states enabled for the majority of readers ... (e.g. throttling, power issues)"), and
  certainly not "when relying on ... Turbo Boost"; disabling the page file ("many users have
  reported in-game stuttering"); restoring global timer resolution behaviour without measuring.
- **What it does not mention at all** (a finding in itself): `NetworkThrottlingIndex`,
  `SystemResponsiveness`, the MMCSS `Tasks\Games` values, Nagle / `TcpAckFrequency` /
  `TCPNoDelay`, core parking, AMD ULPS, TDR. The booster staples are absent from the most careful
  guide.
- **Game Mode** is neither recommended nor condemned: "Game Mode can interfere with process and
  thread priority boosts ... you can experiment with Game Mode enabled and disabled."

## Per tweak

Columns as in our other research docs: is the change reversible, is it driver-free, the anti-cheat
risk for BF6 (EA Javelin), the heat risk on the heat-limited i7-9750H, and the evidence.

### Timer resolution (`NtSetTimerResolution`, `GlobalTimerResolutionRequests`)

- **Change:** `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel\GlobalTimerResolutionRequests = 1`
  plus a process holding a high resolution (its own SetTimerResolution tool).
- **What it really does:** since Windows 10 2004 a timer request only affects the calling process.
  The guide reverse-engineered the kernel: the value "only exists in Windows Server 2022+ and
  Windows 11+". With it, one process can again raise the resolution for everyone, at the cost that
  "background processes are also serviced unnecessarily often".
- **Verdict in the guide:** "I recommend favoring the per-process (non-global) implementation" and
  RTSS for precise frame limiting; measure if you try it. Sometimes 0.507 ms is more precise than
  0.5 ms.
- Reversible: yes. Driver-free: yes. Anti-cheat: none. Heat: more wake-ups on every core.
  Evidence: Sleep() precision measurements, not frame times. Matches our [ISLC](../islc/README.md)
  finding.

### CPU idle states (C-states)

- **Change:** `powercfg /setacvalueindex scheme_current sub_processor 5d76a2ca-e8c0-402f-a133-2158492d58ad 1`
  (idle disable; `0` = default).
- **Guide:** forces C0, "at the expense of higher temperatures and power consumption", keep idle
  states on for most readers, never with Turbo Boost or with Hyper-Threading on.
- Reversible: yes (plan setting). Driver-free: yes. Anti-cheat: none. Heat: **high**, the worst
  item for the target Mac (four of its conditions are against it: laptop, Turbo on, HT on, heat
  limited). Skip.

### Power saving (ASPM, link power, device power saving)

- **Change:** BIOS options; `Get-WmiObject MSPower_DeviceEnable ... enable = $false` for every
  device ("Allow the computer to turn off this device to save power"); NIC power-saving properties
  off in Device Manager.
- **Guide:** "Power-saving has no place on a machine executing real-time tasks."
- Reversible: by hand (the WMI change re-enables on replug). Driver-free: yes. Anti-cheat: none.
  Heat: small on AC. We already turn PCIe ASPM off in our plan; the per-device WMI flag is
  permanent and system-wide, so not a boost step.

### Win32PrioritySeparation

- **Guide (research.md section 5):** decodes the bit fields with a kernel debugger. Values above
  `0x3F` are masked down, so "illogical" values from tweak lists are equivalent to small ones. With
  a fixed quantum, the low bits still matter because the kernel boosts a foreground thread's
  priority by `PsPrioritySeparation` when it finishes a wait, so "a fixed 3:1 quantum should have a
  perceivable difference compared to a fixed 1:1 quantum". No recommended value; "this table has
  changed in later versions of Windows 11 (24H2)".
- Reversible: yes. Driver-free: yes. Anti-cheat: none. Heat: none. Evidence: kernel behaviour, no
  frame-time data. Supports trying GameShift's `0x2A` (fixed, short, 3:1) by feel, nothing more.

### MPO (multiplane overlay)

- **Change:** `reg add "HKLM\SOFTWARE\Microsoft\Windows\Dwm" /v "OverlayTestMode" /t REG_DWORD /d "5" /f`, then reboot.
- **Guide:** only as a way out of being stuck in "Hardware Composed: Independent Flip" when you want
  another presentation mode; check the mode with PresentMon. Not a general performance tweak.
- Reversible: yes (delete the value, reboot). Driver-free: yes. Anti-cheat: none. Heat: none.
  Needs a reboot, so not a boost step.

### Fullscreen optimizations / presentation mode

- **Change:** "Disable fullscreen optimizations" checkbox, or
  `HKCU\SYSTEM\GameConfigStore\GameDVR_DXGIHonorFSEWindowsCompatible = 1`,
  `GameDVR_FSEBehavior = 2`, plus reboot.
- **Guide:** informative only, "not a recommendation of what presentation mode to use". Make sure
  the game is registered in `GameConfigStore` (Game Bar "Remember this is a game"), which also
  "ensures that Game Mode functions properly".
- Useful for us as a check, not a tweak: Game Mode is what we turn on.

### Game Bar / Game DVR

- **Change:** `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR\AppCaptureEnabled` and
  `HKCU\System\GameConfigStore\GameDVR_Enabled`, `0` = off, `1` = on (default).
- **Guide:** disable it when not needed, but enable it temporarily to register games.
- Reversible: yes. Driver-free: yes. Anti-cheat: none. Heat: background recording uses the GPU's
  encoder; off is cooler.

### Memory: page file, memory compression, Superfetch

- Page file: keep it on ("system managed size" on an SSD).
- Memory compression: `Disable-MMAgent -MemoryCompression`, measure first.
- SysMain off (`Services\SysMain\Start = 4`) only when no HDD is present; permanent.
- None of it is a per-session tweak; all are permanent and unmeasured for games.

### Interrupts: MSI mode, interrupt / DPC affinity, RSS, Reserved CPU Sets

- MSI mode via MSI Utility / GoInterruptPolicy (registry under the device's `Interrupt Management`
  key, reboot), interrupt affinity policies, NIC RSS base CPU, `ReservedCpuSets`.
- All need a reboot or driver restart, all need xperf measurement to be done right, and the guide
  warns of BSODs and broken keyboards on mobile devices (`msisadrv`). Skip for a boost tool.

### Network

- The guide's network section is: disable unused adapters, NetBIOS over TCP/IP off, optional DNS
  choice, QoS DSCP 46 policy for the game (with `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\QoS\Do not use NLA = "1"`),
  and "at most two RSS queues/CPUs are being utilized" during online matches.
- No Nagle, `TcpAckFrequency`, `NetworkThrottlingIndex` or interrupt moderation advice.
- DSCP only helps if the router honours it (most home routers and ISPs ignore or strip it).

### Background window message rate (Windows 11 22H2+)

- `HKCU\Control Panel\Mouse\RawMouseThrottleDuration` (default 8 ms, 3 to 20): only matters for
  raw mouse input sent to a background window. A foreground game is not affected.

### VBS / HVCI

- `...\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity\Enabled = 0`; "negatively impacts
  performance", with a 🔒 security warning. Reboot, permanent, security cost. Out of scope.

### Frame limiting

- "Ensure that the GPU isn't fully utilized as lower GPU utilization reduces system latency." A cap
  below what the machine sustains also means less heat, which on this Mac is the real limit. This
  is a setting in BF6 (or RTSS), not something a booster changes, but it is the most useful advice
  in the guide for the target Mac.

## Laptops, heat and anti-cheat

- Laptops: the guide assumes cooling headroom ("The CPU temperature should not increase to the
  point of thermal throttling because you should have already assessed that"). On the target Mac
  that assumption fails, so every "more power" item is a trade against throttling and the 800 MHz
  BD PROCHOT lock (see [prochot](../tweaks/prochot.md)).
- Anti-cheat: it warns that Process Hacker / WindowsD "interfere with anticheats", that some
  anti-cheats need HVCI, Secure Boot or the driver blocklist on, and that Windows Server NIC driver
  workarounds break anti-cheats. Nothing in our scope is affected.

## Adoption items for gamebooster-rs

1. **Adopt: Game DVR background recording check in `status`** (read
   `HKCU\System\GameConfigStore\GameDVR_Enabled` and
   `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR\HistoricalCaptureEnabled`, warn if
   background recording is on). Read-only, no heat, no anti-cheat risk; recording costs GPU time.
2. **Adopt (docs): recommend a frame cap in BF6** below what the Mac sustains, in
   [docs/macbook-pro-2019-slow-cpu.md](../macbook-pro-2019-slow-cpu.md) or the tweaks index. Less
   heat, steadier frame times, no code.
3. **Adopt (docs): note that Game Mode needs the game registered in Game Bar** ("Remember this is a
   game"), in [game mode](../tweaks/game-mode.md). No code.
4. **Try-by-feel: `Win32PrioritySeparation = 0x2A` while boosted** (see
   [GameShift](../gameshift/README.md#adoption-items-for-gamebooster-rs) item 3). The guide's
   research says the value is live and not a no-op; no gain is measured.
5. **Skip: CPU idle states off.** The guide itself advises against it with Turbo and HT on; worst
   heat item for the Mac.
6. **Skip: global timer resolution.** The guide prefers per-process; matches our ISLC finding.
7. **Skip: MSI mode, interrupt affinity, Reserved CPU Sets, MPO, FSE registry, VBS off, page file,
   SysMain.** Reboot-only or permanent; against "restore undoes everything".
8. **Skip: QoS DSCP policy.** Home routers usually ignore it; needs a Group Policy object, not a
   session toggle.
