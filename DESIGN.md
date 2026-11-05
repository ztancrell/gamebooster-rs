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

## Decided

- 2026-11-05: Also help with the Boot Camp Mac's 800 MHz clock lock (BD PROCHOT), not only SGB's
  tweaks. In scope: a low-power boost option (`min_processor_state`, with the existing
  `max_processor_state`) and an 800 MHz warning in `status`. Out of scope: clearing BD PROCHOT
  ourselves (needs a kernel driver; WinRing0 is blocked by Defender, and test signing breaks BF6's
  anti-cheat). ThrottleStop covers that.
- 2026-11-05: Also build SGB's other optimizer tweaks (decoded from its `.dbd` files), but only the
  ones that matter for gaming on the Boot Camp Mac and that `restore` can undo.
- No kernel driver, ever (spec section 5).
- Services are a strict allow-list; start types are never changed.
- Restore runs in reverse order and keeps the state file if a step fails.
