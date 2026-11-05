# Tweaks

One page per tweak gamebooster applies: what it is, what Windows does by default, why it can help
a game, the exact setting, how to try and undo it by hand, and how to measure it. The same text, in
short, is the doc comment at the top of each module in `src/`.

Boost applies them in this order; restore undoes them in reverse.

| # | Tweak | Config switch | Module | Likely impact on BF6 / Boot Camp |
|---|---|---|---|---|
| 1 | [Stop services](services.md) | `services = [...]` | `src/services.rs` | Medium, depends on what runs |
| 2 | [Close apps](close-apps.md) | `close = [...]` | `src/processes.rs` | Medium, depends on what runs |
| 3 | [RAM trim](ram-trim.md) | `ram_trim = true` | `src/ram.rs` | Low; only when RAM is nearly full |
| 4 | [Power plan](power-plan.md) | `power_plan = true` | `src/power.rs` | High: the main one |
| 5 | [Game Mode](game-mode.md) | `game_mode = true` | `src/game_mode.rs` | Low; usually already on |
| 6 | [Game priority](priority.md) | `[[game]] priority = "high"` | `src/priority.rs` | Medium when the CPU is busy |

## How to measure any tweak

Change one thing at a time, and compare the same scene several times with and without it.

1. Install [CapFrameX](https://www.capframex.com/) (it uses Intel's PresentMon underneath), or use
   PresentMon on its own:
   `PresentMon.exe --process_name bf6.exe --timed 60 --terminate_after_timed --output_file run.csv`
2. In BF6, pick something repeatable: the same map, the same route, about 60 seconds, ideally an
   empty Portal/practice match so other players don't change the load.
3. Capture 3 runs without the tweak and 3 with it (`gamebooster restore` / `gamebooster boost`, or
   flip one switch in the config between runs).
4. Compare the **1 % and 0.1 % lows** and the **frame-time graph** (spikes = stutter), not only the
   average FPS. A tweak that removes stutter often barely changes the average.
5. Optional: run [HWiNFO](https://www.hwinfo.com/) in sensors mode to see CPU/GPU clocks,
   temperatures and throttling flags. On a Mac under Boot Camp, thermal throttling is often the real
   limit, and the power plan's effect shows up as steadier clocks.
