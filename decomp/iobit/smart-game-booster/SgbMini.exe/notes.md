# SgbMini.exe

Static strings only (carved `pe_00fd6a54_2492984.bin`, same size). Nothing executed.

The small "Smart Game Booster is boosting into Gaming Mode..." / "restoring back to Normal Mode..."
window (`[frmMiniBoost_M]`). It runs Boost and Restore without the main UI, e.g. from the tray menu,
a hotkey (`BoostHootKey1/2`, `RestoreHootKey1/2`) or auto-boost on game launch, and signals the end
with `/MiniBoostOver`. It carries its own copy of the Boost items (`Boost RAM`, `Boost Service`,
`Boost Process`, `Boost Power Schemes`, `Boost Clipboard`, `Boost AutoUpdate` /
`begin disable windows update.` / `CloseWindowsUpdate`, `Boost OverClock`, `Boost Monitor`,
`Enter ProcessBoost.Boost/Restore`) and a list of other vendors' processes it knows about
(`ASCTray.exe /auto`, `DriverBooster.exe`, `ccSvcHst.exe`, ...). Same Task Scheduler helpers as the
tray (`CreateNewTask_OnLogonEx`, `IsTaskExists`) and the same licence/country/URL-list endpoints. No
background loop of its own beyond the boost run; it is not autostarted.
