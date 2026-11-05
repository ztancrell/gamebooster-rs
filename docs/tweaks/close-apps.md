# Close apps

**What it is.** Ends every running process whose image name is in `close = [...]` (for example
`OneDrive.exe`), after saving each one's full path to the state file. Restore starts each saved
path again (unless it is already running) through `explorer.exe`, so it runs as your normal user
and not with gamebooster's administrator rights.

**Windows default.** Tray and startup apps run all session: cloud sync (OneDrive, Dropbox,
iCloud), chat (Discord, Teams), music, updaters, peripheral and RGB tools, browser background
processes.

**Why it helps.** They use CPU in bursts (a sync, a notification, a redraw), hold RAM, and some
inject overlays into the game's rendering. Closing them leaves the CPU and memory to the game,
most noticeably on a 4-core Mac with 8 or 16 GB.

**Exact setting.** `close = ["OneDrive.exe", "Spotify.exe"]`: names from Task Manager's Details
tab. Empty by default. These are refused in the config: Windows core processes, `explorer.exe`,
`taskmgr.exe`, game launchers (`EADesktop.exe`, `EABackgroundService.exe`, `steam.exe`, Epic),
Defender and the Symantec processes, gamebooster itself, and any configured game.

Limits: closing is like End task, so unsaved work is lost. Apps come back without their original
command-line arguments (as if started from the Start menu).

## Try and undo by hand

```bat
tasklist /fi "imagename eq OneDrive.exe" /v
taskkill /im OneDrive.exe /f
```

Undo: start the app again from the Start menu.

## Measure

- Task Manager > Processes, sorted by CPU and Memory, with BF6 idle in the main menu.
- Task Manager > Performance > Memory "In use" before and after.
- CapFrameX runs with and without the apps closed ([how](README.md#how-to-measure-any-tweak)).
