# Process Lasso: what it really does

Process Lasso (Bitsum LLC, bitsum.com) is a long-running Windows process manager. Its best-known
part, **ProBalance**, lowers background programs for a few seconds while they hog the CPU, so the
program you are using stays responsive. This page comes from reading the official installer of
**Process Lasso 19.0.0.94** (static reverse engineering, 2026-11-05; signed by Bitsum Technologies)
and Bitsum's public docs. **Nothing was run.** Technical notes, hashes and addresses:
[../../decomp/bitsum/process-lasso/README.md](../../decomp/bitsum/process-lasso/README.md).

## Short version

- **What you install:** a signed NSIS installer. It installs a core engine (`ProcessGovernor.exe`)
  that runs as a **Windows service (SYSTEM)**, a GUI, a session agent and small tools. **No kernel
  driver**, no injection into other processes, no bundled offers.
- **What it does to processes:** only documented Windows calls: priority class, CPU affinity, I/O
  priority, Efficiency Mode, working-set trim, power-plan switching. It never touches a process's
  memory.
- **ProBalance is on by default, for everyone (Free and Pro).** Everything else that changes your
  system is off until you turn it on, except the "Bitsum Highest Performance" power plan, which is
  created at install but only activated by Performance Mode.
- **Free vs Pro:** free forever, but "some advanced features" switch off after 14 days, with a
  startup nag until you activate. Commercial use needs a licence after 30 days.
- **Network:** an HTTPS update check, licence activation, and a few server-side text files. Updates
  are signature-checked (`WinVerifyTrust`) before they run. Bitsum says no user or process data is
  sent; nothing found contradicts that.

## Features, one by one

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky**
(can break things, cost security or privacy, or trip anti-cheat), **bloat**. "Mac" means the target
machine: 2019 MacBook Pro, Boot Camp, Windows 10/11, BF6 (EA Javelin anti-cheat).

| Feature | What it really does | Default | Reversible | Verdict (Windows gaming) | Mac + BF6 |
|---|---|---|---|---|---|
| **ProBalance** | Background process using ≥ X % of the machine for 0.9 s, while the machine is ≥ Y % busy → Below normal (exact numbers below); back to normal when it calms down (after ≥ 4.2 s), when you switch to it, or when it exits. Foreground app, its children, services, non-Normal processes and critical Windows processes are never touched | **On** | Yes, automatic | **useful**: real but small gain when something in the background spikes; it does nothing while the PC is quiet | **useful**: the 4-core/6-core/8-core Macs are often CPU-bound in BF6; see the alt-tab note below |
| ProBalance options: Idle instead of Below normal, I/O priority Very low, Efficiency Mode, affinity, core parking off | Stronger restraint | Off | Yes | I/O-low: **useful** for disk-heavy updaters; Idle/affinity: **risky** (voice chat/stream stutter) | keep off, except maybe I/O |
| **Performance Mode** | While a chosen game runs: switch to the "Bitsum Highest Performance" plan, then back. No priority changes. Pauses IdleSaver | Off (Steam auto-detect off) | Yes (also crash-recovery GUID) | **useful** on desktops, mildly | **harmful** on the Mac: see BHP |
| **Bitsum Highest Performance plan** | Copy of Ultimate Performance (or High performance): minimum processor state **100 %**, core parking off, "unpark at once, park slowly" timing | Created at install, not active | Yes (switch plan) | placebo to mildly useful on desktops (same as Ultimate Performance) | **harmful**: 100 % minimum state keeps the CPU hot. That works against our `min_processor_state` / BD PROCHOT approach ([DESIGN.md](../../DESIGN.md)) |
| **IdleSaver** | Switch to a power-saving plan when you're away; back on input | Off in the loader; may be on after the installer's initial config | Yes | **useful** for power, neutral for games | Windows ignores **controller input** as activity, so it can kick in mid-game on a gamepad. Turn it off or rely on Performance Mode |
| **SmartTrim** | At ≥ 65 % RAM load: trims working sets of processes > 196 MB, optionally purges the standby list | Off | Trims are self-healing | **placebo** to **harmful** (trimmed apps page back in, causing stutter). Same as SGB's RAM trim | skip |
| **CPU Limiter** | When a process stays over a CPU threshold, shrinks its affinity to fewer cores for a while | Off (needs rules) | Yes (timed) | **useful** for a known runaway app; never on the game | optional |
| **Persistent priority / affinity / CPU Sets / I/O / GPU priority rules** | Re-applied every time the exe starts; optionally through the IFEO registry, so Windows applies them | Off (user rules) | Yes (delete rule) | **useful** for CPU Sets on hybrid/X3D CPUs; Bitsum itself advises **not** raising game priority | not needed (no hybrid cores) |
| Foreground boost | Foreground process → Above normal | Off | Yes | placebo (Windows already boosts the foreground) | skip |
| IRQControl / interrupt affinity, MSI mode | Registry changes to device interrupt routing, with backup | Off (separate tool) | Yes (Restore Original) | **risky** (can stop a device from starting) | skip |
| Instance limits, disallowed processes, watchdog, keep-running | Admin tools | Off | Yes | bloat for gaming | n/a |
| Core engine service | SYSTEM service, about one sample per second of every process | On | Uninstall | small constant overhead (Bitsum: ≤ 1 % CPU) | fine |

