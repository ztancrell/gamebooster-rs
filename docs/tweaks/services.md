# Stop services

**What it is.** Stops the services listed in `services = [...]` for the session. For each listed
service that is running, the running services that depend on it are stopped first, then the
service itself, each with a 30 s timeout. Only services that were running are recorded; restore
starts exactly those again, in reverse order. The startup type is never changed, so a reboot also
brings them back.

**Windows default.** Many services run all the time: search indexing (`WSearch`), prefetching
(`SysMain`), telemetry (`DiagTrack`), the print spooler (`Spooler`), plus third-party updaters
(Adobe, Google, Apple Software Update) and Boot Camp's own helpers.

**Why it helps.** Each service wakes up now and then to do work: index a new file, check for
updates, upload telemetry. On a 4-core Mac that work competes with the game's threads and causes
the occasional long frame. The gain depends entirely on what runs on your machine.

**Exact setting.** `services = ["WSearch", "SysMain"]` in `gamebooster.toml`: service *key* names,
not the display names. Nothing is stopped by default. Don't list audio (`Audiosrv`), networking,
security software or Apple/Boot Camp keyboard and trackpad services.

## Try and undo by hand

```bat
rem STATE: RUNNING or STOPPED
sc query WSearch
rem services that depend on it
sc enumdepend WSearch
rem asks before stopping dependents too
net stop WSearch
rem undo
net start WSearch
```

Or `services.msc`: find the service, right-click > Stop / Start. The "Service name" in its
Properties dialog is the key name to put in the config.

## Measure

- `gamebooster boost --dry-run` lists the exact services (with dependents) that would stop.
- `sc query <name>` shows STOPPED while boosted.
- Task Manager > Services tab shows the PID; check that process's CPU use in the Details tab
  before you decide a service is worth stopping.
- CapFrameX runs with and without the service stopped ([how](README.md#how-to-measure-any-tweak)).
