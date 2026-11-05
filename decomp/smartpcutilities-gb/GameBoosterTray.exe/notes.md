# GameBoosterTray.exe

Tray resident (ConfuserEx-protected; names only). Started at logon by the elevated `GameBooster`
scheduled task (enabled by default by the installer's finish page). Watches for game launches to enter
and leave Game Mode automatically (`GameModeActivatorRefresh`, `GameModeActivatorCoolDown`), runs Live
Optimization (priority changes), periodic memory optimisation with notifications, global hotkeys
(Game Mode, free memory, My Games), the hardware monitor and temperature alarms, and update checks.
Process name `GameBoosterTray` gets the second WinRing0 service (`R0GameBoosterTray`).
