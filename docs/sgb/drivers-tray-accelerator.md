# SGB 6.0: driver updater, tray and mini windows, network accelerator

Plain-language summary of what these parts of Smart Game Booster do, from static analysis only
(nothing was run, no vendor server contacted). Evidence and addresses are in
`decomp/iobit/smart-game-booster/AutoDriver.exe/notes.md`, `decomp/iobit/smart-game-booster/DriverClean.dll/notes.md`, `decomp/iobit/smart-game-booster/SgbTray.exe/notes.md`,
`decomp/iobit/smart-game-booster/SgbMini.exe/notes.md` and `decomp/iobit/smart-game-booster/SgbAccelerate.dll/notes.md`. None of this is part of
the Boost that gamebooster-rs reimplements (`docs/spec.md` section 6).

## Driver updater (AutoDriver.exe, DrvInstall/, DriverClean.dll)

**What it is.** IObit's Driver Booster engine with SGB branding.

**Where the driver info comes from.** It lists your devices, sends the list to
`s.pcgameboostdrivers.com/scan.php` (fallback `s1/s2.driverboosterscan.com`), and gets back the
"newest" driver for each hardware ID. Packages are downloaded from `f.pcgameboostdrivers.com`
(plain HTTP, checked against an MD5 from the scan reply) in IObit's own package format, so the
drivers are **repackaged by the vendor**, not fetched from Intel, AMD, NVIDIA or Microsoft.

**How it picks.** A driver counts as outdated when the database entry has a newer version or date
than what is installed. Pro users see more entries. Nothing in the code mentions Apple or Boot Camp.

**Install.** It unpacks the package and runs `DpInstX64.exe`, signed by the vendor (ORANGE VIEW
LIMITED). That is IObit's own installer, not Microsoft's DpInst: it calls Windows' DIFx driver API
directly with the **force** flag (installs even over a newer or better-matching driver) and without
legacy mode, so **unsigned packages are refused** (high confidence; `decomp/iobit/smart-game-booster/AutoDriver.exe/notes.md`).
Superseded: this paragraph used to call it re-signed Microsoft DpInst with unknown options.

**Backup and restore.** None found: no driver backup and no restore point. The menu only opens
Windows' own System Restore. Its sibling `DriverClean.dll` deletes old driver packages from the
Windows DriverStore, which is where Device Manager's "Roll Back Driver" takes the old driver from.
Update plus clean can therefore be a one-way trip.

