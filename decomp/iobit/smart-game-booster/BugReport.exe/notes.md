# BugReport.exe notes (crash and debug-log upload)

Static only. Analysed copy: `sgb-extract/carved/pe_015caac4_3106872.bin` (3,106,872 bytes; sha256
prefix `608a9fb5be298dc0`). Method: `strings`.

## What it collects (zip `smgb_debuginfo.zip`)
`bugreport.txt` (madExcept report: exception, call stacks), `*.log` from the SGB log folder
(`AppRun.log`, `Exception.log`, `Registry.log` = licence log, `OverClock.log`), `*.dmp` crash dumps
(also looks up Windows' `CrashControl` `DumpFile`/`MinidumpDir`), `SmartGameBooster.ini`,
`Update\update.ini`, `Driver.ini`, optional screenshot, and a system snapshot: Windows version and
time zone, default browser, `Win32PrioritySeparation`, paging files, memory, the full **process
list** (pid, priority, handles, threads, I/O, memory, runtime, CPU), loaded modules, desktop heap,
volumes (`[%s:] %s, %s, %s, (%s / %s)`), installed hardware.

## Where
`https://stats.gameboosta.com/gamebooster/smgb_bugreport_v2.php?email=%s&type=zip`, plus a ping to
`https://stats.gameboosta.com/multi_app_new.php?action=insert` (`app=smgb6bug&lan=…`, flags for
which SGB exe crashed).

## Silent mode
Every SGB exe has a crash hook that runs `Bugreport.exe /bugreport /smgb /normal|/silent
/files="…" /App "…"`. In `silent` or `nogui` mode the e-mail is set to the placeholder
`silent@silent.com` / `nogui@silent.com` and the zip is uploaded **without showing a window**.
Which mode is used is decided by `MISC_BUGT*` / `BugIgnore` values in `Update\update.ini`
(section `smgb_config`), a file the updater refreshes from the server. Not traced which default
ships. Medium confidence.
