# decomp/exitlag/

Static analysis of **ExitLag** (https://www.exitlag.com/, published by Skowsand Tecnologia Ltda,
Lauro de Freitas, Brazil), the popular paid "ping reducer". Done for comparison with Smart Game
Booster's network accelerator (`../SgbAccelerate.dll/notes.md`) and gamebooster's own `ping`
command. Nothing was executed (no Wine), no account was made, and no ExitLag web service or relay
node was contacted. Plain-language overview: [docs/exitlag/README.md](../../docs/exitlag/README.md).

## Policy

This folder holds **findings in prose only**. No decompiled code, excerpts, scripts, keys or ExitLag
assets are kept in this repository. Extracted and analysed output lived only in the local work
directory `/home/zpt/re/exitlag/` and was deleted after these notes were written; only the
downloaded installer remains there (`/home/zpt/re/exitlag/dl/`).

## How the installer was obtained

`www.exitlag.com` answers every non-browser request (curl, WebFetch) with HTTP 403 (bot
protection), and opening it in a browser was refused by the permission system. The download page as
archived by the Wayback Machine (2025-09-11) links the installer on ExitLag's own CDN as
`https://cdn.exitlag.com/SetupExitLag-<version>-x64.exe`. HEAD requests to that CDN found 5.24.0
as the newest existing version (5.24.1–5.24.4, 5.25.0, 5.26.0 and 6.0.0 return 404), and that file
was downloaded anonymously. No other request went to ExitLag servers.

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05, `https://cdn.exitlag.com/SetupExitLag-5.24.0-x64.exe` (CDN `last-modified` 2026-10-30) |
| Installer | 96,336,080 bytes, sha256 `8e1482aa23169ee442e368fad1ad0c5767d20ce596ab8129b7e4cce24d18dc86`. Inno Setup (setup data 6.7.0), AppVersion **5.24.0**, admin install to `C:\Program Files\ExitLag` |
| Signing | Authenticode **Skowsand Tecnologia Ltda** (DigiCert Trusted G4 Code Signing RSA4096 SHA384 2021 CA1, valid 2024-11-26 to 2027-11-28), verified by binary-refinery. Every ExitLag binary carries the same certificate |
| Tools | binary-refinery (`emit`, `xtinno`, `pemeta`, `perc`), `strings`, `objdump`, `decomp/gdis.py`, LIEF (from the refinery environment) for import/string cross-references. Ghidra was not needed |

## Component index

| Component | sha256 | Notes |
|---|---|---|
| `ExitLag.exe` (Qt 6 / QML client, 117 MB) | `9baa794c84c5a90f1b2c84130877ba8ae5a8b1867ece00866dd3b03d0eccc106` | [app/notes.md](app/notes.md) |
| `libexitlag.dll` 3.27.1 ("API for proxying connections through ExitLag") | `f5e1de65de6945f60d083b7a56127d30dbd1618c2664f03bf87229140d1afcce` | [libexitlag/notes.md](libexitlag/notes.md) |
| `ndextlag.sys` 3.4.0.1 (rebranded Windows Packet Filter NDIS LWF, x64) | `70013fcd32f4745347cbd45c1cc911c0d6939048727f1c8dbcc1da36edf20fa9` | [drivers/notes.md](drivers/notes.md) |
| `nfextlag.sys` 1.7.1.6 (rebranded NetFilter SDK WFP callout, x64) | `1ec0364cdfb04fab4bbdbfc57854b739f906c165a336c670b26953775edf2c47` | [drivers/notes.md](drivers/notes.md) |
| `ndisapi.dll` 3.4.0.1 / `nfapi.dll` 1.5.2.4 (user libraries of the two drivers) | `0493674a…c89347` / `f12b0459…367408` | [drivers/notes.md](drivers/notes.md) |
| `ExitLagPmService.exe` (Intel PresentMon service 2.5.1, re-signed) | `df83f708cfc7549208c7c0d543994f09ff72bc9566b1aa27bf0696e1c1d0b984` | [helpers/notes.md](helpers/notes.md) |
| `handler.exe` (Crashpad handler), `exitlag_autostart.exe`, `exitlag_reboot_required.exe`, `DriverCacheCleaner.exe`, `snetcfg.exe`, `nfregdrv.exe` | see helpers notes | [helpers/notes.md](helpers/notes.md) |
| Inno Setup script (install/uninstall logic) | n/a | [installer/notes.md](installer/notes.md) |

Bundled third-party libraries (from `THIRD_PARTY_LICENSES.txt` and imports): Qt 6 (QML, Charts,
WebView2 host, NetworkAuth), Boost 1.87 (Asio), OpenSSL 3, Protobuf, abseil, fmt, zlib, ICU,
Chromium Crashpad, Intel PresentMon API 2.5.1.
