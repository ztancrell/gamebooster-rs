# SGB's other optimizer tweaks

Besides Boost/Restore, SGB ships an "optimizer" / "system tweak" page of one-off registry values
(51 in `SysSpeed.ini`, 48 in `GB3.ini`, plus the `Opt.dbd` profiles), decoded from its `.dbd` data files. The full table, with a gaming verdict for each, is in
[decomp/SgbData/dbd/README.md](../../decomp/SgbData/dbd/README.md).

gamebooster builds none of them. The only ones it touches at all are read-only checks in
`gamebooster status`: hardware-accelerated GPU scheduling, multiplane overlays (MPO) and Game DVR
background recording (the last two from the WinUtil / PC-Tuning / Tier1Suite research).

## Why none

Most are placebo on Windows 10/11: XP-era memory and TCP keys that Windows ignores, UI delays,
visual effects, NTFS and IE settings. None of them change frame rate or frame times in a game.

`Win32PrioritySeparation` = 0x26 (SGB's "foreground CPU priority") equals the Windows default.
Microsoft documents the value as a 6-bit mask (interval length, fixed/variable, foreground boost)
and the client default 2 (000010) as "shorter, variable intervals" with a 3:1 foreground boost;
0x26 (100110) is what the "Adjust for best performance of: Programs" option writes, and it means
the same thing ([Win32PrioritySeparation, Microsoft Learn](https://learn.microsoft.com/en-us/previous-versions/cc976120(v=technet.10))).
Since 2026-11-06 boost does set a different value, `0x2A` (fixed quantum), while boosted, as a
try-by-feel item with restore; see [priority](priority.md#win32priorityseparation).

Harmful ones (permanent, and not worth it):

- Disabling services (Start=4): permanent unlike our boost-time stop; SENS, seclogon and
  TabletInputService can break login, "Run as" and touch/pen input.
- `Dnscache` Start=4: turns off the DNS Client cache (Windows may refuse; lookups get slower).
- `LargeSystemCache` = 1 (server profile): starves apps of memory on a desktop.
- `HiberbootEnabled` = 1 (fast startup): can leave NTFS volumes and the clock in a bad state when
  dual-booting macOS under Boot Camp.
- `AutoEndTasks` = 1: kills apps at shutdown without asking, losing unsaved work.
- `NtfsMftZoneReoffoation` / `WaitToKilloffoiceTimeout`: SGB's typos; they write junk values.

## Hardware-accelerated GPU scheduling (HAGS)

The one that may matter. It lets the GPU manage its own memory scheduling, which can lower latency
a little. It needs a reboot and a driver that supports it (WDDM 2.7 or newer). Boot Camp's AMD
drivers are old, so it may be missing. That is why `boost`/`restore` don't touch it; `status` prints:

```
GPU scheduling (HAGS): on | off | not set (Windows default)
```

(from `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers` value `HwSchMode`: 2 = on, 1 = off.)

Turn it on by hand: Settings > System > Display > Graphics > Default graphics settings (on Windows 11:
Advanced graphics settings) > Hardware-accelerated GPU scheduling = On, then reboot. If the switch
isn't there, the driver doesn't support it; stop here.

Undo: the same switch to Off, then reboot.

Measure it as in [How to measure any tweak](README.md#how-to-measure-any-tweak) (3 runs each way,
compare 1 % lows). Keep it only if it helps.

## Multiplane overlays (MPO)

Read only. `status` prints:

```
MPO: on (default) | off (OverlayTestMode=5) | off (DisableOverlays=1)
```

(from `HKLM\SOFTWARE\Microsoft\Windows\Dwm` value `OverlayTestMode`, 5 = MPO off, and
`HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers` value `DisableOverlays`, 1 = off; any
other value is printed as "on (...)" with both values.) Turning MPO off is a known fix for
flicker or black screens with some drivers, and PC-Tuning mentions it as a way out of a stuck
"Hardware Composed: Independent Flip" presentation; it is not a frame-rate tweak and needs a
reboot, so boost doesn't touch it. By hand: `reg add "HKLM\SOFTWARE\Microsoft\Windows\Dwm" /v
OverlayTestMode /t REG_DWORD /d 5 /f`, reboot; undo with `reg delete` of the same value, reboot.

## Game DVR background recording

Read only. `status` prints `Game DVR background recording: on | off (GameDVR_Enabled=..,
AppCaptureEnabled=.., HistoricalCaptureEnabled=..)`, from `HKCU\System\GameConfigStore` value
`GameDVR_Enabled` and `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` values
`AppCaptureEnabled` and `HistoricalCaptureEnabled` ("absent" when a value doesn't exist). It counts
as on when `HistoricalCaptureEnabled = 1` and neither of the other two is 0, and then warns:
background recording keeps encoding the last minutes of play (GPU encoder, disk writes). Turn it
off in Settings > Gaming > Captures > "Record in the background while I'm playing a game" = Off.
On the target Mac all three are 0 (2026-11-06), so it prints off.
