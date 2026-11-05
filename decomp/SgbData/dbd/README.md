# SGB `.dbd` containers: decoded

Static analysis only; no vendor binary was run. Code addresses are in the correctly de-filtered
`SgbMain.exe` (sha256 `060f6621…d16f`, see `../../inno_carve.py`).

Decoded output and decoder are not kept in the repo (see decomp/README.md, 'Policy'). The decoder
checked the HAVAL self-test vector copied from SgbMain, each entry's size and CRC-32.

| File | Password (literal in SgbMain) | Entries |
|---|---|---|
| `Opt.dbd` | `asc6_2012_6` (VA 0x8c6671) | 31 INI-style tweak profiles + `Des-ref.txt` |
| `SysSpeed.dbd` | `asc9_speedup` | `SysSpeed.ini` (UTF-16), `GB3.ini` |
| `PriTemp.dbd` | `asc4_2010` | `junkfile.db` (165 sections), `privacy.db` (365 sections), both INI text |
| `Def.dbd` | `asc4_2010` (VA 0x83a868 path; same literal as PriTemp) | 17 malware-scanner signature lists, see [`../scanner/README.md`](../scanner/README.md). `Main.db`/`Signature.db` have compression byte 3: encrypted but not zlib-packed (the decoder must handle it); `BHO.db` appears twice (second copy overwrites the first when extracting) |
| `CYRI.dbd` | `SMGB5_CYRI` | opens with the same decoder (first block is zlib); not extracted (game DB) |

## Container format

All integers little-endian.

- **Trailer** (last 36 bytes): `u64 dir_offset, u32 ?, u32 entry_count, u32 2, u32 dir_packed_len, u32 1, 3×u32 0`.
- **Directory** at `dir_offset`: plain zlib (not encrypted), `entry_count` × 550-byte records:
  `ShortString name` (0x200 bytes), `u64 orig_size, u64 packed_size, u64 stored_size, u64 offset,
  u32 crc32(plain), u8 compression (2 = zlib), u8 encryption (2 = encrypted)`.
- **Entry** at `offset`, `stored_size` = `packed_size` + 24: `salt[8] | IV[16] | ciphertext`.
  The salt and IV are the same fixed bytes in every file, which is why all `.dbd`s share a 24-byte prefix.

## Cipher (DCPcrypt 2, with two quirks)

Callback VA 0x8c4bec (installed by the Opt loader VA 0x8c6554 at container+0xd8; the container
calls it from VA 0x7586c4, then zlib-inflates via VA 0x7583f0):

1. `key = HAVAL-256/5(salt ‖ pw_bytes)` (`TDCP_haval`, VMT 0x7506e8).
   - DCPcrypt's HAVAL pads with `0x80` instead of the standard `0x01` (Final, VA 0x753f98); its
     self-test vector (VA 0xa484b0) matches only with that padding.
   - `pw_bytes` = the first `Length(pw)` **bytes** of the UTF-16LE password (`UpdateStr`, VA 0x5eaa00,
     passes a UnicodeString pointer with the character count as the byte count, the usual Delphi-2009 DCPcrypt bug).
     So `asc6_2012_6` is hashed as `61 00 73 00 63 00 36 00 5f 00 32`.
2. `TDCP_rijndael.Init(key, min(256, 256) bits, IV)`, mode byte +0x34 = 0 = CBC (dispatch VA 0x5eb0fc).
   AES-256-CBC; a trailing partial block is XORed with `E(last ciphertext block)` (DCPcrypt CBC tail).
3. Output is a zlib stream (`78 9c`).

How it was found: the Opt loader names the container class methods (create 0x75793c, open 0x757a10,
extract 0x758a8c) and the callback; VMT class names (`TDCP_haval`, `TDCP_rijndael`) were read at VMT−0x38.
The other two passwords sit next to their `Database\*.dbd` paths in SgbMain's UTF-16 strings.

## What's in it

Summarised below: the Opt Win10 profiles, network default, `Promote.txt`, `Des-ref.txt`, and
`SysSpeed.ini` / `GB3.ini`. Other Opt profiles (XP/Vista/7/8 variants of the same five) and PriTemp
(junk/privacy cleaner path templates for browsers and apps; no performance tweaks) are not described
further. Decoded output and decoder are not kept in the repo (see decomp/README.md, 'Policy').

- **SysSpeed.ini** = the "System Tweak / speed-up" page: 51 items, each a single registry value
  (`RootKey` 2 = HKCU, 3 = HKLM, 4 = HKU; `IsShow=0` items are hidden children applied with their parent via `ModifyN`).
- **GB3.ini** = 48 "GB3_" tweaks with an OS bitmask (1 XP, 2 Vista, 4 Win7, 8 Win8, 16 Win10; RootKey 0 HKCR, 1 HKCU, 2 HKLM).
  A bad search-and-replace (`serv` → `offo`) broke two names: `NtfsMftZoneReoffoation` (meant
  `NtfsMftZoneReservation`) and `WaitToKilloffoiceTimeout` (meant `WaitToKillServiceTimeout`). Those two write junk values; the
  same corruption is in `Promote.txt`.
- **Opt.dbd** = the older ASC-6 "optimizer" profiles (`default`, `dailyuse`, `officework`, `server`,
  `topperformance` per OS) plus network profiles picked by `Main.ini [Main] NetOptType`
  (`net_default`, `LAN`, `dsl`, `modem`). Each section has `DefVal` (Windows default) and `OptVal`.
  `Des` numbers do not match `Des-ref.txt` (stale), so the table describes the registry effect instead.

