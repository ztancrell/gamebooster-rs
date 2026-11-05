# FPS64.dll / FPS.dll / FPSD1264.dll / FPSD12.dll notes (in-game overlay + frame capture)

Carved copies: `FPS64.dll` `pe_038a88ad_1040072.bin`, `FPS.dll` `pe_03264865_854728.bin`,
`FPSD1264.dll` `pe_039a6775_553472.bin`, `FPSD12.dll` `pe_0c2dc247_440832.bin`. PDB
`...\Smart Game Booster\5.3.1\HUD64\FPS.pdb`. Loaded inside the game by SMGBFilter (see its notes).

- **Hooking library**: Microsoft Detours (`.detour`, `.detourc`, `.detourd` sections; `SetThreadContext`
  import is Detours' thread fix-up). Inline code patches on graphics API functions.
- **APIs hooked**: D3D9 (`Direct3DCreate9` dummy device to find vtable; EndScene/Present path,
  "CaptureAviD9"), D3D10/D3D11 (`D3D10/11CreateDeviceAndSwapChain` dummy swap chain ->
  `IDXGISwapChain::Present`, "DX%d_hooked"), OpenGL (`MyWglSwapBuffers`, `MyWglSwapLayerBuffers`).
  D3D12 is in FPSD1264.dll (`HookD12`/`UnHookD12`): dummy `CreateDXGIFactory2` swap chain + command
  queue, hooks `Present` and `ExecuteCommandLists` ("hCommandQueue_Executecmd"). **No Vulkan**
  (no vk* names anywhere).
- **Drawing**: Dear ImGui 1.89.8 (D3D/OpenGL3 backends), D3DX fonts. Shows FPS, plus CPU/GPU
  temperature fed by `UpdateTemperatureInfoForGB3`.
- **Capture**: screenshots via `D3DXSaveSurfaceToFileW` / `D3DX11SaveTextureToFileW` / GDI+;
  video frames copied from the back buffer each Present and handed to VideoRecord64.dll, audio via
  DirectSound / WinMM ("Fps audio record ... win7"). Settings: `...\Smart Game Booster\Hud.ini`,
  `SmartGameBooster.ini`.
- `.fpsseg` is a SHARED section (state shared across all processes that load the DLL). Exports
  `SetHook` (returns 0, a stub) and `GetHook` just read/write flags.
- No network imports.
