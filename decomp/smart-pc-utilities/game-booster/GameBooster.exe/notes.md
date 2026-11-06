# GameBooster.exe

Main window (WinForms, ConfuserEx-protected; names only). Started elevated (manifest or the
`GameBoosterSkipUAC` task with `/SkipUAC`).

## Areas (from control and worker names)

- **My Games**: library from all launchers, cover art, launch, "Boost" from the Explorer right-click
  menu, game-file defragmentation with `Contig.exe` (`BWDefrag`, skipped on SSD).
- **Game Mode** (`BWGameMode`, `BWRefreshGameMode`, `LblPowerPlan`, `LblWinUpdate`, `LblWinDefender`,
  `LblWinTelemetry`, `LblWinGameMode`): stops services, disables scheduled tasks, closes or lowers
  background programs, switches power plan, and (Pro) pauses Windows Update, Defender scans and
  telemetry for the session. Upgrade links (`LnkPowerPlanUpgrade`, `LnkWinDefenderUpgrade`,
  `LnkSchTasksUpgrade`) mark the Pro-only parts.
- **Live Optimization** (`PnlLiveOptimization`): while a game runs, lowers the priority of
  background processes and raises the foreground app, with notifications and a log
  (`LblLastOptimizedPrcs`, `LblLastOptimizedApp`); periodic memory trim.
- **System Optimizer** (`BWApplyTweaks`, `InitTweakCards`, `TweakCard_ToggleChange`, `TabPageTweaks`,
  `FrmWinDefender`): toggle cards for permanent tweaks. Card names (from `Is…Optimized` properties):
  window animations, menu animations, tooltip animations, client-area animations, cursor shadow,
  icon drop shadow, window drag effect (show contents while dragging), font smoothing, UI effects,
  Sticky Keys hotkey, **Windows key** (disable), Windows Widgets, Copilot, core parking, apps
  background activity, Windows Update, Windows Defender, Windows telemetry, Windows Game Mode.
  Pages for scheduled tasks and services (`BWGetSchTasks`, `BWGetServices`) to disable them
  permanently. A restore-from-backup menu item exists (`MnuItmRestoreBackup`, `DirBackup`).
- **Hardware monitor** (`BWHardwareMon`, LibreHardwareMonitorLib): CPU/GPU/board/storage sensors,
  temperature alarms. LibreHardwareMonitorLib 0.9.4 loads its embedded WinRing0 driver for CPU and
  board sensors (service `R0GameBooster`).
- Menu shortcuts to Windows tools (Task Manager, startup programs, programs, power options, network,
  memory diagnostic, task scheduler).

Exact registry values of the tweaks are not recoverable (strings encrypted).
