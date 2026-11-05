# decomp/hone/

Static analysis of **Hone** (https://hone.gg, AuraSide Inc.), done for comparison with Smart Game
Booster, PGWARE GameBoost, Smart PC Utilities' PC Game Booster and Razer Cortex. Nothing was executed
(no Wine). Plain-language overview: [docs/hone/README.md](../../docs/hone/README.md).

## Policy

This folder holds **findings in prose only**. No decompiled code, excerpts, scripts, keys or Hone
assets are kept in this repository. Decompiled and extracted output lived only in the local work
directory `/home/zpt/re/hone/` and was deleted after these notes were written; only the two
downloaded installers remain there (`/home/zpt/re/hone/dl/`).

## Status

The "Download For Free" button on hone.gg points at Overwolf's download service. That returns a small
signed **Overwolf web installer**. Its NSIS script names the package URL it asks Overwolf for. We made
that one anonymous request (no login, no cookies), which redirected to the Hone **ow-electron** setup
on Overwolf's app CDN, and downloaded it. Nothing else was fetched from vendor servers. The only
other requests were reading the hone.gg home and download pages, to find the link.

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05. `https://hone.gg/app/gethone` links to `https://download.overwolf.com/install/Download?ExtensionId=mgkabooemhaamambocobpeoeelpadcjhjgbcfhlc`, which redirects to `download.overwolf.com/installer/prod/<hash>/Hone - Installer.exe` (served gzip-encoded) |
| Web installer | `Hone - Installer.exe`, 3,030,048 bytes, sha256 `11567aeeda1f95051a5bfa4624c802d324663b684affbe603c5e5509ad3f211c`. NSIS, version 2.316.0.1, Authenticode **Overwolf Ltd** (DigiCert G4 code signing, valid 2026-02-19 to 2029-05-10), verified by binary-refinery |
| App package | The installer's `--owelectronUrl` (`download.overwolf.com/setup/electron/<app id>`) redirects to `https://appsdl.overwolf.com/prod/apps/mgkabooemhaamambocobpeoeelpadcjhjgbcfhlc/2.1.0/setup.exe`. Saved as `Hone-2.1.0-setup.exe`: 138,194,232 bytes, sha256 `71809e68e75b272cbcd185c2f14b4937b95d2d6255f2950dc9b5859933dbabb9`. electron-builder NSIS, file version 2.1.0.0, Authenticode **AuraSide, Inc.** (DigiCert G4, valid 2025-01-28 to 2028-01-27) |
| App version | Hone **2.1.0** (Overwolf app id `mgkabooemhaamambocobpeoeelpadcjhjgbcfhlc`), ow-electron on Electron 44.9.0 / Chromium 152 |
| Tools | binary-refinery (`emit`, `pemeta`, `xtnsis`), 7-Zip (NSIS), a throwaway local asar reader (Python `-I`), ilspycmd 9.1 on the user-level .NET 8 SDK, `strings` |

Key component hashes (sha256) from the app package:

| File | sha256 |
|---|---|
| `Hone.exe` (ow-electron host) | `849c394f44dce7691569072f62f91a75d7030e9f475751cdc5209526ea108bd1` |
| `resources/app.asar` | `83e20583b7ef1ed3a9b0471248f2abedd96123d9170dfe197833888ae3037a8c` |
| `HoneUtility.exe` 1.1.70 | `7b4cc38ab88c895bc5af85458877277f33cb66f2e44a14ed80fffcde415dd7f8` |
| `HoneTune.exe` 1.0.3 | `076ccb1b73bc20f3c23cbc2384aea0f508f1200f89e652a01d998d615550a12b` |
| `HoneSvc.exe` 1.0.1 | `3c12e268e92c3ffbaeff3b3797097f1368d1cf2233d645aa38839a61efcbbb3a` |
| `hone-elevation-helper.exe` 1.1.3 | `fe0ed512437ccc8ac3ed663880f3296e9a5dcb2a218431b0a22d5b9edb74fdf4` |
| `SysInfo.exe` 2.0.5 | `5ac1a87955ab08f6b3d25090938f168b29453c4a288130cd714c52f695200363` |
| `GameManager.exe` 1.3.9 | `21a55be4ecf6fa3cffe063840c5d15ebb9de40d8220f796d1a1f356cf62703e9` |
| `hone-bios.exe` | `00951b898bd6f89201402ce36c44bb584a79f148db30e72fda01bc280e085a74` |
| `HWiNFO64.dll` 8.55 (REALiX, signed) | `48ec248a1827b7319450f6f20271bcff2888248fa951e1aa1ce1688e0f5282b9` |

All Hone helpers are signed by AuraSide, Inc.

## Component index

| Component | What | Status |
|---|---|---|
| [Installer](installer/notes.md) | Overwolf web installer, then the Hone electron-builder setup; what gets installed, consent and promo pages, install telemetry. | **Targeted**: NSIS script and strings read; the Overwolf .NET installer could not be decompiled (no managed metadata reachable by ilspycmd), strings only. |
| [App (Electron)](app/notes.md) | Main and renderer JavaScript: login, Free/Premium limits, server-driven optimization catalogue, helper orchestration, telemetry, updates. | **Targeted**: main bundle read for every helper call, endpoints, updater and telemetry; renderer read for the bundled optimization and BIOS catalogues. |
| [HoneUtility](honeutility/notes.md) | Applies and reverts the ~100 one-shot "optimizations" (registry, services, power, network, BCD, NVIDIA/AMD, device MSI/affinity). | **Strings + imports only**: protected with ConfuserEx 1.6 (encrypted constants, tampered metadata); not decompilable statically. What each item writes is inferred from its title, description, bundled data files and imports. |
| [HoneTune (Game Mode)](honetune/notes.md) | Temporary per-game boost: process priority/suspend/kill, service stop, power plan, timer resolution, power throttling, GPU priority, CPU affinity/parking, SnapTap. | **Fully analysed** (names obfuscated, strings and logic readable); decompile deleted. |
| [Other helpers](helpers/notes.md) | HoneSvc (SYSTEM service), elevation helper, hone-bios (AMI SCEWIN), SysInfo + HWiNFO driver, GameManager, HoneRedirect, small tools. | **Targeted** (.NET ones decompiled and deleted; native ones strings). |
