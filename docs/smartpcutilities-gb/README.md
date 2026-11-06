# PC Game Booster (Smart PC Utilities): what it really is

PC Game Booster 8 (smartpcutilities.com, formerly "Game Fire") is a free/paid Windows game booster
written in VB.NET. It has a game library, a **Game Mode** that temporarily stops services, scheduled
tasks and background programs and switches power plan, a **Live Optimization** that changes process
priorities while you play, memory "optimisation", a **System Optimizer** with permanent Windows
tweaks, game-file defragmentation and a hardware monitor. Pro costs about $19/year; Free has Game
Mode basics, library, defrag and monitoring.

Everything here comes from reading the shipped files (static reverse engineering) of installer
8.0.5002, downloaded on 2026-11-05. **Nothing was run.** The program code is encrypted with an
obfuscator (ConfuserEx), so the exact values some tweaks write could not be read; what the installer
does, the power plan, the exception lists and the feature set are certain. Technical notes:
[../../decomp/smart-pc-utilities/game-booster/README.md](../../decomp/smart-pc-utilities/game-booster/README.md).

## Short version

- Game Mode is the same idea as Smart Game Booster's Boost (stop services/tasks, close programs,
  power plan) and seems to be undone when you leave it (it keeps lists of what it changed and has
  "restore services/processes" options). Most of the gain is small on a clean PC.
- The hardware monitor runs **WinRing0**, a known-vulnerable kernel driver that Microsoft Defender
  flags and some anti-cheats block. The installer ships no driver, but the program loads one while
  running (its uninstaller deletes the `R0GameBooster` services that the driver leaves).
- Pro's System Optimizer makes **permanent** changes, including pausing Windows Update, Defender and
  telemetry, disabling the Windows key and turning off visual effects. The uninstaller does not undo them.
- Installs two elevated scheduled tasks (tray at logon, and a "skip UAC" task); both checkboxes are
  ticked by default. Sends install and uninstall tracking with your hardware and Windows details.
- Signed by an individual developer's certificate that expired in April 2026.

## Every part, in one table

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo** (does little or nothing),
**harmful**, **risky** (can break things, cost privacy or security, or trip anti-cheat), **bloat**.
"Reversible" says whether the program itself can put it back.

