# ISLC: what it really does

Intelligent Standby List Cleaner (ISLC), by Wagnardsoft (the author of DDU), is a small free
Windows tool. Gamers run it in the background to "fix stutter". It empties Windows' standby memory
list when free memory gets low, and it can hold a fine timer resolution. This page comes from
reading the official ISLC 1.0.4.8 package (static reverse engineering, 2026-11-05). **Nothing was
run.** Technical notes: [../../decomp/islc/README.md](../../decomp/islc/README.md).

## Short version

- **What you install:** a signed (Wagnardsoft, Sectigo) portable 7-Zip self-extractor or an NSIS
  installer. Inside is one elevated .NET Framework 4.8 GUI program plus a helper DLL. There is no
  service, no driver, and nothing is injected into games.
- **Purge:** `NtSetSystemInformation(SystemMemoryListInformation, MemoryPurgeStandbyList)` with
  `SeProfileSingleProcessPrivilege`. It throws away the **entire** standby list (all priorities),
  the same as RAMMap's "Empty Standby List".
- **When:** it checks every **1000 ms**. It purges when the standby list is **≥ 1024 MB** *and*
  "free" memory (free + zeroed pages, not Task Manager's "Available") is **< 1024 MB**, unless a
  process on the exclusion list is running. With these defaults it purges whenever Windows has
  done its normal job of turning free RAM into cache, which is often.
- **Timer resolution:** off by default. When on, it holds `NtSetTimerResolution` at the chosen
  value (default 1.0 ms) while monitoring runs, and releases it on stop, exit or sleep. It exempts
  itself from Windows 11's "ignore timer requests of hidden windows" rule. An optional checkbox
  sets `GlobalTimerResolutionRequests` = 1 (needs a reboot) so its request applies to every process
  again on Windows 11. Unticking the box deletes the value.
- **Odd part:** for each timer change, it impersonates a SYSTEM token taken from some other process.
  To find one, it tries full-access opens of running processes, which may include your game.
  This does nothing useful; the timer request is per process either way.
- **Autostart:** optional Task Scheduler task "Intelligent StandbyList Cleaner" (logon of any user,
  30 s delay, highest privileges). Off by default.
- **Network:** one update check at start (`wagnardsoft.com/api/islc/version.json`), which only
  shows a notice. **No telemetry**, no account, no ads.

## Features and verdicts

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky**,
**bloat**.

| Feature | Default | What it does | Undo | Verdict |
|---|---|---|---|---|
| Automatic standby purge | off until Start (or "start minimized") | Purges the whole standby list when standby ≥ 1024 MB and free < 1024 MB, every 1 s check | Nothing to undo; the cache is gone and refills from disk | **placebo** on current Windows; **harmful** when it fires during play (re-reads from disk) |
| Manual Purge button / Ctrl+Alt+P hotkey / `-purge` | hotkey off | One purge | as above | **placebo**, occasionally a diagnostic |
| Exclusion list | empty | Pauses automatic purging while listed programs run | n/a | sensible guard (it works the opposite way to what many guides say: listed games stop the purging) |
| Custom timer resolution | off, 1.0 ms | Holds `NtSetTimerResolution` while monitoring | Released on stop, exit, sleep; the kernel drops it if ISLC dies | **placebo** on Windows 10 2004+ and 11 (see below); costs idle power |
| GlobalTimerResolutionRequests (Win 11) | unticked | Registry value so one process's request is global again (after a reboot) | Unticking **deletes** the value, even one you set yourself | **placebo**/power cost; affects every app on the system |
| SYSTEM-token impersonation | with timer only | Full-access opens of processes until a SYSTEM token is found | n/a | **risky** in principle (pointless handle opens, maybe on the game), harmless in practice |
| Autostart task | off | Logon task, highest privileges | Untick deletes it | fine if you want it |
| Update check | always | HTTPS GET of a version file | n/a | fine |

## Does purging the standby list help gaming?

**Short answer: not on current Windows 10/11, except in a rare buggy case, and it can cost you.**

- **What the standby list is.** Pages that were recently used but are no longer in any working set:
  file cache, and pages trimmed from processes. They count as "Available". When something needs
  memory, Windows hands out free/zeroed pages and then repurposes standby pages, lowest priority
  first. This happens in the page-fault path and costs about as much as using a free page, since the
  page is zeroed either way. A large standby list is not "used" memory, and a game asking for RAM
  does not wait for a purge.
  Sources: Microsoft's [RAMMap](https://learn.microsoft.com/sysinternals/downloads/rammap)
  documentation of the page lists, and *Windows Internals* 7th ed., ch. 5 (page list management).
- **Evidence for.** In 2017–2018, many users on Windows 10 1703–1809 reported stutter with a huge
  standby list and "free" memory near zero, in PUBG, Destiny 2 and Battlefield 1. Purging fixed it
  for them. Sources:
  - [Reddit r/Windows10 thread (2017)](https://redlib.ssps.io/r/Windows10/comments/7a0763/standby_memory_issue_causing_stutters_on_creators);
  - [Guru3D: Fix game stutter on Win 10 1703-1809](https://forums.guru3d.com/goto/post?id=5665691);
  - [AddictiveTips](https://www.addictivetips.com/windows-tips/empty-standby-memory-fix-game-stuttering-windows-10);
  - [EA forums, Battlefield](https://forums.ea.com/discussions/battlefield-franchise-discussion-en/standby-memory-issue---stuttering-and-lags-on-multiplayer/6738495).

  ISLC was written for exactly this ("may help users who have stutters in games when using windows
  10 Creator update", in its release post). There is no controlled benchmark, and no Microsoft
  acknowledgement or fix note was found. Later reports say 20H1 and newer no longer need it, but
  this is unverified.
- **Evidence against.**
  - The memory manager's design is above.
  - No independent frame-time measurement shows a gain on Windows 10 21H2+ or Windows 11.
  - On a machine that is not short of RAM, the condition "free < 1 GB" is normal, so ISLC purges
    over and over for nothing.
- **When it hurts.**
  - Every purge throws away cached files: game assets, shaders, DLLs and the browser. Whatever needs
    them next reads them from disk again. That means hard faults, and map loads or alt-tabs that are
    slower, not faster.
  - A purge during a match can itself cause a hitch. With the default thresholds it can fire
    mid-game: users report a "moment of freeze" when it runs.
  - With an SSD the cost is small but never negative; with a hard drive it is large.
  - The exclusion list exists precisely to stop purging while the game runs.

**Verdict:** placebo on current Windows; harmful when it purges during play or on a hard drive. It
is useful only as a diagnostic if you really see a full standby list with ~0 free memory *and*
stutter. In that case the real fix is more RAM, or closing what fills it.

## Does a timer request from another process affect the game on Windows 11?

**No, unless you set `GlobalTimerResolutionRequests`. And games set their own resolution anyway.**

- Since Windows 10 version 2004, `timeBeginPeriod`/`NtSetTimerResolution` "no longer affects global
  timer resolution". Only processes that asked get the finer resolution. Since Windows 11, a
  window-owning process that is minimised or fully covered does not get it either
  ([Microsoft: timeBeginPeriod](https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod);
  opt-out flag `PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION` in
  [SetProcessInformation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessinformation);
  background in Bruce Dawson's
  ["Windows Timer Resolution: The Great Rule Change"](https://randomascii.wordpress.com/2020/10/04/windows-timer-resolution-the-great-rule-change/)).
- So on Windows 10 2004+ and 11, ISLC's request changes only ISLC's own timers. ISLC exempts
  itself from the hidden-window rule, so its request survives in the tray, but it is still only
  its own.
- `GlobalTimerResolutionRequests` = 1 (Windows 11 / Server 2022+, read at boot) restores the old
  global behaviour. Sources:
  - [valleyofdoom/TimerResolution](https://github.com/valleyofdoom/TimerResolution), a public
    research tool that documents the value;
  - the Great Rule Change comments;
  - [Wagnardsoft forum thread on the option](https://www.wagnardsoft.com/forums/viewtopic.php?p=12471).

  Only then would ISLC's request reach the game.
- **Does the game care?**
  - Games that sleep in their frame limiter or render loop call `timeBeginPeriod(1)` themselves, as
    most engines do. Those already get 1 ms while in front, and a global request adds nothing.
  - A finer request (0.5 ms) can make `Sleep`-based frame caps a little more precise. Gains in forum
    tests are within noise, and nobody has published a frame-time study for BF6.
  - Costs: more timer interrupts, more idle power and heat (Microsoft's own remark: it "can also
    reduce overall system performance" and "prevent the CPU power management system from entering
    power-saving modes").
- **Verdict:** placebo on Windows 11 by default. With the global switch it is a system-wide power
  cost for a gain no one has measured. Same verdict as Hone's timer item
  ([Hone](../hone/README.md)).

## For a 2019 MacBook Pro under Boot Camp playing BF6

The machine: 16 GB RAM (base for the 2019 15"/16" models), an Intel 9th-gen CPU with the BD PROCHOT
800 MHz problem, an AMD Radeon Pro 5300M/5500M, an Apple NVMe SSD, and Windows 10 or 11 via Boot
Camp.

- **RAM.** BF6 lists 16 GB as its recommended amount. With BF6, Steam/EA app, Discord and a browser,
  free+zeroed memory will sit under 1 GB most of the time. ISLC with the defaults would then purge
  over and over during matches, each time dropping cached game data that the SSD must re-read.
  - The SSD makes that cheap, but it is never a gain.
  - If you see real paging (hard faults/sec high in Resource Monitor), the fix is closing apps. That
    is what gamebooster-rs's close-apps step does, not purging cache.
- **CPU.** ISLC's 1-second poll is negligible, even at 800 MHz.
- **Timer.**
  - The custom timer adds timer interrupts on a heat-limited CPU. With `GlobalTimerResolutionRequests`
    it applies to every process.
  - Skip it. BF6 sets whatever resolution it needs.
  - To check, run `powercfg /energy` (as admin, while the game runs). The report lists every process
    holding a timer request ("Platform Timer Resolution: Outstanding Timer Request").
- **EA Javelin.**
  - ISLC does not inject into or read BF6. The purge is a system call that does not touch the game
    process.
  - The exclusion monitor uses a process snapshot and WMI start/stop events without opening the game.
  - The one interaction is the SYSTEM-token search during timer changes. It may try a full-access
    `OpenProcess` on `bf6.exe` if the game comes before a SYSTEM process in the list. Javelin
    protects the game process, so the open is expected to be denied or stripped. That is harmless,
    but it is exactly the kind of handle access anti-cheats log.
  - ISLC is a signed, well-known tool and there are no reports of bans for it. Untested here.
  - If you use ISLC at all, leave the timer option off; then this never happens.
- **Overall:** nothing on this Mac needs ISLC. If you want to try it as an experiment:
  - add `bf6.exe` to its exclusion list (no purging mid-match);
  - leave the timer off;
  - compare with `gamebooster bench`.

## Compared with gamebooster-rs's RAM trim, Hone and SGB

- **gamebooster-rs RAM trim** ([ram-trim](../tweaks/ram-trim.md)) calls `EmptyWorkingSet` once on
  other processes when boosting. Their pages move to the standby/modified lists, so they stay cached
  and come back cheaply. **ISLC does the opposite end:** it throws away the standby list itself,
  including pages that a trim just put there.
  - Running both is the worst combination. First the trim moves background apps' pages to standby,
    then ISLC deletes them, so those apps must reload from disk.
  - Neither helps unless memory is truly short.
- **Hone** has no standby purge in Game Mode. It does hold the maximum timer resolution for the
  session and sets `GlobalTimerResolutionRequests`, deleting it on stop. That is the same mechanism
  as ISLC's checkbox, with the same "deletes your own value" problem. Judged placebo there too.
- **SGB** "clears RAM" with `SetProcessWorkingSetSize(-1,-1)` (a working-set trim, not a standby
  purge) and has no timer-resolution code ([spec evidence](../spec.md); [SGB](../sgb/README.md)).
- **ISLC is the most honest of them:**
  - it does one thing, says what it does, and ships debug symbols;
  - it has no telemetry and no permanent system changes, except the opt-in registry value and task.
  
  Its premise dates from a 2017-era Windows 10 problem.

## Worth adopting into gamebooster-rs (reversible, no driver)?

**Nothing to adopt.**

- **Standby purge:**
  - It cannot be undone: the cache is simply gone.
  - It is not measurably useful on Windows 10 21H2+/11.
  - It works against our own RAM trim.

  It fails the DESIGN.md rule ("only gaming-relevant, reversible tweaks").
  - A one-shot purge before the game starts would at most turn later cache hits into disk reads.
  - The gentler "purge low-priority standby only" (command 5) has the same problem, only smaller.
- **Timer resolution:** already rejected (see [Hone](../hone/README.md) and
  [priority](../tweaks/priority.md#power-throttling-off)).
  - On Windows 11 the global switch needs a reboot and changes every process.
  - The per-process request does not reach the game.
  - Evidence for a gain is missing.
- **Exclusion idea (pause while the game runs):** irrelevant to us, since we do nothing periodic.
- **What ISLC confirms for us:**
  - the RAM trim should stay off by default and be described as "only when memory is short";
  - `gamebooster bench` (hard faults, load times, frame times) is the right way to decide.

  If someone wants to test ISLC on the Mac, run `gamebooster bench compare` with and without it
  (exclusion list empty vs. `bf6.exe` excluded). Do that before considering anything.
  Note that `bench` capture of BF6 is currently blocked by its anti-cheat
  ([measuring](../measuring.md#status)); until that is solved, use Resource Monitor (hard faults/sec)
  and map-load times.
