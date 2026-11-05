# SGB: GPU overclock, in-game FPS overlay and recording

Plain-language summary of three Smart Game Booster 6.0 features that `gamebooster-rs` does **not**
reimplement. Source: static reverse engineering of the shipped binaries (nothing was run). Details and
evidence: `decomp/OverDrive.dll/`, `decomp/GPUlib.dll/`, `decomp/SMGBFilter64.dll/`,
`decomp/FPS64.dll/`, `decomp/SgbHUD64.exe/`, `decomp/VideoRecord64.dll/`, `decomp/sgbGameRecord.exe/`,
`decomp/RobloxFPS.dll/`.

## GPU overclock (Boost action A2, `OverDrive.dll`)

- **How**: talks to the graphics driver's own tuning interface. AMD: ADL (`atiadlxx.dll`),
  OverDrive 5, 6 and 8 / OverdriveN. NVIDIA: NVAPI (`nvapi.dll`, hidden function IDs, P-state
  offsets). No driver of its own, no BIOS flashing.
- **What it changes**: GPU core clock and memory clock only. No voltage increase found; fan and
  power-limit functions are read and logged.
- **How much**: an "OC level" from `SmartGameBooster.ini` (`[Boost] OCLevel`). Level 1 = half of the
  headroom the driver allows; level 3 = the user's own MHz (`OC_GPU`, `OC_RAM`); otherwise the full
  driver-allowed headroom, capped by a per-model reference clock from SGB's GPU database. It never
  goes beyond what the driver allows, but it does not test stability or watch temperature.
- **Undo**: a `Reset` function writes the default clocks back; a log is kept in
  `...\Logs\OverClock.log`.
- **Your 2019 MacBook Pro (Radeon Pro 5300M/5500M, Boot Camp)**: very likely **not supported**. Every
  path is gated on the driver reporting OverDrive support, and AMD/Apple's Boot Camp driver for these
  mobile chips does not offer manual tuning. Expect the option to be greyed out or do nothing. Check
  yourself: if AMD Radeon Software on your Mac has no "Performance > Tuning" page, SGB can't overclock
  either. On a thin laptop that is already heat-limited, an overclock would mostly add heat anyway.
- `GPUlib.dll` only *reads* per-process GPU usage (the method Process Hacker uses). Harmless.

## In-game FPS overlay (HUD)

How the counter gets into a game:

1. `SgbHUD64.exe` (and `SgbHUD32.exe` for 32-bit games) loads `SMGBFilter64.dll` and calls its
   `SetHook`, which installs a **system-wide Windows hook** (`SetWindowsHookEx`, `WH_CBT`, all threads).
   Windows then loads `SMGBFilter64.dll` into **every program with a window** on your desktop.
2. Inside each program the DLL checks the name against SGB's game list and an exclusion list
   (`FpsFilterList.txt`: browsers, Office, Explorer, ...). In a matching game it loads `FPS64.dll`.
3. `FPS64.dll` uses **Microsoft Detours** to patch the game's graphics calls in memory:
   Direct3D 9, Direct3D 10/11 (DXGI `Present`), OpenGL (`wglSwapBuffers`), and Direct3D 12 through
   `FPSD1264.dll` (`Present` and `ExecuteCommandLists`). **Vulkan is not supported.** It draws the
   overlay with Dear ImGui.

A second global hook (`WH_KEYBOARD_LL`) watches all keystrokes for the overlay, screenshot and record
hotkeys.

## Recording and screenshots

- Inside the game, `FPS64.dll` copies each frame from the game's back buffer at `Present`, records
  sound (DirectSound/WinMM), and hands both to `VideoRecord64.dll`. That DLL starts
  `sgbGameRecord.exe` and passes the data through shared memory.
- `sgbGameRecord.exe` encodes with FFmpeg 4.x (`LibAV\` folder, via the FFVCL Delphi wrapper):
  **software MPEG-4 Part 2** (`-c:v mpeg4 -q N`), BGR24 frames, float or 16-bit audio. No hardware
  encoder (NVENC/AMF) is used even though the FFmpeg build includes them, so recording costs CPU.
  Falls back to a plain GDI screen grab.
- Screenshots: saved by D3DX / GDI+ to the folder set in the HUD settings.

## Privacy and security notes

- None of these components (OverDrive, GPUlib, SMGBFilter, FPS64, VideoRecord, sgbGameRecord,
  SgbHUD) contains networking code. Nothing found sends screenshots, video or keystrokes anywhere.
- The keyboard hook sees **every keystroke on the system** while the HUD runs (needed for hotkeys).
  No code was found that stores them, but it is a broad permission.
- The CBT hook puts an SGB DLL into every windowed program, including password managers and
  browsers (the filter list only stops `FPS64.dll`, not `SMGBFilter64.dll` itself).
- Logs: `%LOCALAPPDATA%\PCGameBoost\Smart Game Booster\Logs\` (`OverClock.log`, `VideoLogger_*.log`).
- `RobloxFPS.dll` is a copy of the open-source Roblox FPS unlocker: it rewrites memory inside the
  Roblox process. Roblox-only.

## Safe to use with BF6?

**Boost: yes. FPS overlay, recording, screenshots and Roblox unlocker: no, keep them off for BF6.**

- BF6 uses EA Javelin, a kernel-level anti-cheat. What SGB's overlay does (a global hook that puts an
  unknown, non-whitelisted DLL into `bf6.exe`, then Detours code patches on DXGI `Present` and D3D12
  `ExecuteCommandLists`) is exactly the technique cheat overlays use. Kernel anti-cheats routinely
  block or report foreign DLLs loaded into the game, and patched `Present` functions are a common
  integrity check. Expected outcomes, best to worst: the DLL is blocked and the overlay just doesn't
  show; the game refuses to start or crashes; the account is flagged. EA does not publish an overlay
  allow-list we could check, and SGB is not a known whitelisted overlay (as Steam, Discord, NVIDIA
  and AMD overlays are). This is an inference from the technique, not a tested result.
- Even with the overlay off for BF6, the `SMGBFilter64.dll` hook loads into **every** windowed
  process while `SgbHUD64.exe` runs, the game included. To be safe, turn the in-game FPS / HUD feature
  off entirely (so `SgbHUD64.exe` is not running) before starting BF6.
- The Boost actions (power plan, services, processes, priority) do not touch the game's memory and
  are what `gamebooster-rs` reimplements. For an FPS counter in BF6 use the game's own performance
  overlay or AMD's/Steam's overlay instead.
- GPU overclock does not touch the game process either, so it is not an anti-cheat concern; on the
  MacBook it most likely does nothing anyway.
