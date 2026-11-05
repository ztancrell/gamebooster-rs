# Installer, service, licence, updates — notes

Process Lasso 19.0.0.94 installer `processlassosetup64.exe` (sha256 `92cc0aaa…2804b`, see
[../README.md](../README.md)). Static only: the NSIS script was extracted with binary-refinery
`xtnsis` and read for its actions. The helpers were read through strings and imports. All output was
deleted.

## What the installer does

- Standard NSIS installer: EULA page, language choice, install folder, and options for a Start-menu
  and desktop shortcut and two optional "default/advanced config" sections. Writes
  `HKLM\Software\ProcessLasso` (install dir, language) and an Uninstall entry (publisher "Bitsum").
- After copying files it runs `InstallHelper.exe` steps in order: stop running copies, clear old
  logs, first-install or migrate config, `/powerinstall` (creates the BHP power plan), `/install`
  (registers the core engine), `/env_path_install` (adds the folder to PATH, for `pl.cmd`),
  `/enable_update_check`, `/initconfig` (writes the default `prolasso.ini`), and
  `/startgovernorservice`. Finally it starts the GUI with `/install`.
- The 32-bit stub on 64-bit Windows checks the architecture. Only on mismatch does it open the
  32-bit download URL in the browser; nothing else is downloaded during install.
- No bundled third-party offers were seen in the script.

## How the core engine runs

- **Service mode** (the default, `/governor_start_type service`): a Windows service named
  `ProcessGovernor`, display name "Process Lasso Core (Process Governor)". `srvstub.exe` is the
  service host and launches `ProcessGovernor.exe` with an exit-event name. As a service it runs as
  **SYSTEM**, so it can manage every user's processes and services (subject to protected-process
  rules).
- **Per-user mode** (`current`): started at logon through a Task Scheduler task "with elevated
  rights" (InstallHelper's errors: "Can not set %s to start at login with elevated rights"), or the
  `Run` key.
- `start-governor.bat` tries `net start ProcessGovernor`, else starts the exe with `/manual`.
- `bitsumsessionagent.exe` ("Process Lasso Session Agent") runs in user sessions so that the
  SYSTEM-side engine can launch the GUI on the user's desktop (`CreateProcessAsUserW`,
  `WTSQuerySessionInformationW` in the governor).
- **No kernel driver** in the package. Privileges it enables: `SeDebugPrivilege` (open other
  processes), `SeIncreaseQuotaPrivilege` (working-set trim), `SeProfileSingleProcessPrivilege`
  (standby purge).
- **IRQControl** (separate tool, signed, 1.0.0.26) and InstallHelper write device
  `Interrupt Management\Affinity Policy` and `MessageSignaledInterruptProperties` registry values.
  They back up the originals under `HKCU\Software\ProcessLasso\InterruptAffinityBackup` and offer
  "Restore Original". This changes hardware interrupt routing; the GUI warns it can cause stutter or
  a device that fails to start.

## Licence (high level; no bypass looked for or described)

- EULA: Process Lasso is free indefinitely, but "some advanced features are disabled after 14 days".
  Commercial use requires purchase after 30 days. The Server Edition is a 30-day trial.
- The governor has log actions `PL_ACTION_TOOK_PRO_FEATURES_DISABLED` and `…LICENSE_EXPIRED`. The
  GUI has a nag dialog (`IDD_NAG`, `NagDismissedCount`) shown at startup until activation, with
  "Activate my license", "Buy Now" and an in-product coupon link.
- Activation: `plActivate.exe` against `https://activate.bitsum.com/` (activate, check, deactivate
  actions; looks like a standard e-commerce licence API). The key is stored as `prolasso.key` /
  config. Beta builds "unlock all Pro features".
- ProBalance itself is not a Pro-only feature (Bitsum's docs: on by default for everyone).

## Updates and network

- Version check: `https://update.bitsum.com/userservices/versioninfo.php` (HTTPS). The installer
  turns it on (`/enable_update_check`); the INI key is `Advanced/AutomaticUpdate`.
- `QuickUpgrade.exe` downloads the new installer (`pl4sfx.exe`) over HTTPS (WinINet) and imports
  `WinVerifyTrust`, so the download's Authenticode signature is checked before it runs. If that
  fails, it sends the user to the download page with an error code.
- Text the program fetches from the server: `https://lic.bitsum.com/versioninfo/processlasso/`
  `…/welcome.txt`, `rules.txt`, `pro.txt`, `commercial.txt` and `attribs-ex.txt` (server-driven
  messages and app attributes).
- Telemetry: Bitsum's FAQ says "Process Lasso doesn't send any user or process data to Bitsum" and
  connects only for updates and licences. Nothing in the strings contradicts that: no analytics SDK
  or crash-upload endpoint was found. `bitsum.com/minidumps` is a help link for sending dumps by hand.
  Static reading cannot prove what the update request contains (version and probably licence state).
