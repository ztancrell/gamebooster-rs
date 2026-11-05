# Tweaks

One page per tweak gamebooster applies: what it is, what Windows does by default, why it can help
a game, the exact setting, how to try and undo it by hand, and how to measure it. The same text, in
short, is the doc comment at the top of each module in `src/`.

Boost applies them in this order; restore undoes them in reverse.

Measured on the target Mac: to be filled in from `gamebooster bench` runs
([measuring](../measuring.md)); until then the impact column is an expectation, not a result.

| # | Tweak | Config switch | Module | Likely impact on BF6 / Boot Camp | Measured on the target Mac |
|---|---|---|---|---|---|
| 1 | [Stop services](services.md) | `services = [...]` | `src/services.rs` | Medium, depends on what runs | not yet |
| 2 | [Close apps](close-apps.md) | `close = [...]` | `src/processes.rs` | Medium, depends on what runs | not yet |
| 3 | [RAM trim](ram-trim.md) | `ram_trim = true` | `src/ram.rs` | Low; only when RAM is nearly full | not yet |
| 4 | [Lower other processes](priority.md#lower-other-processes) | `lower_others = true` (default off), `lower_others_class`, `lower_others_exclude` | `src/priority.rs` | Low to medium when background apps use CPU | not yet |
| 5 | [Power plan](power-plan.md) | `power_plan = true`, `max_processor_state = 100`, `min_processor_state` (unset = 100) | `src/power.rs` | High: the main one | not yet |
| 6 | [Game Mode](game-mode.md) | `game_mode = true` | `src/game_mode.rs` | Low; usually already on | not yet |
| 7 | [Accessibility shortcuts](accessibility-shortcuts.md) | `accessibility_shortcuts_off = true` | `src/accessibility.rs` | None on FPS; stops Sticky Keys pop-ups | not yet |
| 8 | [Fastest DNS](network.md#fastest-dns) | `fastest_dns = true` (default off) | `src/network.rs` | Low: faster lookups/connects, not in-game ping | not yet |
| 9 | [Game priority](priority.md) | `[[game]] priority = "high"` | `src/priority.rs` | Medium when the CPU is busy | not yet |
| 9b | [Game power throttling off](priority.md#power-throttling-off) (same step as 9) | `game_power_throttling_off = true` | `src/priority.rs` | Low; only when the game is in the background or on battery | not yet |

Not a tweak: `gamebooster ping` ([network](network.md#ping)) measures latency to cloud regions and
changes nothing.

Not built: [SGB's other optimizer tweaks](optimizer.md) (placebo or harmful on Windows 10/11);
`gamebooster status` only reports whether hardware-accelerated GPU scheduling is on.

Not a tweak, but related: [Stuck at 800 MHz (BD PROCHOT)](prochot.md), the problem `gamebooster
status` warns about, and how to fix it.

## How to measure any tweak

Change one thing at a time, and compare the same scene several times with and without it. The
full method is in [measuring](../measuring.md).

1. `gamebooster bench --label <name>` (it runs PresentMon for you), then
   `gamebooster bench compare a.json b.json`. [CapFrameX](https://www.capframex.com/) (also
   PresentMon underneath) can open the raw CSV `bench` saves next to each JSON, for the graph.
2. In BF6, pick something repeatable: the same map, the same route, about 60 seconds, ideally an
   empty Portal/practice match so other players don't change the load.
3. Capture 3 runs without the tweak and 3 with it (`gamebooster restore` / `gamebooster boost`, or
   flip one switch in the config between runs).
4. Compare the **1 % and 0.1 % lows** and the **frame-time graph** (spikes = stutter), not only the
   average FPS. A tweak that removes stutter often barely changes the average.
5. Optional: run [HWiNFO](https://www.hwinfo.com/) in sensors mode to see CPU/GPU clocks,
   temperatures and throttling flags. On a Mac under Boot Camp, thermal throttling is often the real
   limit, and the power plan's effect shows up as steadier clocks.
