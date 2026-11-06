# GameBoosterCore.dll

VB.NET class library, ConfuserEx-protected (method bodies encrypted; see [../README.md](../README.md)).
Everything here is from namespace, type and member names plus the few plain strings.

| Namespace / type | What the names show |
|---|---|
| `GameMode.GameModeManager`, `GameModeReport` | Starts/stops Game Mode. Builds three lists (`GetGameModeServices`, `GetGameModeSchTasks`, `GetGameModeProcesses`) minus the exception files and vendor list (`GetGameModeExcluded…`). Keeps `_gameModeSvcsLst`, `_gameModeSchTasksLst`, `_gameModePrcsLst` so it can put them back; options `RestoreOptimizedServices`, `RestoreOptimizedProcesses`, `AutoExitGameMode`. A report window lists what was changed (`OptimizedServicesCount`, `OptimizedSchTasksCount`, `OptimizedProcessesCount`, `OptimizedVFXCount`). |
| `SharedPreferences.GameModePreferences`, `CorePreferences`, `MyGamesPreferences`, `PreferencesManager` | Settings: auto Game Mode on game start (`AutoGameMode`, `AutoGameModeExplorer`), Game Mode log/report, hotkeys (Game Mode, free memory, My Games), memory optimisation (`OptimizeMemory`, `MemoryOptimizeInterval`, `MemoryMaxUsage`, `ClearStandbyMemory`, `NotifyOfMemoryOptimization`), `OptimizeFileExplorer`, `OptimizeBackgroundProcesses`, `OptimizeForegroundApps`, `BoostOptionInShellMenu`, beta updates, promotions (`AppPromotionsEnabled`, `ThirdPartyPromotionsEnabled`), `AutoSendError`. |
| `MyGames.*` | Game library: `SteamProvider` (reads Steam `libraryfolders`/config VDF), `EpicGamesProvider`, `BattleNetProvider`, `MsStoreProvider`, Uplay, Windows Game Explorer, Program Files scan (`GameScanner`). Launch status per game. |
| `VDF.*` | Steam VDF parsing (accounts, library folders). |
| `Components.ComponentsManager`, `ComponentInfo`, `ComponentImpactInfo` | Startup programs / installed apps with publisher and "impact" (reads the Uninstall keys). |
| `LicensingHelper.LicenseManager` (+ `LicenseInfo`, `LicenseType`, `LicenseStatus`, `UpdateChannel`) | Free / Pro / Pro trial; name + email + key; online activate, revoke, renew, validate; a signed licence file. Two RSA public keys are embedded in plain text (used to verify licence and update data). Not studied further, by design. |
| `AppUpdates.AppUpdater`, `AppUpdateInfo`, `AppUpdateAction` | Update check data model (see [UpdateCheck.exe](../UpdateCheck.exe/notes.md)). |
| `UserGuide` | Plain URLs of the online manual (`/product-manuals/game-booster/…`: my-games, system-status, files-defragment, game-adviser, diagnostic-report, live-gaming-mode, settings-editor, tamper-protection, update-check). |
