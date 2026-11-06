# Safety and restore

- Before the first change, boost writes everything restore needs to
  `gamebooster-state.json` next to `gamebooster.exe` (previous power plan, the services it stops,
  the full paths of the apps it closes, the processes it lowers with their old priority and I/O
  priority, the old Game Mode value, the old accessibility shortcut flags, the old DNS setting, the
  old `Win32PrioritySeparation` value (or that it was absent), the old IFEO entries with the
  registry priority method). If that write fails, nothing is
  changed. The file's presence means "boosted"; boosting again doesn't overwrite it.
- `auto` with the registry priority method records the old IFEO entries in
  `gamebooster-auto-ifeo.json` (same folder) before writing them; `restore` puts them back and
  deletes it. While that file exists, `boost` skips the IFEO step (`auto` already wrote them).
- After a crash, a power cut or a reboot, just run `gamebooster restore`. Restore is idempotent:
  running services, running apps and an already-deleted plan are fine. If starting a service,
  restoring the power plan, writing Game Mode, `Win32PrioritySeparation`, the accessibility flags or DNS fails, the state
  file is kept so you can run restore again; it is deleted only after a clean restore.
- Nothing is permanent: service startup types aren't touched, so a reboot restarts services too.
  Power plan by hand: `powercfg /setactive 381b4222-f694-41f0-9685-ff5bb260df2e` (Balanced) and
  `powercfg /delete a2921d59-9c60-439e-b61a-21c441a0df53`.
  DNS by hand: `netsh interface ip set dns "<adapter>" dhcp`.
- Each tweak is independent: one failing is printed and the others still run.
- Run `gamebooster restore` **before** any "reset / restore default power plans" button (for
  example WinUtil's, which runs `powercfg /restoredefaultschemes`): that deletes our plan, and
  restore then has to fall back to Balanced instead of your old plan.
