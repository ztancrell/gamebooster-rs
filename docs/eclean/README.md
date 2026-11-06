# eclean Optimizer: what it really is

eclean is a Windows "keep your PC tidy" suite from eclean Labs AB (Sweden), at
[eclean.gg](https://eclean.gg/products/eclean/features/optimizer/). It has a Cleaner (a built-in copy
of the community Winapp2 database), an Uninstaller, a Booster page (startup apps, services,
scheduled tasks, automatic DNS), live hardware views, a **Performance mode** on the dashboard, and
the **Optimizer**: a catalogue of 47 Windows settings, each explained, sorted into Recommended,
Trade-off, Advanced and Expert, with a per-change history and undo. It also has an *Atlas mode*
that applies AtlasOS playbook toggles (eclean partners with AtlasOS). Free, Pro ($6.99/month or
$59.99/year, 2 PCs) and Family. The marketing is mild by booster standards: no FPS numbers, and its
FAQ says registry cleaning "has not delivered measurable speed on any Windows since XP".

Version **1.4.4** (file and product version), the official
`https://update.eclean.gg/windows/latest/eclean-latest_x64-setup.exe` (the site's Download link; no
account, no payment), downloaded 2026-11-06, `Last-Modified` 2026-10-26, SHA-256
`eb0608baa743026e603d65825a73ed920c4bdb176ffa91fc975b57930b4f7fb0`, 26,855,040 bytes.
**Nothing was run.**

## How it's built

