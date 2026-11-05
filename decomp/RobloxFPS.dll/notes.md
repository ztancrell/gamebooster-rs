# RobloxFPS.dll notes (Roblox FPS unlocker)

Carved copy: `pe_0c265a47_355328.bin` (i386, PDB `SMGB_gongjie\UnlockRobloxFPS`). Exports
`SetFPSCapEx`, `WatchFPU`. Strings match the open-source rbxfpsunlocker ("Failed to get process base!
Restart Roblox FPS Unlocker", "AttachedProcessesCount", "variable scan"): it finds Roblox processes,
scans their memory for the task scheduler and overwrites the frame-rate cap. Writes into another
process's memory. Roblox-only; irrelevant to BF6.
