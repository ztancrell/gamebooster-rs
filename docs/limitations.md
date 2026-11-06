# Limitations and tips

## Limitations

- Game priority (process method) is set only on games already running when you boost. Start the game first, or run
  `boost` again once it's running, or use `boost --watch` or `auto`. The watcher and `auto` only
  run while their console is open; there is no service or logon task.
- Closed apps are relaunched without their original command-line arguments.
- The power plan values follow the spec; the sleep timeout is 0 (never), an assumption.
- `AutoGameModeEnabled` is per user: run elevated from your own account.
- Not implemented (on purpose): GPU overclock, monitor presets, clipboard clearing, Windows Update
  changes, network acceleration (relay servers; `ping` and fastest DNS are the substitute), tray icon cleanup, killing `explorer.exe`, SGB's other optimizer registry tweaks (see [optimizer](tweaks/optimizer.md)).
- Run on one real machine so far (the target Mac: Boot Camp, BF6); checklist and results in
  [docs/hardware-test.md](hardware-test.md).

## Tips

- Cap BF6's frame rate (in-game) a little below what the Mac sustains when hot: less heat, steadier
  clocks and frame times than running uncapped into the thermal limit.
- Game Mode only applies to games Game Bar recognises: if BF6 isn't detected, open Game Bar
  (Win+G) in the game and tick "Remember this is a game". See [Game Mode](tweaks/game-mode.md).
