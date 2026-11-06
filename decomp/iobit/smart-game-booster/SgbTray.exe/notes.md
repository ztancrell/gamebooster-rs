# SgbTray.exe

Static strings only (carved `pe_00d56e1c_2620472.bin`, same size). Nothing executed.

Notification-area resident. Menu: "Open Smart Game Booster", Settings, "Boost into Gaming Mode".

- **Autostart**: a logon scheduled task created through the Task Scheduler COM API
  (`CreateNewTask_OnLogonEx`, `TaskScheduler_TLB_V2`, `_TASK_RUNLEVEL`). SgbMain's config strings
  place `SmartGameBooster Startup` next to `SgbTray.exe" /auto`, so the task is most likely named
  `SmartGameBooster Startup` and runs `SgbTray.exe /auto`. A second task, `SmartGameBooster SkipUAC`,
  lets the tray start SgbMain elevated without a UAC prompt (`/c Schtasks /run /tn "`, `/skipuac`).
  Exact task XML not recovered.
- **Background work** (timers, `TTimer`/`SetTimer`; intervals not recovered):
  process enumeration (`CreateToolhelp32Snapshot`, `EnumProcesses`, `NtQuerySystemInformation`) to
  spot game launches (`AutoGameLaunch`, "Auto PC Boost", SVIP-only); CPU/GPU/disk temperature via the
  CPUID SDK and WMI ("CPU temperature is too high!"); RAM > 90 % popup; new-game detection (Epic
  manifests, Steam); 7-day "not optimised" reminder (`Pop.ini`); graphics/audio driver-update popups
  that launch `AutoDriver.exe`; auto defrag counts; session change notifications
  (`WTSRegisterSessionNotification`).
- **Network**: `Do timer exec SgbUpdater.` (runs `SgbUpdater.exe /auto`), licence check
  `https://smgblicense.itopupdate.com/check.php`, stats `https://stats.gameboosta.com/gamebooster/...`,
  `https://api.gameboosta.com/`, country lookup `ip-api.com` / `ipinfo.io`, and base-URL fallback
  lists `https://s3.amazonaws.com/myfilelist/list*` / `https://storage.googleapis.com/myfilelist/list*`
  ("AntiBan_TestList.json", used when the vendor's domains are blocked).

## Timers (DFM resources parsed, handlers disassembled)

Superseded: "intervals not recovered" above. TRdTimer components in the `TPF0` form resources
(parsed with a small DFM reader; handler addresses from the published-method table):

| Form / timer | Interval | Start | Handler | What it does |
|---|---|---|---|---|
| `TFormGB4Tray.Timer1` | 1000 ms | on | `0x5bb7e8` | Re-enables the watcher object at `[form+0x384]` if off; if `MinutesBetween(Now, lastUpdate) >= UpdateTime*60` runs `SgbUpdater.exe /auto` and stores Now. `UpdateTime` = `Update\Update.ini [Main] UpdateTime`, default 12 (`0x5b73f0`; the shipped ini has no such key), and `lastUpdate` starts at tray launch (`0x5b742a`). So: **updater every 12 h, first 12 h after the tray starts**. Then `FindWindow("TfrmSmartGameBoosterMain")` to track whether the main window is open. |
| `TFormGB4Tray.RdTimer_SendProstat` | 300000 ms (5 min) | on | `0x5ba648` | Reads `[SysOther] PostDaysBt` from `Update.ini` and runs ProductStat's `/postcache /days <PostDaysBt> /cachepath "<…>\PCGameBoost\ProductData"` (`0x526714`): flushes cached usage stats. |
| `TFormGB4Tray.rdtmr1` | 2000 ms | off (one-shot) | `0x5ba7f8` | If the scheduled task `SMGB_Tray_Boost_Game` exists (`0x5109d8`, `IsTaskExists`), runs it (`0x510914`). |
| `TfrmTempretureAlarm.rdtmrClose` | 30000 ms | off | `0x5b3ef4` | Auto-closes the temperature alarm popup after 30 s. |
| `TfrmTempretureAlarm.rdtmrStopAlarmSound` | 5000 ms | off | `0x5b3f60` | Stops the alarm sound after 5 s. |
| `TTrayTipForm.Timer2` / `Timer3` | 500 / 100 ms | off | `0x5b54c0` / `0x5b54ec` | Tray-tip popup show/fade steps. |

Direct `SetTimer` calls (`0x40218c`) are UI/skin animation (100-200 ms). One runtime
`TTimer.SetInterval(1000)` at `0x58eb01`. The process-watch, temperature, RAM and driver nags run on
the watcher object/threads, not on these DFM timers; their periods were not traced.
