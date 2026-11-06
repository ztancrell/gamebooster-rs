# decomp/razer/cortex/

Static analysis of **Razer Cortex** (https://www.razer.com/cortex), done for comparison with Smart Game
Booster and PGWARE GameBoost. Nothing was executed (no Wine). Plain-language overview:
[docs/razer-cortex/README.md](../../../docs/razer-cortex/README.md).

This folder holds **findings in prose only**. No decompiled code, excerpts, scripts or Razer assets are
kept in this repository; those stay in the local work directory (`/home/zpt/re/razer-cortex/`).

## Status

The official download is a small signed **web bootstrapper** (the generic "Razer Installer", LWI2) with no
Cortex code. With the user's approval we then made the same two anonymous requests the installer makes
(`discovery3.razerapi.com` for the release-channel hash, `manifest3.razerapi.com` for the product list;
no login, no PC model sent) and downloaded the three Cortex modules it lists from Razer's CDN. MD5s matched
the manifest. Decompiled output was used locally for research and then deleted; nothing derived from it
except these notes is kept.

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05, `https://rzr.to/cortex-download` (link on the product page) redirecting to `https://dl.razerzone.com/drivers/GameBooster/RazerCortexInstaller.exe` |
| File | `RazerCortexInstaller.exe`, 14,923,464 bytes, sha256 `e4eabf1a0cf030b773a1a14ed22f4cb337c5f85638863260215b6315d83eb570` |
| Version | Razer Installer **2.6.0.890**, linked 2026-06-02 (VS2022 17.12) |
| Signature | Authenticode, Razer USA Ltd. (DigiCert G4 code signing, valid 2025-01-14 to 2028-04-11); verified by binary-refinery |
| Tech stack | Native C++ x86 extractor + embedded DotNetZip self-extractor holding a .NET Framework 4.8 WPF app (Razer.Installer.*), AWS SDK (Kinesis), Serilog/NLog, native Razer device-detection and BLE DLLs |
| Tools | binary-refinery (`perc`, `pemeta`), Python `zipfile`, ilspycmd 8.2 on a user-level .NET 8 SDK |

Cortex modules (from the manifest, downloaded 2026-11-05 from `manifest-assets.razersynapse.com`):

| Module | File | sha256 |
|---|---|---|
| Razer Cortex 11.12.0.14 | `RazerCortexSetup_11.12.0.14.exe`, 262,554,168 bytes | `2a9609385749b82a5f04841aa37abaf843c8e52eb09736a83611470968bb24c2` |
| Razer Game Manager 3.15.0.1200 | `RazerGameManager_3.15.0.1200.exe`, 62,649,280 bytes | `352655029a1f3d9ba2425a2818475d2b82e29d16f874a852b426ac49edff3171` |
| Razer Central 7.23.0.1220 | `RazerCentral_v7.23.0.1220.exe`, 121,821,664 bytes | `aace29226ac0cdf023dc06ebf4cfaac54597138d42810da0cbe99b8a90dc3fd8` |

Tools for the app: binary-refinery `xtinno`, ilspycmd 9.1, Ghidra headless (native boost DLLs), a local
SQLCipher binding to read the System Booster database.

Also fetched (static CDN file the installer reads): `https://manifest-assets.razersynapse.com/lwi/manifest.json`,
which only names the device-detection update `DetectManager-v1.1.0.76.zip` (not downloaded; not
booster-related).

## Component index

| Component | What | Status |
|---|---|---|
| [Installer (LWI2)](installer/notes.md) | Bootstrapper: picks the app from its own file name, fetches the product manifest, downloads and runs each module installer, cross-promotes other Razer apps, sends install-funnel telemetry. | **Complete** for behaviour, download/verify path, telemetry and promotion (all Razer .NET assemblies decompiled). Native device-detection/BLE DLLs: strings only. |
| [Razer Cortex app](cortex/notes.md) 11.12.0.14 | Game Booster, System Booster, overlay/FPS, clips, Boost Prime, AppGet, rewards, telemetry, launcher service. | **Targeted**: every Game Booster item, the full System Booster database, setup script, autostart/service/tasks, endpoints and integrity checks read in code. Razer Central and Game Manager modules: downloaded, not analysed. |
