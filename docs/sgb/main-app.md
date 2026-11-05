# Smart Game Booster 6.0: what the main app does

Plain-language guide to `SgbMain.exe`, the main SGB window, apart from Boost and Restore themselves
(those are in `docs/spec.md`). Found by reading the program's code and data files; nothing was run.
Addresses and details: `decomp/SgbMain.exe/features.md`.

## The smaller Boost steps

- **Monitor Tune (A1).** Changes an external monitor's picture mode over its control cable (DDC/CI):
  display mode, colour preset, brightness and contrast. It writes the old values to `Monitor.ini`
  first and puts them back on Restore. Only works with monitors that accept DDC/CI; never on XP or a
  laptop/Mac built-in screen. Useful? Only if you like a different picture mode for games.
- **GPU overclock (A2).** Hands over to `OverDrive.dll`: asks it how far the card can go, then tells it
  to overclock and notes the new clocks in `OverDrive.ini`. Restore calls "Reset". Risky; can make
  games crash.
- **Clear clipboard (A6).** Empties the clipboard. No effect on games.
- **Windows Update (A8).** Switches automatic updates to "off" through Windows' own update settings
  object, if Windows lets it (on Windows 10/11 this is usually locked and SGB gives up). Restore puts
  the old setting back.
- **Remove dead tray icons (A11).** **Does nothing.** In 6.0 it only writes "Remove Dead Tray Icons"
  to a debug log. Do not implement it.

## Finding and starting games

- SGB looks for installed games in Steam, Epic, GOG, EA/Origin, Ubisoft, Battle.net, Riot, Roblox,
  WeGame, the Microsoft Store (matched against a list of 22,703 Store game IDs) and Windows' own games.
  Each game gets a record in `UserGame.ini`: name, path, launcher, and the program names that mean "the
  game is running".
- It starts a game through its launcher's link (for example `steam://rungameid/…`) or directly.
- **Auto Boost** (boost when you start a game) is watched by the tray program `SgbTray.exe`, not by
  the main window. The tray program notices the game starting and tells the main window to boost. It
  is a paid (SVIP) feature.
- **Game Desktop** hides all other windows and the desktop icons while you play, leaving only the
  taskbar and tray.
- **In-game overlay** (`fps.dll`/`FPS64.dll`): FPS, CPU/GPU temperature, screenshots and video
  recording, with hotkeys. How it gets into games and why it is risky with anti-cheat:
  `gpu-hud-recording.md`.

## "Can You Run It" game database

A 16 MB encrypted file that opens into a 63 MB database of 162,942 games with minimum and recommended
CPU, graphics card, memory and Windows version, plus score tables for 9,471 CPUs and 3,397 graphics
cards. SGB scores your PC and says whether each game should run. It also reports your searches and
"can't run" results to the vendor.

## Other tools in the window

- **Startup manager.** Lists programs, services and scheduled tasks that start with Windows, with
  "how many users disabled this" ratings from the vendor. Its shipped blacklist suggests disabling
  **Windows Defender** and **Malwarebytes** at startup, alongside adware. Do not follow that advice.
- **System Tweak, Internet Booster.** Registry and network tweaks, some with typos that write junk
  values (see `decomp/SgbData/dbd/README.md`; how the scanner decides what is a "problem": `cleaners.md`). Browser settings edits too, including re-signing
  Chrome's protected preferences so Chrome accepts the change.
- **Junk and privacy cleaner, browser extension manager.** Deletes temp files, browser history,
  cookies and similar. How it scans, what it deletes and how its numbers overstate the result:
  `cleaners.md`.
- **Security check.** Reads which antivirus and firewall Windows reports and can turn Defender's
  real-time protection and the firewall back on. It also detects about 100 installed antivirus
  products (used to block the accelerator when Kaspersky is present); no uninstall option is offered.
- **Malware scanner.** A small, outdated signature scanner (database from 2021) with a quarantine
  folder; it counts tracking cookies as threats. See [security-features.md](security-features.md).
- **Driver updater, Game defrag, temperature alarms.**
- **Game accelerator.** A VPN-style tunnel to the vendor's servers (`SgbAccelerate.dll`).
- **Account sign-in, upsell and promotion windows, skins, languages.**

## Privacy and security notes

- **Statistics are sent** to several vendor servers, including one that receives your hardware
  details (`smgb_hw.php`), daily/monthly "active user" pings, and searches in the game database. It
  also looks up your IP location (ip-api.com, ipinfo.io).
- **"SkipUAC"** creates a scheduled task so SGB can run with administrator rights without the
  Windows permission prompt.
- The proxy password is kept in a settings file.
- SGB checks the registry for other products from the same vendors (IObit, iTop) and promotes them.

## What is worth copying for gamebooster-rs

Nothing here beyond what `docs/spec.md` already ranks. Monitor Tune and the overclock are
hardware-specific and optional; the clipboard and tray steps have no effect; the rest is cleaner,
tweaker and upsell software unrelated to Boost.
