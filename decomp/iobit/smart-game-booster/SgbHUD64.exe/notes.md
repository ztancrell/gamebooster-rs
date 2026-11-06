# SgbHUD64.exe / SgbHUD32.exe notes (HUD controller)

Carved copies: `SgbHUD64.exe` `pe_03a2d975_3654856.bin`, `SgbHUD32.exe` `pe_0436173d_2413256.bin`.
Delphi VCL apps (form `frmHookMainForm`). They load SMGBFilter(64).dll and `fps64.dll`, publish the
game list (`HUD64\GameProcess.txt`, `GameState.ini`) and exclusion list (`HUD64\FpsFilterList.txt`,
1,113 bytes: browsers, Office, IObit tools, chat apps, `explorer.exe`, `svchost.exe`, ...) and set
hotkeys/position/colour. Their own two `SetWindowsHookExW` calls (0x6a09ea WH_GETMESSAGE on the
current thread, 0x6ab600 WH_MSGFILTER) are Delphi VCL internals, not the injection; the global hook
comes from SMGBFilter. Two copies exist because a hook DLL can only enter processes of its own
bitness.
