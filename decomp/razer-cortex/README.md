# decomp/razer-cortex/

Static analysis of **Razer Cortex** (https://www.razer.com/cortex), done for comparison with Smart Game
Booster and PGWARE GameBoost. Nothing was executed (no Wine). Plain-language overview:
[docs/razer-cortex/README.md](../../docs/razer-cortex/README.md).

This folder holds **findings in prose only**. No decompiled code, excerpts, scripts or Razer assets are
kept in this repository; those stay in the local work directory (`/home/zpt/re/razer-cortex/`).

## Status: installer done, Cortex itself not obtained

The official download is a small, signed **web bootstrapper** (the generic "Razer Installer", called LWI2
inside). It contains no Cortex code. It asks two Razer web APIs which product version to install and where
to download it from; only that API answer contains the Cortex package URL and checksum. Calling Razer's
APIs was outside this task's rules (only the installer and static CDN files may be fetched), so the Cortex
program, its Game Booster, System Booster, background service and overlay were **not analysed**. Every
Cortex feature row in the docs is marked *not verified* until a package is obtained.

Ways forward (a decision for the user): allow two anonymous GETs to `discovery3.razerapi.com` and
`manifest3.razerapi.com` (exactly what the installer does), or obtain the Cortex module installer from a
machine that already ran the Razer installer (the installer keeps downloaded module installers in `%windir%\Installer\Razer\Installer2\DownloadCache`).

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05, `https://rzr.to/cortex-download` (link on the product page) redirecting to `https://dl.razerzone.com/drivers/GameBooster/RazerCortexInstaller.exe` |
| File | `RazerCortexInstaller.exe`, 14,923,464 bytes, sha256 `e4eabf1a0cf030b773a1a14ed22f4cb337c5f85638863260215b6315d83eb570` |
| Version | Razer Installer **2.6.0.890**, linked 2026-06-02 (VS2022 17.12) |
| Signature | Authenticode, Razer USA Ltd. (DigiCert G4 code signing, valid 2025-01-14 to 2028-04-11); verified by binary-refinery |
| Tech stack | Native C++ x86 extractor + embedded DotNetZip self-extractor holding a .NET Framework 4.8 WPF app (Razer.Installer.*), AWS SDK (Kinesis), Serilog/NLog, native Razer device-detection and BLE DLLs |
| Tools | binary-refinery (`perc`, `pemeta`), Python `zipfile`, ilspycmd 8.2 on a user-level .NET 8 SDK |

Also fetched (static CDN file the installer reads): `https://manifest-assets.razersynapse.com/lwi/manifest.json`,
which only names the device-detection update `DetectManager-v1.1.0.76.zip` (not downloaded; not
booster-related).

## Component index

| Component | What | Status |
|---|---|---|
| [Installer (LWI2)](installer/notes.md) | Bootstrapper: picks the app from its own file name, fetches the product manifest, downloads and runs each module installer, cross-promotes other Razer apps, sends install-funnel telemetry. | **Complete** for behaviour, download/verify path, telemetry and promotion (all Razer .NET assemblies decompiled). Native device-detection/BLE DLLs: strings only. |
| Razer Cortex app (`gamebooster2`) | Game Booster, System Booster, FPS overlay, rewards, updater, background service. | **Not obtained** (see Status). |
