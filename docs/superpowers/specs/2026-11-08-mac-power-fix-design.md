# MacBook Pro 2019 power fix in gamebooster-rs (design)

Date: 2026-11-08. Status: design, not built. Evidence: the mbp16-power project
(`/home/zpt/projects/mbp16-power`, DESIGN.md "Where we are" and the 2026-11-08 run entries).

Labels: **(user)** = the user decided it in the brainstorm; **(my call)** = Claude's default, open to
change; **(builder)** = left to whoever builds it.

## Intent and why

A student buys a 2019 16" MacBook Pro for college, installs Windows with Boot Camp and plays BF6.
After a few minutes the frame rate collapses and the mouse lags. They have never heard of
MorePowerTool, ThrottleStop or the SMC and shouldn't have to. **(user)**

What actually happens (measured in mbp16-power, 2026-11-07/08):

- The SMC in the T2 chip runs a zone-0 power budget. It compares the smoothed CPU+GPU draw (SMC key
  `PZ0F`) with a budget (`PZ0E`): ~82 W cold, falling to ~65-72 W as the case warms. While the draw
  is over budget, `PZ0T` climbs and the SMC cuts the GPU (`MSLG` -> `EGPS` steps), then the CPU
  (`MSLC`, a clock cut the reported ratio hides), then fires PROCHOT. The budget is hard-coded in
  signed SMC firmware; nothing in Windows raises it.
- So the only lever is to keep the total draw under the warm budget: cap the GPU and the CPU together.
- Measured preset: **GPU 32 W + CPU max 1700 MHz**, stock driver flag, BF6 Borderless, Low preset
  (11:15 run, ~7 min of play): draw peaked 65.6 W against a budget that bottomed at 70.2 W (~5 W
  margin); `PZ0T`, `MSLC`, `MSLG` stayed 0, no PROCHOT, GPU limit 32 W throughout; FPS median 53,
  10th percentile 47. 1800 MHz reached the budget edge (`PZ0T` climbing after ~7 min); 1600 MHz kept
  ~8-10 W margin but was CPU-bound (FPS dips the user felt in busy scenes).
- BF6 in Fullscreen mode (profile `GstRender.FullscreenMode 1`) ran at ~20 FPS / ~50 ms GPU time per
  frame on the same hardware; switching to Borderless (`2`) gave 56-60 FPS at once (10:28:58).