**Boot Camp Mac risk: high.** Apple's Boot Camp drivers (audio, Wi-Fi/Bluetooth, keyboard and
trackpad, AMD/Intel graphics) are Apple-packaged and often older by version or date. Nothing tells
the updater to leave them alone, so it will offer to swap them for generic ones. Generic graphics or
audio drivers on a Mac can lose features, fail to install, or misbehave (for example, the speaker
tuning that Apple's audio package includes). Without a backup or restore point, getting back means
reinstalling the Boot Camp support software by hand.

**Verdict:** don't use it on the Mac. Get drivers from Apple's Boot Camp support software, or
straight from AMD/Intel when you know the Mac model is supported.

## Tray (SgbTray.exe) and mini window (SgbMini.exe)

**Tray.** Starts at logon through a Task Scheduler task (most likely `SmartGameBooster Startup`,
running `SgbTray.exe /auto`). A second task, `SmartGameBooster SkipUAC`, starts the main program
elevated without a UAC prompt. While running, on timers, it:
- scans the process list to spot games starting (auto-boost is SVIP-only);
- reads CPU, GPU and disk temperatures and warns when they are high;
- shows popups when RAM is over 90 %, after 7 days without a boost, when a new game is installed,
  and when graphics or audio drivers "need updating";
- runs the updater (`SgbUpdater.exe /auto`) every 12 hours (first run 12 hours after the tray
  starts), checks the licence (`smgblicense.itopupdate.com`),
  and sends cached usage stats (`stats.gameboosta.com`) every 5 minutes.
Its timers: a 1-second housekeeping tick (updater due-check, main-window check), the 5-minute stats
flush, a one-shot 2-second start of the `SMGB_Tray_Boost_Game` task, and popup timers (temperature
alarm closes after 30 s, its sound stops after 5 s). The process-watch and nag checks run on separate
threads whose periods weren't traced. Superseded: "The poll intervals weren't recovered."
(`decomp/iobit/smart-game-booster/SgbTray.exe/notes.md`).

**Mini window.** The small "boosting into Gaming Mode / restoring to Normal Mode" window. It runs
the same Boost and Restore steps without the main UI, from the tray menu, a hotkey or auto-boost.
It isn't autostarted and does nothing in the background.

**Verdict:** the tray is ordinary background bloat with nags and telemetry, and it costs a little
CPU while you play. gamebooster-rs needs none of it: Boost and Restore run on demand.

## Network accelerator (SgbAccelerate.dll + WinDivert driver)

**What it is.** The iTop VPN client core (same company) aimed at game traffic. Runs on the vendor's
servers; with no account and no servers it does nothing.

**Flow.**
1. Log in to `api.gameboosta.com` (email, Google, activation code or an iTop VPN account) and get a
   token.
2. Download the node list: groups by country, city or game, with P2P, IPv6 and VIP flags.
3. Choose a node. "Smart Acceleration" picks by game region, then by geographic distance from
   your IP's geolocation, by server load and by measured ping. You can also pick a region or node
   by hand.
4. Ping: the time to open a TCP connection to the node, plus a UDP probe that also measures packet
   loss. These are the Latency and Loss Rate columns.
5. Connect. It loads the `itvwd` packet driver (renamed WinDivert), points your DNS at 127.0.0.1
   (the old DNS is saved and put back on disconnect), and may block IPv6.
6. The driver grabs **every outbound TCP and UDP packet** on the PC. A second, sniff-only handle
   maps each connection to its process, and only the game's connections are sent into the tunnel.
   Everything else is reinjected unchanged.
7. Tunnel: KCP over UDP, ChaCha20-encrypted, to the node. A server-controlled filter drops
   BitTorrent, eD2k, mail, web and some DNS through the tunnel, so it can't be used as a free VPN.
   There is a kill switch.
8. On disconnect: `sc stop itvwd`.

**More detail from a full decompile of the DLL** (evidence in `decomp/iobit/smart-game-booster/SgbAccelerate.dll/notes.md`):
- The DLL never sees your password. The main program logs in, then hands the DLL two session
  tokens. A node that rejects the token, or that is full, sends an error back and the connection
  stops.
- The main program gives the DLL up to 16 nodes. About every 10 seconds the DLL checks that the
  current node still answers, and switches to another one in the list if it doesn't. After three
  failed checks in a row it tells the main program, which drops the connection. It doesn't
  reconnect by itself.
- "Encryption" is weak. ChaCha20 is used with only 8 bytes (64 bits) of key and, as far as could be
  seen, a fixed all-zero nonce. Treat it as scrambling, not privacy. The code also has "none",
  "xor" and "rc4" modes.
- The packet driver's filter always leaves out traffic to the nodes themselves, so the tunnel
  doesn't capture itself.
- The kill switch is only a stored on/off setting in the DLL. Nothing in the DLL was found acting on
  it, so whether it really blocks traffic when the tunnel drops is unconfirmed.
- DNS is changed and restored by the main program, not the DLL. The DLL only drops lookups for names
  on a server-supplied block list.

**System changes.** If the Base Filtering Engine (BFE) or Windows Firewall services are disabled,
it sets them to start automatically and starts them, and doesn't change them back. It also changes
the DNS settings and some netsh interface settings while connected. If it crashes, DNS can be left
pointing at 127.0.0.1, and then nothing resolves.

**Free tier.** A few hours of "Free Boost Time" on a separate free server pool, then SVIP (paid).
Some regions are blocked.

**Anti-cheat risk with BF6 (EA Javelin): real, size unknown.** A kernel driver that intercepts and
rewrites all game packets is exactly what lag-switch cheats use, and WinDivert is the usual tool for
that. Several anti-cheats are known to refuse to run, or to flag, when WinDivert is loaded.
Renaming it to `itvwd` doesn't hide its code from signature checks. Nothing in these files says how
Javelin treats it. It could refuse to start, kick, or flag the account. The driver can also stay
loaded after you stop accelerating if `sc stop` fails.

**Verdict:** skip it for BF6. It helps only when your ISP's route to EA's servers is bad, it costs
money after the trial, it reroutes all your traffic through a kernel driver, and it is a plausible
anti-cheat trigger. If routing really is the problem, a well-known gaming VPN that EA tolerates is
the safer test, and that's a separate decision.