| Feature / part | What it really does | Reversible | Verdict |
|---|---|---|---|
| Game Mode: stop services | Stops running services not in its exception list (antivirus, DHCP, Windows Installer, Search, Gaming Services, NVIDIA are spared); user-editable list. Can also change start types. | Yes, on exit (option) | useful (small); risky if a needed service is stopped |
| Game Mode: scheduled tasks | Disables scheduled tasks for the session (Pro for the full list). | Yes (option) | placebo to small |
| Game Mode: close / lower programs | Closes or lowers background programs, sparing AV, launchers, shell and the vendor list. | Restores priorities; closed programs are not restarted | useful (small) |
| Game Mode power plan | Switches to its imported plan: High performance with minimum CPU 100 % on AC, core parking off, PCIe link power saving off, Wi-Fi max performance, long sleep/screen timers, no password on wake. | Yes (switches back) | useful on desktops; **harmful on hot laptops** |
| Pause Windows Update / Defender scans / telemetry (Pro) | Done through Defender/PowerShell settings; needs Tamper Protection turned off by the user. | Session pause in Game Mode; permanent in System Optimizer | **risky** (security); tiny gain |
| Live Optimization (Pro) | Lowers background process priority, raises the foreground app, while gaming. | Yes (restore option) | useful (small) |
| Memory optimisation | Trims every process's working set and purges the standby list on a timer or hotkey. | n/a | placebo / harmful (causes page faults and stutter) |
| System Optimizer visual effects (Pro) | Turns off window/menu/tooltip animations, cursor and icon shadows, drag-window contents, font smoothing. | Backup folder + System Restore point; not undone on uninstall | placebo for games |
| System Optimizer: Windows key, Sticky Keys hotkey | Disables the Windows key and the Sticky Keys shortcut. | As above | useful (comfort), not performance |
| System Optimizer: Widgets, Copilot, background apps, core parking, Windows Game Mode | Turns off Widgets/Copilot/app background activity; core parking off; toggles Windows Game Mode. | As above | placebo to small |
| System Optimizer: services and tasks pages | Disables services/scheduled tasks permanently. | Backup; not on uninstall | risky |
| Defragment game files | Runs Sysinternals Contig on a game folder (skipped on SSD). | n/a | placebo on SSD, small on HDD |
| Hardware monitor / temperature alarms | LibreHardwareMonitor 0.9.4 with the **WinRing0** driver. | Driver service left until uninstall | **risky** (anti-cheat, Defender alert, vulnerable driver) |
| Game library | Finds Steam, Epic, Battle.net, Microsoft Store, Uplay games; launch with boost. | n/a | harmless |
| "Boost" right-click menu on .exe files | Adds `exefile\shell\GameBooster`. | Removed on uninstall | harmless |
| Scheduled tasks `GameBooster`, `GameBoosterSkipUAC` | Elevated tray at logon; on-demand elevated start without UAC. Both enabled by default. | Removed on uninstall | bloat; minor risk (always-elevated tray) |
| Install/uninstall tracking | POST to `smartpcutilities.com/installtrack.html`: version, edition, Windows build, language, RAM, screen size, date/time; opens thanks/uninstall web pages. | – | privacy cost (minor) |
| Promotions | Fetches the vendor's and **third-party** offers (can be turned off in settings). | – | bloat / nag |
| Licensing | Free, 30-day Pro trial, Pro (name + email + key), activated and checked online; expiry reminders and upgrade nags. | – | nag |
| Updates | Update check, download, Authenticode/signed-manifest check, beta channel. | – | probably fine (URL encrypted; cert expired) |
| Crash/feedback reports | Optional automatic error sending; feedback form. | – | harmless if left off |
| Installer | Advanced Installer, admin, signed by an individual; no services, drivers or Run keys; no bundled third-party installers. | Uninstall removes tasks, power plan, driver services, optional settings | harmless |

The website's comparison charts show "0 Average FPS" placeholders; no measured results are given.
No fake scan or fake progress was found in the parts readable, but the code bodies could not be read.

## On a 2019 MacBook Pro under Boot Camp playing BF6

Don't use it there. The WinRing0 driver is the main problem: BF6 uses kernel anti-cheat (Javelin) and
Defender flags WinRing0, so the hardware monitor is a real risk. The Game Mode power plan forces 100 %
minimum CPU on AC with core parking off, which adds heat on a machine that already throttles and can
fall into the 800 MHz BD PROCHOT lock (see [prochot](../tweaks/prochot.md)). Memory trimming can
add stutter. Stopping services and lowering background programs is the only part that may help a
little, and gamebooster-rs already does that reversibly.

## Compared with Smart Game Booster, PGWARE GameBoost and gamebooster-rs

- **SGB**: same model (temporary Boost with restore, plus permanent optimizer tweaks, monitoring,
  library, nags). SGB ships its own drivers and a network accelerator; PC Game Booster has no
  network part but loads WinRing0 for sensors. Both phone home; SGB much more.
- **PGWARE GameBoost**: registry-only, permanent, mostly placebo; PC Game Booster does real,
  temporary work instead.
- **gamebooster-rs** already covers the useful core (power plan, allow-listed services, game
  priority, lowering other processes, Game Mode, restore). Worth considering, all reversible and
  driver-free:
  - **PCIe Link State Power Management off** and **wireless adapter "maximum performance"** as
    optional power-plan settings during boost (restore puts the old values back). Low value on the
    Mac on battery.
  - An **exception list by publisher** (never touch processes/services signed by AV vendors,
    launchers, GPU vendors), as a safety net for `lower_others`. Built: `NEVER_LOWER_PUBLISHERS`
    in `src/priority.rs` (by version-info CompanyName, not signature; processes only).
  - Not worth adopting: memory trimming / standby purge, minimum CPU 100 % (fights the Mac's
    thermals), visual-effects tweaks, pausing Defender/Update, WinRing0 sensors.
