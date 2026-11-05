# Data files

## `GameBooster-PowerPlan.pow` ("PC Game Booster Power Plan", GUID `2e7846dd-b84d-4e35-a2a0-99df8a4a76d5`)

A registry hive (decoded with `hivexregedit`). Setting GUIDs mapped to their Windows names; AC = plugged in,
DC = battery. Personality is High performance (`245d8541…` = 1).

| Setting | AC | DC | Windows default (High performance) |
|---|---|---|---|
| Minimum processor state | **100 %** | 75 % | 100 % / 5 % (Balanced: 5 %) |
| Maximum processor state | 100 % | 100 % | same |
| Core parking, minimum cores (`0cc5b647…`) | 100 % (parking off) | – | 100 % on High performance |
| System cooling policy | Active | Passive | same |
| PCI Express Link State Power Management | Off | Off | Off on High performance; Balanced: Moderate |
| Wireless adapter power saving | Maximum performance | Maximum performance | DC: Medium on Balanced |
| Turn off hard disk after | 3 h | 1 h | 20 min |
| Turn off display after | 3 h | 1 h | 10–15 min |
| Sleep after | 3 h | 30 min | 30 min / 15 min |
| Hibernate after | Never | – | 3 h |
| Dim display | 0 | 0 | |
| Display brightness | 100 % | 100 % | |
| Require password on wake | No | No | Yes |
| Lid / power / sleep button | Sleep / Shut down / Sleep | same | |

In short: a copy of Windows' High performance plan with longer screen/disk/sleep timers, no
password on wake, and 75 % minimum CPU on battery. Imported by the installer; Game Mode can switch
to it (`GetInstalledPowerPlans`, `PowerSetActiveScheme`; the Pro feature "Game Mode power plan").

## Exception lists (`%PROGRAMDATA%\Smart PC Utilities\PC Game Booster\`)

Plain text, one name per line ending in `;`. They list what Game Mode and Live Optimization must
**not** touch, which implies they touch everything else of the matching kind:

- `Process Exceptions` (~200): the booster itself, Explorer/shell hosts, `dwm`, `conhost`, antivirus
  processes (Avast, AVG, Avira, McAfee, Norton, ESET, Kaspersky, Bitdefender, Sophos, Malwarebytes …),
  Steam, Ventrilo, PunkBuster (`pnkbstra`), Logitech SetPoint, Realtek audio, NVIDIA helpers.
- `Live Exceptions` (~30): system processes never deprioritised by Live Optimization (`winlogon`,
  `csrss`, `dwm`, `RuntimeBroker`, Game Bar, `steamwebhelper`, WebView2, …).
- `Service Exceptions` (~55): antivirus services, DHCP, Windows Installer, Windows Search, Gaming
  Services, GameInput, NVIDIA display container.
- `Task Exceptions`: the booster's own tasks, Norton, "CreateExplorerShellUnelevatedTask".
- `Vendors` (~45): publishers whose processes/services are never touched (AV vendors, Valve, EA,
  Ubisoft, Epic, NVIDIA, the vendor itself).
- The list of names that **are** stopped is not shipped; it is built at run time (`GetGameModeServices`,
  `GetGameModeSchTasks`, `GetGameModeProcesses`) and the user edits it in the Game Mode editor.

## Others

- `Games.db`: SQLCipher-encrypted SQLite (no readable header). Probably the "Game Adviser" game
  database (genres, cover art); not opened.
- `Contig.exe` 1.70 (Sysinternals, Microsoft-signed): the "defragment game files" feature runs it
  on a game folder (`FileDefragEngine`, `BWDefrag`, `MnuItmDefragGame`).
- `Release Notes.url`: link to the release-notes page.
