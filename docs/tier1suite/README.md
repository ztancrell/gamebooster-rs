# Tier1Suite (Tier1Settings): what it really is

Tier1Suite is a Windows "PC optimization suite" from Tier1Settings, the people behind the
tier1settings.com gaming-settings articles ([product page](https://tier1settings.com/tier1suite/)).
It has four modules: **Tier1Tune** (a catalogue of 43 tweaks and system checks, each with Apply and
Revert), **Tier1Timer** (holds a fine timer resolution, per game), **Tier1Stretch** (stretched and
custom resolutions) and **Tier1Monitor** (a timer "latency" benchmark and per-game FPS capture).
It comes as Free and Pro. Unlike most boosters, every catalogue entry explains itself honestly in the
app: several entries say outright that they are placebo, myths or desktop-only. Many "risky" ones are
report-only on purpose.

Version **1.4.2** (`1.4.2+810a6bc14789ad5bb8727aec00b4d704123e74c5`), the official MSI
`https://downloads.tier1settings.com/tier1suite/downloads/Tier1Suite.msi`, downloaded 2026-11-06,
SHA-256 `319923407b2cf83a9302ad1afe141f5c18e968bd2af028ff2663721742d02a47`, 75,038,720 bytes.
**Nothing was run.** The MSI was unpacked with 7-Zip. The main program is a .NET 8 WPF single-file
bundle (not Native AOT). It was unpacked with a small bundle-manifest reader. Inside are a composite
ReadyToRun image (`Tier1Suite.r2r.dll`) and the component assemblies, which still hold their IL:
`Tier1Suite.dll`, `Tier1Suite.Core.dll` and `Tier1Suite.Modules.{Tune,Timer,Stretch,Monitor}.dll`.
The facts below come from the assemblies' metadata, their string tables (the catalogue text is in
them, in full) and the IL of the catalogue constructors. Those constructors give the risk level,
hive, value and type of each tweak, and the Free/Pro list. All IL dumps were deleted afterwards (see
[decomp/README.md](../../decomp/README.md#policy)).

## Short version

- **No kernel driver.** The MSI installs `Tier1Suite.exe`, Intel's `PresentMon.exe` and nothing else
  that runs in the kernel. But it installs a **LocalSystem service**, `Tier1SuiteBroker`
  (`Tier1Suite.exe --service`, described as "Applies the Windows settings you choose in Tier1Suite
  without an administrator prompt, and re-applies them if Windows resets them"). It also adds an HKLM
  Run entry (`Tier1Suite.exe --background`) and a `tier1suite:` URL protocol.
- **Tweaks are persistent, not per session.** An applied tweak stays until you press Revert. A
  "watchdog" in the broker re-applies it if Windows changes it back. Revert is real: each tweak saves
  the old value, or notes that the value was absent, before writing. Revert puts that back or deletes
  the value. When elevated it also creates a System Restore point (`srclient.dll`) first, and after a
  crash it reverts unfinished transactions.
- **The catalogue is almost all plain registry values**, plus two service start types, three power
  settings and one `netsh` DNS change. The standby-list cleaner has **no purge** in this build. It
  only watches and says when it would purge. The kernel DPC/ISR capture and the built-in frame-time
  parser are also "not implemented in this build".
- **Free vs Pro:** Free gets the 6 system checks plus 14 named entries. Pro gets the remaining 23 tweaks,
  Timer and Stretch Auto Mode, and unlimited custom resolutions. The check is client-side: a tier in
  the app, backed by a cached server validation. Not examined further.
- **Network:** licence calls, telemetry and an "advisor" chat (`/advisor/chat`, `/advisor/explain`)
  go to an AWS API Gateway (`rvfcrclte7.execute-api.us-east-1.amazonaws.com`). There are also Google
  Analytics 4 Measurement Protocol events (`G-Y8XCRVFKMY`), a `version.json` update check and
  report sharing (`/report/share`, `/report/delete`). A key works on 3 PCs.

## The tweak catalogue (43 entries)

Risk is the app's own rating: Safe, Moderate or Advanced. "Report" means detect-only: it reads and
explains, and never writes. **Free** marks the Free tier; everything else needs Pro. Revert is the
same for every registry entry: the previous value is written back, or deleted if it did not exist.
None of the entries needs a driver. Anti-cheat risk is "none" unless noted: these are ordinary,
documented settings, and nothing touches a game process. **Heat** is for a heat-limited laptop CPU
such as the target Mac's i7-9750H.

### Latency (12)

| Tweak | Exact change | Risk | Heat | Notes |
|---|---|---|---|---|
| Core parking min cores 100 % | Power setting *core parking min cores* (`0cc5b647-c1df-4637-891a-dec35c318583`, subgroup `54533251-82be-4824-96c1-47b60b740d00`) = 100 on **AC only** (`PowerWriteACValueIndex`), old AC value saved | Moderate | more idle heat | The app says it rarely helps. High performance (our base plan) already doesn't park on AC |
| C-states (OS idle disable) | Power setting *processor idle disable* (`5d76a2ca-e8c0-402f-a133-2158492d58ad`) = 1, AC only | Advanced | **high**: cores never idle; the app warns it can lower boost clocks | Bad for a hot laptop |
| Fullscreen optimizations | Report: `HKCU\System\GameConfigStore` `GameDVR_FSEBehaviorMode` / `GameDVR_HonorUserFSEBehaviorMode` | Safe | none | Advises leaving FSO on |
| Game DVR background recording (Free) | `HKCU\System\GameConfigStore` `GameDVR_Enabled` = 0 (DWORD) | Safe | less GPU work when it was on | Pairs with *Game Bar capture* below |
| Games task GPU priority | `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games` `GPU Priority` = 8 | Moderate | none | 8 is already the Windows default for the Games task on stock installs, so usually a no-op |
| GPU low-latency mode | Report only; says NVAPI/ADL are not queried, points at the driver panel | Safe | none | |
| HAGS | `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers` `HwSchMode` = 2, reboot | Safe | none | We already report this in `status` |
| HPET | Report: parses `bcdedit /enum {current}` for `useplatformclock` | Advanced | none | Advises leaving it unset |
| Mouse acceleration (Free) | `HKCU\Control Panel\Mouse` `MouseSpeed` = "0" (string; thresholds not touched), sign out | Safe | none | Raw-input games (BF6) ignore it |
| Disable MPO | `HKLM\SOFTWARE\Microsoft\Windows\Dwm` `OverlayTestMode` = 5, reboot | Moderate | none | Only for flicker/black flashes |
| System responsiveness | `...\Multimedia\SystemProfile` `SystemResponsiveness` = 10 | Moderate | none | The app says its own tests found no effect |
| Timer resolution (Free) | Report: `NtQueryTimerResolution`. Tier1Timer does the setting | Safe | none | See Tier1Timer below |

### Windows (15)

| Tweak | Exact change | Risk | Heat | Notes |
|---|---|---|---|---|
| Background apps (UWP) | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppPrivacy` `LetAppsRunInBackground` = 2 | Moderate | none | Store apps only, as the app says |
| Delivery Optimization | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization` `DODownloadMode` = 0 | Moderate | none | No peer upload/download |
| Game Bar capture (Free) | `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` `AppCaptureEnabled` = 0 | Safe | less GPU work when on | |
| Game Mode (Free) | `HKCU\Software\Microsoft\GameBar` `AutoGameModeEnabled` = 1 | Safe | none | Same as ours |
| Memory integrity (HVCI) | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` `Enabled` = 0, reboot | Advanced | none | Security trade-off. **Anti-cheat:** some require it on; the app warns |
| Notifications | `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications` `ToastEnabled` = 0 | Moderate | none | The app itself says Do Not Disturb is the better tool while gaming |
| Power plan | Activates Ultimate Performance (`e9a42b02-d5df-448d-aa00-03f14749eb61`, made with `PowerDuplicateScheme` if missing), else High performance (`PowerSetActiveScheme`) | Moderate | **high** (min CPU 100 %) | Revert: re-activates the old plan; deletes Ultimate (`PowerDeleteScheme`) if it created it |
| Power throttling | `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling` `PowerThrottlingOff` = 1 (machine-wide) | Moderate | **more**: no app is throttled | We already do this per game |
| Search indexing | `WSearch` start type → Disabled (`ChangeServiceConfigW`; old start type saved via `QueryServiceConfigW`) | Moderate | none | Breaks Start/Explorer search |
| Startup delay (Free) | `HKCU\...\Explorer\Serialize` `StartupDelayInMSec` = 0 | Safe | none | Login only |
| Startup items (Free) | Report: HKCU/HKLM `...\CurrentVersion\Run` and `Explorer\StartupApproved\Run` | Safe | none | No bulk disable, on purpose |
| Taskbar widgets (Free) | `HKCU\...\Explorer\Advanced` `TaskbarDa` = 0 | Safe | none | Desktop only. The app has a wrapper for values Windows guards ("Windows protects this setting from other apps"); recent Windows 11 guards `TaskbarDa` this way |
| Transparency (Free) | `HKCU\...\Themes\Personalize` `EnableTransparency` = 0 | Safe | none | Desktop only |
| VBS | `HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard` `EnableVirtualizationBasedSecurity` = 0, reboot | Advanced | none | As HVCI |
| Visual effects | `HKCU\...\Explorer\VisualEffects` `VisualFXSetting` = 2 | Safe | none | Desktop only |

### Network (5)

| Tweak | Exact change | Risk | Heat | Notes |
|---|---|---|---|---|
| Adapter power saving | Active adapter's instance key under `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}\NNNN` (matched by `NetCfgInstanceId`): `PnPCapabilities` = 24 | Moderate | tiny | Takes effect only after the adapter reloads (you disable/enable it, or reboot) |
| DNS | `netsh interface ipv4 set dnsservers ... static` 1.1.1.1 + 1.0.0.1; saves DHCP vs static list | Moderate | none | Our `fastest_dns` does this, but measured |
| Nagle (Free) | `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{adapter}` `TCPNoDelay` = 1, `TcpAckFrequency` = 1 | Safe | none | TCP only; BF6 uses UDP |
| Throttling index | `...\Multimedia\SystemProfile` `NetworkThrottlingIndex` = 0xFFFFFFFF | Advanced | none | |
| QoS (Free) | Report: `HKLM\SOFTWARE\Policies\Microsoft\Windows\Psched` `NonBestEffortLimit` | Safe | none | Calls the "20 % reserved" claim a myth |

### Storage & memory (5)

| Tweak | Exact change | Risk | Heat | Notes |
|---|---|---|---|---|
| Page file (Free) | Report: `...\Session Manager\Memory Management` `PagingFiles` | Advanced | none | Advises system-managed |
| Shader cache (Free) | Report: sizes of NVIDIA/AMD/Intel/`D3DSCache` folders under local app data | Safe | none | Never deletes |
| Standby list cleaner | Polls free RAM and standby size (WMI `Win32_PerfRawData_PerfOS_Memory` `StandbyCache*Bytes`) every 1 s; "would purge" when free < 1024 MB and standby > 1024 MB | Moderate | none | **The purge call is not in this build**, so it never clears. No `NtSetSystemInformation` anywhere |
| SysMain | `SysMain` start type → Disabled | Moderate | none | |
| TRIM (Free) | `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` `DisableDeleteNotify` = 0 | Safe | none | Fixes a bad old tweak; normally already 0 |

### System facts, report-only (6, all Free)

GPU driver version and age (WMI `Win32_VideoController`; flags a driver older than about a year),
memory speed vs JEDEC default (XMP/EXPO, `Win32_PhysicalMemory`), Resizable BAR (size of the GPU's
largest PCI memory window), system drive type (`MSFT_PhysicalDisk`), TPM and Secure Boot
(`Win32_Tpm`, `SecureBoot\State\UEFISecureBootEnabled`) and Windows version. Risk: Safe for driver and
drive, Moderate for memory and ReBAR, Advanced for TPM and Windows version (the risk is in the BIOS or
OS change they point to).

## The other modules

- **Tier1Timer**: `NtSetTimerResolution` (and `NtQueryTimerResolution`). A requested value, default
  label "Set timer resolution to 0.5 ms", is held by the app's process. Releasing the request is the
  revert. With Auto Mode (Pro) it sets a per-game value when a watched game starts and drops back when
  the game exits. Since Windows 11 a timer request applies only to the process that makes it. A game
  that wants a fine timer asks for one itself, so this is placebo there, as
  [ISLC](../islc/README.md) already found.
- **Tier1Monitor**: the "latency benchmark" is a user-mode thread at raised priority
  (`SetThreadPriority`). It repeatedly sleeps for a target time and records the **sleep overshoot**
  (mean, min, max, deviation, interrupt rate). The "check for driver latency spikes" verdict
  (`stutter.dpc`) uses the same proxy. Real kernel DPC/ISR capture through the NT Kernel Logger ETW
  session is "not implemented in this build". Per-game FPS (60 s: average, 1 % low, P99 frame time)
  runs the bundled PresentMon through the broker service, which is ETW. That is the same mechanism
  our `bench` uses, so BF6's anti-cheat would presumably block it too.
- **Tier1Stretch**: lists display modes (`EnumDisplaySettings`), applies with `ChangeDisplaySettingsEx`
  and reverts after 15 s unless kept. Custom resolutions are written as an EDID override under the
  monitor's registry key, or through `nvapi64.dll` on NVIDIA, with an adapter restart
  (`SetupDiCallClassInstaller`, `DIF_PROPERTYCHANGE` disable/enable).

## On a 2019 MacBook Pro under Boot Camp playing BF6

It is not harmful if you pick carefully, but most of it is either already in gamebooster-rs or
does nothing for BF6. Avoid the Power plan, C-states, Core parking and Power throttling entries: all
four mean more heat on a CPU that is heat-limited. Leave HVCI/VBS alone: it needs a reboot, it is a
security trade-off, and anti-cheats are moving toward requiring it. The always-on LocalSystem broker
and the Run entry are more resident software than these tweaks justify.

## Adoption items for gamebooster-rs

Ranked. Only items that make sense on the target Mac, can be judged by feel, and that `restore` can
undo count.

1. **Game DVR / Game Bar capture check**: `HKCU\System\GameConfigStore` `GameDVR_Enabled` and
   `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` `AppCaptureEnabled`. **Adopt as a
   read-only `status` line** that warns when background recording is on (like HAGS). Background
   recording keeps a video encoder running on the Radeon Pro 5300M. It is off by default, so a write
   step is only worth it if the warning ever fires.
2. **Toasts off while boosted**: `HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications`
   `ToastEnabled` = 0, old value restored. **Try by feel**: HKCU, reversible and harmless. It stops
   pop-ups stealing focus mid-match, but Focus/Do Not Disturb already does this, and it may need the
   notification service to pick it up.
3. **GPU driver age in `status`**: date from `Win32_VideoController.DriverDate`. **Try**:
   Boot Camp often ships old AMD drivers, and a line saying "driver is N months old" is cheap and
   changes nothing.
4. **Network adapter power saving** (`PnPCapabilities` = 24): **skip**. It is persistent, works only
   after an adapter reload (a short disconnect), and our plan's wireless-adapter setting already
   covers Wi-Fi power saving on AC.
5. **Disable MPO** (`OverlayTestMode` = 5): **skip**. It needs a reboot and fixes only flicker this
   Mac has not shown. Note it in a doc if flicker appears.
6. **HVCI / VBS off**: **skip**. It needs a reboot, it is a security trade-off, and anti-cheats
   increasingly expect it on. `restore` cannot undo it within a session.
7. **C-states off, core parking 100 %, Ultimate plan, machine-wide power throttling off**: **skip**.
   They all add heat. Core parking on AC is already 100 % in the High performance base of our plan,
   and we turn off power throttling per game.
8. **SysMain / WSearch disabled**: **skip**. They change start types, which DESIGN rules out; WSearch
   can already go in the `services` stop list.
9. **Standby-list cleaner, timer resolution, sleep-overshoot "DPC" check**: **skip**. The cleaner
   purges nothing in this build, timer resolution is already rejected ([ISLC](../islc/README.md)),
   and the overshoot test measures the timer, not the game.
10. **Nagle, network throttling index, GPU priority, SystemResponsiveness, QoS**: **skip**. These are
    TCP-only, already the default, or, by Tier1's own account, no effect.
11. **Game Mode, DNS, HAGS report**: already in gamebooster-rs.
