# Research

What other game boosters and tuning guides do, feature by feature, with verdicts and adoption items.
Prose only: vendor code is not kept in this repo.

What Smart Game Booster itself does, feature by feature, with verdicts: [docs/sgb/](sgb/README.md).
The same for PGWARE GameBoost (registry-only, nothing worth adopting): [docs/pgware/](pgware/README.md).
The same for Smart PC Utilities' PC Game Booster (temporary Game Mode, loads the WinRing0 driver for sensors): [docs/smartpcutilities-gb/](smartpcutilities-gb/README.md).
The same for Razer Cortex: [docs/razer-cortex/](razer-cortex/README.md).
The same for Hone (server-driven tweak catalogue, encrypted engine, temporary Game Mode): [docs/hone/](hone/README.md).
The same for ISLC, Intelligent Standby List Cleaner (standby purge and timer resolution, both placebo or harmful on Windows 10/11): [docs/islc/](islc/README.md).
The same for ExitLag (relay "ping reducer" with two kernel packet drivers; helps only on a bad ISP route): [docs/exitlag/](exitlag/README.md).
The same for Process Lasso (ProBalance dynamic priority restraint, read from the binary; the basis of `lower_others_mode = "dynamic"`): [docs/process-lasso/](process-lasso/README.md).
The same for JetBoost (BlueSprig, a rebranded IObit booster from 2012; stops a Windows 8-era service list, hard-kills apps, forces min CPU 100 % even on battery; nothing new to adopt): [docs/jetboost/](jetboost/README.md).
The same for Wise Game Booster (permanent XP-era registry tweaks, kills programs, stops services with a resume list; uploads your game list): [docs/wise-game-booster/](wise-game-booster/README.md).
The same for Tier1Suite (Tier1Settings; 43-entry registry/power catalogue with honest notes and real revert, LocalSystem broker that re-applies, no driver; adoption: a Game DVR check in `status`): [docs/tier1suite/](tier1suite/README.md).
The same for BoosterX (Eazfuscator-protected settings manager; its "6.6× less CPU load" CS2 claim is idle-desktop debloat incl. Defender off, worth ~2–3 % FPS; adoption: try a background raw-input throttle): [docs/boosterx/](boosterx/README.md).
The same for eclean Optimizer (Tauri/Rust, 47 explained persistent tweaks with real undo, demand-start broker, no driver; its gaming tweaks are default-equal or placebo, the Pro Expert ones add heat or remove security; adoption: maybe `WerSvc`/`dmwappushservice` in the services list): [docs/eclean/](eclean/README.md).
Open-source tools and guides, read from source, with adoption items:
[GameShift](gameshift/README.md) (per-session apply/revert with a crash journal; mostly folklore tweaks, several of which need a reboot to work),
[valleyofdoom PC-Tuning](pc-tuning/README.md) (measure-first guide; keeps idle states on, prefers per-process timer resolution, silent on Nagle/MMCSS),
[Chris Titus WinUtil](winutil/README.md) (gaming-relevant tweaks only; its Ultimate Performance undo resets all power plans).
