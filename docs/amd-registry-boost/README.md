# windows-amd-registry-boost: what its AMD registry tweaks really do

windows-amd-registry-boost (<https://github.com/Userbash/windows-amd-registry-boost>, by "sanya"
(GitHub user Userbash), GPLv3) is a single PowerShell script, `main.ps1` ("AMD GPU Optimizer
v1.0"), that writes about 200 values into the AMD display driver's registry keys plus a few
Radeon Software (Adrenalin) settings under `HKCU`/`HKLM\Software\AMD`. It asks two questions
(overlay on or off; "5000 / 6000 / 7000 series" or "9000 series"), takes a `.reg` backup and
tells you to reboot. It claims "the most performance and stability" from "hidden driver
settings that are not available in the standard Adrenalin Software interface".

**Source read:** commit `55fd1bdc50b3700ab8203e01764ff5b5c3abe4a5` (2025-12-12), shallow clone,
read 2026-11-07: `main.ps1` (469 lines), `README.md`, `README.ru.md`, `LICENSE`. Nothing was run.
Checked against AMD's Adrenalin support article
[DH3-012](https://www.amd.com/en/resources/support-articles/faqs/dh3-012.html) and the reporting
on Anti-Lag+ bans ([Tom's Hardware](https://www.tomshardware.com/news/amd-pulls-drivers-that-cause-counter-strike-2-bans-after-valve-roasted-them),
[PC Gamer](https://www.pcgamer.com/amd-yeets-its-latest-gpu-driver-in-response-to-anti-lag-causing-bans-in-counter-strike-2)).
AMD does not document the driver's private registry values; where this page says what a value
does inside the driver, that is **inference** from the value names, from which INF / Adrenalin
writes them, and from tuning-community reports, and is marked as such.

Target judged: the user's GPU, **AMD Radeon Pro 5500M 8 GB (Navi 14, RDNA1)** in the 16-inch
2019 MacBook Pro, under Boot Camp, on Apple's Boot Camp AMD driver or a newer Adrenalin /
community (bootcampdrivers.com) driver, playing BF6 (DirectX 12, EA Javelin anti-cheat). On
that machine the 5500M drives the internal panel (inference from the usual Boot Camp setup: the
Intel iGPU is not used by Windows), so a display-breaking value means a black internal screen.

## Short version

- **Almost all of it is no-op folklore.** About 120 of the values are Catalyst / XP-era names
  (AGP, HyperMemory, 3DNow!, Truform, D3D9 caps, OpenGL options, the `ati2mtag` service) that a
  2019+ RDNA1 driver does not read, or that are already the INF default. BF6 is DirectX 12, which
  the D3D9 caps and OpenGL keys cannot touch.
- **The parts that are live are mostly harmful on a laptop**: power gating, SCLK deep sleep and
  stutter mode off, thermal auto-throttling off, power containment off, PCIe LTR forced to 1.
  Where the driver honours them (unknown on Navi: the SMU firmware and PowerPlay table enforce
  power and thermal limits; inference) they add idle and load heat, which on this Mac is the
  real limit (see [prochot research](../tweaks/prochot-research.md)). None makes a frame faster.
- **A few are ordinary Adrenalin settings** written behind the UI's back: Radeon Boost, Chill,
  FRTC and compute preemption off, Anti-Lag on (profile 1), tessellation and texture filtering
  overrides, ReLive (`DvrEnabled`) and the overlay off, AMD telemetry (AUEP) off. These you can
  set in Adrenalin itself, where they exist. Turning **Chill off** removes the one frame cap the
  [prochot research](../tweaks/prochot-research.md) found helpful on this Mac.
- **Its restore is not a real undo.** The backup is a `reg export` of the display class subkey
  only; `reg import` puts back old values but never deletes values the script created, so after
  "restore" every new value stays. The `HKCU\Software\AMD\*`, `HKLM\Software\AMD\Install`,
  `HKLM\Software\AUEP`, `HKCU\Software\ATI\ACE\...` and `Services\ati2mtag` / `amdwddmg` writes
  are not backed up at all. Choosing "apply again" overwrites the backup with the tweaked state.
  The restore point is created with `-ErrorAction SilentlyContinue` inside `try`, so "[OK]
  restore point created" prints even when Windows skipped it (Windows allows one restore point
  per 24 h by default).
- **Adapter detection is loose**: the first subkey of the display class
  `{4d36e968-e325-11ce-bfc1-08002be10318}` whose `DriverDesc` matches `AMD`, `Advanced Micro
  Devices`, `ATI` or `Radeon`, case-insensitive substring (`-match`), so any adapter name
  containing "ati" matches too. Only one adapter is tweaked.
- **Nothing in it is worth adopting as a boost step** for gamebooster. At most a read-only
  ReLive line in `status`.

## The tweaks

Columns: what is set; the claim (from the menu text and README, which give no per-value claims;
"perf" = the general "max performance" claim); what it really does on an RDNA1 driver; when it
takes effect; risk; does it apply on the Boot Camp 5500M. KMD = the display class subkey
`HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\NNNN`
(read by the kernel driver when it starts: needs a driver restart or reboot, inference from how
AMD's INF writes them under `HKR`). UMD = `NNNN\UMD` (read by the user-mode D3D driver when a
game creates its device: the next game start, inference).

### Power and clocks (KMD, DWORD)

| Value (set to) | Claim | What it really does on the 5500M | Takes effect | Risk | Verdict |
|---|---|---|---|---|---|
| `EnableUlps` = 0 | perf, stability | ULPS = Ultra Low Power State for idle secondary GPUs in CrossFire / hybrid setups. Real key, still in AMD INFs (inference). Single-GPU Mac: little or nothing; at most the GPU never enters its deepest idle state, more idle power. | reboot | heat, battery | folklore for one GPU; skip |
| `PP_SclkDeepSleepDisable` = 1 | perf | Stops the core clock dropping to deep sleep between bursts. If honoured: higher idle/low-load power and heat, no FPS under load (the clock is already high). | reboot | heat | harmful on this Mac |
| `PP_ThermalAutoThrottlingEnable` = 0 | perf | Name says "thermal auto-throttling off". Pre-Vega PowerPlay key; on Navi the SMU firmware does thermal throttling from the PowerPlay table, so likely ignored (inference). If honoured, it removes a thermal safeguard on a GPU sharing a heat pipe with the CPU. | reboot | **thermal / hardware** | never on the Mac |
| `PP_DisablePowerContainment` = 1 | perf | Power containment = the PowerTune power limit. Same as above: probably ignored on Navi (limit lives in the PPTable, which MorePowerTool edits via `PP_PhmSoftPowerPlayTable`); if honoured, more GPU watts, which is exactly the VRM/PROCHOT trigger on the 16-inch 2019 model. | reboot | **heat, PROCHOT, black screen** | never on the Mac |
| `PP_DisableVoltageIsland` = 1 | perf | Old PowerPlay voltage-island control. Likely not read on Navi (inference). | reboot | low | no-op |
| `PP_GPUPowerDownEnabled` = 1 | perf | Lets a hybrid laptop power the dGPU down when idle. The opposite direction of the rest of the list. Mac: the dGPU drives the panel, so nothing (inference). | reboot | none | contradictory no-op |
| `DisablePowerGating` = 1 | perf | Global power gating of idle GPU blocks off. If honoured: more idle and load leakage, heat. No FPS. | reboot | heat | harmful on this Mac |
| `DisableSAMUPowerGating` = 1 | perf | SAMU only exists on GCN 1.x/2.x (2013-2015). Not on Navi. | — | none | no-op |
| `DisableUVDPowerGatingDynamic` = 1, `DisableVCEPowerGating` = 0 | perf | UVD/VCE are the pre-Navi video blocks; Navi 14 has VCN 2.0. No game effect either way. | — | none | no-op |
| `DisableDrmdmaPowerGating` = 1 | perf | SDMA copy-engine power gating off. More power, no FPS. | reboot | heat | skip |
| `StutterMode` = 0 | "less stutter" | Display-controller memory "stutter" (self-refresh) power saving, not frame stutter. Off = more memory power at idle; community reports it as a flicker fix. Nothing for frame times. | reboot | heat | folklore (name confusion) |
| `EnableAspmL1` = 1, `EnableAspmL0s` = 0 | perf | GPU PCIe link power states. L1 **on** is a power-saving state, so it contradicts the "performance" goal; Boot Camp firmware decides ASPM anyway (inference). Our power plan already sets PCIe link state off. | reboot | low | no-op / contradictory |
| `DisableDMACopy` = 1, `DisableBlockWrite` = 0 | stability | Old copy-path switches; `DisableDMACopy` reportedly moves copies off the DMA engine (slower) and was a Catalyst-era flicker workaround. `DisableBlockWrite` = 0 is the default. | reboot | low (possible slowdown) | folklore |
| `LTRSnoopL1Latency`, `LTRSnoopL0Latency`, `LTRNoSnoopL1Latency`, `LTRMaxNoSnoopLatency` = 1 (profile "5000-7000"); deleted (profile "9000") | "aggressive" latency | PCIe Latency Tolerance Reporting: tells the platform how long the GPU can wait for memory. 1 (ns, presumably) means "never let the package sleep", which blocks deep package C-states; more heat on a heat-limited i9 laptop, no measured gain. Not default values (inference); the "9000" profile deleting them only matters if they existed. | reboot | heat, PROCHOT | folklore, harmful here |

### Adrenalin feature defaults (KMD, DWORD)

| Value (set to) | Claim | What it really does | Takes effect | Risk | Verdict |
|---|---|---|---|---|---|
| `KMD_RadeonBoostEnabled` = 0, `KMD_FRTEnabled` = 0 | perf | Global Radeon Boost (dynamic resolution) and Frame Rate Target Control off. Both are off by default; FRTC is DX9-11 only per AMD's older article. | driver restart / Adrenalin | none | default-equal |
| `KMD_ChillEnabled` = 0 | perf | Radeon Chill (frame cap / idle FPS limit) off globally. Default off, but it is the frame cap that helped on this Mac ([prochot research](../tweaks/prochot-research.md)); forcing it off fights that. | driver restart / Adrenalin | none, but loses a heat tool | skip on the Mac |
| `KMD_DeLagEnabled` = 1 ("5000-7000") / 0 ("9000") | lower input lag; "9000" fixes Discord/web lag | Global Radeon Anti-Lag. Real feature, available on RDNA1 in Adrenalin, helps only when GPU-bound. The plain driver-side Anti-Lag is not Anti-Lag+, which hooked game DLLs and caused CS2 VAC bans in 2023 (AMD pulled 23.10.1); still, set it per game in Adrenalin, not globally. | driver restart / Adrenalin | low | user setting, not a tweak |
| `KMD_EnableComputePreemption` = 0 | lower latency | Turns off compute preemption. If honoured: long compute work cannot be interrupted, so a long shader can hit the Windows GPU timeout (TDR: black flash, "driver stopped responding", game crash). No measured latency win. | reboot | **TDR / crash** | never |
| `AllowSnapshot`, `AllowSubscription`, `NotifySubscription` = 0 | perf | Meaning undocumented (names suggest Radeon Software notification / snapshot features). | unknown | unknown | unknown, skip |
| `AllowRSOverlay`, `AllowSkins` = "false" (overlay off) or "true" | "max FPS" | Radeon Software in-game overlay (Alt+R) and skins. Overlay off avoids one injected overlay; no FPS when the overlay is not shown. | Radeon Software restart | none | user setting |

### Legacy D3D9 / OpenGL / AGP values (KMD)

| Values | What it really does | Verdict |
|---|---|---|
| 31 `OGL*` DWORDs (`OGLWaitVerticalSync`, `OGLEnableTripleBuffering`, `OGLTruformMode`, `OGLDisableProgPCILatency`, ...), `UseNewOGLRegPath`, `OGLCustomSettings` | Catalyst-era OpenGL options. Modern OpenGL settings live under `UMD` / Adrenalin profiles (inference). BF6 is DX12. | no-op |
| `DirAnisoQuality`, `MultipleSizeAGP`, `AntiAlias`, `TemporalAAMultiplier`, `HyperMemory`, `HardwarePageFlip` | AGP (gone since PCIe), HyperMemory (2005 Radeon X300 era), old AA/AF control. | no-op |
| 38 REG_SZ "1" caps (`FastZClearEnabled`, `HierarchicalZEnable`, `ColorCompression`, `PureDevice`, `TclEnableBackFaceCulling`, `GL_S3_s3tc`, ...) and `ZCompressionMode` "3", `ZFormats` "7", `PixelCenter`, `AnisoDegree`, `AnisoType`, `AntiAliasSamples`, `ValidateVertexIndex`, `EnableUntransformedInLocalMem` "0" | D3D9-era capability strings. AMD INFs still carry many of them with these same values (inference from INF knowledge), so this mostly rewrites defaults; DX12 does not read them. | no-op |
| 23 `DALRULE_RESTRICT<WxH>MODE` = 0, `DALRULE_DISPLAYSRESTRICTMODES` = 0, `DALRULE_AUTOGENERATELARGEDESKTOPMODES` = 1 | Display Abstraction Layer rules that hide or auto-generate desktop modes. Off/on here mainly changes which resolutions the mode list shows; picking an odd generated mode on the internal panel can give a black or garbled screen. No performance meaning. | skip (display risk) |
| `Services\ati2mtag\Device3\OpenGL\private`: `Enable3DNow`, `EnableAGPTextures`, `EnableTCL`, ... = 1; same under `Services\amdwddmg\Device3\...` if that service exists | `ati2mtag` is the pre-2012 Catalyst service name; current drivers use `amdkmdag` (inference). `New-Item -Force` **creates** a stray `Services\ati2mtag` key on systems that never had it. 3DNow! is an AMD CPU instruction set removed in 2010 and never on Intel. | no-op, leaves junk |

### UMD 3D settings (`NNNN\UMD`, REG_BINARY UTF-16 digits, the format Adrenalin uses)

| Value (set to) | What it really does | Verdict |
|---|---|---|
| `Tessellation_OPTION` "2", `Tessellation` "1" | Most likely "override application settings" with maximum tessellation level 1x (inference from the Adrenalin UI and value mapping), so tessellation is effectively off. Lower GPU load in games that tessellate, visibly flatter geometry. AMD: lower levels "can reduce overhead" ([DH3-012](https://www.amd.com/en/resources/support-articles/faqs/dh3-012.html)). DirectX scope on DX12 not stated by AMD. | image-quality trade, Adrenalin setting |
| `TFQ` "2" | Texture Filtering Quality; AMD offers Standard (recommended default), High, Performance. "2" is probably Performance (inference). Tiny gain, slight shimmer. | Adrenalin setting |
| `ShaderCache` "2" | Shader cache mode (probably "on" vs "AMD optimized"; inference). Default is AMD optimized; no gain expected. | default-ish |
| `FlipQueueSize` "1" | Old Catalyst "max pre-rendered frames". Whether current drivers still read it is unverified; DX12 games manage their own frame queue. | folklore |
| `VSyncControl` "1", `AntiAlias` "1", `AntiAliasSamples`/`AnisoDegree`/`AnisoType` "0", `SurfaceFormatReplacements` "1", `TurboSync` (Enhanced Sync) "0", `EQAA` "0", `MLF` (morphological AA) "0", `AntiStuttering` "1", `EnableTripleBuffering` "0", `HighQualityAF` "1", `CatalystAI` "1", `Main3D`/`Main3D_DEF` "1", `GI`, `AAF`, `TextureOpt`, `TextureLod`, `ForceZBufferDepth`, `ExportCompressedTex`, `PixelCenter`, `SwapEffect`, `ASTT`/`ASD`/`ASE`, `PowerState` | Application-controlled / off values, mostly equal to Adrenalin's defaults (VSync "off unless application specifies", AA "use application", Enhanced Sync / EQAA / MLAA off). Some names (`CatalystAI`, `ASTT`) are Catalyst-era. | default-equal |
| `UMD\DXVA`: `MosquitoNoiseRemoval*`, `Deblocking*`, `3to2Pulldown`, `BlueStretch*`, `LRTCCoef`, `Fleshtone*`, `DynamicRange` "0", `StaticGamma_ENABLE`, `DynamicContrast_ENABLE`, `WhiteBalanceCorrection` | Video playback post-processing. Nothing for games; `DynamicRange` may change video black levels. | irrelevant |
| `UMD\DXC`: `AllowBoost` "1", `AllowDelag` "1" | Per-API allow flags for Radeon Boost / Anti-Lag (inference). Contradicts `KMD_RadeonBoostEnabled` = 0. | no-op |

### Radeon Software and telemetry (HKCU / HKLM\Software)

| Value (set to) | What it really does | Verdict |
|---|---|---|
| `HKCU\Software\AMD\DVR\DvrEnabled` = 0 (overlay off) or 1 | Radeon ReLive recording off/on (inference: the setting behind Adrenalin's Record & Stream switch). With Instant Replay on, the encoder runs in the background like Game DVR. Off is a real, small saving. | real, but a user setting |
| `HKCU\Software\AMD\CN`: `RSXBrowserUnavailable`, `CN_Hide_Toast_Notification`, `SystemTray`, `AllowWebContent` = "true"/"false", `AutoUpdateTriggered` = 0, `BuildType` = 0 | Radeon Software UI: built-in browser, toasts, tray icon, web content. `BuildType` = 0 is undocumented and may change which update channel Radeon Software thinks it is on (inference). No game effect. | cosmetic; `BuildType` unknown |
| `HKLM\Software\AMD\Install\AUEP` = 1, `HKLM\Software\AUEP\RSX_AUEPStatus` = 2 | AMD User Experience Program (telemetry) opt-out (inference from names). Normally chosen at install or in Adrenalin Preferences. Privacy, not FPS. | permanent preference, skip |
| `HKCU\Software\ATI\ACE\Settings\ADL\AppProfiles\AplReloadCounter` = 0 | A counter the old ATI ACE / ADL stack uses to decide when to reload app profiles (inference). Resetting it does nothing for games. | no-op |

## On a 2019 MacBook Pro under Boot Camp playing BF6

- **Which key it would hit.** The Mac usually has one AMD adapter at display class subkey `0000`,
  but after a driver reinstall (Apple's package swapped for a community or AMD Boot Camp driver)
  the active one can be `0001` or higher while the old subkey lingers. The script takes the
  **first** matching `DriverDesc`, which can be the stale subkey, so its writes may land on a
  driver instance that is not in use.
- **Apple's Boot Camp driver** is an older AMD branch with Apple's INF; Adrenalin features
  (Boost, Chill, Anti-Lag, ReLive) may be missing or different, and the `KMD_*` / `HKCU\Software\AMD`
  values then do nothing (inference: the driver only reads names its build knows). Newer
  community/Adrenalin drivers on the 5500M read more of them.
- **RDNA1 (Navi 14)**: SAMU, UVD, VCE, AGP, HyperMemory, Truform and 3DNow! keys cannot apply.
  The power keys (`PP_*`, power gating, deep sleep, LTR) are the ones that might be honoured, and
  every one of them adds heat to a machine whose CPU and GPU share cooling and whose VRM triggers
  BD PROCHOT under combined load ([prochot research](../tweaks/prochot-research.md)). Expect lower
  sustained clocks, not higher FPS.
- **BF6 is DX12**: the D3D9 caps, OpenGL keys, DXVA video keys and FRTC do not touch it. Of the
  UMD overrides, only tessellation and texture filtering could plausibly change BF6 (DX12 scope
  not confirmed by AMD), and both are visible image-quality trades the user can set per game in
  Adrenalin.
- **Anti-cheat**: registry values the AMD driver reads are not game files, and no injection is
  involved, so Javelin is not expected to care (inference; untested). The anti-cheat-relevant
  AMD history is Anti-Lag+, which this script does not touch.
- **Recovery**: a bad display value (DAL rules, power tables, compute preemption) on the Mac means
  a black or crashing internal screen with no iGPU fallback. Getting back needs WinRE / Safe Mode
  (three interrupted boots), then the `.reg` import, which still leaves the created values, so in
  practice a clean driver reinstall (DDU + reinstall the Boot Camp/community driver).
- **Never apply on the Mac**: `PP_ThermalAutoThrottlingEnable` = 0, `PP_DisablePowerContainment`
  = 1, `KMD_EnableComputePreemption` = 0, `DisablePowerGating` / `PP_SclkDeepSleepDisable` /
  LTR = 1, the DAL mode rules, and `KMD_ChillEnabled` = 0 if you rely on Chill as a frame cap.
  In short: do not run this script on the Mac.

## Adoption items for gamebooster-rs

None of these is built; implementation is the user's decision.

How it would be done, for any item that writes a display-driver value: find the active adapter's
subkey from the device, not by scanning names: enumerate present display devices
(SetupAPI `SetupDiGetClassDevs` with `GUID_DEVCLASS_DISPLAY`, `DIGCF_PRESENT`), keep the one with
hardware ID `PCI\VEN_1002`, and open its driver key with `SetupDiOpenDevRegKey(..., DIREG_DRV)`
(or read its `Driver` property, `{4d36e968-...}\NNNN`). Record in the state file the subkey,
whether each value existed and its old type and data; restore deletes created values and
writes back old ones. KMD values also need a driver restart (disable/enable the device, or
`pnputil /restart-device`), which blanks the only screen on the Mac for a few seconds and can
fail; that alone rules out KMD writes as a boost step.

1. **Maybe (read-only): ReLive recording line in `status`.** Read
   `HKCU\Software\AMD\DVR\DvrEnabled` and report "AMD ReLive recording: on | off | not present",
   next to the Game DVR line. Needs confirming on the Mac which driver writes it and what 1 means
   (Instant Replay vs just enabled). Zero risk; helps only if ReLive is on. No write, no restart.
2. **Maybe (docs only): point to Adrenalin per-game settings** for BF6 on the 5500M: Chill or
   in-game cap (already our advice), Anti-Lag per game, ReLive off. No code; where the Boot Camp
   driver lacks Adrenalin, the in-game cap is the answer.
3. **Skip: every KMD power value** (`EnableUlps`, `PP_*`, power gating, `StutterMode`, ASPM, LTR).
   Heat on a heat-limited Mac, unknown whether Navi honours them, needs a driver restart, and two
   remove thermal/power safeguards.
4. **Skip: `KMD_EnableComputePreemption` = 0.** TDR and crash risk, no measured gain.
5. **Skip: `KMD_*` feature flags (Boost, Chill, FRTC, Anti-Lag).** User settings in Adrenalin;
   driver restart to change from the registry; Chill off works against our frame-cap advice.
6. **Skip: UMD tessellation / texture filtering overrides.** Image-quality choices the user makes
   per game; writing them globally behind Adrenalin's back can be overwritten by Adrenalin's own
   profile store and is a visible downgrade, not a boost.
7. **Skip: legacy D3D9/OpenGL/AGP/DXVA values, DAL rules, `ati2mtag` keys, AUEP, `CN` UI values,
   `AplReloadCounter`.** No-op for a DX12 game, display risk (DAL), or permanent preferences.
8. **Lesson, not a feature: undo must delete what it created.** The script's `reg export` /
   `reg import` backup shows the failure our state file avoids: record absence, not just old
   values, and cover every key touched.

PowerPlay "maximize performance" in our power plan (ATI graphics power setting, see
[power plan](../tweaks/power-plan.md)) is already the supported, restorable way to keep the GPU
from downclocking; nothing here improves on it.
