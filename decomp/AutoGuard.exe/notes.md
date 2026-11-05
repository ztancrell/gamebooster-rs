# AutoGuard.exe

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0277284c_3672120.bin`. Delphi, "Auto Guard for Smart Game Booster".

Background malware/browser-add-on scanner: imports `Scan.dll` (see `../Scan.dll/notes.md`), modes
`/AutoGuard` and `/AutoGuardIdle` (runs when idle; `"%s" /AutoGuard`), databases `BHO.db`, `BHO.ign`,
`Coreext.db`, `File.db`, `FileInfo.db`, `folder.db`, UI strings `%s threat(s) found`, browser names
(`chrome.exe`, `firefox.exe`). Same helper set as the other Delphi tools (AntiBan lists, `ip-api.com`/
`ipinfo.io` country lookup, `Bugreport.exe /bugreport /smgb`). SgbMain has the `AutoGuardConfig`
setting, `rdbtn_autoguard` toggle and `DoReciveData AutoGuard`.
