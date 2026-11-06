# Installer notes (smart_game_booster_enseo_hp_setup.exe, Inno Setup 6.4.2)

Static only. Source: `emit setup | xtinno meta/setup.json` (sections) and the IFPS disassembly in
`/home/zpt/re/sgb-extract-db/embedded/script.ps`.

- Declarative entries: 3 tasks (desktop icon, pin to taskbar, Quick Launch), 5 shortcuts, 1 registry
  key (`HKLM\SOFTWARE\PCGameBoost\Smart Game Booster`), 1 run entry (`SgbMain.exe /needskipuac`,
  condition `CheckAutoRun`). No `[Ini]`, no services, no Run key, no browser entries.
- Script work: geo check `GET http://ip-api.com/json/` via WinHttp; if `"countryCode":"CN"` it shows
  `RegionRestrictions` and stops. Silently uninstalls older Game Booster 3 / SuperbGameBoost / SGB
  Alpha / SGB 5 (`/sp- /verysilent /suppressmsgboxes`). Pins the taskbar icon (`PinTools\ICONPIN*.exe`).
  Installs drivers with `DrvInstall\DpInstX32/X64.exe`. Runs SgbInit (see `../SgbInit.exe/notes.md`)
  which creates the scheduled tasks and drivers, and `UninstallPromote.exe /install smgb6` (install
  beacon); on uninstall `/uninstall smgb6`, `taskkill`, and cleanup of pending renames.
- Also installs DirectX runtime DLLs to System32 (see `../README.md`).
- **No bundled third-party offers, no browser/homepage changes** were found in the script strings.
