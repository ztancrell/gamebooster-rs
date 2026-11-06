# decomp/wagnardsoft/islc/

Static analysis of **Intelligent Standby List Cleaner (ISLC)** by Wagnardsoft
(https://www.wagnardsoft.com/), the author of DDU. Done for comparison with the other boosters
and with gamebooster-rs's RAM trim. Nothing was executed (no Wine). Plain-language overview:
[docs/islc/README.md](../../../docs/islc/README.md).

## Policy

This folder holds **findings in prose only**. No decompiled code, excerpts, scripts or ISLC assets
are kept in this repository. Decompiled and extracted output lived only in the local work directory
`/home/zpt/re/islc/` and was deleted after these notes were written. Only the two downloaded
packages remain there (`/home/zpt/re/islc/dl/`).

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05. The wagnardsoft.com home page links to the ISLC release page, which links to the forum thread (`/forums/viewtopic.php?t=1256`). The thread is behind a Cloudflare browser check, so its download links were read from the Internet Archive copy of 2026-10-05. Both files were then fetched from Wagnardsoft's own host, `download.wagnardsoft.com/ISLC/`. Their sha256 values match the ones published in the thread. |
| Portable | `ISLC v1.0.4.8.exe`, 706,056 bytes, sha256 `599315a1852b951e0adae2be46fce30bb32aab7f8a4c9112471a5a88320e3085`. A 7-Zip self-extractor (7-Zip SFX 26.02 stub). |
| Installer | `ISLC v1.0.4.8_setup.exe`, 1,221,976 bytes, sha256 `6292d1c55f671aef93c4e8e2df554753e7457b361771844fe92a812801956e3c`. NSIS 3 Unicode, file version 1.0.4.8. |
| Signing | Both packages and both assemblies have a valid Authenticode signature from **Wagnardsoft** (Quebec, CA), issued by Sectigo Public Code Signing CA R36, serial `042eff54cdead1ac3609e2b5c99afa3f`, valid 2026-05-15 to 2027-08-13. Checked with binary-refinery `pemeta`. |
| App version | ISLC **1.0.4.8** (release post dated 9 September; files dated 2026-09-09). .NET Framework 4.8 WPF app, x86, manifest `requireAdministrator`. |
| Tools | 7-Zip (SFX and NSIS extraction), binary-refinery `emit`/`pemeta`, ilspycmd (user-level .NET SDK), `strings`. |

Component hashes (sha256). The portable and installer packages contain identical binaries.

| File | sha256 |
|---|---|
| `Intelligent standby list cleaner ISLC.exe` 1.0.4.8 (1,189,136 bytes) | `11e4ed0b322adced3214c5a73ca1e807140341be6327b2a47a4c52434e73322f` |
| `ISLC.Core.dll` 1.0.0.0 (49,424 bytes) | `11dc5458940033634e40b855ec117a07edadf78d3c3fe2aca989098a46c590ac` |
| `Intelligent standby list cleaner ISLC.pdb` | `7b7ea246dc8a7a1cc5d9e26166012217f55e40b6ffc2d0bb0cc5404af04d1eba` |
| `ISLC.Core.pdb` | `908409cad227f1b5d2eb369feaf2b53b192b986d9bab468adb19fb3240e696e0` |

There are also 18 satellite resource assemblies (UI languages), `ReadMe.txt`, `LICENSE.txt`
(proprietary freeware) and `THIRD-PARTY-NOTICES.txt` (Newtonsoft.Json 13.0.3, merged into the exe).
Debug symbols are shipped, and the code is not obfuscated.

## Component index

| Component | What | Status |
|---|---|---|
| [Notes](notes.md) | Standby purge, trigger logic and defaults, timer resolution, Windows 11 global-timer switch, exclusion list, autostart task, hotkey, command line, update check, SYSTEM-token impersonation, settings file. | **Fully analysed.** Both assemblies were decompiled and read; the decompile was deleted. |
| Installer | NSIS installer that copies the same files. Uninstaller stub only. | Listed with 7-Zip, but the script was not recovered. No service, driver or extra payload is in the archive. |