A **Tauri 2** app: Rust back end, SvelteKit front end in WebView2, packed in an NSIS installer
(unpacked with 7-Zip). Inside are `eclean.exe` (the app, 68 MB), `eclean-privileged-service.exe`
(the broker, 13 MB; it shares one Rust core with the app), the WinGet engine
(`WindowsPackageManager.dll`, for app updates), the WebView2 bootstrapper and
`resources/backend_config.json` (API base `https://api.eclean.gg`, Sentry on). Both binaries are
Rust (nightly 2026-10-21) with clear strings and source paths. The front end is embedded in
`eclean.exe` as brotli-compressed assets; they were pulled out with a small script that follows
Tauri's asset table. The English UI text (every Optimizer entry's title, summary, description and
trade-off) comes from those assets. The registry paths and value names come from the broker's string
table, and the written values come from the immediates in the broker's apply and revert routines,
read from an `objdump` disassembly of a few functions. That disassembly was deleted afterwards (see
[decomp/README.md](../../decomp/README.md#policy)).

## Short version

- **No kernel driver.** No `.sys`, no driver loading, no WinRing0/PawnIO. No miner, stealer or
  exfiltration code found.
- **A broker service**: `eclean Privileged Service` (`eclean-privileged-service.exe --service`),
  registered as a **demand-start** service, not auto-start. It is reached over the named pipe
  `\\.\pipe\eclean-privileged-service-v1` and runs only "cataloged" commands. Its
  own description says it does this "without granting the GUI a generic elevated executor". It
  checks the caller's token (`caller_not_trusted`, `caller_not_local_admin`) and audits requests.
  That is a narrower design than Tier1Suite's always-on broker.
- **Autostart:** a per-user Run entry, **on by default** at first run (`--autostart`), plus
  Task Scheduler tasks under `\eclean` (scheduled cleaner, low-space trigger).
- **Phones home:** `api.eclean.gg` (account, entitlements, device limit, feature flags,
  notifications polling, referrals), Sentry crash reports (`ingest.de.sentry.io`), OpenTelemetry
  (`otel.eclean.gg`), product analytics (`insights.eclean.gg`), the Tauri updater
  (`update.eclean.gg`, minisign-signed manifest) and Discord Rich Presence. The DNS feature measures
  public DoH/DoT resolvers (Cloudflare, Google, Quad9, AdGuard, OpenDNS, HE). That is a lot of
  telemetry for a tidy-up tool, but nothing points at more than usage and crash data.
- **Optimizer changes are persistent**, not per session. Each one records a "restore point" of the
  old registry values (and the old `bcdedit` nx policy) and can be reverted from its history. Many need a
  reboot (MMCSS, the timer key, VBS, mitigations). Pro's *Stay optimized* (`optimizations/keep`)
  re-applies changes that a Windows update undid.
- **Pro gating** is an account entitlement checked by the app (`features/auth/service/entitlements`).
  The Expert block sits behind `optimizer.expert_tweaks`. Not examined further.
- **Performance mode** is the only per-session "booster": a power-plan switch, foreground priority,
  stopping six services, all restored when it ends. See below.

## Optimizer catalogue (gaming-relevant entries)

The other ~30 entries are privacy, UI and update settings (telemetry personalization, advertising
ID, Spotlight, Copilot, Recall, widgets, web search, autoplay, LLMNR, SMBv1, hibernation, update
reboots, file extensions, classic context menu and so on). They are not performance tweaks and are
not listed. Revert for every row: the old value saved in the restore point is written back, or the
value is deleted if it was absent. Tier: **R** = Recommended (no trade-off text), **T/A** =
Trade-off or Advanced (has trade-off text), **E** = Expert (Pro; listed after the
`optimizer.expert_tweaks` entitlement in the broker's catalogue). No row needs a driver.

| Tweak (UI title) | Exact change | Tier | Reboot | Risk (heat / anti-cheat / security) |
|---|---|---|---|---|
| Prioritize foreground apps | `HKLM\SYSTEM\CurrentControlSet\Control\PriorityControl` `Win32PrioritySeparation` = `0x26` (38) | R | no | none. `0x26` behaves the same as the client default `2` (short, variable, 3:1) |
| Optimize MMCSS | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile` `SystemResponsiveness` = 10 | R | yes | none |
| Disable network throttling | same key, `NetworkThrottlingIndex` = `0xFFFFFFFF` (revert path uses 10) | R | yes | none |
| Disable Svchost splitting | `HKLM\SYSTEM\CurrentControlSet\Control` `SvcHostSplitThresholdInKB` = a computed value (from RAM size; not recovered exactly) | T/A | yes | less service isolation |
| Disable Game Bar | `HKCU\System\GameConfigStore` `GameDVR_Enabled` = 0; `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR` `AppCaptureEnabled` = 0; `HKCU\SOFTWARE\Microsoft\GameBar` `ShowStartupPanel`, `GamePanelStartupTipIndex`, `UseNexusForGameBarEnabled` | T/A | no | none; loses clips and overlay |
| Disable background apps | `HKCU\...\CurrentVersion\BackgroundAccessApplications` `GlobalUserDisabled`; `HKCU\...\CurrentVersion\Search` `BackgroundAppGlobalToggle` | T/A | no | Store apps only |
| Disable Delivery Optimization peering | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization` `DODownloadMode` (HTTP only) | R | no | none |
| Optimize NTFS | `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` `NtfsDisableLastAccessUpdate`, `NtfsDisable8dot3NameCreation` | T/A | no | loses last-access dates |
| Disable SleepStudy logging | disables the SleepStudy Task Scheduler task | R | no | none |
| Disable core isolation | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` `EnableVirtualizationBasedSecurity` = 0 and `...\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` `Enabled` = 0 | T/A | yes | **security**: no HVCI; anti-cheats are moving toward requiring it |
| Disable CPU mitigations | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management` `FeatureSettingsOverride` / `FeatureSettingsOverrideMask`; `...\Session Manager\kernel` `DisableExceptionChainValidation` (SEHOP), `MitigationOptions`, `MitigationAuditOptions`; `...\Session Manager` `ProtectionMode`; `bcdedit /set {current} nx` (policy AlwaysOff/AlwaysOn/OptIn/OptOut; the old policy is saved). Exact values not recovered | E (Pro) | yes | **harmful**: Spectre/Meltdown, CFG, SEHOP and possibly DEP protections off. Anti-cheats that check these can refuse to run |
| Disable Fault Tolerant Heap | `HKLM\SOFTWARE\Microsoft\FTH` (turns FTH off, clears its tracked-app list) | E (Pro) | no | crash-prone apps crash more |
| Enable timer resolution | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel` `GlobalTimerResolutionRequests` = 1 | E (Pro) | yes | **heat**: one app's fine timer then wakes every core, idle power up. Honoured only on Windows 11 / Server 2022+ |
| Disable core parking | `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\0cc5b647-c1df-4637-891a-dec35c318583` `ValueMax` = 0 (revert writes 100) | E (Pro) | no | **heat** if it has the effect the UI promises |
| Disable Modern Standby | `HKLM\SYSTEM\CurrentControlSet\Control\Power` `PlatformAoAcOverride` (the legacy override; value not recovered, 0 is the usual one) | E (Pro) | yes | sleep can break on some PCs |

## Performance mode (dashboard)

A timed session, manual or automatic (up to 32 executable names that start it), that ends on a
timer. It does three things, and undoes each when it ends:

1. **Power plan**: switches to an "aggressive" plan: *Atlas Power Scheme* if AtlasOS installed one,
   otherwise **High performance** (`PowerEnumerate`, `PowerSetActiveScheme`). The previous scheme's
   GUID is saved and re-activated.
2. **Foreground priority**: a worker follows the foreground window. If its process is at Normal,
   Below normal or Idle priority, it is raised to **Above normal** (`SetPriorityClass`, `0x8000`). The
   old class is stored (pinned by process creation time against PID reuse) and put back when the
   window loses focus or the mode ends.
3. **Background services**: stops `DiagTrack`, `dmwappushservice`, `WSearch`, `SysMain`, `Spooler`
   and `WerSvc` through the broker, records which were running, and starts those again on exit.
   Start types are not changed.

No timer resolution, no RAM purge, no process killing. Whether Performance mode is Free or Pro was
not determined.

## Claims vs reality

| Claim (eclean.gg / in-app text) | What the code does | Verdict |
|---|---|---|
| "Reduce background work that competes with responsiveness and frame consistency" | Performance mode stops six services and raises the foreground app to Above normal; the Optimizer turns off Game DVR and Store background apps | **Real but small**: the same session tweaks gamebooster-rs already makes. Game DVR off is the one with a measurable effect, and only when recording was on |
| Prioritize foreground apps "reduces input lag in games" and "improves game frame times" | Writes `Win32PrioritySeparation` = `0x26` | **Placebo**: `0x26` is the documented client default `2` spelled out (short, variable, 3:1), so nothing changes on Windows 10/11 client |
| Optimize MMCSS: "improving game frame times and reducing input latency" | `SystemResponsiveness` 20 → 10, read at boot | **Placebo for most games**: it only changes how much CPU MMCSS reserves for low-priority work while MMCSS-registered (audio/video) threads run. Tier1Suite's own tests found no effect |
| "A more responsive connection" / Disable network throttling | `NetworkThrottlingIndex` = `0xFFFFFFFF` | **Placebo for gaming**: it only lifts a cap on non-multimedia packet processing while MMCSS multimedia plays, which is far above a game's packet rate |
| Disable Game Bar "frees the CPU and memory Windows uses to record your games" | Game DVR and capture off | **Real** when background recording was on (it is off by default) |
| Enable timer resolution, "lower latency in some apps" | `GlobalTimerResolutionRequests` = 1 | **Misleading / harmful on a laptop**: Windows 11 only (no effect on the target's Windows 10 21H2). Where it works it brings back the old global timer, which raises idle power. Games that want a fine timer already ask for one themselves. The app does say "Your PC uses more power, even when idle" |
| Disable core parking, "avoids brief delays when idle cores wake" | `ValueMax` = 0 on the *core parking min cores* setting's registry entry | **Misleading**: this edits the setting's allowed range, not the active plan's value. High performance (Performance mode's plan) already keeps cores unparked on AC. Adds heat if it works as promised |
| Disable CPU mitigations, "cost the most on older CPUs" | Spectre/Meltdown overrides, SEHOP, mitigation options, `bcdedit` nx | **Harmful**: a real but small CPU gain on old CPUs at a broad security cost, and a reboot. The app is honest that it leaves the PC "open to CPU attacks" |
| Disable core isolation, "which some games are sensitive to" | VBS and HVCI off | **Real gain on some games at a security cost**. As in [Tier1Suite](../tier1suite/README.md); anti-cheats increasingly expect it on |
| Disable Svchost splitting, "can significantly reduce RAM usage" | Raises `SvcHostSplitThresholdInKB` | **Real but irrelevant**: it saves some tens of MB of process overhead on a 16 GB machine, at the cost of service isolation. Needs a reboot. No FPS effect |
| Disable Fault Tolerant Heap, "removes a slowdown" | FTH off, list cleared | **Mostly placebo**: FTH only shims apps that kept crashing. If a game is on that list, it is better to fix the crash |
| "Every change keeps its own history, and supported ones can be undone" | Per-change restore points with the old values (and the old nx policy); revert writes them back | **Real** |
| Automatic DNS: "switches you when a provider is clearly faster" | Benchmarks public DoH/DoT resolvers, sets DNS with `netsh` | **Real, small**: DNS speed affects lookups, not in-game ping. Same as our `fastest_dns` |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Not harmful if you stay out of the Expert block and away from core isolation, but nothing in it is
new for gamebooster-rs. Performance mode is a lighter version of our boost (High performance plan,
priority, a service stop list). Game DVR off is already a `status` line. The Expert entries
(timer resolution, core parking, mitigations) all add heat or remove security on a heat-limited CPU,
and the timer key does nothing on Windows 10 21H2. The persistent Optimizer changes plus the
default-on autostart and five telemetry endpoints are more resident software than the tweaks
justify.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **`WerSvc` and `dmwappushservice` as suggested `services` entries**: **try by feel**. eclean's
   Performance mode stops `DiagTrack`, `dmwappushservice`, `WSearch`, `SysMain`, `Spooler` and
   `WerSvc` and starts again only the ones that were running. That is exactly our `services`
   mechanism. `WerSvc` (Windows Error Reporting) and `dmwappushservice` (WAP push routing, telemetry
   plumbing) are harmless to stop for a session. The only cost is that a crash during the session is
   not reported to Microsoft. Mention them in [services](../tweaks/services.md) only if the list
   there is ever extended. No default change: `services = []` stays.
2. **Foreground-follow priority (Above normal for whatever window is in front)**: **skip**. We
   already raise the named game, and `lower_others` lowers the rest. Following the foreground would
   also touch BF6 from outside the process, which EA Javelin blocks (see
   [priority](../tweaks/priority.md)); our registry method is the working route.
3. **Session expiry timer for boost**: **skip**. `auto` already restores when the game exits, which
   is what eclean's timer approximates.
4. **`Win32PrioritySeparation` = `0x26`**: **skip**. It equals the client default; we already write
   `0x2A` (fixed quantum) as a try-by-feel item.
5. **`SystemResponsiveness`, `NetworkThrottlingIndex`, Svchost split, FTH, NTFS last-access**:
   **skip**. These are placebo for games, reboot-only, or both.
6. **`GlobalTimerResolutionRequests`, core parking `ValueMax`, CPU mitigations, VBS/HVCI, Modern
   Standby override**: **skip**. They add heat, remove security, need a reboot, or have no effect on
   Windows 10 21H2. `restore` cannot undo a reboot-time setting within a session.
7. **Game DVR off, High performance plan, fastest DNS**: already in gamebooster-rs (`status` line,
   power plan, `fastest_dns`).
