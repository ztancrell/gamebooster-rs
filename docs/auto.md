# Automatic boost

`gamebooster auto` (as administrator; leave the window open) runs the same watcher without
boosting first. When a configured `[[game]]` starts it runs the normal `boost` (`bf6.exe started:
boosting`) and then watches like `boost --watch`. When the last game has exited it waits
`auto_restore_delay_secs` (default 30 s; `last game exited: restoring in 30 s`); a game starting
again within that time keeps the boost (a restart or a launcher hand-off). Then it runs the normal
`restore` (`restored; waiting for a game`) and waits for the next game.

- It only restores boosts it made. If you were already boosted when `auto` started (or boost by
  hand while it waits), it watches that boost like `boost --watch` and leaves it for you to restore.
- The window's title says what it is (`gamebooster auto: boosts BF6 while it runs. Closing this window
  undoes the boost`). Started by the logon task (a window of its own) it starts minimised; run by hand
  in a terminal, the terminal is left as it is.
- After `install` on the gated Mac (see [commands](commands.md)), `auto` puts BF6 back in Borderless
  when it starts and after each restore (30 s after BF6 exits, once the game has finished rewriting
  its profile), if BF6 or Steam Cloud changed it; nothing is printed when it is still Borderless.
- If you run `restore` in another window while a game is running, it doesn't boost again until all
  games have exited and one starts.
- Ctrl+C or closing the window while it is boosted restores; while it waits, it just exits. Windows
  allows about 5 s after the window is closed: if restore doesn't finish in time, the state file
  stays, so run `gamebooster restore`.
- If a restore step fails, the state file is kept and `auto` treats it as a boost it didn't make:
  run `gamebooster restore`.
- With `game_priority_method = "registry"`, `auto` writes the games' IFEO entries when it starts,
  before any game, because Windows reads them only when the game starts (a boost after that is too
  late). It keeps them across its boosts and restores and puts them back exactly when it exits.
  They are recorded in `gamebooster-auto-ifeo.json`, so if `auto` is killed, `gamebooster restore`
  puts them back (run while `auto` is still open, it ends that hold as well). If a boost is still
  active when `auto` exits (one it didn't make), the entries go to that boost's state file and its
  `restore` puts them back.

## Start at logon

A scheduled task can start `auto` when you log on, as administrator, in its own console window
(closing that window restores, as above). `gamebooster install` (or a double-click on the exe)
registers exactly this task itself, replacing one named `gamebooster`, and starts it;
`gamebooster uninstall` removes it. By hand, run once in an administrator PowerShell:

```powershell
$u = "$env:COMPUTERNAME\$env:USERNAME"
$a = New-ScheduledTaskAction -Execute 'C:\gamebooster-rs\gamebooster.exe' -Argument 'auto' -WorkingDirectory 'C:\gamebooster-rs'
$t = New-ScheduledTaskTrigger -AtLogOn -User $u
$p = New-ScheduledTaskPrincipal -UserId $u -LogonType Interactive -RunLevel Highest
$s = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit 0 -MultipleInstances IgnoreNew
Register-ScheduledTask -TaskName gamebooster -Action $a -Trigger $t -Principal $p -Settings $s
```

The settings override Task Scheduler defaults that would break `auto`: no start on battery, stop
when unplugged, and a 72-hour kill. Use `$env:COMPUTERNAME`, not `$env:USERDOMAIN` (over SSH the
latter is `WORKGROUP` and registration fails). Remove with
`Unregister-ScheduledTask gamebooster -Confirm:$false`; start it now with `Start-ScheduledTask gamebooster`.
