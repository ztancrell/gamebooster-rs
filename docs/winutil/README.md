# Chris Titus WinUtil: the gaming-relevant tweaks

WinUtil (<https://github.com/ChrisTitusTech/winutil>, MIT) is a PowerShell/WPF "Windows utility":
app installer, debloat tweaks, preferences, power plans, DNS, Windows ISO builder. Most of it is
privacy and debloat. Only the parts that can affect a game are covered here. It is the tool whose
"Ultimate Performance" plan was active on the target Mac when the ~2.29 GHz cap was found (see
[DESIGN.md](../../DESIGN.md#decided), 2026-11-05).

**Source read:** commit `2ef5e5dc7852634bc40ad789a3d1115ee3913c41` (2026-11-03), shallow clone,
read 2026-11-06: `config/tweaks.json` (66 entries), `functions/public/Invoke-WPFUltimatePerformance.ps1`,
`functions/private/Invoke-WinUtilTweaks.ps1`. Nothing was run.

## Short version

- Tweaks are data: each entry in `config/tweaks.json` lists registry values with a `Value` and an
  `OriginalValue`, plus optional `InvokeScript` / `UndoScript` lines. **Undo writes the fixed
  `OriginalValue` (the Windows default), not what you had before.** So "undo" is "reset to
  default", and it is not session-scoped: tweaks stay until you undo them.
- Gaming-relevant entries: Ultimate Performance plan, MPO, Game Mode, Visual Effects, Mouse
  Acceleration, Sticky Keys, Delivery Optimization, Teredo / IPv6, Background Apps, svchost
  splitting, Services to Manual. No timer resolution, Nagle, MMCSS, `NetworkThrottlingIndex`,
  `Win32PrioritySeparation`, HAGS or Game DVR entries at this commit (one appx-removal path sets
  `GameDVR\AppCaptureEnabled = 0` when the Xbox apps are removed).
- **The "Ultimate Performance - Disable" button runs `powercfg /restoredefaultschemes`.** That
  deletes every custom plan, including gamebooster's plan if you are boosted, and leaves Balanced
  active. Run `gamebooster restore` before pressing it.
- The plan button at this commit is a plain duplicate of the built-in Ultimate Performance
  template; nothing in it sets *Maximum processor state* to 80 %. The 80 % cap found on the target
  Mac came from somewhere else (an older WinUtil, ThrottleStop, or a manual edit); it is not in this
  code.

## Per tweak

All are driver-free and none touches the game process, so the anti-cheat risk for BF6 is nil
unless noted. "Heat" is on the heat-limited i7-9750H.

| Tweak | Exact change (apply / undo) | Reversible? | Heat | Evidence / notes |
|---|---|---|---|---|
| Ultimate Performance plan ("Performance Plans - NOT FOR LAPTOPS") | `powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61`, then `/setactive` the new GUID. Undo: `powercfg /restoredefaultschemes`. | Only by resetting all plans (custom plans lost). | High: no idle parking, minimum state 100 %, timers as on a workstation. | Ultimate is High performance with fewer power-saving timers; little difference from High performance in games. Its own label says not for laptops. Our plan (High performance copy, restored on `restore`) covers it. |
| Multiplane Overlay (combo) | *Enabled*: delete `HKLM\SOFTWARE\Microsoft\Windows\Dwm\OverlayTestMode` and `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\DisableOverlays`. *Disabled (Compatibility)*: `OverlayTestMode = 5`. *Fully Disabled*: also `DisableOverlays = 1`. | Yes (back to "no value" = Windows default). | None | WinUtil says "applied immediately"; PC-Tuning and most guides say reboot. Fixes flicker / stutter / black screens on some multi-monitor and AMD/NVIDIA driver combinations; it can also cost: Independent Flip and overlay planes are what make borderless windowed as fast as fullscreen. Fix for a symptom, not a speed-up. |
| Game Mode (toggle) | On: `HKCU\Software\Microsoft\GameBar\AllowAutoGameMode = 1` and `AutoGameModeEnabled = 1`. Off: both `0`. | Yes | None | Same as ours except the extra `AllowAutoGameMode` (an older value; Settings' Game Mode switch is `AutoGameModeEnabled`). We write only `AutoGameModeEnabled` and record the old value. |
| Visual Effects - Set to Best Performance | `HKCU\Control Panel\Desktop`: `DragFullWindows = "0"`, `MenuShowDelay = "200"`; `...\WindowMetrics\MinAnimate = "0"`; `HKCU\Control Panel\Keyboard\KeyboardDelay = "0"`; Explorer `ListviewAlphaSelect`, `ListviewShadow`, `TaskbarAnimations = 0`; `VisualEffects\VisualFXSetting = 3` (custom); `HKCU\Software\Microsoft\Windows\DWM\EnableAeroPeek = 0`; more. Undo writes fixed defaults. | Only to defaults | None | Desktop snappiness, nothing in a fullscreen game. |
| Mouse Acceleration (toggle) | `HKCU\Control Panel\Mouse`: on `MouseSpeed = "1"`, `MouseThreshold1 = "6"`, `MouseThreshold2 = "10"`; off all `"0"`. | Yes | None | BF6 reads raw input, so the Windows pointer curve does not apply in game. Desktop preference only. |
| Sticky Keys (toggle) | `HKCU\Control Panel\Accessibility\StickyKeys\Flags`: `506` (on) / `58` (off). | Yes, persistent | None | We already turn the hotkeys off session-only via `SystemParametersInfo` (see [accessibility shortcuts](../tweaks/accessibility-shortcuts.md)). |
| Delivery Optimization - Disable | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeliveryOptimization\DODownloadMode = 0`; undo deletes it. | Yes | Less | Stops peer-to-peer update upload/download from using the line. A per-session version is stopping `DoSvc` (GameShift does this). |
| Teredo - Disable | `HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters\DisabledComponents = 1`, `netsh interface teredo set state disabled`; undo `= 0`, `state default`. | Yes | None | Breaks Xbox Live party / NAT on some setups; no effect on BF6's IPv4 UDP. `DisabledComponents` needs a reboot. |
| IPv6 - Set IPv4 as Preferred | `DisabledComponents = 32` (0x20). | Yes | None | Only changes address selection order; reboot. No gaming effect when the game server is IPv4. |
| Background Apps - Disable | `HKCU\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications\GlobalUserDisabled = 1`. | Yes | Less | Windows 10 era switch; Store apps only. Small. |
| svchost.exe Splitting - Disable | `HKLM\SYSTEM\CurrentControlSet\Control\SvcHostSplitThresholdInKB` = installed RAM in KB; undo `3670016`. | Yes, reboot | None | Fewer processes, less memory; no frame-time effect shown. Makes a hung service take others with it. |
| Services - Set to Manual | Changes start types of a long service list to Manual. | Only to its listed defaults | Less | Against our rule (start types are never changed). |
| Hibernation - Disable | `powercfg /hibernate off`, `HibernateEnabled = 0`. | Yes | None | Frees disk space; no gaming effect. Our plan already sets never hibernate while boosted. |

## On the target Mac

- WinUtil's Ultimate plan is the wrong tool on this Mac: it is "NOT FOR LAPTOPS" by its own label,
  and its undo wipes all custom plans. gamebooster's own plan already does the useful part and puts
  the old plan back.
- If the plan list ever looks wrong after WinUtil, `gamebooster status` shows the active plan and
  warns when *Maximum processor state* is below 100 %.
- MPO is the one WinUtil item worth knowing for BF6: if the game flickers, shows black frames or
  stutters in borderless mode, *Disabled (Compatibility)* and a restart is the standard fix.

## Adoption items for gamebooster-rs

1. **Adopt (docs + `status`): warn about `powercfg /restoredefaultschemes`.** In the README /
   [power plan](../tweaks/power-plan.md) doc: run `gamebooster restore` before WinUtil's
   "Ultimate Performance - Disable" (or any "reset power plans"), because it deletes our plan. No
   code needed: `restore` already copes (it skips deleting a plan that is gone, and activates
   Balanced if the old plan was a custom one that was wiped too).
2. **Adopt (`status`, read-only): report MPO state** (`OverlayTestMode`, `DisableOverlays`), like
   HAGS: `MPO: default | disabled (OverlayTestMode=5) | fully disabled`. Helps explain flicker or
   stutter; changes nothing.
3. **Try-by-feel (docs only): MPO off as a manual fix** with the exact `reg add` / `reg delete`
   lines and "restart Windows", for flicker or borderless stutter. Not a boost step: it needs a
   restart, so restore could not undo it in-session.
4. **Try-by-feel: `DoSvc` in the suggested services list** (stopped while boosted, restarted by
   restore). No heat; helps only if updates are being shared at that moment.
5. **Skip: Ultimate Performance plan.** Our High-performance-based plan with restore is the same
   thing for games, without wiping plans.
6. **Skip: Game Mode's `AllowAutoGameMode`.** Ours writes `AutoGameModeEnabled`, the Settings
   switch, and Game Mode verified on in `status` on the target Mac.
7. **Skip: Visual Effects, Mouse Acceleration, Sticky Keys flags, Teredo/IPv6, Background Apps,
   svchost splitting, Services to Manual, Hibernation.** Desktop-only, already covered, permanent,
   or reboot-only.
