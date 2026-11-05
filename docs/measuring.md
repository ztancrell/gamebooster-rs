# Measuring

Every tweak has to earn its place on the target Mac (DESIGN.md): `gamebooster bench` records the
game's frame times, and `gamebooster bench compare` says whether two runs really differ or only
by run-to-run noise.

## Commands

Run as administrator (PresentMon reads Windows' ETW events, which needs admin), with the game
running and in a match:

```powershell
.\gamebooster.exe bench                 # 60 s
.\gamebooster.exe bench 90 --label power-off
.\gamebooster.exe bench compare bench\2026-11-05_201500_normal.json bench\2026-11-05_202100_boosted.json
```

`bench [seconds] [--label <name>]` (10 to 3600 s, default 60):

- Picks the first game from `[[game]]` in the config that is running; errors if none is.
- Records it with PresentMon for that many seconds (it stops early if the game exits). Switch to
  the game right after starting it.
- Writes `bench\<date>_<time>_<label>.json` next to gamebooster.exe, plus the raw PresentMon CSV with
  the same name (open it in CapFrameX or a spreadsheet for the frame-time graph). The label
  defaults to `boosted` when a boost is active (the state file exists), else `normal`.
- The JSON holds: label, game exe, duration, start time, whether boosted, the stats below, the
  average and 1 % low FPS of each 10 s chunk, the boost-relevant config keys at bench time (power
  plan on/off and min/max processor state, Game Mode, accessibility shortcuts, RAM trim, number of
  services and apps to close, lower_others and its class, fastest_dns, games), the gamebooster
  version and the PresentMon version.

Example summary (made-up numbers):

```
bf6.exe 60 s, boosted:
  4213 frames, avg 70.2 FPS, median 14.10 ms
  1% low 48.3 FPS, 0.1% low 31.0 FPS, stutters (> 2x median) 12
  avg FPS per 10 s: 69.8 70.1 71.0 70.4 69.5 70.3
Saved C:\gamebooster\bench\2026-11-05_202100_boosted.json (raw frames: C:\gamebooster\bench\2026-11-05_202100_boosted.csv)
```

Stats, from PresentMon's `MsBetweenPresents` column (time between the game's frames; `FrameTime`
if a PresentMon version has no `MsBetweenPresents`):

| Stat | Meaning |
|---|---|
| avg FPS | frames / total frame time |
| median ms | the typical frame time |
| 1% low / 0.1% low FPS | 1000 / the 99th / 99.9th percentile frame time (nearest rank): how slow the worst frames are |
| stutters | frames longer than twice the median |

## How to measure fairly

- **Same scene every run**: the same map, spot and route, or a replay; an empty Portal or practice
  match so other players don't change the load. Look at the same thing.
- **Warm-up run first** and throw it away: shaders compile and files cache on the first pass.
- **At least 3 runs per setting**, alternating: normal, boosted, normal, boosted, ... so slow drift
  (heat, background updates) hits both settings equally.
- **One tweak at a time**: flip one config key (for example `power_plan = false`), `restore`,
  `boost`, bench; label the runs after the key (`--label power-off`). `bench compare` lists the
  config keys that differ between two runs, so you can check you changed only one.
- **Plugged in**, same brightness, nothing else running that you don't run every time.
- **Same thermal state**: a Mac heats up over minutes and then throttles. Let it settle (a few
  minutes in game) before the first run, and keep the pauses between runs equal.

## Reading `bench compare`

```
              A                     B
label         normal                boosted
game          bf6.exe               bf6.exe
...
avg FPS       68.0                  70.2
1% low FPS    44.1                  48.3
config        power_plan: false -> true

avg FPS: B - A = +2.2 (+3.2 %): real difference (Welch t = 3.10 > 2.23, ~95 %)
1% low FPS: B - A = +4.2 (+9.5 %): within run-to-run noise (Welch t = 1.40 <= 2.23)
```

Method: each run is cut into 10 s chunks, and the chunks' average FPS (and 1 % low) are the
samples. Welch's t-test (two samples, unequal variance) compares the two runs' chunk means; if
|t| is above the two-sided 95 % critical value for the Welch degrees of freedom, the difference is
called real, else within run-to-run noise. Runs under 20 s have too few chunks to judge.

Caveat: chunks of one run are not fully independent (heat and the scene drift slowly), so the
test is optimistic. Trust a "real difference" only when it repeats across your 3+ pairs of runs,
in the same direction. A warning is printed when the runs are on different games or their
durations differ by more than 1.5x.

## PresentMon

`bench` uses Intel's [PresentMon](https://github.com/GameTechDev/PresentMon) console app, version
2.6.0 (`PresentMon-2.6.0-x64.exe`), pinned by SHA-256
`b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`. On first use it is downloaded
with `curl.exe` from the official GitHub release into `tools\` next to gamebooster.exe; before every
run its hash is checked with `certutil -hashfile`, and a file that doesn't match is deleted and not
run. It reads the present events Windows already logs (ETW); nothing is injected into the game, so
anti-cheat has nothing to see. Command line used:

```
PresentMon-2.6.0-x64.exe --process_name <game> --timed <seconds> --terminate_after_timed
  --terminate_on_proc_exit --no_console_stats --session_name gamebooster_bench
  --stop_existing_session --output_file bench\<name>.csv
```

### Third-party notice

PresentMon is not part of gamebooster and is not redistributed in this repository; gamebooster
downloads the official release. Its licence:

```
Copyright (C) 2017-2024 Intel Corporation

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"),
to deal in the Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom
the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE
OR OTHER DEALINGS IN THE SOFTWARE.
```
