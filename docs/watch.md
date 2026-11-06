# Watch mode

`gamebooster boost --watch` boosts as usual, then stays running in the console, checking once a
second:

- **Games.** When a configured `[[game]]` starts, it sets its priority and power throttling off, so
  you don't have to run `boost` again after launching BF6. When the game exits, both end with it.
- **Dynamic lower others** (`lower_others = true`, `lower_others_mode = "dynamic"`). Like Process
  Lasso's ProBalance: one of your Normal-priority background apps (same filter as `lower_others`,
  and never the window in front or its child processes) goes to Below normal when the whole
  machine is busy and that app has used a big share of it for 0.9 s; it goes back to Normal once it
  is quiet and has been restrained for at least 4.2 s, or at once when you switch to it. The
  thresholds come from Process Lasso's table by logical CPU count (16 threads: machine >= 10 %,
  app >= 5 %, release under 2 %); see [game priority](tweaks/priority.md#dynamic-mode).

It prints one line per change (`restrained chrome.exe (pid 4120, 9 % CPU) to below normal`,
`released chrome.exe (pid 4120) after 6 s: quiet again`) and a status line at most every 10 s when
something changed (`[2:30] CPU 34 %, 1 restrained (chrome.exe), game running`). Ctrl+C or closing
the window releases what the watcher restrained and prints "Still boosted: run `gamebooster
restore` when you're done playing.": the rest of the boost stays until you run `restore`. Each
restraint is recorded in the state file as it happens, so if the watcher is killed, `restore`
still undoes it. If you run `restore` in another window, the watcher notices and stops.
