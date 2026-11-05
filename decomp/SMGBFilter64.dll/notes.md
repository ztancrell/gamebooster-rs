# SMGBFilter64.dll / SMGBFilter.dll notes (HUD injector)

Carved copies: 64-bit `carved/pe_0410e075_181448.bin` (`HUD64\SMGBFilter64.dll`), 32-bit
`carved/pe_0774b936_154312.bin` (`SMGBFilter.dll`). PDB `F:\SMGB 5\HUD\FPS\SMGBFilter\...\DragDrop.pdb`.
Exports `SetHook`, `GetHook`, `SetFPSGameInfo`, `SetFPSFilter`, `SetFPSHotKey`, `SetAviHotKey`,
`SetScreenshotHotkey`, `GetFPSStatInfo`. No network imports.

## Injection (0x180002d00_SetHook_global_hooks.asm)
- `SetHook` calls `SetWindowsHookExW(WH_CBT, proc, hThisDll, 0)` and
  `SetWindowsHookExW(WH_KEYBOARD_LL, proc, hThisDll, 0)`. Thread id 0 = **global**: Windows maps
  this DLL into every GUI process of the same bitness on the desktop. The CBT proc only calls
  `CallNextHookEx`; the hook is purely a DLL-loading vehicle.
- In each process, DllMain matches the process against the game list the HUD published in shared
  memory (`SetFPSGameInfo`: CreateFileMapping "hMapGame"/"hMapGameeList", exe name and path) and the
  exclusion list (`FpsFilterList.txt`, dbgview, SGB's own HUD). On a match it calls
  `LoadLibraryA("...\FPS64.dll")` (FPS.dll for 32-bit), which does the drawing hooks.
- So SMGBFilter itself is loaded into every windowed process (including the game) while the hook is
  installed; FPS64.dll only into matched games.
- `WH_KEYBOARD_LL` sees every keystroke system-wide; it is used for the FPS/record/screenshot
  hotkeys ("fpsKey", "aviKey", "screenshotKey"). Nothing found that logs or sends keystrokes.
