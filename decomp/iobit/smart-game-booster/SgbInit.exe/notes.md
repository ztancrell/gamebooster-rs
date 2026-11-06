# SgbInit.exe notes (first-run init, tasks, anti-tamper)

Static only. Analysed copy: `sgb-extract/carved/pe_00000000_3195448.bin` (3,195,448 bytes; sha256
prefix `07d409752712d8d1`). Method: `strings`. The list-file copy step is in `../SgbData/README.md`.

Run by the installer as `SgbInit.exe /init /ver 6.0.1.1231 /force /f /inspkg "…" /insur "…"
[/upgrade /prever … | /bigupgrade] [/insincn] [/insthandle …]`, `/InitAfterInstall`, `/isboost`,
`/drvrestore`, `/unsopt`, `/UnInstallDelFile`, `/clearpendingfilerename`.

- **Scheduled tasks** (Task Scheduler COM; the register code references the `SYSTEM` account, run level not traced): `SmartGameBooster SkipUAC`
  (starts SGB elevated without a UAC prompt; installer launches `SgbMain.exe /needskipuac`) and
  `SmartGameBooster Startup` (`SgbTray.exe" /auto`, the "run with Windows" option `FRunWithWindows`).
  It also lists all tasks (`schtasks /query /fo LIST`) for the startup manager.
- **Default settings** written to `%APPDATA%\PCGameBoost\Smart Game Booster\SmartGameBooster.ini`
  (`Autoupdate`, `update_method`, `FRunWithWindows`, `FShowNewsIcon`, `FbShowRecommad`, …).
- **Network driver**: unpacks `itvwd32.sys`/`itvwd64.sys` (WinDivert) from `Update\*.dat`.
- **Install stats**: `http://ascstats.pcgameboost.com/kwstats.php` with
  `k=%S1&s=%S2&c=%S3&p=%S4&type=newinstall` (plain HTTP; search-keyword/source attribution), and
  ProductStat3 posts.
- **Reads Explorer UserAssist** (`…\Explorer\UserAssist\{F4E57C4B…}\Count`, `{75048700…}\Count`),
  Windows' record of which programs the user ran. Purpose not traced (likely "was SGB launched").
- **Anti-crack**: looks for proxy DLLs next to the exe (`PYG.dll`, `version.dll`, `winmm.dll`,
  `msimg32.dll`), checks file signatures (signer allow-list incl. Chengdu Zhagu / ORANGE VIEW) and
  the key; on failure shows "You are using an unofficial version of Smart Game Booster. Your system
  may be vulnerable to malicious attacks." and links the official download.
- Checks for 360 Total Security / MPC AdCleaner / iSafe (likely to adapt browser launching).
