# HoneUtility.exe 1.1.70: notes

The engine behind Hone's one-shot "optimizations". It is a .NET Framework 4 console program, signed by
AuraSide, Inc., and protected with **ConfuserEx 1.6**: renamed symbols, encrypted constants, and
metadata tampered so that ilspycmd refuses it ("illegal tables"). Static decryption was out of
scope, so **the exact registry values each item writes could not be read**. This page lists what can
be shown from its imports, plain strings, bundled data files and the app's catalogue text.

## Interface

- The app runs it elevated with an optimization id and a mode (apply or "revert"). It also takes
  `--check` (hardware flags) and a Boost-Up size estimate. HoneSvc runs it with **AES-encrypted
  arguments** (a key compiled into both programs) for two ids: `RqbxUufM` "Optimize device affinity"
  and `oPu4x2Ef` "Optimize Message Signaled Interrupts".
- Plain strings: `SOFTWARE\Hone` and its `TweaksSvc` value (bit flags telling HoneSvc which device
  tweaks to re-apply), `%NvDriverStore%`, `%Steampathfolder%`, the game-detection file, an AMD
  download URL, and `SeShutdownPrivilege`.

## Capabilities (from imports)

- Registry read and write, token duplication and `CreateProcessWithTokenW`. That is the usual way to
  run as SYSTEM or TrustedInstaller and write keys that admins normally cannot (Defender, protected
  service keys). Privilege names such as `SeRestorePrivilege` appear in its strings, but some may
  come from the bundled TaskScheduler library, so this is inferred, not verified.
- **Services:** create, delete, change start type (`ChangeServiceConfig`/`ChangeServiceConfig2`),
  stop, and enumerate dependants. So "Disable Windows Services" changes **start types**, which is
  persistent, unlike Game Mode's temporary stop.
- **Scheduled tasks:** TaskScheduler COM (register, disable).
- **Power:** imports a power scheme from a file, sets the active scheme, writes AC/DC values, unhides
  settings, deletes schemes. It uses `HoneV5.pow` ("Hone Ultimate Power Plan V4", built on Ultimate
  Performance). The bundled Balanced/High performance `.pow` files are probably used to put stock
  plans back.
- **Hosted PowerShell** (out-of-process runspace). The netsh, TCP, congestion-provider, DSCP (QoS
  policy) and NIC advanced-property items are most likely PowerShell/netsh based.
- **Devices:** SetupAPI/cfgmgr32 and `DeviceIoControl`, for MSI mode and interrupt affinity policy
  under each device's `Interrupt Management` registry key, and for NIC settings.
- `NetUserModalsGet/Set` and `NetUserDel` (account policy and user removal; which item uses them is
  unknown), `ExitWindowsEx` (reboot prompt), file hashing and AES (for its own argument and file
  handling).
- **NVIDIA:** unzips and runs NVIDIA Profile Inspector to import `HoneProfile.nip` into the driver's
  global **Base Profile** (about 80 settings). Revert imports `Base_Profile.nip`, which is an **empty**
  Base Profile. That resets the global profile to driver defaults instead of restoring what the user
  had. Notable values in Hone's profile:
  - max pre-rendered frames 1, power management "prefer maximum performance", shader cache 10 GB;
  - **threaded optimization off**, V-Sync forced off, **G-SYNC disabled globally**;
  - FXAA/Ansel/MFAA off, DLSS preset overrides on.
- **Others shipped with it:** `TotalReg.exe` (third-party registry editor), offered as "Install
  enhanced registry editor"; installers for Process Explorer and the Samsung NVMe driver (catalogue
  items).

## Reversibility (what can be said)

- A "revert" mode exists and the app syncs per-item applied state. Hone stores some backups under
  `HKLM\SOFTWARE\Hone`: `PowerPlanBackup` is confirmed for HoneTune, and the website claims originals
  are saved. System Restore points are offered as a safety net.
- Not verifiable per item. Items that clearly cannot be undone exactly:
  - the NVIDIA revert (resets to defaults);
  - service start-type changes, if no backup is kept;
  - "Install …" items;
  - BCD edits (revertible only if the old values were recorded).

## Status

Strings and imports only; nothing decompiled is kept.
