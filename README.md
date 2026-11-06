# gamebooster-rs

> **Made with AI.** The code, docs and research in this repo were written by Claude Opus 5.5
> (Anthropic) through Claude Code. I direct the project, make the decisions and test on real hardware.

A small Windows CLI that tweaks the system for gaming with `boost` and undoes every change with
`restore`. No kernel driver, no network accelerator. Written in Rust for playing Battlefield 6 under
Boot Camp on a Mac; every tweak is explained, with the manual commands to try it yourself.

## What boost does

In this order; restore undoes them in reverse. Details, manual commands and how to measure each one:
[docs/tweaks/](docs/tweaks/README.md).

1. [Stop services](docs/tweaks/services.md) you list. Restore starts them again.
2. [Close apps](docs/tweaks/close-apps.md) you list. Restore relaunches them.
3. [RAM trim](docs/tweaks/ram-trim.md) of other processes (off by default, not undoable).
4. [Lower other processes](docs/tweaks/priority.md#lower-other-processes): lower CPU and disk priority (off by default).
5. [Power plan](docs/tweaks/power-plan.md): a `gamebooster` copy of High performance, optional Turbo off or MHz cap.
6. [Game Mode](docs/tweaks/game-mode.md) on.
7. [Accessibility shortcuts](docs/tweaks/accessibility-shortcuts.md) (Sticky / Filter / Toggle Keys hotkeys) off.
8. [Fastest DNS](docs/tweaks/network.md#fastest-dns) on the active adapter (off by default).
9. [Win32PrioritySeparation](docs/tweaks/priority.md#win32priorityseparation) `0x2A`.
10. [Game priority](docs/tweaks/priority.md) High and power throttling off for configured games.

The full step-by-step description is in [docs/commands.md](docs/commands.md#what-boost-does-in-order).

## Install and update

1. Download `gamebooster.exe` from the
   [`latest` release](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest) into
   `C:\gamebooster`. If SmartScreen warns: **More info** > **Run anyway**.
2. In a terminal: `cd C:\gamebooster`, then `gamebooster init` (writes `gamebooster.toml`).
3. Later: `gamebooster update` replaces the exe with the latest release.

Build from source (Linux cross-compile; on Windows plain `cargo build --release`):

```sh
cargo build --release --target x86_64-pc-windows-gnu
cargo test
```

More: [docs/install.md](docs/install.md).

## Quick start

In a terminal **as administrator**, in `C:\gamebooster`:

```bat
gamebooster auto       :: boost when a game starts, restore after it exits
gamebooster boost      :: or boost by hand (after starting the game)
gamebooster restore    :: undo everything
gamebooster status     :: what is active, power plan, CPU clock
```

## Commands

| Command | What it does |
|---|---|
| `boost` | Apply the enabled tweaks (`--dry-run` prints the plan). |
| `boost --watch` | Boost, then keep running: game priority on launch, dynamic lower others. |
| `auto` | Boost when a game starts, restore after the last one exits. |
| `restore` | Undo the last boost from the state file. |
| `status` | Show boost state, power plan, Game Mode, HAGS, MPO, Game DVR, CPU clock. |
| `ping [--route [host]]` | Latency, jitter and loss to cloud regions, or per hop. Changes nothing. |
| `bench`, `bench compare` | Record frame times with PresentMon and compare two runs. |
| `snapshot`, `compare` | Save and diff a read-only snapshot of system state. |
| `init`, `update` | Write the example config; update the exe. |

Boost, restore and bench need administrator rights. Full reference: [docs/commands.md](docs/commands.md).

## Config

`gamebooster init` writes [gamebooster.example.toml](gamebooster.example.toml) as
`gamebooster.toml` next to the exe. Every key is explained in that file; each tweak can be switched
off on its own. Unknown keys are errors. All keys with defaults in one table:
[docs/config.md](docs/config.md).

## Safety

- Before the first change, boost writes everything restore needs to `gamebooster-state.json`; if
  that write fails, nothing is changed.
- After a crash, a power cut or a reboot, just run `gamebooster restore`. It is idempotent and keeps
  the state file until a clean restore.
- Nothing is permanent: service startup types aren't touched, so a reboot restarts services too.
- By hand: `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced),
  `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`, `netsh interface ip set dns "<adapter>" dhcp`.
- Run `gamebooster restore` before any "restore default power plans" button.

Details: [docs/safety.md](docs/safety.md).

## More

- [Automatic boost](docs/auto.md) (`auto`) and [watch mode](docs/watch.md) (`boost --watch`)
- [Measuring](docs/measuring.md) with `bench` (blocked on BF6 for now)
- [Hardware test](docs/hardware-test.md): checklist and results on the target Mac
- [MacBook Pro 2019 stuck at a low clock](docs/macbook-pro-2019-slow-cpu.md)
- [Limitations and tips](docs/limitations.md)
- [Research](docs/research.md): what other boosters and tuning guides do, with verdicts
- [Spec](docs/spec.md) the behaviour follows; [design](DESIGN.md)
