# Install, update and build

Every push to `main` that changes the program (not docs-only pushes) is tested and built by GitHub Actions
([.github/workflows/release.yml](../.github/workflows/release.yml)) and published as `gamebooster.exe`
on the rolling [`latest` release](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest).
That one exe is all Windows needs: the example config and the capture scripts are built in.

## Student install (double-click)

On the 16" MacBook Pro 2019 (Boot Camp, Radeon Pro 5500M) playing BF6:

1. Download `gamebooster.exe` from the
   [Releases page](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest).
2. Double-click it (SmartScreen: **More info** > **Run anyway**; UAC: **Yes**).
3. Read the summary, press Enter.
4. Open Steam > BF6 and play.

The double-click runs `gamebooster install`: it copies itself to `C:\gamebooster-rs`, writes
`gamebooster.toml` there, applies the Mac power fix (CPU 1700 MHz while BF6 runs, GPU 32 W table,
BF6 Borderless; only on that exact Mac, see [commands](commands.md)), and
registers and starts the logon task that runs `gamebooster auto` (see [Automatic boost](auto.md)).
Undo all of it: `C:\gamebooster-rs\gamebooster.exe uninstall` in a terminal. The downloaded copy
can be deleted afterwards.

## Manual install

1. Download `gamebooster.exe` from the
   [Releases page](https://github.com/ztancrell/gamebooster-rs/releases/tag/latest) and put it in
   `C:\gamebooster-rs`. Windows SmartScreen may warn the first time you run it: **More info** >
   **Run anyway**. If PowerShell still refuses it, run `Unblock-File C:\gamebooster-rs\gamebooster.exe`.
2. In a terminal: `cd C:\gamebooster-rs`, then `gamebooster init`. It writes `gamebooster.toml` next to
   the exe (it never overwrites an existing one); edit it in Notepad.

Later: `gamebooster update` downloads the latest `gamebooster.exe` with `curl.exe` (Windows 10 1803
or later), checks it is a Windows program that runs, and swaps it in; it prints the old and new
version (`gamebooster --version`: crate version and git commit). The replaced exe is kept as
`gamebooster.exe.old` until the next command deletes it. Updating while boosted is fine: the state
file stays compatible, so `restore` works with the new version.

## Build

Needs Rust. From Linux (cross-compile; needs the `x86_64-pc-windows-gnu` target and mingw-w64):

```sh
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/gamebooster.exe
```

On Windows: `cargo build --release` gives `target\release\gamebooster.exe`.

`cargo test` runs the platform-independent unit tests (config, state file, order, parsers) on any
OS. The binary itself only does anything on Windows. `build.rs` embeds the git short hash (or
`GITHUB_SHA` in CI) for `--version`.
