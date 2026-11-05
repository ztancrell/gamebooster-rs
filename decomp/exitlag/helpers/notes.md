# Helper programs

All signed by Skowsand Tecnologia Ltda unless noted.

| File | sha256 | What it is |
|---|---|---|
| `ExitLagPmService.exe` 2.5.1.0, 3,595,968 B | `df83f708cfc7549208c7c0d543994f09ff72bc9566b1aa27bf0696e1c1d0b984` | Intel's open-source PresentMon service ("Service for Intel PresentMon API clients"), re-signed by ExitLag. Installed as an **auto-start service** by setup; collects frame-time data (ETW) for the overlay through `PresentMonAPI2.dll` (`e6b9761b…16ec88`) |
| `handler.exe`, 603,328 B | `e61ae982fd3654f9d0ca230cb1fdd4aa44c36a98f5a44056bc05e4fe3390bf71` | Chromium **Crashpad handler** (no version info): writes minidumps of the client and uploads them over WinHTTP with annotations and attachments; the upload URL is passed at run time, not stored in the file |
| `exitlag_autostart.exe`, 114,360 B | `64b7f5a0bd5f26f721fd713123ffbdb1ebbb4ac095009358c9e605add91dc909` | Target of the logon scheduled task the client creates; starts `ExitLag.exe` |
| `exitlag_reboot_required.exe`, 113,848 B | `9c254ee6beb0d2cd7fcf4a7867b61885d2ee83e8351066c9ca9bfcf96af5c722` | Holds the named object `Global\ExitLag_IsRebootRequired` after the NDIS driver install |
| `DriverCacheCleaner.exe`, 257,728 B (setup temp) | `5d0b21386ad0290bb8835509210836845d70c47a7bc003def9e99e712dd57dd1` | Removes old `ndextlag`/`nfextlag` packages from the driver store |
| `snetcfg.exe` 5.2.3790.0, 15,360 B | `516c5643cf378bdbc28191db75f85aed6988f21fe176c6d198ec21e76540c944` | Microsoft's DDK "network config sample", **unsigned**; installs/removes the NDIS filter binding |
| `nfregdrv.exe` (setup temp) | n/a | NetFilter SDK driver registration tool |
