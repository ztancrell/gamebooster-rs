# Smart Game Booster 6.0: what it really is

Smart Game Booster (SGB) is a Windows "game booster" sold by the IObit / iTop group. Its free core,
**Boost**, makes a handful of local Windows changes before you play and undoes them afterwards. Around
that core sit a cleaner, a driver updater, an in-game FPS overlay, a paid VPN-style "network
accelerator", and a lot of telemetry, licensing and upsell code.

Everything here comes from reading the shipped files (static reverse engineering). **Nothing was
run.** Details:

- [main-app.md](main-app.md): the main window, the smaller Boost steps, game detection, other tools.
- [gpu-hud-recording.md](gpu-hud-recording.md): GPU overclock, in-game FPS overlay, recording.
- [telemetry-licensing-updates.md](telemetry-licensing-updates.md): what it sends home, licensing,
  updates, installer, dark patterns.
- [drivers-tray-accelerator.md](drivers-tray-accelerator.md): driver updater, tray and mini windows,
  network accelerator.
- [../spec.md](../spec.md): the exact Boost/Restore behaviour that gamebooster-rs copies.
- Technical notes per program: [../../decomp/README.md](../../decomp/README.md).

## Every feature, in one table

Verdicts: **useful** (does something real for games), **placebo** (does little or nothing),
**risky** (can break things, cost privacy, or trip anti-cheat), **bloat** (runs in the background
without helping games).

| Feature / program | What it really does | Verdict |
|---|---|---|
| Boost: power plan | Switches to a high-performance plan with sleep and screen-off disabled; restores the old one. | useful |
| Boost: stop services, close programs | Stops listed Windows services and closes chosen background programs; restarts them on Restore. | useful |
| Boost: lower other programs | Code exists (TPriorityController) but nothing ever calls it, so SGB 6.0 never lowers other programs. gamebooster-rs offers it as its own opt-in `lower_others`. Superseded: earlier said SGB drops them to lowest priority. | dead code |
| Boost: free RAM | Asks every program to give back memory it isn't using. Helps only when RAM is nearly full. | placebo |
| Boost: Game Mode, Windows Update | Turns on Windows Game Mode (usually already on) and tries to pause automatic updates (usually locked). | placebo |
| Boost: clear clipboard | Empties the clipboard. | placebo |
| Boost: remove dead tray icons | Does nothing in 6.0 (only writes a log line). | placebo |
| Boost: monitor tune | Switches an external monitor to a "game" picture mode over its cable, puts it back after. | placebo |
| Boost: GPU overclock (`OverDrive.dll`) | Raises graphics clocks within what the driver allows; no stability check. Likely does nothing on a Boot Camp Mac. | risky |
| Game finder and launcher | Finds games in Steam, Epic, GOG, EA, etc., and starts them. | useful |
| Auto Boost (`SgbTray.exe`) | Watches for a game to start and boosts automatically. Paid (SVIP). | useful (paid) |
| Tray program (`SgbTray.exe`) | Always running: temperature, RAM and driver pop-ups, updates, licence checks, statistics. | bloat |
| Mini window (`SgbMini.exe`) | Small "boosting…" window that runs Boost without the main UI. | useful |
| Game Desktop | Hides other windows and desktop icons while you play. | placebo |
| In-game FPS overlay (`SgbHUD64.exe`, `SMGBFilter64.dll`, `FPS64.dll`) | Puts an SGB DLL into every windowed program and patches the game's drawing calls to show FPS. | risky (anti-cheat) |
| Recording, screenshots (`VideoRecord64.dll`, `sgbGameRecord.exe`) | Records gameplay with slow software encoding. | risky (anti-cheat), costs CPU |
| Roblox FPS unlocker (`RobloxFPS.dll`) | Rewrites Roblox's frame cap in memory. | risky |
| GPU usage monitor (`GPUlib.dll`) | Reads per-program GPU usage. | harmless |
| "Can You Run It" database | Scores your PC against 162,942 games' requirements; reports searches to the vendor. | bloat |
| Startup manager | Lists startup programs; its blacklist even suggests disabling Windows Defender. | risky |
| System tweaks, Internet booster | Registry and network tweaks, some with typos that write junk values. | risky |
| Junk/privacy cleaner, malware scanner, security check | Deletes temp files and browser data; small signature scanner; re-enables Defender/firewall. | bloat |
| Driver updater (`AutoDriver.exe`, `DriverClean.dll`) | Installs vendor-repackaged drivers with no backup, and deletes old ones so Windows can't roll back. Would replace Apple's Boot Camp drivers. | risky |
| Network accelerator (`SgbAccelerate.dll` + WinDivert) | VPN-style tunnel for game traffic through the vendor's servers, via a packet driver. Paid after a trial. | risky (anti-cheat) |
| Telemetry (`ProductStat3`, `AUpdate`, `BugReport`, `UninstallPromote`, `SgbInit`) | Usage pings, hardware details, searches, crash reports with your process list; no off switch. | bloat, privacy cost |
| Licensing (`Register.dll`) | Online check tied to CPU and disk serial; edits your hosts file without asking. | risky |
| Updater (`SgbUpdater.exe`) | HTTPS manifest, files checked by hash and signature. | useful (reasonably safe) |
| Feedback form (`FeedBack.exe`) | Sends a message you type. | harmless |
| Upsell pop-ups | "Limited Time 85% OFF", SVIP expiry nags, accelerator countdown. | bloat |

## What gamebooster-rs reimplements and why

gamebooster-rs ([DESIGN.md](../../DESIGN.md)) is a small command-line tool (`boost` / `restore` /
`status`) that copies only the Boost steps that plausibly help a CPU/GPU-bound game on a Boot Camp
MacBook running BF6: the power plan, stopping an allow-list of services, closing chosen programs,
freeing RAM, Game Mode, and game priority (optionally lowering other programs). Everything it changes
is recorded and undone by `restore`.

It leaves out everything else on purpose: no overlay or packet driver (anti-cheat risk, and no kernel
driver ever), no driver updater (Boot Camp risk), no overclock or monitor tune (hardware-specific,
no effect on the Mac), no tray-icon step (SGB's does nothing), no telemetry or licensing. In place of
the network accelerator it has a `ping` command and an optional "fastest DNS" step.
