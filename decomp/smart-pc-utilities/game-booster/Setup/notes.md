# Setup (Advanced Installer MSI, x64 package)

Read with `../msitables.py` from the x64 MSI (the x86 MSI is the same package for 32-bit Windows).
Edition property `EDITION=Professional`; one installer for Free and Pro (the licence decides).

## Files and registry

- Installs to `C:\Program Files\Smart PC Utilities\PC Game Booster\`: the eight vendor binaries, the
  third-party DLLs, `Games.db`, `Contig.exe` + `Eula.txt` (Sysinternals), licence/open-source RTFs.
- Exception lists go to `C:\ProgramData\Smart PC Utilities\PC Game Booster\` (see [Data](../Data/notes.md)).
- Registry: `HKLM\Software\Smart PC Utilities\Game Booster` `Path`, `Version`;
  `HKCU\…\Game Booster\DetectInterference`=1; creates `HKCR\exefile\shell` (the app later adds a
  "Boost with PC Game Booster" right-click entry `exefile\shell\GameBooster`, removed on uninstall).
- Shortcuts: desktop, Start menu, an Uninstall shortcut.
- **No services, no drivers, no Run key** in the package.

## Custom actions (install)

| Action | What it does |
|---|---|
| `CreateGameBoosterPowerPlan` | `powercfg /import "%TEMP%\GameBooster-PowerPlan.pow" 2e7846dd-b84d-4e35-a2a0-99df8a4a76d5`: adds the "PC Game Booster Power Plan" (not made active by the installer). |
| `CreateSchTaskGameBooster` | Task `GameBooster`: at logon + 1 min, runs `GameBoosterTray.exe`, RunLevel **HighestAvailable** (elevated), created disabled. |
| `CreateSchTaskSkipUAC` | Task `GameBoosterSkipUAC`: on demand, runs `GameBooster.exe /SkipUAC` elevated. Lets the app start as admin without a UAC prompt. |
| `AllowAutoStart`, `AllowUACSkip` | Finish-page checkboxes, **both ticked by default**: enable the two tasks (`schtasks /CHANGE /ENABLE`). |
| `MigrateAppConfigFiles`, `UpgradeAppConfigFiles`, `UpgradeAppRegistry`, `MigrateGameModeConfig` | VBScript that moves settings from the old "Game Fire" product (Documents/LocalAppData folders, `HKCU\Software\Smart PC Utilities\Game Fire`). |
| `SendInstallerData` | Advanced Installer `DataUploader.dll`: HTTP POST to `https://www.smartpcutilities.com/installtrack.html` with product name/version/language, date and time, edition, install/uninstall action, Windows build and version, system language, product and upgrade codes, physical and virtual memory, screen width and height. Runs on install **and** on uninstall. |
| `OpenThanksPage` | Opens `…/thanks.html?product=gamebooster&v=…&l=…&os=<build>` in the browser after install. |
| `TerminateProcessGBTray`, `TerminateProcessGFTray` | Close the running tray (also the old Game Fire `GFTray.exe`). |

## Custom actions (uninstall)

- Deletes the power plan (`powercfg /delete 2e7846dd-…`) and both scheduled tasks.
- `sc stop` / `sc delete` **`R0GameBooster`** and **`R0GameBoosterTray`**. These are the kernel-driver
  services LibreHardwareMonitorLib creates at run time when it loads its embedded **WinRing0**
  driver ("R0" + process name). So the program loads a kernel driver while running, although the
  installer ships none.
- Asks "remove all preferences?"; if yes (the `AI_CLEAN_RESOURCES_UNINSTALL` checkbox defaults to
  ticked), VBScript deletes the program folder, `%LOCALAPPDATA%\…\PC Game Booster`,
  `%PROGRAMDATA%\…\PC Game Booster` and `HKCU\Software\Smart PC Utilities\Game Booster`.
- Opens `…/uninstall.html?product=gamebooster…` (feedback page) and sends the tracking POST again.
- **No uninstall step restores tweaks** (visual effects, services set by the System Optimizer, Windows
  Update / Defender / telemetry changes). Whether the app undoes them itself before uninstall is not
  visible (method bodies encrypted). Deleting the preferences also deletes the app's backup folder.
