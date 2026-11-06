# Game Mode

**What it is.** Sets `HKEY_CURRENT_USER\Software\Microsoft\GameBar`, value `AutoGameModeEnabled`
(REG_DWORD) to `1`. The old value is saved first, or noted as absent. Restore writes the old
value back, or deletes it if it was absent.

**Windows default.** On, since Windows 10 version 1903. When the value is absent Windows treats
Game Mode as on. So this is usually a no-op that only makes sure it wasn't turned off.

**Why it helps.** With Game Mode on, Windows prefers the foreground game for CPU and GPU
scheduling, postpones Windows Update driver installs and restart notifications while you play,
and lowers background activity. Small but free.

**Exact setting.** `game_mode = true` in the config. The value lives in your user's registry
hive: run gamebooster elevated from your own account, not from a different administrator account,
or it changes that account's setting instead.

Game Mode only applies to games Game Bar recognises. If BF6 isn't picked up, press Win+G in the
game, open the Game Bar settings and tick "Remember this is a game".

## Try and undo by hand

Settings > Gaming > Game Mode, or:

```bat
reg query HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled
reg add HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled /t REG_DWORD /d 1 /f
rem undo if it was absent
reg delete HKCU\Software\Microsoft\GameBar /v AutoGameModeEnabled /f
```

## Measure

- `gamebooster status` prints the current value.
- The effect is small; you'd need several CapFrameX runs with Game Mode off and on to see a
  difference ([how](README.md#how-to-measure-any-tweak)). Mostly it prevents interruptions.
