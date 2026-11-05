# SgbMain.exe: everything beyond Boost/Restore

Static analysis of the de-filtered `sgb-extract/app/cmAppVer/SgbMain.exe` (sha256 `060f6621…d16f`).
Nothing was executed. Addresses are VAs. Data files are from `sgb-extract-db/data/{app}/6.0.1/` unless
another path is given. Plain-language summary: `docs/sgb/main-app.md`.

## Method

- **Class list**: Delphi 2009+ VMTs found by scanning for a dword equal to its own address + 0x58
  (vmtSelfPtr = −0x58); class name ShortString at VMT−0x38, parent at VMT−0x30. 1,466 classes.
  Superseded: `notes.md` used −0x4C for the self pointer, which finds only 113.
- **Method names**: SGB logs `Class.Method` strings everywhere; 281 unique ones were collected from
  the UTF-16 string table and give most entry points.
- **Code**: `refs.py` (string → referencing function) and `dis.py` (annotated objdump), both in this
  directory. REA/Ghidra was not used here: the REA server was serving another agent's binary at the
  time, so this pass relied on the two scripts.
- Correction to `notes.md`: 0x5458a0 is the **debug-log** function (UnicodeString →
  `OutputDebugStringW`, ends ≈0x5458f0). The PSAPI loader (`LoadLibraryW("PSAPI.dll")`,
  `GetProcAddress EnumProcesses…`) is the next function, starting just after it at ≈0x5458f4.

## Boost actions not yet covered by `notes.md`

