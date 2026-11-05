# gamebooster-rs design

## Idea

A small Windows CLI (`boost` / `restore` / `status`) that reimplements the local Boost/Restore tweaks
of an existing game booster (SGB), as specified in [docs/spec.md](docs/spec.md): power plan,
services, processes, RAM trim, Game Mode, game priority. Everything it changes is recorded and undone
by `restore`. No kernel driver, no network accelerator. Target machine: a 2019 MacBook Pro running
Windows via Boot Camp, playing BF6.

## Open questions

- Spec section 8 unknowns (SGB's default service list, start-type changes, RAM trim API): answered
  by an SGB capture run, see [docs/hardware-test.md](docs/hardware-test.md).
- Does Turbo off plus a low minimum processor state stop the 800 MHz BD PROCHOT lock on the Mac?
- Automatic boost mode (boost when a configured game starts, restore when the last one exits)?
  Deferred 2026-11-05. The `boost --watch` watcher now exists (it already applies game priority
  when a game starts), so auto-boost could build on it; still deferred.
- How to measure BF6 when its anti-cheat blocks ETW capture (bench incomplete, 2026-11-05): confirm
  the cause, then choose BF6 overlay / screen-update counting / another game.

## Decided

- 2026-11-05: Also help with the Boot Camp Mac's 800 MHz clock lock (BD PROCHOT), not only SGB's
  tweaks. In scope: a low-power boost option (`min_processor_state`, with the existing
  `max_processor_state`) and an 800 MHz warning in `status`. Out of scope: clearing BD PROCHOT
  ourselves (needs a kernel driver; WinRing0 is blocked by Defender, and test signing breaks BF6's
  anti-cheat). ThrottleStop covers that.
- 2026-11-05: `max_cpu_mhz` (optional, unset = no cap): writes *Maximum processor frequency* into
  our plan (AC and DC), the driver-free version of the fixed-multiplier BD PROCHOT workaround.
  800 to 6000 MHz or 0; with `max_processor_state` below 100 both apply, the lower wins.
- 2026-11-05: Also build SGB's other optimizer tweaks (decoded from its `.dbd` files), but only the
  ones that matter for gaming on the Boot Camp Mac and that `restore` can undo.
  Outcome: none built; `status` reports HAGS (`HwSchMode`) read-only (see docs/tweaks/optimizer.md).
- 2026-11-05: No copy of SGB's network accelerator (vendor relay servers + packet driver). Instead:
  a `ping` command that reports latency to game-server regions (changes nothing), and an optional
  "fastest DNS" step on boost that restore undoes.
- 2026-11-05: `ping` reports jitter and loss per region (`ping_samples`, default 10), and
  `ping --route [host]` does an mtr/pathping-style per-hop check (ICMP via `IcmpSendEcho2`, no
  admin, no driver) to tell router vs ISP vs far end. From the ExitLag research
  (docs/exitlag/README.md, adoption items 1-2).
- 2026-11-05: Lower other processes (SGB's TPriorityController, spec 2.4a), opt-in
  (`lower_others`), Below normal by default, only the user's own Normal-priority processes.
- 2026-11-05: `lower_others` also skips by publisher (exe version-info CompanyName substrings,
  built-in list plus `lower_others_exclude_publishers`), so renamed or new vendor exes are safe.
- 2026-11-05: Turn off the Sticky / Filter / Toggle Keys hotkeys while boosted
  (`accessibility_shortcuts_off`, default on: harmless and reversible). Only the hotkey flags, never
  a feature the user has on; session only (`SPIF_SENDCHANGE`, not persisted), so a crash heals on
  sign-out or reboot. Idea from Razer Cortex.
- 2026-11-05: Our power plan copies the user's lid, power button and sleep button actions (AC and
  DC) from the previous plan, like brightness, instead of SGB's "lid does nothing".
- 2026-11-05: Every tweak must be measurable: keep what helps on the target Mac, document what
  doesn't. `gamebooster bench` records frame times (PresentMon, no injection) and compares runs
  with run-to-run noise accounted for. (Status: incomplete, see Open questions)
- 2026-11-05: While boosted, opt the running game out of Windows power throttling (EcoQoS)
  (`game_power_throttling_off`, default on: harmless and process-scoped, nothing stored). Applied
  with game priority; restore hands control back to Windows. No timer-resolution flag. From the
  Hone research (docs/hone/README.md, adoption item 1).
- 2026-11-05: Watch mode + dynamic `lower_others`: `boost --watch` stays running after the boost
  (until Ctrl+C or the console closes), sets game priority when a game starts, and with
  `lower_others_mode = "dynamic"` restrains busy background processes ProBalance-style (Process
  Lasso's per-CPU-count thresholds, Below normal only, never the foreground app). Each restraint is
  journalled in the state file; stopping the watcher releases them, `restore` undoes the rest.
  Measure with bench (off / once / dynamic) before keeping dynamic.
- No kernel driver, ever (spec section 5).
- Services are a strict allow-list; start types are never changed.
- Restore runs in reverse order and keeps the state file if a step fails.
