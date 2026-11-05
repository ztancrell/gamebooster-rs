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
