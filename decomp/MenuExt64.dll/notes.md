# MenuExt64.dll (+ MenuExt32.dll)

Static strings, imports and exports only (objdump/strings); nothing executed. Carved: `pe_02fc9695_248376.bin` (x64), `pe_02f9845d_201272.bin` (x86). MSVC/ATL COM server.

Explorer context-menu handler (`IContextMenu`, class `CUnLockerMenu`, `ContextMenuHandlers`, standard
`DllRegisterServer` exports). Adds a "Smart Game Booster" entry that runs `SgbMain.exe /Game …` on
the selected program (add as game). Reads `SmartGameBooster.ini` and `MenuRight.dat`. PDB
`D:\work\SMGB6\...\RightMenu\PfShellExtension\...`. No network.
