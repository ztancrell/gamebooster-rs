# Game priority

**What it is.** Sets the CPU priority class of each running `[[game]]` process (default High).
Restore sets running games back to Normal; nothing is needed if the game has exited. The game has
to be running when this step runs: start BF6, then run `gamebooster boost`. If you boosted
before starting the game, run `gamebooster boost` again: while boosted, it only re-applies
priority.

**Windows default.** Every app starts at Normal priority; the scheduler shares CPU time between
all Normal threads, with a small boost for the window in front.

**Why it helps.** At High, the game's threads run before every Normal-priority background thread
that is ready at the same moment, so a background burst can't delay a frame. It only matters when
the CPU is busy (4-core Macs, large BF6 maps); on an idle CPU it changes nothing. Don't use
Realtime: it can starve input, audio and the driver threads the game itself depends on, which
causes stutter or even a frozen system.

**Exact setting.**

```toml
[[game]]
exe = "bf6.exe"     # check the real name in Task Manager > Details while BF6 runs
priority = "high"   # realtime, high, above_normal, normal, below_normal, low
```

`steam.exe`, `explorer.exe`, launchers and Windows core processes are refused as games.

## Try and undo by hand

Task Manager > Details > right-click `bf6.exe` > Set priority > High. Or in PowerShell (admin):

```powershell
(Get-Process bf6).PriorityClass = 'High'
(Get-Process bf6).PriorityClass = 'Normal'   # undo
```

Windows forgets the priority when the game exits; that is also an undo.

## Measure

- Task Manager > Details, column "Base priority" (right-click a column header > Select columns).
- CapFrameX with some background load running (a big download or a file copy), once at Normal and
  once at High ([how](README.md#how-to-measure-any-tweak)).
