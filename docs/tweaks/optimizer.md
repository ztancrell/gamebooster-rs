# SGB's other optimizer tweaks

Besides Boost/Restore, SGB ships an "optimizer" / "system tweak" page of one-off registry values
(51 in `SysSpeed.ini`, 48 in `GB3.ini`, plus the `Opt.dbd` profiles), decoded from its `.dbd` data files. The full table, with a gaming verdict for each, is in
[decomp/SgbData/dbd/README.md](../../decomp/SgbData/dbd/README.md).

gamebooster builds none of them. The only one it touches at all is a read-only check of
hardware-accelerated GPU scheduling in `gamebooster status`.

## Why none

Most are placebo on Windows 10/11: XP-era memory and TCP keys that Windows ignores, UI delays,
visual effects, NTFS and IE settings. None of them change frame rate or frame times in a game.

`Win32PrioritySeparation` = 0x26 (SGB's "foreground CPU priority") equals the Windows default.
Microsoft documents the value as a 6-bit mask (interval length, fixed/variable, foreground boost)
and the client default 2 (000010) as "shorter, variable intervals" with a 3:1 foreground boost;
0x26 (100110) is what the "Adjust for best performance of: Programs" option writes, and it means
the same thing ([Win32PrioritySeparation, Microsoft Learn](https://learn.microsoft.com/en-us/previous-versions/cc976120(v=technet.10))).

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