## ProBalance, exactly

The thresholds are **percent of the whole machine**. Bitsum's INI reference only says "varies". The
binary shows they depend on the number of logical CPUs:

| Logical CPUs | Machine busy ≥ | Process ≥ | Restored below | Over-threshold time | Min. restraint | Max. restraint |
|---|---|---|---|---|---|---|
| 4 | 23 % | 15 % | 5 % | 900 ms | 4.2 s | none |
| 8 (13" MacBook Pro 2019) | 11 % | 8 % | 4 % | 900 ms | 4.2 s | none |
| 12 (16", i7-9750H) | 10 % | 7 % | 3 % | 900 ms | 4.2 s | none |
| 16 (16", i9-9880H/9980HK) | 10 % | 5 % | 2 % | 900 ms | 4.2 s | none |

In words, on a 16-thread Mac: when the PC is at least 10 % busy, and a background program has used
at least 0.8 of a thread for 0.9 s in a row, it drops to Below normal. It stays there at least 4.2 s,
until it uses less than 2 % (about a third of a thread). The per-process trigger always sits near one
busy thread ("a thread spinning flat out"), so ordinary background work is left alone. Full table and
evidence: [governor notes](../../decomp/bitsum/process-lasso/governor/notes.md).

## Mac (Boot Camp) and BF6

- **Anti-cheat.** Process Lasso never reads or writes game memory and loads no driver. By default it
  never touches the game at all: the foreground app is excluded, and Performance Mode only switches
  the power plan. Bitsum reports no bans in its history. EAC-style anti-cheats strip the access
  rights it would need, so rules aimed at the game itself simply fail; Bitsum suggests launcher
  affinity or IFEO registry rules instead. For BF6 with Javelin: **low risk, nothing aimed at
  `bf6.exe` needed**.
- **Alt-tab caveat.** Only the *current* foreground app is excluded. If you alt-tab out while BF6
  keeps using a lot of CPU, ProBalance can lower BF6 to Below normal after 0.9 s. It goes back to
  normal when you switch back. On Javelin the call will likely just fail (access denied), which is
  harmless. Add `bf6.exe` to ProBalance's exclusions to be tidy.
- **Power plan.** Do not use Performance Mode/BHP on the Mac. A 100 % minimum processor state
  keeps the i7/i9 hot, which makes BD PROCHOT throttling worse. Keep our own power plan steps.
- **Worth it on the Mac?** ProBalance alone, yes. It is the one Process Lasso feature that matches the
  Mac's real problem (a CPU-bound game sharing 4–8 cores with background spikes). Everything else is
  either already in gamebooster-rs (priority, power throttling), harmful there (BHP, SmartTrim), or
  irrelevant (CPU Sets, IRQ).

## Comparison with gamebooster-rs `lower_others`

| | gamebooster-rs `lower_others` ([priority.md](../tweaks/priority.md)) | ProBalance |
|---|---|---|
| When | Once, at `boost`; until `restore` | Continuously, per second |
| Whom | **Every** own Normal-priority process except exclusions | Only a process that is **actually hogging** (≥ ~1 thread for 0.9 s) while the PC is busy |
| How long | Whole boost | ≥ 4.2 s, until it calms down; no maximum |
| Lowered to | Below normal (or Idle) | Below normal (or Idle); optional I/O Very low, EcoQoS, affinity |
| Exclusions | Name list, publisher list, games, `close` list, SYSTEM/services | Foreground and its children, services, non-Normal, critical processes, user list |
| Foreground | Not special (a Normal foreground app other than the game is lowered) | Never touched; restored at once when it becomes foreground |
| Restore safety | Journal (pid + creation time) | In-memory; exits restore everything |
| Needs | Nothing resident | A resident engine (service) |

Static lowering has two problems: it lowers dozens of idle apps that never compete, and it lowers
things you use (a browser with a guide, Discord when not excluded). ProBalance lowers only the one
process that is actually competing, and only while it competes. Bitsum's own measurements are
responsiveness tests, not game frame times. Neither approach has frame-time evidence on our Mac yet.

## Adoption proposal: dynamic restraint (ProBalance-style), opt-in

**Built** (2026-11-05) as `lower_others_mode = "dynamic"` with `gamebooster boost --watch`: see
`src/watch.rs` and [docs/tweaks/priority.md](../tweaks/priority.md#dynamic-mode). Differences from
the proposal below: the config key is `lower_others_mode = "once" | "dynamic"` (not
`lower_others = "dynamic"`), no `dynamic_*` overrides (Process Lasso's table only), and stopping
the watcher releases its restraints but leaves the rest of the boost on (run `restore` separately).
Not yet measured (item 7).

**Design question first:** gamebooster-rs is a one-shot CLI (`boost` / `restore`). Dynamic restraint
needs a process that keeps running while you play. This is new ground, next to the deferred
"automatic boost mode" open question in DESIGN.md, and it needs a decision there before anyone builds
it. A minimal form that keeps the CLI idea: `gamebooster boost --watch` stays in the foreground of
its console (or minimised) until the game exits or Ctrl+C, then runs `restore`.

Proposed behaviour, all reversible and driver-free, using only APIs we already call
(`SetPriorityClass`, the existing journal):

1. **Config.** `lower_others = "dynamic"` (next to today's `true`/`false`, which stay as they are).
   Optional `dynamic_*` overrides for the numbers below. Off by default.
2. **Sampling.** Once per second: one process snapshot plus `GetProcessTimes` per candidate, and
   `GetSystemTimes` for the machine total. Percent = CPU-time delta ÷ (wall time × logical CPUs).
3. **Candidates:** exactly today's `should_lower` set: same user, Normal priority, not in
   `NEVER_LOWER`, publisher list, games, `close` or `lower_others_exclude`. Plus two ProBalance
   rules: **skip the foreground window's process and its children**, and restore at once a
   restrained process that becomes foreground.
4. **Defaults.** Use Bitsum's per-CPU-count table (evidence in the governor notes), so on the Mac:
   machine ≥ 10–11 %, process ≥ 5–8 % (by thread count), sustained for **900 ms** (with 1 s sampling:
   one full sample over the threshold). Restore when
   below 2–4 % **and** restrained ≥ **4.2 s**. No maximum.
5. **Action.** Below normal only. Idle stays the `lower_others_class` choice and is not recommended.
   No affinity, no EcoQoS, no I/O priority in v1. I/O Very low could be a later flag, since it helps
   with disk-heavy updaters and is equally reversible.
   Superseded 2026-11-06: `lower_others_io` (default on) sets I/O priority Low (not Very low) with
   each lowering or restraint; see [game priority](../tweaks/priority.md#io-priority).
6. **Safety.** Every lowering goes into the same journal (pid, creation time, old class) **before**
   the call. Restore/exit puts back only matching pid + creation time, exactly like `lower_others`
   today. A crash leaves at most a few processes at Below normal until they exit, the same as now.
   Print one line per restraint/restore, for example `restrained chrome.exe (9 % CPU) / restored
   after 6 s`.
7. **Measure.** `gamebooster bench`, BF6 or a stand-in game, with a background load (a browser
   video plus a big download or a CPU burner). Compare `lower_others` off / `true` / `"dynamic"`. Keep
   dynamic only if 1 % lows improve or stay equal with fewer processes touched.

Cost estimate: about one new module (sampling loop + state map), reusing `priority.rs`'s exclusion
and journal code. No new dependencies.
