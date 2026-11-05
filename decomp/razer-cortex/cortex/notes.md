# Razer Cortex 11.12.0.14: notes

Findings about the Cortex app itself, from static reading of the package the Razer installer downloads.
Written in our own words; no vendor code, keys or assets are reproduced. Nothing was executed.

## Package and what gets installed

The installer's product list (fetched anonymously from `manifest3.razerapi.com`, release channel `prod`,
with OS=Windows 11 x64 and no PC model) lists three modules for Cortex (`GameBooster2`). All three are
installed silently as administrator; the user sees one "Razer Cortex" install.

| Module | Version | Size | What |
|---|---|---|---|
| Razer Cortex (primary) | 11.12.0.14 | 263 MB | Inno Setup package, installs to `C:\Program Files (x86)\Razer\Razer Cortex` |
| Razer Central ("Natasha") | 7.23.0.1220 | 122 MB | Razer account / SDK service (`RazerCentralService`), shared by Razer apps |
| Razer Game Manager 3 ("GMS3") | 3.15.0.1200 | 63 MB | Game detection service shared by Razer apps |

Razer Central and Game Manager were downloaded and their checksums checked, but they were **not
analysed**. Downloads are on `manifest-assets.razersynapse.com`; MD5s matched the manifest (sha256s in
[../README.md](../README.md)).

The Cortex setup (Inno Setup, `PrivilegesRequired=Admin`):

- Registers and starts **`CortexLauncherService`**, a native Windows service running as SYSTEM. It listens
  on a named pipe and starts processes in the user's session with a duplicated user or elevated token. In
  practice it lets Cortex start elevated without a UAC prompt.