SGB default on/off: the data has no per-item "ticked" flag. The scanner lists every item whose current
value differs from `OptVal` (`CompareStr`/`CompareMath`). Reversible: every item is one registry value; the
Windows default is recorded (`DefVal` in Opt, nothing in SysSpeed/GB3), so you can undo it by hand. Items that `Delete`
a value cannot be undone from the data alone.

### Tweaks (Win10-relevant, deduplicated across SysSpeed/GB3/Opt win10 profiles)

Gaming column for a 2019 MacBook Pro under Boot Camp (Win10/11, NVMe SSD, Intel CPU, AMD dGPU):
**helps** = plausible measurable effect, **none** = placebo / no-op on Win10, **harm** = can hurt.

| Tweak | Change | Gaming |
|---|---|---|
| Foreground CPU priority | HKLM `…\PriorityControl\Win32PrioritySeparation` = 38 (0x26: short, variable, 3:1) in SysSpeed; 26 (0x1A) in Promote | none: 0x26 equals the client default 2 (see docs/tweaks/optimizer.md) |
| HW-accelerated GPU scheduling | HKLM `…\GraphicsDrivers\HwSchMode` = 2 (GB3_48, Win10) | maybe; driver-dependent, needs reboot; Boot Camp AMD drivers are old, so test first |
| Kernel not paged | HKLM `…\Memory Management\DisablePagingExecutive` = 1 | none on 16–32 GB machines |
| `IoPageLockLimit` = 128 MiB | HKLM Memory Management | none (ignored since XP SP1) |
| `LargeSystemCache` = 0 (= Windows default); server profile sets 1 | HKLM Memory Management | none at 0; **harm** at 1 on a desktop |
| Worker threads +10 | `AdditionalCritical/DelayedWorkerThreads` (GB3_12/13, XP only) | none |
| NTFS last-access off / 8.3 names off / `NtfsMemoryUsage`=0 | HKLM `…\FileSystem\…` | none for games (Win10 already defaults last-access to system-managed) |
| Network throttling / QoS | HKLM Policies `Psched\NonBestEffortLimit`=0, `MaxOutstandingSends`=65535 | none (QoS reserve myth) |
| TCP legacy | `TcpWindowSize`/`GlobalMaxTcpWindowSize` = 8760, `Tcp1323Opts`, `SackOpts`, `DefaultTTL`=64, `TcpNumConnections`, `ServiceProvider` priorities | none on Win10 (keys ignored); 8760 window would **harm** on XP |
| DNS cache | `Dnscache\Parameters` negative-cache = 0, TTL caps 10800 | none |
| `Dnscache` service Start=4 (Promote) | disables DNS Client | **harm** (breaks DNS caching; Win10 may refuse) |
| Disable services (Start=4) | AxInstSV, AppMgmt, CertPropSvc, Netlogon, PeerDistSvc, SCPolicySvc, SNMPTRAP, TrkWks, MSiSCSI, WebClient, WinRM, WerSvc, PNRP/p2p, iphlpsvc, TabletInputService, SENS, seclogon, upnphost, DiagTrack, HomeGroupProvider | none for FPS (idle services); permanent unlike our Boost-time stop; SENS/seclogon/TabletInputService can break things (**harm**) |
| SysMain delayed start | `SysMain\DelayedAutoStart`=1 | none |
| UI timings | HKCU Desktop `MenuShowDelay`=0, `HungAppTimeout`=4000, `WaitToKillAppTimeout`=5000, `AutoEndTasks`=1, `ForegroundLockTimeout`=150000, `MouseHoverTime`=100, Aero hover times 0 | none for games; `AutoEndTasks` can lose unsaved work at shutdown |
| Visual effects off | `VisualFXSetting`=3, `TaskbarAnimations`=0, `ListviewAlphaSelect`=0, shadow/animation `DefaultApplied`=0, `MinAnimate`=0 | none in fullscreen games |
| Error reporting / debugger | `AeDebug\Auto`=0, PCHealth `DoReport/ShowUI`=0 | none |
| Boot / disk | `Dfrg\BootOptimizeFunction\Enable`=Y, `AutoChkTimeout`=5, `HiberbootEnabled`=1 (fast startup) | none; fast startup can confuse Boot Camp dual-boot clocks/NTFS mounts (**harm**) |
| Explorer misc | `Max Cached Icons`=2000, `AlwaysUnloadDLL`=1, `DesktopProcess`=1, `NoSimpleNetIDList`, `NoNetCrawling`, `link` shortcut-arrow text, `.scf` handler, show-hidden fix | none |
| Security/privacy | `RemoteRegAccess`=1, `NoDriveTypeAutoRun`=221, SEHOP on, `CWDIllegalInDllSearch`=2, CEIP/location/inventory/biometrics policies | not performance (neutral to good) |
| IE / WMP | `MaxConnectionsPerServer`=10, IE/WMP update checks off | none (IE is gone) |
| LanmanWorkstation/Server | `MaxCmds/MaxThreads`=30, `MaxMpxCt`=2048, NetBT `NameSrvQueryTimeout`=3000 | none |

Bottom line: for gaming on Boot Camp, only `HwSchMode` is worth considering, optionally and after
testing (`gamebooster status` reports it; see docs/tweaks/optimizer.md). `Win32PrioritySeparation`
0x26 equals the Windows client default. Superseded: an earlier version of this note said 0x26 helps
and that our Boost already sets it; it doesn't. The rest is legacy XP-era placebo, and
the service-disabling and `Dnscache` items are harmful as permanent changes.
