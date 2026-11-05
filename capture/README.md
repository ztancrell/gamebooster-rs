# Capture what Smart Game Booster's "Boost" changes

Two PowerShell scripts for Windows PowerShell 5.1 (built into Windows 10/11). Nothing to install.

- `snapshot.ps1 -Out <file.json>` saves a read-only snapshot of the system state that game boosters
  usually change. It reads and changes nothing.
- `compare.ps1 -Before <a.json> -After <b.json>` prints only what changed, grouped as: power, services,
  processes (started / stopped / priority / affinity), registry, Windows Update, BCD, timer resolution,
  free memory.

The same scripts check gamebooster's own boost and restore: see
[docs/hardware-test.md](../docs/hardware-test.md), which also says which spec unknown each part of
an SGB diff answers.

## Steps (on the Boot Camp Windows install)

1. Copy the `capture` folder to the Windows side, for example `C:\capture`.
2. Open PowerShell as administrator: Start menu, type `PowerShell`, right-click
   **Windows PowerShell**, **Run as administrator**. Use your normal Windows account, since the
   script reads your user's settings (HKCU).
3. Allow the scripts for this window only:
   ```powershell
   Set-ExecutionPolicy -Scope Process Bypass
   cd C:\capture
   ```
4. Make sure Smart Game Booster is **not** boosted yet. If it is, press its button to restore first.
5. Decide about Battlefield 6:
   - If Smart Game Booster only boosts while the game is running (it waits for or detects the game),
     launch BF6 now and wait until you are in the main menu.
   - Otherwise do **not** launch BF6. Then the comparison shows only the booster's changes.
6. Take the "before" snapshot:
   ```powershell
   .\snapshot.ps1 -Out before.json
   ```
7. Press **Boost** in Smart Game Booster and wait until it says it is done.
8. Take the "after" snapshot:
   ```powershell
   .\snapshot.ps1 -Out after.json
   ```
9. Compare and save the result to a file:
   ```powershell
   .\compare.ps1 -Before before.json -After after.json | Out-File -Encoding utf8 diff.txt
   ```
10. Send `diff.txt` (and if possible `before.json` and `after.json` too).

What to look for: each snapshot ends with a line like `wrote before.json (... processes, ... services ...)`.
Lines starting with `note:` mean something could not be read; the most common is `bcdedit failed`,
which means the window was not opened as administrator.

## What the snapshot does not capture

- Per-process I/O priority, memory priority and thread priorities (only priority class and affinity).
- What "memory cleaning" freed, other than the change in free memory (`Memory` section).
- GPU driver settings (AMD / Boot Camp driver panels), network adapter properties, firewall, DNS.
- Timer resolution: on Windows 11 this is per process, so the value shown is what this PowerShell
  process sees, which may not be what the game gets.
- Short-lived processes that start and exit between the two snapshots, and any change the booster
  undoes before the second snapshot.
- Registry keys outside the list in `snapshot.ps1` (see the `registry` section of the script).
