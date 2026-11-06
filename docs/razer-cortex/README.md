# Razer Cortex: what it really does

Razer Cortex (razer.com/cortex) is Razer's free PC "game booster", game launcher and rewards app. This page
comes from reading the official installer and the Cortex 11.12.0.14 package it downloads (static reverse
engineering, 2026-11-05). **Nothing was run.** Technical notes:
[../../decomp/razer/cortex/README.md](../../decomp/razer/cortex/README.md).

## Short version

- Installing Cortex silently adds **three** Razer programs: Cortex, Razer Central (account service) and
  Razer Game Manager (game detection service). It also adds a SYSTEM service plus a scheduled task whose
  purpose is to **start Cortex as administrator without a UAC prompt**, and it starts with Windows.
- **Game Booster** runs automatically when a game starts (auto boost is on by default). Its default
  boost:
  - switches to a "Razer Game Booster Plan" (High performance with sleep and power saving turned off);
  - trims every program's memory;
  - **empties your clipboard**;
  - turns off background apps, Windows telemetry, error reporting, Cortana and the Sticky Keys shortcut.
  Everything except the clipboard is put back when the game exits. Stopping services and closing apps
  happen only for what is ticked in its lists.
- **System Booster** writes about 120 registry values. It is the same old list SGB uses (IObit lineage),
  almost all preselected, with a working "restore". Mostly placebo on Windows 10/11, a few harmful.
- The app's screens are a **web page loaded from Razer's servers** that drives an elevated back end on
  your PC. Some downloaded add-ons are loaded **without a signature check**, and one feature **turns off
  winget's installer hash check**.
- Telemetry is on by default with no switch in the app. It covers games you play, boost use and a
  hardware inventory (CPU, GPU, RAM, mice, keyboards, cameras, audio), plus a machine id from the
  motherboard and disk serial numbers. The installer adds a fingerprint from the MAC address, disk serial
  and Windows id, linked to your Razer ID if one was ever used.
- **No kernel driver, no injection into games**: the FPS counter uses Windows event tracing, and the
  overlay is a separate window. Anti-cheat risk for BF6 is low (see below).

## Every part, in one table

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky** (can
break things, cost privacy or security, or trip anti-cheat), **bloat**.

| Part | Default | What it does | Reversible | Verdict |
|---|---|---|---|---|
| Auto boost on game start | on | Boost when Cortex sees a known game start, restore when the last one exits | yes | useful as a mechanism (only as good as the items below) |
| Power plan | on | Copy of High performance: CPU min/max 100 %, PCIe and Wi-Fi power saving off, no sleep, no display or disk timeout | yes (previous plan reactivated; plan removed on uninstall) | **useful** on a plugged-in desktop, mildly; on a laptop on battery it costs battery and heat |
| CPU unpark | on | Only unhides the "core parking" power setting; does not change it | yes | placebo |
| RAM trim | on | Empties every program's working set | n/a | placebo / slightly harmful (memory is paged back in, causing stutter) |
| Clear clipboard | on | Deletes the clipboard | **no** | harmful (data loss), no gain |
| Background apps off | on | Store apps can't run in the background | yes | placebo on most PCs |
| Disable telemetry policy | on | Windows telemetry policy while gaming | yes | placebo for FPS |
| Disable error reporting | on | Windows Error Reporting off | yes | placebo |
| Disable Cortana | on | Policy plus closes Cortana | yes | placebo on Windows 11 (Cortana is gone) |
| Sticky Keys shortcut off | on | No Sticky Keys pop-up from pressing Shift 5 times | yes | **useful** (comfort, not FPS) |
| CPU core affinity | off | Squeezes other processes onto 2–4 cores | yes | harmful / **risky** (can starve audio, launchers, anti-cheat) |
| Windows key off | off | Blocks the Win key while boosted | yes | useful for some (comfort) |
| Game Desktop | off | Kills Explorer, shows a Cortex desktop | yes (restarts Explorer) | placebo / risky |
| Stop services | ticked list (~60 suggested) | Print spooler, time sync, search, SysMain, Remote Desktop, credential and smart-card services and more | yes (restarted) | mostly placebo, some **risky** |
| Close programs | ticked list | Terminates apps such as VLC, TeamViewer, KeePass, Task Manager; relaunches them on restore | partly (unsaved work lost) | **risky** |
| System Booster (registry, ~120 values, all preselected) | manual | Same IObit-era list as SGB: timeouts, NTFS, TCP/IP, IE, visual effects, a dozen services disabled, Windows ads/suggestions off, MMCSS "Games" profile | yes ("restore before" or "Windows defaults") | mostly **placebo**; Windows ads/suggestions off is fine; disabling services and `AutoEndTasks` can be **harmful** |
| Boost Prime (game settings optimizer) | manual | Rewrites a game's graphics config via add-ons downloaded per game | depends on the game | **risky** (unsigned code runs as admin) |
| FPS counter / overlay | user | ETW-based FPS, external overlay window | yes | useful (Windows Game Bar does the same) |
| Game clips | user | Screen recording with auto-clips for CS2 / Dota 2 / LoL | yes | bloat for most; fine if wanted |
| System Clean, Defrag | manual | Junk cleaner, defragmenter, optional schedule | n/a | bloat (Windows does both) |
| AppGet (app installer) | manual | Installs apps via winget; on a hash mismatch it **disables winget's hash check** system-wide and retries | no (setting stays on) | **risky** |
| Remote play (Neuron) | manual | Streaming server; local client accepts any TLS certificate | yes | **risky** |
| Deals, Razer Gold/Silver, "Play to Paid" | on (shown) | Store, rewards for playing promoted games (heartbeat every 5 min), notifications | n/a | bloat / mild dark pattern |
| Launcher service + "skip UAC" task + autostart | on | SYSTEM service and scheduled task start Cortex elevated without a prompt; Run key and a 3-minute delayed launch | yes (uninstaller removes them) | **risky** (permanent admin path) |
| Razer Central and Game Manager | on (silent) | Account and game detection services shared with other Razer apps | uninstall separately | bloat |
| Telemetry | on, no in-app switch | Usage, games played, hardware inventory, machine id; install telemetry before the licence screen | — | **risky** (privacy) |
| Updates | automatic | App via the Razer installer (HTTPS plus MD5, no signature check); the UI page is live from the CDN | — | **risky** in design, OK in practice |

