# Hone: what it really does

Hone (hone.gg, by AuraSide Inc., distributed through Overwolf) is a popular Windows "PC optimizer" for
gamers. It offers a catalogue of about 100 one-click Windows, network, GPU and BIOS tweaks, plus a
**Game Mode** that boosts while you play. This page comes from reading the official installer and the
Hone 2.1.0 package it downloads (static reverse engineering, 2026-11-05). **Nothing was run.**
Technical notes: [../../decomp/hone/README.md](../../decomp/hone/README.md).

## Short version

- **What you install:** an Overwolf web installer (signed by Overwolf) downloads Hone 2.1.0, an
  Electron app (Overwolf's "ow-electron") with about a dozen helper programs signed by AuraSide.
- **Login is required.** Free gives you **10 optimizations** (+1 per referral, up to 15). Premium
  removes the limit, unlocks premium-only items, "Boost-Ups" and Game Mode's CPU/GPU options, and
  removes ads.
- **Two kinds of change:**
  1. **Optimizations** are permanent registry, service, power, network, BCD, GPU-driver and even
     **BIOS** changes. The program that applies them (HoneUtility) is **encrypted with ConfuserEx**,
     so the exact values could not be read. The table below says what each item is described to do.
     Hone says it saves originals for "supported" settings and offers a System Restore point, but this
     could not be checked per item.
  2. **Game Mode** (HoneTune) is temporary and properly restored: power plan, timer resolution,
     process priority/suspend/kill, stopping services, CPU affinity, GPU priority.
- **The catalogue includes outright security downgrades:** disable Windows Defender, UAC, VBS, CPU
  "mitigations" (Spectre/Meltdown), SmartScreen. Hone's own descriptions warn about some of them.
- **No Hone kernel driver and no injection into games.** But Hone can load two third-party kernel
  drivers:
  - HWiNFO's sensor driver, for hardware monitoring;
  - AMI's flash driver, through SCEWIN, for the BIOS editor.
- **Two optimizations install a permanent `HoneSvc` service** (device MSI/interrupt affinity). It
  re-applies those settings at every device or power event.
- **Telemetry:** Mixpanel (including its session-replay library), Sentry, Hone's own error service,
  a hardware fingerprint, game-session and benchmark uploads, Overwolf install analytics (partly over
  plain HTTP), and an uninstall ping.

## Optimizations, one by one

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky** (can
break things, cost security or privacy, or trip anti-cheat), **bloat**.

"What it changes" is from Hone's own item descriptions, the bundled data files and HoneUtility's
imports. HoneUtility's encrypted code was not read, so the registry values are the **usual** ones for
such a tweak and could not be confirmed. Defaults: nothing is applied until you click it. "Reversible"
says what Hone claims and what the mechanism allows.

### Performance and latency

| Item | What it changes | Reversible | Verdict |
|---|---|---|---|
| Optimize Windows power plan | Imports "Hone Ultimate Power Plan V4" (built on Ultimate Performance: min CPU 100 %, no idle/parking/power saving) and activates it | Yes (switch plan; stock plan files shipped) | **useful** on a plugged-in desktop, mildly; laptops: heat and battery |
| Lock CPU at 100 % | CPU kept from idling (idle states off in the plan) | Yes, by plan | **harmful** on laptops (heat → throttling), placebo on desktops |
| Disable power throttling | Power-throttling (EcoQoS) policy off system-wide | Probably (one registry value) | placebo for a foreground game; slightly more power for background apps |
| Disable power-saving features | Device/USB/PCIe power saving off, hibernation-type features | Unknown | placebo; costs battery |
| Disable hibernation | `powercfg -h off` (also removes Fast Startup) | Yes (`-h on`) | placebo for FPS; frees disk space |
| Optimize Windows timer resolution | Timer resolution request / Windows 11 global timer setting; ships `SetTimerResolution.exe` | Likely | placebo on Windows 11 (games set their own); can raise idle power |
| Optimize BCDEdit settings | Boot options (typically dynamic tick, platform clock/tick, TSC sync) | Only if old values were saved | placebo at best; **risky** (can worsen timing, boot issues) |
| Optimize Message Signaled Interrupts | Sets MSI mode for devices in their `Interrupt Management` keys; creates **HoneSvc** to re-apply | Claimed | mostly placebo (modern drivers already use MSI); **risky** with drivers that don't support it (no boot or device failure) |
| Optimize device affinity | Interrupt affinity policy per device (GPU/NIC/USB on chosen cores); creates **HoneSvc** | Claimed | placebo/**risky**; gains unproven on Windows 10/11 |
| Disable dynamic interrupt steering | Changes how Windows spreads interrupts (Hone 1.4 item) | Unknown | placebo/risky |
| Optimize CSRSS settings | Raises `csrss.exe` CPU/IO priority via Image File Execution Options | Probably | placebo |
| Optimize memory management | Memory manager values (paging executive, large system cache, page file) | Unknown | placebo; page-file changes can be **harmful** on 8–16 GB machines |
| Optimize I/O operations | Filesystem/disk values (last-access, 8.3 names, I/O queue values) | Unknown | placebo |
| Optimize drives | TRIM/defrag scheduling, drive health | n/a | placebo (Windows already does this) |
| Disable Superfetch (SysMain) | Service start type to Disabled | Claimed | placebo; slower app starts on HDDs |
| Disable Windows services | **Start type** of a server-defined list set to Disabled | Claimed | mostly placebo, some **harmful** (breaks features silently) |
| Optimize Windows scheduled tasks | Disables a list of scheduled tasks | Claimed | placebo |
| Disable mitigations | Spectre/Meltdown and similar CPU mitigations off (`FeatureSettingsOverride`) | Yes (registry) | **risky** (security); measurable gain only on older Intel in some cases |
| Disable VBS | Virtualization-based security / memory integrity off | Yes | **risky**; can give a few % on some systems, but anti-cheats increasingly expect it on |
| Disable fullscreen optimizations | Per-user GameDVR/FSE compatibility flags | Yes | placebo on current Windows (can break flip-model benefits) |
| Disable Multi-Plane Overlay (MPO) | DWM `OverlayTestMode` | Yes | **useful** only if you have MPO flicker/stutter; otherwise placebo |
| Disable Windows Game Bar / Xbox | Game Bar, Game DVR, Xbox services off | Yes | placebo (useful only if you never use them) |
| Enable Hone Game Mode | Turns on Game Mode features (see below) | Yes | see Game Mode |
| Game optimizations | Bundle of game-related registry values (MMCSS "Games" profile, GPU priority, `SystemResponsiveness`) | Claimed | placebo |
| Optimize mouse | Mouse acceleration off ("Enhance pointer precision"), queue sizes; `SetDpi.exe` | Yes | **useful** (acceleration off) if you want raw input; rest placebo |
| Set visual effects for performance | Animations, shadows and similar off | Yes | placebo on modern GPUs |
| NVIDIA control panel settings / NVIDIA general optimizations | Imports Hone's profile into the driver's **global** profile (about 80 values: low latency, max performance power, threaded optimization **off**, V-Sync off, **G-SYNC off globally**, 10 GB shader cache) | **Only to driver defaults** (the revert imports an empty profile, so your own settings are lost) | mixed: some fine, G-SYNC/threaded-optimization off can be **harmful** |
| Lock NVIDIA GPU at max boost clock | GPU kept at max P-state | Probably | **harmful** (heat, idle power), no gain in GPU-bound games |
| Disable NVIDIA HDCP / enable legacy sharpening / NVIDIA telemetry | Driver registry values | Yes | placebo; HDCP off breaks protected video |
| Optimize AMD settings | AMD driver (Adrenalin) registry values | Unknown | placebo/risky (old drivers) |
| Optimize integrated Intel GPU | iGPU driver values | Unknown | placebo |

### Network

| Item | What it changes | Reversible | Verdict |
|---|---|---|---|
| Remove network bandwidth limits | `NetworkThrottlingIndex` off, QoS reserved bandwidth 0 | Yes | placebo (the "20 % reserved" is a myth) |
| Optimize TCP settings / Netsh settings | TCP auto-tuning, RSS, ECN, timestamps, initial RTO, Nagle-type values via netsh/registry | Claimed | placebo; auto-tuning off can **harm** download speed |
| Optimize congestion provider | TCP congestion control (for example CTCP/BBR) | Yes | placebo for game UDP traffic |
| Optimize DSCP | QoS policy marking game traffic | Yes | placebo (home routers and ISPs ignore DSCP) |
| Optimize network adapter settings / disable task offload | NIC advanced properties (interrupt moderation, power saving, offloads off) | Claimed | mostly placebo; offload off raises CPU use |
| Disable SMBv1 & SMBv2 | File-sharing protocols off | Yes | SMBv1 off is good security; **SMBv2 off breaks file sharing** |
| Disable VPN support / Wi-Fi / Bluetooth | Services and devices off | Yes | **harmful** if you use them; placebo otherwise |

### Privacy and Windows clean-up (about 50 items)

Telemetry off for Windows (DiagTrack), Windows Update/Store, taskbar/Start, developer tools, Chrome,
Edge, Firefox, Office, Visual Studio and NVIDIA. Also: general privacy, Cortana, Copilot, OneDrive,
Windows Insider, error reporting, the Program Compatibility Assistant, Storage Sense, search indexer,
search highlights, notifications and Action Center, Focus Assist, Mobility Center, My People,
HomeGroup, touch settings, Fax and printing, lock screen and its blur, startup effects, transparency
and dynamic tiles. Shell taste items: classic context menu, classic Alt-Tab, old volume pop-up, compact
Explorer, all tray icons shown, no "- Shortcut" suffix, JPEG wallpaper quality, taskbar transparency,
Windows Settings ads removed. Browser items: hardware acceleration off, updates off, background off.

- **Reversible:** yes for nearly all (policy and registry values).
- **Verdict:** placebo for FPS. The privacy and ad items are fine if you want them. Several are
  **harmful**:
  - browser updates off (security);
  - browser hardware acceleration off (more CPU use);
  - search indexer off (slow search);
  - printing off.

### Security downgrades

| Item | Verdict |
|---|---|
| Disable Windows Defender | **risky** (no antivirus; Hone itself warns) |
| Disable UAC | **risky** (every program gets admin silently) |
| Disable SmartScreen | **risky** |
| Disable VBS, Disable mitigations | **risky** (see above) |

### Installers and extras

| Item | Verdict |
|---|---|
| Install Process Explorer, Samsung NVMe driver, enhanced registry editor (TotalReg) | bloat (installers, not optimizations); the Samsung NVMe driver is **risky** on Windows 10/11 (the inbox driver is fine) |
| Game configs / "pro player" settings | Rewrites a game's settings file; revert offered. Fine if you like the preset. |
| Boost-Ups (premium bundles) | Apply many items at once; same verdicts as the items |

### BIOS optimizations (hone-bios + AMI SCEWIN)

Reads and writes BIOS setup values on AMI Aptio boards:

- **off:** C-states and C1E, SpeedStep/Speed Shift, spread spectrum, vendor auto-installers, fast
  boot;
- **on:** Turbo, Resizable BAR / Above 4G decoding, Hyper-Threading;
- plus about 80 other options.

It loads AMI's flash kernel driver.

- **Reversible:** only from Hone's export, or "load defaults" in BIOS setup.
- **Verdict:** Resizable BAR on is **useful** if your GPU supports it. C-states and Speed Shift off are
  **harmful** on laptops and dubious on desktops. Writing NVRAM from Windows is **risky** (a bad write
  can stop the PC booting).

### Game Mode (temporary, restored at game exit)

| Part | Default | What it does | Restored | Verdict |
|---|---|---|---|---|
| Power plan | on | Activates the Hone plan | yes | **useful** on AC power, mildly |
| Timer resolution | always on | Maximum timer resolution for the session. On Windows 11 it also sets `GlobalTimerResolutionRequests` (needs a reboot to take effect) | resolution released; the registry value is **deleted** on stop, even if you had set it yourself | placebo |
| Power throttling | always on | The game process is opted out of EcoQoS | ends with the process | placebo for a foreground game; **useful** for minimised/alt-tabbed games on hybrid CPUs |
| Visual effects | always on | UI animations off | not confirmed | placebo |
| Processes | server lists | Priority change, **suspend**, or **kill** | priority and suspend undone; killed apps not restarted | priority: useful-ish; suspend/kill: **risky** |
| Services | server lists | Stop or pause | restarted | mostly placebo |
| CPU affinity / no CPU0 / no E-cores / no SMT (premium) | off | Game pinned to chosen cores, others moved away | yes | **risky**; sometimes helps on hybrid or dual-CCD CPUs |
| Core parking (premium) | off | Min unparked cores 100 % | yes (backed up in registry) | placebo on Windows 10/11 |
| GPU priority (premium) | off | GPU scheduling priority class for listed processes | ends with the process | placebo/unproven |
| SnapTap | manual | Global keyboard hook, last-key-wins for opposite keys | yes | **risky**: banned in some competitive games; Hone blocks it only for listed anti-cheats, and **EA Javelin is not on its list** |

### Other parts

| Part | Default | Verdict |
|---|---|---|
| HoneSvc (SYSTEM-level auto-start service, created by the MSI/affinity items) | off until those items | **risky** (permanent, re-applies device tweaks after every device or power event) |
| Elevation helper (named pipe, signed-exe allow-list) | while running | OK design, still a standing admin broker |
| HWiNFO sensor driver (SysInfo) | when monitoring and benchmarking | **risky** for anti-cheat compatibility (it is a well-known signed driver, usually tolerated) |
| Telemetry, hardware fingerprint, session replay library | on | **risky** (privacy); no way to turn telemetry off was found in the bundle |
| Updates | automatic | App via electron-updater (SHA-512 over HTTPS, no publisher-signature check configured); helpers via API with SHA-256 checks and signature-pinned elevation. OK in practice. |
| Ads (Free), referrals, optimization counter | on | mild dark pattern |

## Marketing claims, checked

- **"Up to 45 % FPS increase"** is a single user testimonial on the home page, not a measurement. Hone's
  own page also says no fixed result applies.
- The items that can measurably move FPS or latency are few:
  - the power plan (desktop, CPU-bound games);
  - Resizable BAR (via the BIOS editor);
  - disabling MPO (only on systems with MPO problems);
  - disabling VBS or mitigations (a few %, by giving up security);
  - mouse acceleration off (feel, not FPS).
- Everything else is the same registry folklore found in SGB, Razer Cortex and PGWARE: timer,
  BCD, TCP/Nagle, CSRSS, MMCSS, services.
- **"Reduce PC-side delay" / network:** Hone itself states it does not route traffic or reduce ping.
  The TCP/DSCP items do nothing for UDP game traffic.
- **"Restore supported settings":** true for Game Mode (verified). For optimizations it is a claim the
  encrypted engine prevents checking. The NVIDIA revert provably resets to defaults instead of your
  previous settings.
- **"2.5M+ gamers", Trustpilot 4.7:** not checkable from the code.

## For a 2019 MacBook Pro under Boot Camp playing BF6

The machine has an Intel 9th-gen CPU with the 800 MHz BD PROCHOT lock, an AMD Radeon Pro 5300M/5500M
and old Boot Camp drivers.

- **Worth anything:**
  - Game Mode's power plan, on AC only, and better with a capped max processor state. The Mac is
    heat-limited, so the "100 % minimum" style plans add heat and can make throttling worse.
  - Mouse acceleration off.
  - Windows ads and suggestions off, if you want them off.
- **Avoid:**
  - **Lock CPU at 100 %**, Disable C-states-style power items, Disable power saving. More heat on a
    machine already throttling.
  - **BIOS optimizations.** The Mac has Apple EFI, not AMI Aptio. SCEWIN won't work, and you must not
    try.
  - **MSI mode and device affinity.** Apple's Boot Camp drivers are old and untested with forced MSI.
    A failure can mean no keyboard, trackpad or boot. These also install HoneSvc.
  - **BCDEdit, Disable Windows services, Disable Bluetooth/Wi-Fi.** Bluetooth off kills Magic
    Mouse/AirPods.
  - **Optimize AMD settings.** Hone targets Adrenalin; the Boot Camp Radeon driver is Apple's older
    package.
  - Security downgrades (Defender, UAC, VBS, mitigations).
  - **SnapTap.**
  - All NVIDIA items do nothing here.
- **Anti-cheat (EA Javelin):**
  - Hone does not inject into BF6. Game Mode's changes are ordinary admin API calls.
  - BF6 requires Secure Boot. Hone's catalogue does not touch Secure Boot, but it lets you turn off
    VBS and mitigations, which is the wrong direction for a kernel anti-cheat.
  - The HWiNFO driver (loaded for monitoring) is a signed, widely used driver. Javelin's stance on it
    is unknown.
  - SnapTap's anti-cheat guard does not know Javelin, and SOCD key tools are banned in some games.
  - Overall: low risk if you use only Game Mode without SnapTap and skip the monitoring/benchmark
    features. Untested.
- **Overall:** for this Mac, Hone adds a login, telemetry and an optimization quota in exchange for a
  power plan and a mouse setting. gamebooster-rs already does the power plan with a max-state cap for
  the PROCHOT problem.

## Compared with SGB, Razer Cortex and gamebooster-rs

- **Same folklore as SGB and Cortex:** the optimization list overlaps heavily with SGB's `.dbd` tweaks
  and Cortex's System Booster (TCP, MMCSS, services, telemetry, visual effects). Hone goes much
  further:
  - MSI/interrupt affinity with a permanent service;
  - BCD and CPU-mitigation edits;
  - Defender/UAC/VBS off;
  - NVIDIA driver profile rewrites;
  - BIOS NVRAM writes.
- **Better than SGB/Cortex:**
  - Game Mode restores properly, with backups in the registry and crash recovery.
  - The elevation broker checks signatures, unlike Cortex's "start anything as admin" service.
  - Game Mode has no clipboard wipe or RAM-trim gimmick.
- **Worse:**
  - Login required, and a paywall on item count.
  - Heavier telemetry, including session replay and a hardware fingerprint.
  - An encrypted tweak engine (you cannot audit what it writes).
  - Optional third-party kernel drivers.
- **gamebooster-rs already covers** the useful core: a High-performance-based plan with restore, chosen
  services and apps, game priority, Game Mode, and accessibility shortcuts off. It also adds a max
  processor state cap for the Mac.

## Worth adopting into gamebooster-rs (reversible, no driver)

1. **Built** (`game_power_throttling_off`, default on; see [game priority](../tweaks/priority.md#power-throttling-off)).
   **Opt the game out of power throttling (EcoQoS)** with `SetProcessInformation(ProcessPowerThrottling)`
   on the game process while boosted.
   - Process-scoped: it ends when the game exits and needs nothing stored.
   - Evidence: HoneTune does exactly this, always on. Microsoft documents the API.
   - Effect: small on a foreground game; it helps when the game is minimised or behind another
     window, which Windows otherwise throttles on battery.
   - Low value on the Mac (no hybrid cores), so optional.
2. **Back up power settings under our own registry key / state file before changing them, and restore
   from it after a crash.** We already keep a state file. HoneTune's per-scheme AC/DC backup with
   "restore incomplete" reporting is the same idea, so this only confirms our design.
3. **Before risky actions, check for known anti-cheat processes** (HoneTune's list) and include EA
   Javelin (`EAAntiCheat.GameService`), which Hone misses. Use it to skip suspending or re-prioritising
   such processes. gamebooster-rs already skips by publisher, so this is a small hardening idea, not a
   new feature.

Not adopted:

- **Timer resolution.** On Windows 11 the global switch needs a reboot, and Hone's restore deletes a
  value the user may have set. Placebo.
- **GPU scheduling priority class.** Undocumented D3DKMT call with no evidence of gain.
- **CPU affinity/parking.** Risky; no gain on Windows 10/11.
- **Everything permanent** (MSI, BCD, services, security, NVIDIA, BIOS): against our "restore undoes
  everything, no driver" rule.