- Cortex also creates **scheduled tasks**: `RazerCortexSkipUAC_…` ("Run RazerCortex without UAC
  notify", highest privileges) and `RazerCortexDelay3MinLaunch_…` (start Cortex 3 minutes after logon).
  The uninstaller runs Cortex with `-DeleteAllSch` to remove them.
- Autostart: an `HKLM\…\CurrentVersion\Run` value `RazerCortex` with `-autorun` (a Cortex setting can
  remove it).
- Registers the `razercortex://` URL protocol, desktop / Start-menu shortcuts and, if needed, installs the
  Microsoft WebView2 runtime.
- On uninstall it also runs `UninstallPowerPlans.exe` (removes the "Razer Game Booster Plan") and stops
  and removes the service. It cleans up older Razer Game Booster leftovers (`RzKLService`, old folders).
- **No kernel driver** in the Cortex package.
- The package ships a separate telemetry helper, `RazerCortex.InstallBigData.exe`, which reports the
  install.

## Architecture

- A .NET 10 back end (`RazerCortex.exe` / `RazerCortex.dll` plus one module assembly per feature) runs
  elevated and serves a token-protected local SignalR/HTTP API on `127.0.0.1`. CORS allows only Razer's CDN
  origin.
- **The user interface is a web app loaded at runtime from Razer's CDN**
  (`deals-assets-cdn.razerzone.com/cortex10/v{version}/…`; also an overlay page and a "Big Picture"
  page) and shown in WebView2. So the screens and what they ask the elevated back end to do are delivered
  online by Razer and can change without a Cortex update.
- The actual boost work is in native C++ DLLs: `Boost.dll` (Game Booster), `Tweaks.dll` (System
  Booster), `FileCleaner.dll`, `Defrag.dll`, `FPSManager.dll` / `Razer.PresentMonImpl.dll` (FPS counter),
  `K_FPS.dll` (overlay window). Note: the Game Booster's code paths still carry the old Razer Game Booster
  structure, and the System Booster database has the same layout and largely the same entries as the IObit
  database used by Smart Game Booster (see [../../SgbData/README.md](../../SgbData/README.md)).

## Game Booster (`Boost.dll`, settings in `BoostSettings\`)

**Trigger.** "Auto boost" is **on by default**. Boost starts when a game is started from Cortex's library,
or when the overlay / game monitor detects a known game starting. Restore runs when the last running
known game exits, or on manual Restore, hotkey or Cortex exit. Five specific games (by Razer game id) get a
"safe boost" variant. There is also a manual Boost button and hotkeys.

**Default items** (`boost.ini`, value 1 = on). Each has a matching Restore.

| Item | Default | What it does | Restore |
|---|---|---|---|
| Power | on | Creates "Razer Game Booster Plan" (a copy of High performance) and activates it. Min and max processor state 100 %, cooling policy active, PCIe link-state power management off, Wi-Fi adapter max performance, never turn off the hard disk / display / dim, no sleep on battery and no hibernate on AC, no wake password, multimedia "prevent idling" when sharing and "optimize video quality". Lid, power and sleep button actions and display brightness are copied from the current plan. | Reactivates the previous plan; the plan is deleted on uninstall |
| CPU unpark | on | Edits the registry metadata of the "core parking min cores" power setting (unhides it, sets its allowed maximum to 100 %). It does **not** change the min-cores value, so by itself nothing gets unparked. | Puts the attribute back |
| RAM | on | Trims the working set of other processes (EmptyWorkingSet), with an exclusion list | Nothing to restore (pages fault back in) |
| Clipboard | on | **Empties the clipboard** | Not restorable (contents lost) |
| Background apps | on | Sets `GlobalUserDisabled` under `…\BackgroundAccessApplications` (HKCU) | Reverts the value |
| Disable telemetry | on | Writes the `AllowTelemetry` policy under `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection` | Reverts |
| Disable report problems | on | Turns Windows Error Reporting off under `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | Reverts |
| Disable Cortana | on | Sets the `AllowCortana` policy and terminates `Cortana.exe` | Reverts and relaunches Cortana |
| Sticky Keys | on | Turns off the Sticky Keys shortcut (SystemParametersInfo) | Reverts |
| CPU affinity ("CPU core") | off | Pins non-game processes of SYSTEM / LocalService / NetworkService / the user onto a few cores chosen by core count (e.g. 2 of 8). Whitelist includes Razer, Steam, Search, Easy Anti-Cheat and Vanguard processes, but **not EA Javelin**. | Restores the saved masks |
| Win key | off | Disables the Windows key while boosted | Reverts |
| Game Desktop | off | Kills `explorer.exe` and shows a Cortex game desktop | Restarts Explorer |
| Services | user list | Stops ticked Windows services. Defaults come from a built-in list with about 60 ticked for Windows 10/11 (e.g. Print Spooler, Windows Time, Windows Search, SysMain, WER, Remote Desktop, Font Cache, BitLocker, Netlogon, Credential Manager, Smart Card policy, Offline Files, telemetry `DiagTrack`). Many are not running anyway. | Restarts what it stopped |
| Processes | user list | **Terminates** ticked user processes. The suggestion list includes VLC, iTunes, uTorrent, TeamViewer, CCleaner, KeePass, Evernote, Task Manager and others. A whitelist protects system, Razer and launcher processes. | Relaunches them from their saved path and arguments (their unsaved state is lost) |

A raised-priority list (`PPConfigList.ini`) names Steam, its web helper and overlay as "upper" and
`conhost` / `SearchUI` as "lower".

## System Booster (`Tweaks.dll`, encrypted database `CortexCleanDB.db`)

- **Database:** an SQLCipher database with a fixed key inside the DLL, which we read locally. It holds 76
  tweak categories with about 120 registry writes, almost all **selected by default**. Each stores the
  original value, the recommended value and the Windows default.
- **Restore:** two buttons, "restore to before" (from a backup XML) and "restore Windows defaults". So the
  System Booster is reversible as long as the backup survives.
- **Contents:** the same family as SGB's optimizer list:
  - Shutdown and hang timeouts lowered, `AutoEndTasks`.
  - `DisablePagingExecutive`, `IoPageLockLimit`.
  - `Win32PrioritySeparation` = 0x26.
  - Menu, hover and animation delays.
  - NTFS last-access and 8.3 names off.
  - AutoRun off, `AutoChkTimeout` 5 s.
  - Legacy TCP/IP parameters (TTL, PMTU, RFC 1323, SACK, `GlobalMaxTcpWindowSize`).
  - Internet Explorer connection counts.
  - DNS negative-cache times 0.
  - A dozen rarely-used services set to Disabled (Peer Distribution, iSCSI, SNMP trap, NAP, Certificate Propagation, ActiveX installer, Application Management, Smart Card policy, WebClient, WinRM, Offline Files, WER, PNRP/P2P, RPC Locator).
  - Windows "suggestions" / ContentDeliveryManager off, advertising id off, Bing search off, consumer features off.
  - MMCSS Games profile: Priority 6, Scheduling High, `SystemResponsiveness` 0, `NetworkThrottlingIndex` off.
  - `SvcHostSplitThresholdInKB` raised.
  - A `remoteregaccess` value under `SecurePipeServers\winreg`.

## Other modules (high level)

- **FPS counter and overlay:**
  - The FPS counter uses an ETW / PresentMon session (no hooking of the game).
  - The overlay is a separate top-most window that shows a web page from Razer's CDN.
  - No DLL injection into games was found. The game-start event records each game's anti-cheat type.
- **Game clips / recording** (`Razer.VideoRecorder.exe`):
  - It captures with Windows Graphics Capture; MinHook is used only inside the recorder's own process.
  - Auto-clip plug-ins for CS2, Dota 2 and LoL read the games' official event APIs.
  - Clip plug-ins and an OpenVINO model can be updated from the CDN.
- **Boost Prime / game settings optimizer:** downloads per-game .NET plug-ins from the CDN and loads them
  into the elevated back end to rewrite a game's graphics config. There is **no signature or hash check**
  on these plug-ins, only HTTPS.
- **System Clean, Defrag:** junk cleaner and a defragmenter. A scheduled clean task exists.
- **AppGet:** installs and updates third-party apps through `winget`, from a Razer-curated "popular apps"
  list. If winget reports a hash mismatch, Cortex **turns on winget's `InstallerHashOverride` admin setting
  and retries with the hash check skipped**; the setting stays on afterwards.
- **Neuron (remote play / streaming):**
  - It downloads its server component.
  - The installer must pass a Windows signature check (any valid signature, not specifically Razer's).
  - Its local HTTP client accepts **any TLS certificate**.
- **Deals / rewards:**
  - A deals store and a Razer Gold games list.
  - "Play to Paid" sends a heartbeat every 5 minutes while a promoted game runs, to earn rewards.
  - Rewards / Razer Silver pages come from `cortex.razer.com` and `deals.razer.com`.
  - Notifications are refreshed every 30 minutes.
  - A Razer ID login (OAuth via `oauth2.razer.com`) is needed for rewards and sync, not for boosting.
- **Telemetry:**
  - A `DataAcquisition` module sends usage events (game names and platforms, boost start/restore and how it was triggered, clean results, notification clicks, campaign ids).
  - It also sends a hardware inventory: CPU, GPU, RAM, mice, keyboards, cameras and audio devices.
  - Endpoints: Razer Analytics (`maxineapi.razersynapse.com`) and a WebSocket at `cortex-wss.razer.com`.
  - Data collection is switched on in `appsettings.json` (`CollectData:IsEnable = true`); no in-app switch was found.
  - A `Razer.MachineID` helper derives an id from motherboard and disk serial numbers.
  - A remote "BigReportConfig" on the CDN tunes reporting.
- **Updates:** Cortex updates come through the same Razer installer and manifest (HTTPS plus MD5 from the manifest, no signature check; see [../installer/notes.md](../installer/notes.md)). The web UI updates itself from the CDN.

## Anti-cheat (BF6 / EA Javelin)

- No injection into game processes, no kernel driver, and FPS comes from ETW.
- The overlay is an external window, which Javelin tolerates for other overlays. A conflict cannot be ruled out without testing, but nothing here behaves like a cheat.
- The risky parts are the boost items that touch other processes:
  - **Process killing:** low risk as long as the Javelin processes are not ticked; they are not in the suggestion list.
  - **CPU affinity** (off by default): would also try to restrict EA's anti-cheat service, which is not on its whitelist. A protected process would normally refuse, so the likely effect is nothing. Keep it off.