## For a 2019 MacBook Pro under Boot Camp playing BF6

- **Worth anything:**
  - The power plan, but only on AC power. The MacBook's Intel CPU and AMD GPU are limited by heat, so
    "100 % minimum processor state" mostly adds heat and fan noise; any gain is small.
  - The Sticky Keys shortcut off.
- **Avoid:**
  - Clipboard clearing (on by default).
  - CPU affinity.
  - Closing programs.
  - The System Booster's service disabling: Boot Camp relies on some standard services, and the Apple
    keyboard / trackpad drivers are not on Cortex's whitelists.
  - Game Desktop.
- **Anti-cheat (EA Javelin):** Cortex doesn't inject into BF6 and installs no driver. The overlay is an
  external window and FPS comes from Windows event tracing. With CPU affinity off (the default) nothing
  touches EA's anti-cheat processes. Risk is low, but untested.
- **Overall:** for this machine Cortex adds two background services, a SYSTEM service and telemetry in
  exchange for a power plan that Windows' own "Best performance" mode or `powercfg` gives without any of
  that.

## Compared with SGB, PGWARE and gamebooster-rs

- **Same lineage as SGB:** the System Booster database and the Game Booster's service and process lists
  are the same IObit-era material. Cortex's per-item Restore is cleaner than SGB's, and the clipboard
  clear and RAM trim are just as pointless.
- **More reach than SGB or PGWARE:**
  - A permanently elevated back end driven by a web UI from Razer's servers.
  - Unsigned downloadable add-ons.
  - Several services, and more telemetry.
- **gamebooster-rs already covers** the useful part: a High-performance-based power plan, stopping chosen
  services and closing chosen apps, all with restore, plus priorities and Game Mode.
- **Worth adopting into gamebooster-rs** (reversible, no driver):
  1. Turn off the **Sticky Keys / Filter Keys shortcuts** while boosted and restore afterwards
     (SystemParametersInfo). This is comfort, not FPS, and fully reversible. Built 2026-11-05:
     [accessibility shortcuts](../tweaks/accessibility-shortcuts.md).
  2. If gamebooster-rs ever gets an automatic mode: restore only when the **last** watched game exits
     (Cortex counts running games), so a second game doesn't get un-boosted. Built 2026-11-06:
     `gamebooster auto` ([Automatic boost](../auto.md)).
  3. Optional, for a custom power plan: copy the user's **lid and power-button actions and screen
     brightness** from the current plan into the boost plan, so switching plans doesn't change them.
     Built 2026-11-05 (lid, power button, sleep button): [power plan](../tweaks/power-plan.md).

  Everything else is placebo, harmful or already covered.
