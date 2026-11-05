# HoneTune.exe 1.0.3 (Game Mode): notes

A .NET console program with obfuscated names but readable strings and logic. It was decompiled
locally and the output deleted. It runs elevated for the length of a game session and undoes its
changes when the game (or the parent Hone process) exits, on manual stop, or at the next start if a
previous session crashed (it keeps state in `Global\HoneTune_*` shared memory and in
`HKLM\SOFTWARE\Hone\PowerPlanBackup`).

## Session model

- The app passes a target (process name, PID, or a benchmark exe path), a session id and the action
  lists. HoneTune tracks the game's child processes, so a launcher-started game is followed, and it
  ends the session when the last tracked process exits.
- A second start sends the new operations to the running instance over shared memory, so one session
  can cover several games.
- On stop it shows a "Game Mode stopped" toast through `hone-notifier.exe`, and it reports session data
  (start and end times, end reason, RAM saved, actions taken) back to the app, which uploads it.

## What it can change (all temporary)

| Option | Mechanism | Restored at the end |
|---|---|---|
| Process actions (from the server's mode lists) | Per process name or PID: priority (low … realtime), **suspend** (suspends every thread), or **kill** | Priority back to normal; suspended threads resumed. Killed programs are not restarted. |
| Service actions | Stop, or pause where the service supports it, through the Service Control Manager | Services restarted or resumed |
| Custom power plan (default on) | Imports or activates the Hone plan, remembering the original scheme GUID | Original scheme reactivated |
| Core parking (premium) | Raises "processor performance core parking min cores" (`0cc5b647-…`) to 100 % on AC and DC in the active scheme; the old values are backed up under `HKLM\SOFTWARE\Hone\PowerPlanBackup\CoreParking` | Old AC/DC values written back, backup removed |
| Timer resolution (always on) | `NtSetTimerResolution` to the maximum. On Windows 11 it also sets `GlobalTimerResolutionRequests` = 1 under `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel`, so one process's request applies system-wide again (takes effect after a restart) | Resolution released; the registry value is **deleted** (also needs a restart), even if the user had set it before |
| Power throttling (always on) | `SetProcessInformation` (ProcessPowerThrottling) to opt the game out of EcoQoS/power throttling | Process-scoped; ends with the process |
| Visual effects (always on) | Turns off UI animation effects via `SystemParametersInfo` | Restore not confirmed from strings |
| GPU priority (premium) | `D3DKMTSetProcessSchedulingPriorityClass` on listed processes (GPU scheduling class) | Process-scoped |
| CPU affinity / pinning (premium) | CPU Sets / affinity masks: the game on chosen cores, other processes moved off them. Variants: avoid CPU 0, avoid E-cores (Intel hybrid), avoid the second CCD (AMD multi-CCD), avoid SMT siblings. Skips `audiodg`, `explorer`, `csrss`-type system processes and a short list of launchers and peripheral agents. | Affinities and CPU Set exceptions reset |
| SnapTap (separate command) | Global **low-level keyboard hook** (`WH_KEYBOARD_LL`) doing SOCD resolution: the last pressed of two opposite keys wins | Hook removed on stop |

## Anti-cheat handling

- Before SnapTap starts (and for a status query), HoneTune scans running processes for anti-cheat
  names: BattlEye (`BEService`, `bedaisy`), Easy Anti-Cheat, Riot Vanguard (`vgk`/`vgc`), PunkBuster,
  ACE, EQU8 and Denuvo. It also has a "high-risk game" list (for example Rainbow Six's BattlEye exe).
  If one is found, SnapTap is refused with a toast.
- **EA Javelin (BF6) is not on the list.** SnapTap would start while BF6 runs. Game Mode itself does
  no anti-cheat check. Its affinity and suspend actions only touch processes the server's mode lists
  name.
- No injection into games, no driver. Everything uses documented user-mode APIs, run as administrator.
