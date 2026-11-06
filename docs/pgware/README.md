# PGWARE GameBoost: what it really is

GameBoost (PGWARE LLC, pgware.com) is a small paid Windows "game and internet booster". You pick your
Windows version, processor and internet type, press **Optimize now**, and it writes about 30 registry
values, then asks you to restart. **Restore** puts some of them back. That is the whole product: no
background program, no per-game boost, no services, no power plan, no overlay, no driver.

Everything here comes from reading the shipped files (static reverse engineering) of the installer
downloaded on 2026-11-05 (installer version 4.6.8.2026, the programs inside say 3.10.21.2024).
**Nothing was run.** Technical notes: [../../decomp/pgware/gameboost/README.md](../../decomp/pgware/gameboost/README.md).

## Short version

- On Windows 10/11 almost every value it writes is already the Windows default, ignored since
  Windows Vista, aimed at Internet Explorer, or misspelled / in the wrong place. The only real
  scheduling change makes the foreground game get slightly **less** CPU priority.
- The paid "Maximum Boost" differs from the free "Boost" in one thing: how many HTTP connections
  Internet Explorer-era networking opens per server (8 instead of 4).
- The "Finding 3D graphics device… Finding modem information…" progress screen is fake: a timed
  animation that detects nothing. The processor choice isn't used at all.
- The updater downloads and runs a new installer as administrator over plain HTTP with no signature
  check. Nothing it ships is code-signed.
- Uninstalling does not undo the changes. Restore only truly restores 9 values; it deletes the rest,
  including anything you had set there before.

## Every part, in one table

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo** (does little or
nothing), **harmful**, **risky** (can break things, cost privacy or security, or trip anti-cheat),
**bloat**. Nothing in GameBoost touches game processes, so none of it is an anti-cheat risk.

| Feature / part | What it really does | Verdict |
|---|---|---|
| CPU priority (`Win32PrioritySeparation` = 1) | Changes the Windows default 2 (foreground programs get 3× longer CPU turns) to 1 (2×). The opposite of the "prioritize full-screen games" claim. | harmful (slightly) |
| Memory (`DisablePagingExecutive` 0, `LargeSystemCache` 0, `IoPageLockLimit` 4 KB) | First two are the Windows defaults; the third has been ignored since Windows Server 2003. | placebo |
| TCP (`Tcp1323Opts`, `TcpWindowSize`, `GlobalMaxTcpWindowSize` 256960, `DefaultTTL` 64, `SackOpts`, PMTU, `TcpMaxDupAcks`, keep-alive) | XP-era settings. Window sizes are ignored since Vista (Windows tunes them itself); the rest are defaults or make no speed difference. Same for every internet type except dial-up and ISDN. | placebo |
| QoS policy (`Psched\NonBestEffortLimit` 50, `MaxOutstandingSends` 65535) | Sets a machine policy that only affects programs reserving bandwidth through QoS. Games don't. | placebo |
| "Maximum Boost" (`MaxConnectionsPerServer` 4 / 8) | Connections per web server for Internet Explorer-style downloads. Nothing to do with games. Paid only. | placebo |
| Prefetch (`EnablePrefetcher` 1) | Written to the wrong registry key, so Windows never reads it. | placebo |
| Shell and IE (`SmoothScroll` off, `MinAnimate`, `Use StyleSheets`, `Append Completion`, shortcut-resolve policies) | Cosmetic or browser/Explorer behaviour; one turns off smooth scrolling in lists, one stops Windows searching for the target of a broken shortcut. | placebo |
| Legacy junk (`Messenger\Start` 4, `P3\HackFlags`, `Error Mode`, `ContigFileAllocSize`, `SendAlert`, `NoPopUpsOnBoot`, `BootOptimizeFunction`) | Services that no longer exist (it creates empty keys for them), a misspelled value (`Error Mode` instead of `ErrorMode`), and XP-era settings. | placebo (leaves junk) |
| Restart prompt | Every Optimize and Restore asks you to reboot. | placebo |
| Restore | Puts back 9 saved values; deletes the other 24 (on Windows 10/11), so anything you or your admin had set there is lost. Uses the OS/internet choice shown at Restore time. | risky (minor) |
| Uninstall | Leaves all changes in place. | risky (minor) |
| Fake progress screen | 33 "Finding …" steps of timed animation; no detection happens. | dark pattern |
| Updater (`GameBoostUpdate.exe`) | Checks every 45 days over HTTP; downloads the installer over HTTP and runs it as admin, no signature or hash check. Anyone on your network path could swap it. | risky (security) |
| Licensing (`GameBoostRegister.exe`, `/online`) | Timed trial; name + serial, machine-limited; checked online over HTTP on every start with a machine ID (disk volume serial); the server can revoke it. "You have been reported!" piracy scare; TrialPay "free serial for trying offers". | bloat, minor privacy cost |
| Installer | Inno Setup, admin, unsigned. No services, tasks, Run keys, drivers or bundled offers. | harmless |
| Telemetry | Only the licence and update checks above. No analytics or crash uploads. | harmless |

## On a 2019 MacBook Pro under Boot Camp playing BF6

Nothing here helps. None of the values affect GPU, power, thermals or the 800 MHz BD PROCHOT clock
lock (see [prochot](../tweaks/prochot.md)); the network values are ignored by Windows 10/11 and by
BF6's UDP traffic; the one scheduling change slightly lowers the game's foreground boost. It is also
harmless to BF6's anti-cheat (registry only, nothing injected), but there is no reason to install it,
and the HTTP updater is a real security hole.

## Compared with Smart Game Booster and gamebooster-rs

- **SGB** does real, temporary, per-session work (power plan, stopping services, closing programs,
  lowering other processes) and undoes it; GameBoost does none of that. GameBoost's registry list
  matches the kind of permanent "optimizer" tweaks in SGB's `.dbd` files, which gamebooster-rs
  already reviewed and declined (see [optimizer](../tweaks/optimizer.md)). SGB is far bigger, with
  much more telemetry, but its updater is HTTPS with hash and signature checks; GameBoost's is not.
- **gamebooster-rs**: nothing new worth adopting. Every GameBoost value is a default, obsolete,
  misplaced, or (for `Win32PrioritySeparation`) counter-productive. gamebooster-rs's game-priority
  step already does what GameBoost's "CPU prioritization" claim promises, directly and reversibly.