- The CPU cap works without a kernel driver: the power plan's *Maximum processor frequency*
  (gamebooster's existing `max_cpu_mhz`) held the CPU at the cap and ~16-21 W, like ThrottleStop's
  clamped PL1 did.

## What the student does (user)

1. Download one `gamebooster.exe`.
2. Run it (double-click).
3. Read the output.
4. Open Steam > BF6 and play. Nothing else, ever.

## Goals

- **Set and forget, and automatic (user).**
  - Once: install a GPU soft PowerPlay table capped at **32 W** (`PP_PhmSoftPowerPlayTable`,
    REG_BINARY, under `HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000`).
  - While BF6 runs: CPU cap **1700 MHz** via the existing `max_cpu_mhz` power-plan setting, applied
    by the existing `auto` watcher, lifted when BF6 quits (`auto`'s 30 s delay). Everyday use keeps
    full CPU speed.
  - The first run registers the logon scheduled task from docs/auto.md (`gamebooster auto`, highest
    run level, the battery/time-limit overrides) itself, then starts it. No PowerShell for the student.
- **The whole existing boost stays on (user).** The user has tested the services/priority/DNS steps
  on the Mac. The Mac fix is added to it, not a separate mode.
- **BF6 Borderless, set by gamebooster (user).** Set `GstRender.FullscreenMode 2` and
  `GstRender.WindowBordersEnable 0` in
  `%USERPROFILE%\Documents\Battlefield 6\settings\steam\PROFSAVEbf6mp_profile` (plain text,
  `Key value` lines), only while `bf6.exe` is not running, keeping a backup; `restore` puts the old
  values back.
- **Mac-only gate (user).** The Mac parts (GPU table, 1700 MHz default, Borderless) run only when all
  of these match:
  - `Win32_ComputerSystem` Manufacturer `Apple Inc.`, Model `MacBookPro16,1`
  - GPU `PCI\VEN_1002&DEV_7340&SUBSYS_020F106B` (AMD 5500M, Apple subsystem 106B)
  - VBIOS `113-D32206U1-019` (the 8 GB card measured)
  Otherwise: one line ("Mac power fix skipped: not a MacBookPro16,1 with the 5500M") and the normal
  boost runs. **(my call: the rest of the boost still runs)**
- **Plain output.** The run ends with a short summary a student understands: what was set, that it
  applies itself whenever BF6 starts, and `gamebooster restore` to undo everything.

## Non-goals

- No kernel driver (for now, user). A signed PL1/MSR driver was discussed: possible (Macs Fan
  Control's signed kernel driver does not trip EA Javelin online) but costly (EV certificate,
  Microsoft attestation signing, BSOD risk). Revisit only if the power-plan cap falls short.
- No `PP_Apple_Bootcamp_Enable` flag change, no GPU-cut bypass (mbp16-power's cutbypass). With the
  draw under budget the SMC never cuts, so the stock flag is fine (10:18 and 11:15 runs, flag 1).
- No ThrottleStop, no MorePowerTool, no Macs Fan Control needed.
- No SMC writes, no firmware changes (the T2 route stays research only in mbp16-power).
- Other laptops, other GPUs, other games' presets. Linux: out of scope (amdgpu ignores ATIF
  forced-power-state requests, so Linux runs as if bypassed; the SMC's MSLC/PROCHOT still apply;
  inferred, not measured).

## Anti-goals (would be a failure even if it "works")

- Leaving the machine changed in a way `restore` can't undo, or that survives an uninstall silently.
- Writing a power table to any GPU other than the exact model/subsystem/VBIOS above.
- Running hotter or less safely than stock.
- Needing the student to understand registry keys, power tables or the SMC to use or undo it.
- Breaking BF6's anti-cheat.

## Constraints

- Admin rights: boost/restore already refuse to run without them. A double-click must get them
  without the student knowing "Run as administrator" (builder: elevation manifest or self-relaunch
  via `runas`, as mbp16-power's capture.cmd does).
- `restore` semantics (docs/safety.md): everything is recorded in `gamebooster-state.json` before
  the first change; restore is idempotent and keeps the state file if a step fails.
- ThrottleStop running overrides `max_cpu_mhz` (docs/config.md). Warn if `ThrottleStop.exe` runs.
  A PL1 left in the CPU by ThrottleStop persists until reboot.
- The soft PowerPlay table is read when the driver starts: apply it with a GPU restart
  (`pnputil /restart-device <instance id>`, instance id from `pnputil /enum-devices /class Display`
  matching `VEN_1002&DEV_7340`; see mbp16-power `tools/cutbypass/src/main.rs` `gpu_id()` and
  `restart_gpu()`, ~line 194). **(my call: restart, not a reboot)**
- BF6's profile is rewritten by the game on exit and can be synced by Steam Cloud; only edit it
  while the game is closed.
- Std-only Rust, Windows APIs via the existing patterns (gamebooster-rs has no extra dependencies).

## How it fits the existing code (builder, pointers)

- `src/main.rs`: today a run with no command prints usage and needs `gamebooster.toml` next to the
  exe (`Config::load` errors without it). A double-click needs a no-argument path: first-run setup
  (write the config with `install::init`, apply the Mac parts, register and start the `auto` task,
  print the summary, wait for Enter so the window doesn't vanish). **(builder: shape it)**
- `src/power.rs`: `max_cpu_mhz` / `PROC_FREQ_MAX` already exist. On the gated Mac the default becomes
  1700 (my call: the Mac default lives in the written config, so the student can see and change it).
- `src/watch.rs` (`watch::run`, the `auto` watcher) and docs/auto.md (logon task): reuse as is;
  first run registers the task. **(my call: start the `auto` console minimised with a plain window
  title saying what it is and that closing it undoes the boost)**
- `src/state.rs`: add the GPU table's previous value (absent, or the old bytes) and the BF6 profile
  backup to the state, or to a separate install record (see open questions).
- `src/install.rs`: `init`, `update` and the release URL already exist; distribution is the
  existing GitHub release exe.
- `status`: report whether the Mac fix is applied (table present and 32 W, BF6 Borderless, task
  registered).

## Decided by the user (2026-11-08 brainstorm)

- Target user: a student who never heard of MPT/ThrottleStop; download, run, read, play.
- Set-and-forget plus automatic: GPU table once, CPU cap via the existing `auto` watcher.
- The whole existing boost stays on.
- Mac-only gate on Apple / MacBookPro16,1 / 5500M Apple subsystem / VBIOS.
- gamebooster switches BF6 to Borderless itself (with backup and restore).
- No kernel driver for now.

## Claude's calls (change any of these)

- Apply the GPU table with a GPU restart instead of asking for a reboot.
- Keep the GPU table on outside games (32 W is below the stock 50 W and harmless for everyday use).
- The `auto` console starts minimised with a plain title.
- On a non-matching machine, skip only the Mac parts; the rest of the boost runs.
- The 1700 MHz default is written into the config, visible and changeable.

## Open questions

- **Where the 32 W table bytes come from.** The table now in the Mac's registry was made with
  MorePowerTool from the stock vBIOS table (`C:\Users\bootcamp\Desktop\5500m-modded.mpt` /
  `5500m-unmodded.mpt` also exist). Options: ship that blob (simplest; it is AMD/Apple vendor data,
  redistribution may be a concern, and it also carries 950 mV and other MPT edits), or build it at
  run time by reading the card's own table and patching only the PPT field (no vendor data shipped;
  needs the table layout). Not settled.
- **Restore of the GPU table.** `restore` ends a boost, but the table is meant to stay. Does
  `restore` remove it (simple, matches "restore undoes everything") or does it need its own
  `uninstall`? Today's text in the summary ("restore undoes everything") assumes the former.
- **Borderless on every boost or once?** Once at first run, or re-checked on each `auto` boost
  (the game or a settings reset can flip it back, as happened around 09:00 on 2026-11-08)?
- **15-minute confirmation still owed** for 32 W + 1700 MHz (only ~7 min measured). If it fails,
  fall back to 30 W or 1600 MHz.
- **Future, not now:** an adaptive cap that reads the SMC budget (`PZ0E`/`PZ0F`) and loosens or
  tightens `max_cpu_mhz` live; needs SMC access (Macs Fan Control's driver holds it exclusively).