| Action | Code | What it does |
|---|---|---|
| A1 Monitor (`TFrameMonitorDDC`, `TMonitorManager`, `TMonitorInfo`) | Boost 0x7c07bc, Restore 0x7c0f7c, VCP get/set 0x7bd5e0–0x7bdcd0, dxva2 loader strings 0x7bc334 | Skips on Windows XP. Loads `dxva2.dll` dynamically (all the Monitor Configuration API names are at 0x7bc378–0x7bc98c). Enumerates monitors (`GetPhysicalMonitorsFromHMONITOR`), reads EDID from `HKLM\System\CurrentControlSet\Enum\<id>\Device Parameters\EDID` (0x7bf724), parses the capabilities string (`AnalyzeVCP`, 0x7be8b4). Boost reads `Monitor.ini` `[Main] MatrixEnabled, GamingModeEnabled, GamingModeValue` and per monitor `<id>_Boost PresetMode/Brightness/Contrast/ColorTemp`; first saves the current values to `<id>_original` (defaults if read fails: Brightness 100, Contrast 100, ColorTemp 2), then writes the boost values. VCP codes: **0xDC** display mode (`DCMode`/PresetMode), **0x14** select colour preset (`ColorTemp`), **0x10** luminance, **0x12** contrast, each via `SetVCPFeature`, followed by a Sleep. Restore writes `_original` back. Preset names: Standard, Games, Office, Multimedia, Movie, User, Sports, Professional, Balance, Energy saving, Demonstration, Dynamic. `Monitor.ini` lives in the user data dir (`%APPDATA%\PCGameBoost\Smart Game Booster\`). |
| A2 Overclock (`TOverClockBoost`) | loader 0x7b8da4, Boost 0x7b9460, range 0x7b92b8 | `LoadLibrary("OverDrive.dll")` and `GetProcAddress` of **OverClock, Reset, CanOverClock, GetGPUInfo, GetRange**. Skips if `OverDrive.ini [OverClock] exception` is set. Boost: GetRange, then `OverClock(&gpu, &mem, 0)` (fn ptr 0xa57c44, returns 1 on success, fills the applied values); logs percent = value / range (×0.5×100), saves `OverClockGPU`/`OverClockMem` to `OverDrive.ini`. Restore calls `Reset`. Level comes from the settings `[Boost] OCLevel, OC_GPU, OC_RAM` (0x7b9d14–0x7ba118). SgbMain does not reference `GPUlib.dll` by name; that one is loaded by OverDrive.dll or another module. |
| A6 Clipboard (`TClipbrd`) | 0x7b7bc4 | Reads the `[Clipbrd] Clear` setting, then VCL `Clipboard.Clear` (global 0x4eb5f4 → OpenClipboard/EmptyClipboard). No restore. |
| A8 Windows Update (`TUpdateBoost`, `TUpdateController`) | Boost 0x7c26c8, `CloseWindowsUpdate` 0x7c253c, `SetUpdateState` 0x7c2e68 | `CoCreateInstance` of the Automatic Updates object, then `IAutomaticUpdatesSettings`: exits if the service is disabled (`ServiceEnabled Exit`), if the setting is read-only (`ReadOnly Exit`) or required by policy (`Required Exit`); otherwise saves the old NotificationLevel (`WindowsUpdate` key) and sets the new one ("set new configure."). Matches spec 2.5; nothing new. |
| A9 Game Mode (`TGameMode`) | strings 0x7c2200 | `AutoGameModeEnabled` as in the spec. |
| A11 Tidy tray icons | 0x7c7ae9 in TBoost.Execute (`0x7c7a2e_…logonly.asm`) | **Does nothing in 6.0.** The string "Remove Dead Tray Icons" is only passed to 0x5462f4 → 0x5458a0 (`OutputDebugStringW`); no window or tray API follows, the next step is the finish callback. The tray-related window classes (`ToolbarWindow32`, `TrayNotifyWnd`) are referenced only from the Game Desktop exclusion table below. |
| Boost log | 0x7c7a4d | Writes `Logs\Boost.txt` (CopyFileW of the run log). |

## Game detection, launching and auto-boost

- **Scanner** (`TGetGamesInfo`, `TGameScaner` 0x80b7a8, config `GameScaner.ini`): reads launcher data
  for Steam (`libraryfolders`, `GetSteamGamesList`, launch `steam://rungameid/<id>`), Epic
  (`\Epic\EpicGamesLauncher\Data\Manifests\`, `com.epicgames.launcher://apps/%s?action=launch&silent=true`),
  GOG (`SOFTWARE\GOG.com\Games\`), Origin/EA (`\Origin\LocalContent\` + Uninstall keys), Uplay/Ubisoft
  (`SOFTWARE\Ubisoft\Launcher\Installs\`, `uplay://launch/`), Battle.net (`battlenet://`, agent cache),
  Riot (`RiotClientInstalls.json`, `riotclient://launch-product=%s&launch-patchline=live`), Roblox,
  WeGame, Microsoft Store (`WindowsApps`, matched against `Database\MSGameDb.db`: 22,703 rows
  `data_aid, data_pfns (package family name), data_m (Store ID)`), Windows system games, and a WMI
  check (`FindGameExistsWMI`). Each game record: `gameid, Name, GamePathAndFileName, WorkPath, Param,
  Launch/LaunchFile, MonitorFiles, GameType, IsUserGame, UserIcon`, saved in `UserGame.ini` (plus
  `WebGame.ini`, `GameInfo.txt`, `HotGame.ini`). `MonitorFiles` = the exe names that mean "this game is running".
- **Launch**: from the game list (`Btn_LaunchGame`), via launcher URLs above or `CreateProcess` /
  `ShellExecuteEx` of the exe; `AutoGameLaunch` and `/Game`, `-game`, `-gamedesk` switches.
- **Auto-boost when a game starts**: the process watching is **not in SgbMain**. `SgbTray.exe` has
  `OnProcessStart`, `TGameMonitorInfo`, `FGameMonitorList`, `FStreamAuotoBoost`, `GameAutoscan`, and a
  GPU-usage monitor (`EtGpuMonitorInitialization`, Process Hacker style). SgbMain is told to boost by
  message (`TfrmSmartGameBoosterMain.WMBoost`) or command line (`/boost`, `/boostnow`, `/Boost`).
  The setting is `FAutoGameMode` / "Enable Auto Boost …", gated by SVIP ("Upgrade to SVIP" in SgbTray).
- **Hook/HUD** (`THookGameManger` 0x814948): loads `fps.dll` / `SMGBFilter.dll` and calls `SetHook,
  GetHook, SetFPSHotKey, SetScreenshotHotkey, SetAviHotKey, SetFPSFilter, GetFPSStatInfo, SetFPSGameInfo,
  UpdateTemperatureInfoForGB3, SetScreenshotDir, SetAviDir, SetFpsPos, SetCaptureFileType,
  SetVideoRecordParam, Get/SetAudioDevice, SetFontColor, SetCustomFontParam, SetTextPosition,
  SetTextShadowInfo` (0x8140a8–0x8142ec): in-game FPS/CPU/GPU overlay, screenshots, video recording.
- **Game Desktop** (`TGameDesk` 0x7c51d8, `0x7c51d8_TGameDesk_HideDeskApp.asm`): `EnumWindows` and
  `ShowWindow` hide every visible top-level window except 13 classes (table 0xa4be84: shell tray,
  taskbar, IME, SGB's own) and hides the desktop icons (`Progman` → `SHELLDLL_DefView`); hotkeys
  `ExitGameDeskTop1/2`. Also "LauchGameWithVirtualDesktop".

## Startup manager (`TMainCtl`, `TStatupBase`, `TStartupPost`, `TGetDataFromServer`)

Lists Run keys, startup folders, services, scheduled tasks and browser plugins (`PStartUpItem`,
`PServiceItem`, `PTaskScheduleItem`, `PBroswerPluginItem`), and can disable, delete or delay them.
Data under `Startup Manager\` (`config.ini, delayEx.ini, delStartups.ini, OptimizeRecord.ini,
OptFailed.ini, Ignore.ini, boottime.dat, disableRate.db, discription.db`) and these shipped files:

| File | Format | Content |
|---|---|---|
| `startupService.db` | UTF-16 INI (not SQLite), 1,289 lines, 15 sections (`[xp_x86]` … per Windows version/arch) | `ServiceName=StartType` recommendations (2 auto, 3 manual, 4 disabled), e.g. Alerter=4, Messenger=4. Used by the startup manager's service tab, not by Boost. |
| `startupWhite.db` | SQLite: `whiteList` 197 rows (MD5, StartupName, FileName, MFName, Type), `signatureList` 9 | Items never suggested for disabling. |
| `startupBlack.db` | SQLite `blackList`, 7 rows | searchsettings, application updater, couponshelper ×2, BEHelper, **mbamscheduler (Malwarebytes)**, **windefend / msmpeng.exe (Windows Defender)**. |
| `startupSignature.db` | SQLite `signatureList`, 18 signers | Trusted publishers, includes IObit, Avast, AVG, Opera, Yandex, Lavasoft and several Chengdu companies. |
| `StartupDRate.db` | SQLite `disableRate`, 13,367 rows (MD5, "99.85%", recommendation) | Crowd "users who disabled this" rates. |

## CYRI ("Can You Run It") game database

`Database\CYRI.dbd` (16.1 MB) decodes with `../SgbData/dbd/dbd_decode.py`, password `SMGB5_CYRI`, into
two entries (not committed, regenerate with the script):

- `CYRI.db` (62.6 MB SQLite): `CYRIGame` 162,942 games (SMGBID, GameName, Name_s, Categories,
  Min/Rec CPU text + score, Min/Rec GPU text + score, Min/Rec RAM, Min/Rec OS, GameImagePath,
  DownLoadPath, Sort, OnlineCount, UserCount); `CPUScore` 9,471 and `GPUScore` 3,397 (Score, aliases);
  `GPUScore_Hid` 49,806 (PCI hardware ID → score); `TopGame` 1,018 (embedded JPEG hex thumbnails).
- `ScoreCFG.ini` (UTF-16, 129 KB): `[main_GPU]`/`[main_CPU]` name normalisation aliases.

`TCYRIGame` (`MatchMinPC`, `MatchRecPC`, `SearchGame_*`) compares the PC's CPU/GPU score with the
game's. It also posts back: `PostFailUserPC`, `PostSearchCount`, `UpdateUserCount`; updates come from
`CYRIupdate.ini`/`/UpdateCYRIDB`. `GPUInfo.db` (separate SQLite, `GPU_NVIDIA` 1,072 rows …) is the GPU spec table.

## Settings and data files

- **User data dir** `%APPDATA%\PCGameBoost\Smart Game Booster\`: `Main.ini` (main settings),
  `SmartGameBooster.ini` (also under `%USERPROFILE%\AppData\LocalLow\…`, holds `StartParam`),
  `Monitor.ini`, `OverDrive.ini`, `ServiceBoost.ini`, `WinSvc.ini`/`UserWinSvc.ini`,
  `3rdSvcWL.txt`/`User3rdSvcWL.txt`, `ProcWL.txt`/`ProcBL.txt`/`UserProc*.txt`, `AppSafe.txt`,
  `Score.db` (process suggestion DB, 1,923 rows), `UserGame.ini`, `GameState.ini`, `Pop.ini`,
  `Driver.ini`, `Process.ini`, `Logs\` (Boost.txt, Registry.log, JunkFile/Privacy logs).
- **Settings keys** (unit around 0x6cc000–0x6d2200): `CloseToTray, FRunWithWindows, AutoMinimize,
  FAutoGameMode, FBoostRestore, FRestoreProcesses, FRestoreServices, BoostHootKey1/2, RestoreHootKey1/2,
  ScreenHotkey1/2, RecordHotkey1/2, RecordFullScreen, VideoQuality, DisplayFPS1/2, FPScolor, FOverlayConer,
  FDisplayTempInGame, FWarnTempCPU/VGA/HD/Board, FIsCelsiusTemp, FnWarnProcessCPU/Memory, GameAutoscan,
  MonitorSteam, MonitorTune, OverClock, OCLevel, OC_GPU, OC_RAM, Language, FSkinName, Transparency,
  Autoupdate, update_method, SkipUAC, silentMode, FEnableProxy, FProxyServer, FProxyPort, FProxyUserName,
  FProxyPassword`. The proxy password is stored in the INI; no encryption call was seen near it.
- Command-line switches: `/boost /boostnow /Boost /Game -game -gamedesk /ScanGame /Settings /silent
  /skipuac /autorun /STARTUP /TurnOn /TurnOff /UpdateCYRIDB /MacthCYRI /AutoDefrag /AutoDriver
  /AutoGuard /bugreport /postnow /url /appgoto /Proxy /MiniBoostOver -Tray -Main …`.

## Every other feature in the class list

| Class(es) (VMT) | Feature |
|---|---|
| `TfrmSmartGameBoosterMain` | Main window, tray icon (`WMTrayNotify`), boost message handler (`WMBoost`). |
| `TfrmGameList`, `TframeOneGameShow`, `TfrmGameInfo`, `TfrmSelectGameVer`, `TfrmDetailGame`, `TShowAllGame` | Game library UI, launch, add local game, pick game version. |
| `TfrmBoostConfig`, `TfromBosstResult`, `TfrmBoostAntimator`, `TframeCircle1` | Boost configuration dialog, result screen, animation. |
| `TFrameBoosterProxy`, `TfrmServerArea`, `TServerListManager`, `TTopVPNConfig`, `TiTopSocks5Server`, `TSocksMonitorThread` | "Game accelerator" (online network booster, iTop VPN based): server list, region pick, local SOCKS5 server; engine in SgbAccelerate.dll. |
| `TCYRIFrame`, `TCYRIGame`, `TCYRIGetHotGame`, `THotGameFrame` | "Can You Run It" requirements check, hot games. |
| `TFrameMonitorDDC`, `TFormMonitor`, `TFormUserPresetMode` | "Monitor Tune": DDC/CI presets (A1). |
| `TFormHUDPanel`, `TFramePerForMance`, `TCurveFrame`, `TTemperatureForm`, `THookGameManger` | FPS/temperature overlay, performance curves, screenshot/video record (via fps.dll, Temperature.dll). |
| `TGameDefragFrame` (0x91292c), `DefragConfig.ini` | Defragment game folders. |
| `TTweaksFrame` (0x90e868), `TSysSpeedBusiness`, `TDeepOptimization`, `TDepthopt` | System Tweak page (`SysSpeed.ini`, `GB3.ini` from `SysSpeed.dbd`, see `../SgbData/dbd/`). |
| `TSSDDetection`, `TWMIDiskInfo`, `TUSBDevice` | Disk/SSD detection for tweaks. |
| `TInternetBooster`, `TInterSpeed`, `TIEBooster`, `TFirefoxBooster`, `TOperaBooster`, `TEdgeOptimize`, `TIBChromiumEdge`, `TSystemOptimize`, `TTCPState` | Internet booster: TCP/IP registry tweaks and browser preference edits (backups `ASCInternetBak.dat`, `ASCNETSHBak.dat`, `ASCFirefoxBak.dat`, `ASCEdgeBak.dat`). |
| `TJunkfilesScanner`, `TPrivacyScanner`, `TClearIEHistoryThread`, `TRecyclebinOpration`, `TAllBrowserCleaner` (+Chrome/FF/Opera/IE/Edge cleaners), `THmacHelper`/`TChromeHmacHelper` | Junk and privacy cleaner; browser plugin/extension manager. Templates in `PriTemp.dbd`. The Chrome HMAC helper recomputes Chrome's `Secure Preferences` MACs so extension/setting edits are accepted. |
| `TSecurityGuard` (0x81e…), `TFireWallMgr` | "Security" page: WMI `SecurityCenter2` AntiVirusProduct/FirewallProduct, "fix" by `PowerShell Set-MpPreference -DisableRealtimeMonitoring 0` (turns Defender real-time **on**) and enabling the firewall (`/enable /av /as /fw`). |
| `TMalwareScanner`, `TScanConfig` | Small malware scanner (signature DBs from `Def.dbd`/`.sgdt`, `MalwareIgnore.ini`, quarantine `\QuarantineZone\`). |
| `TAVUninstall`, `TAVProdItem` (0x8fb4bc) | Lists installed AV products from Uninstall keys (`DisplayName, Publisher, NotUnInst`) and can run their uninstallers (`msiexec`). |
| `TMainCtl`, `TStatupBase`, `TStartupPost`, `TGetDataFromServer` | Startup manager (above); posts startup items to the server for ratings. |
| `TFrameDriver`, `TDrvCfg`, `TMultiDownload`, `TInstallThread` | Driver updater (downloads from `f.pcgameboostdrivers.com`, `driverboosterscan.com`). |
| `TTaskSchedule`, `TCreateTaskThread`, `TTaskSchedulerInterfaces1/2` | Scheduled tasks, including **SkipUAC**: task "SmartGameBooster SkipUAC" runs SGB elevated without a UAC prompt (`schtasks /run /tn`). |
| `TfrmSettings`, `TFrameEdit1`, `TRdHotKey` | Settings and hotkeys. |
| `TFormSignInUp`, `TFormLoginInfo`, `TFormLoginSetting` | Account sign-in (Microsoft Graph `me/photos` for the avatar). |
| `TfrmLevel1Sale`, `TfrmProSales`, `TfrmGiftPromotion`, `TFormSVIPToExpPromote`, `TFormPromoteLeft`, `TWhatsNewForm`, `TfrmMainMsgBox` | Upsell, promotions, what's new. |
| `TFormCrack` (0x966470) | Licence/usage state (`pr.dat`, `FirstInstallDate`, `LastDay_DayActive_`), detects other IObit/iTop products in the registry, launches `StatUser.exe /u http://ascstats.pcgameboost.com/active.php` (daily/monthly active stats). |
| `TfrmAbout`, `TSkinManger`, `TLanguage` | About, skins, translations. |
| `TMetroInfo`, `TGetMetroThread` | Store/UWP app enumeration (`MetroAppCache.ini`). |
| `TSessionShareMemory`, `TMemUsage`, `TPipeThread` | Shared memory / pipes to SgbTray and `datastate.dll` (`\\.\pipe\datastate_`). |

## Network endpoints (privacy)

Telemetry/statistics: `stats.gameboosta.com` (`app_stats.php`, `iusage_v2.php`, **`smgb_hw.php`**
hardware upload, `gethot.php`), `ascstats.pcgameboost.com/active.php`, `ascstats.iobit.com`
(`usage.php`, `base-info.php`), `stats.reportcpanel.com`. Licence: `smgblicense.itopupdate.com/check.php`.
Geo-IP: `ip-api.com/json/`, `ipinfo.io`. Updates: `update.pcgameboost.com`, `update.superbgameboost.com`.
Download lists: `s3.amazonaws.com/myfilelist/list-*`, `storage.googleapis.com/myfilelist/list-*`
(lists for other iTop/IObit products: itop, ivc, pdf, dpm, ied, smgb).

## Cleaners, System Tweak scanner, and who calls TPriorityController (2026-11-05, REA/Ghidra + `dis.py`)

Plain-language version: `docs/sgb/cleaners.md`. Templates: `PriTemp.dbd` (password `asc4_2010`) decodes
with `../SgbData/dbd/dbd_decode.py` into `junkfile.db` (165 INI sections) and `privacy.db` (365); not
committed, regenerate. `Junk.dbd`, `RZJunk.dbd`, `SDJunk.dbd` are named in code (0x847470…) but not shipped.

**TPriorityController.Boost has no caller** (`0x7c8acc_TPriorityController_no_Boost_caller.asm`). The dword
0x7c4278 appears only in the VMTs of TPsPriorityBoost (0x7c33d4) and TPriorityController (0x7c445c),
which share every slot. The only instance is TBoost+0xa0 (created 0x7c676a). Every read of +0xa0 is the
destructor, the stop-flag setters (0x7c6dc9, 0x7c6e5d), the callback setter (0x7c88fe) and three
accessors used only by the Boost settings dialog (0x7cf4c8, 0x7cf51b, 0x7d26ff): `[Boost] Priority`
(on/off, default 1) and `[Boost] PriorityClass` (default 3). TBoost.Execute dispatches +0x84…+0x9c,
+0xa4, +0xa8 and skips +0xa0. So the "lower others to IDLE" code is dead and the game-exclusion
question is moot (the list builder 0x7c3a9c has no game exclusion anyway). SgbTray.exe imports
SetPriorityClass/GetPriorityClass and reads the same two keys (strings, `-db` copy): most likely where
the game's own class is set. Not read in code.

**Junk / privacy engine** (`TJunkfilesThread`, `TPrivacyThread`; same structure).

| Step | Code | What |
|---|---|---|
| Template | parser strings 0x859200… (junk), 0x885af4… (privacy) | Per section: `KindID` (tree group), `Recommended`, `CheckFile`/`CheckReg` (is the app present), `DelFile` (`dir|mask[|RECURSE]`), `DelReg` (key or value), `ExcludeFile`, `SpecialChecked`/`SpecialDelete` (built-in handlers such as `WIN_RECYCLEBIN`, `GC_SAVEPASSWORDS`), `DllName/DllScan/DllFix` (only `DriverClean.dll`), `WarnSec`. |
| Scan | `SelfScan` 0x84787c, `AnalyzeAndScan` ~0x849d00 | Enumerates the template paths (32-bit and, separately, x64 views), records files with sizes and existing keys/values. Browser history/cookies are cleaned with SQL (`delete from urls/visits/cookies/moz_places…`, 0x87ad14–0x88356c). No call to `Random` (0x40372c, 36 callers) anywhere in 0x841000–0x88a000; the `Sleep`s there are 10–100 ms waits in loops. |
| Fix | `SelfFix` 0x848244 → `FixItem` 0x84866c (`0x848244_…`, `0x84866c_…asm`) | Enables SeBackup/SeRestore and loads **other users' `ntuser.dat` hives** (0x82b97c, RegLoadKeyW) so their keys are cleaned too. Files: skipped if gone or excluded; if in use (0x8346e4, exclusive CreateFileW) → `MoveFileExW(DELAY_UNTIL_REBOOT)` (0x834270) and the "(Delete on Restart)" list; else delete (0x834da8, optional overwrite passes = "Secure Deletion"), failure → reboot list. Keys/values: if a backup stream is open, exported as `.reg` text first (0x8405a0, 0x840d18), then deleted; failures go to a fail list. |
| Report | 0x85da60 (`0x85da60_JunkResult_fixed_count.asm`) | "N items fixed" = number of ticked result rows (0x85db3d); size = sum of scan-time sizes of the ticked results (0x85db8f). Not reduced by failures or reboot-deferred files. |

Inflation, precisely: (1) the post-clean "fixed" count and size are the selection, not a measured
result; (2) files in use are counted as cleaned although they are only queued for the next reboot
(labelled "(Delete on Restart)" in the list); (3) registry `DelReg` items (MRU lists, UserAssist,
tray/menu caches) are counted as items alongside files. No evidence of invented items or of a scan that
does not look at the disk. Template defaults (`Recommended`): junk 140 of 164 = 1, including
`WindowsDownloadsFolder` (`WIN_Downloads`, scan 0x850cb8 not read in detail) and `WinInstaller`
("Windows Installer Unused Files"); privacy: every cookies, saved-passwords and saved-form-data item = 0
(except `Edge - Session` and `TD - Session` = 1). Whether `Recommended` sets the initial tick was not
traced in UI code.

**Registry cleaner**: none. No invalid-reference scanner strings (SharedDLLs, MUICache, ActiveX/COM,
"invalid entries"); the only registry deletions are the cleaner templates above.

**System Tweak scanner** (`TSysSpeedBusiness`, data `SysSpeed.ini`/`GB3.ini` from `SysSpeed.dbd`):

- Issue test 0x8a1bd0 (`0x8a1bd0_TSysSpeedBusiness_IsNotOptimized.asm`): open the key with
  `TRegistry.OpenKey(key, CanCreate=True)` (`mov cl,1` 0x8a1da5; 0x4abcb0 → RegCreateKeyExW), so **the scan
  creates missing keys**. Value missing → issue unless `OptVal=Delete`. `CompareStr`: issue if current ≠
  OptVal. `CompareMath`: issue if OptVal < current. So any value left at Windows' default (usually absent)
  counts as an issue.
- Backup 0x8a4d90 (`0x8a4d90_…asm`): before writing, the current value (or `NULL` if absent, 0x8a3420) is
  saved as `OptValBefore` in `Tweaks.ini`. Restore 0x8a477c writes it back via 0x8a4be4 → 0x8a65c4
  (`NULL`/`Delete` → DeleteValue). Internet Booster and Startup Manager keep their own records
  (`Internet Booster\Config.ini`, `Startup Manager\OptimizeRecord.ini`, `TTweaksFrame.Restore` 0x90fbbc).
- `Logs\Registry.log` (0x6cbcbc, fn ~0x6cbb10) is a log, not a backup.
