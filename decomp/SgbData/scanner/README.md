# SGB malware scanner, security check, AV detection, remaining data files

Static analysis only; no vendor binary was run, nothing was contacted. Addresses are VAs in the
de-filtered `SgbMain.exe` (`/home/zpt/re/sgb-extract/app/cmAppVer/SgbMain.exe`). Data from
`/home/zpt/re/sgb-extract-db/data/{app}/6.0.1/`. The raw signature lists are **not** kept in the
repo; regenerate them with `python3 -I ../dbd/dbd_decode.py Def.dbd asc4_2010 <outdir>` (outside the repo).
Plain-language version: [docs/sgb/security-features.md](../../../docs/sgb/security-features.md).

## 1. Malware scanner ("Account Guard" / "Auto Scan Spyware")

### Engine: IObit's own, no licensed engine

- Classes `TMalwareScanner` (unit `uMalwareScanner`, VMT 0x823e74), `TScanConfig`, with interfaces
  `ICache`, `IEnum`, `IDatabase`, `IScan`, `IRepair`, `IScanThread` from unit `PScanBase`. Scan types
  (strings at 0x824884…): `FileDirScan`, `RegScan`, `CookieScan`, `MemScan` (running processes), `LinkScan` (shortcuts).
- No third-party engine: the installer ships no engine DLL, and SgbMain has no engine/SDK names
  (Bitdefender, Avira, ClamAV etc. appear only as browser paths or in the AV-detection list below).
  It is the IObit "spyware removal" module from Advanced SystemCare 4–era code: the database password is
  `asc4_2010`, the same as `PriTemp.dbd` (ASC 4, 2010).
- Created by the security page loader VA 0x83a508: `TSecurityGuard` (0x83a5c9) and
  `TMalwareScanner` (0x83a6fd), with path `Database\Def.dbd` (0x83a6e9) → VA 0x8250fc, which also reads
  `MalwareIgnore.ini` (`[Malware Scan Ignore]`, keys `PATH, VIRUSNAME, SCANTYPE, DBNAME, INDEX`; VA 0x8224f8).

### Database: `Database\Def.dbd`

The ordinary `.dbd` container ([../dbd/README.md](../dbd/README.md)), password `asc4_2010` (found by trying every
string in SgbMain against the first entry; the literal sits in SgbMain at 0x8d49a0). 17 entries:

| Entry | Lines | Format | What |
|---|---|---|---|
| `Main.db` | 1 | `LastUpdate=2021-03-15`, `Reference=265012` | Database version stamp |
| `MD5.db` | 550,714 | `id,ver,Name,md5` | Whole-file MD5 of known bad files. 485,377 `Trojan.Generic`, 40,358 `Malware`, 21,892 `Adware`, 988 `Bundler`, 874 `Worm`, 573 `Backdoor`, 347 `HackTool`, 127 `BitCoinMiner`, 19 Ransom. The `ver` field takes 21 values 1904–2054 (database builds). |
| `File.db` | 22,229 | `id,ver,Name,path` | Exact file paths with env macros (`%PROGRAMS%\Protection Center\Buy.lnk`) |
| `Folder.db` | 6,745 | `id,ver,Name,folder` | Folder paths (`%PROGRAMFILES%\PointingToDeals`) |
| `FileInfo.db` | 716 | `id,ver,Name,0,text` | Version-info strings (company/product: `http://infohold.co.kr`) |
| `Reg.db` | 17,227 | `id,ver,Name,key` | Registry keys/values |
| `Guid.db` | 11,304 | `id,ver,Name,{CLSID}` | COM GUIDs, looked up under `GuidKey.db` (42 roots) and `GuidVal.db` (14 Run/Toolbar keys) |
| `BHO.db` (2nd copy) | 412 | `id,ver,Name,file` | Internet Explorer helper objects |
| `TrackingCookie.db` | 7,245 | `id,ver,domain=Name` | Cookie domains, 774 labelled just `Tracking Cookie` (doubleclick, 00fun.com …) |
| `Coreext.db` | 440 | name | Browser-extension names (`qvo6`, `v9.com`, `salesresourcepartners`) |
| `BHO.db` (1st), `Site.db` (4,806 domains), `TC.db` (212 ad/tracker domains) | | | Older lists; their names are not among the scanner's file strings (0x824a7c–0x82995c), so likely unused |
| `Signature.db`, `File.ign` | 0 | | Empty. `Signature_Add.db` and the `*.ign` ignore lists are referenced but not shipped |

Totals over the 7 detection lists: 609,347 entries. By family: trojan 506,750, malware 40,358,
adware 23,945, misleading (rogue AV, "PC optimizers") 12,545, backdoor 7,252, worm 1,734, PSW 1,440,
fraudtool 1,357, bundler 988, PUP 915, unwanted 729, spyware 682, hijacker 525. Samples:

```
3904670,1963,Trojan.Generic,0000230ecb17a1e1cfd818dc7c78d79e        (MD5.db)
1000000,1000,Misleading.Protection Center,%PROGRAMS%\Protection Center\About.lnk   (File.db)
300003,1000,Adware,%PROGRAMFILES%\PointingToDeals                  (Folder.db)
2000001,1000,Misleading.AndromedaAv,HKCR\*\shell\AV                (Reg.db)
500002,1000,00Fun.com=Tracking Cookie                               (TrackingCookie.db)
```

No byte patterns, no heuristics, no emulation, no unpacking: every rule is an exact MD5, an exact
path, a registry key, a GUID or a cookie domain. Names are generic and many entries target
2008–2012 rogue antivirus and toolbars (`antivirus2008pro`, CoolWebSearch, EasyPDFCombine toolbar).

### Scope of a scan

- File MD5 check only inside a fixed list of ~90 folders (table at 0x822708…0x823c9c), mostly
  **non-recursive** (`:RECn` = depth n): `%WINDIR%`, `%SYSDIR%`, `%SYSDIR%\Drivers`, `%ROOTDRIVE%`, `%TEMP%:REC3`,
  `%APPDATA%`, `%PROGRAMFILES%`, `%DESKTOP%:REC3`, `%STARTUP%:REC9`, Recycle Bin, plus XP-era paths
  (`Local Settings`, `KazaaBackupFiles`, `NetMeeting`, `outlook Express`). Only extensions
  `*.exe,*.txt,*.ocx,*.sys,*.cpl,*.cmd,*.ser,*.bin,*.com,*.bat,*.vxd` (0x8222cc; no `.dll`, no `.scr`, no archives).
  `%LOCALAPPDATA%`, `Downloads` under the user profile and `ProgramData` are not in the list.
- Registry, GUID/BHO, running-process (`MemScan`), shortcut (`LinkScan`) and browser-cookie
  (`CookieScan`: Firefox, Chrome, Edge; asks you to close the browser) checks against the lists above.
- A scan cache `sw_ScanCache`; idle-time auto scan (`Btn_AutoScan`), listed as a Pro feature ("Auto Scan Spyware").

### Updates

The shipped database is dated **2021-03-15** (`Main.db`) in a 6.0.1 build whose other files are from
2024–2026. No code path that downloads a newer `Def.dbd` was found: SgbMain has no URL or update
key naming `Def.dbd`, and `Update\Update.ini` (`[MainControl]`, `[FileUpdate] FList=` empty) has no
signature-update entry. Only a whole-program update would replace it. Confidence: medium (absence of evidence).

### On detection

Results show `Threat Type`, `Risk Level` (Low/Middle/High), action `Repair` / `No Action` / `Ignore`
(`[M_MalwareScanner]` in `English.lng`). Repair moves files to `<data dir>\QuarantineZone\`
(path built at VA 0x822236), deletes registry items and cookies; `Ignore` writes `MalwareIgnore.ini`.
The UI then says "N threats removed". (Exact move-vs-delete per type not traced; quarantine folder confirmed.)

## 2. Meaningful protection or upsell?

Evidence, not opinion:

- Database last updated 2021-03-15, no signature update channel found; 85 % of rules are
  `Trojan.Generic` MD5s, defeated by any one-byte change in the file.
- No DLL scanning, fixed shallow folder list, no heuristics, no real-time protection.
- **Tracking cookies count as threats.** `CookieScan` uses 7,245 cookie domains (incl. `doubleclick.net`,
  plain `Tracking Cookie` labels); the UI groups them under "Spyware threats" with
  `24=Cookies: Potential risk of privacy leak` and the headline `<a>%s</a> threats found`
  (`M_Account_threats`). On any machine that has browsed the web this produces a non-zero "threats" count.
- Marketing text frames it as account protection ("Scan and remove spywares to ensure game account
  and trade security", "It will affect your game security. Click View to check details and remove now.").

So: it can catch some old, unmodified, known samples in a few folders; as protection it is far
below Windows Defender, which is already present. Its practical output on a normal PC is mostly
cookies reported as "threats". High confidence on the data, medium on the UI counting (strings, not traced code).

## 3. Security check and "antivirus uninstaller"

### Security check (`TSecurityGuard`, 0x81e7f4; "Security Guard" page)

- WMI `ROOT/SecurityCenter2` (and `ROOT/SecurityCenter` on XP): `AntiVirusProduct`, `AntiSpywareProduct`,
  `FirewallProduct` (`displayName`, `productState`, `onAccessScanningEnabled`, `productUptoDate`). Shows
  "Your PC is not fully protected!" when one is off.
- Fix antivirus (VA 0x81ecdc): `cmd.exe /c PowerShell Set-MpPreference -DisableRealtimeMonitoring 0`, i.e. turns
  Defender real-time protection **on**; for third-party AV it only tells you to open it. Fix firewall:
  `netsh firewall set opmode enable` (0x81d342). It never turns protection off
  ("For the sake of system security, it cannot be turned off"). Benign.

### "Antivirus uninstaller" (`TAVUninstall` VMT 0x8fb4bc, `TAVProdItem`)

It is a detector, not an uninstall feature. `TAVUninstall` reads `DetectionEx.ini` (plain UTF-16 in
the program folder, 100 sections `[P_<vendor>]` with boolean `DisplayName=` expressions like
`('avast' & ('Antivirus' | 'Security' | 'Premier'))`) and matches them against the Windows Uninstall
keys (`DisplayName`, `Publisher`, `NotUnInst`). COMODO Internet Security is also found through its
`cmderd` service (VA 0x8fb5a8). Products covered: Avast, AVG, Avira, Bitdefender, Kaspersky, Microsoft
Security Essentials, Norton/Symantec, Trend Micro, ESET, McAfee, Ad-Aware, VIPRE, Panda, G Data,
F-Secure, Sophos, Emsisoft, Webroot, ZoneAlarm, COMODO, Dr.Web, Quick Heal, 360, Tencent, Kingsoft,
Baidu, Ahnlab, Spybot, SUPERAntiSpyware, TotalAV, HitmanPro.Alert, ClamAV/Immunet and ~60 smaller ones.
No IObit product is in the list.

Callers found: VA 0x90605c (loads `DetectionEx.ini`, compares publishers) and VA 0x9c4d00, the network
accelerator, right before it loads `iTopClientExt.dll`; that pairs with the message
`FKaspersky=Network Acceleration is unavailable due to Kaspersky. Please uninstall Kaspersky first`.
The neighbouring uninstaller unit can run `UninstallString` / `msiexec.exe /x` (0x8f94d4, 0x8f9930), but no
static call from the AV detector reaches it (calls are virtual; not resolved) and the language file has
no "uninstall antivirus" text. `UninstallRote.dbd` / `PlugDB.dbd` (IObit Uninstaller databases,
password `asc4_2010`) are not shipped; SgbMain copies them from an installed IObit product ("copy IUDB", VA 0x8ffeac).
Confidence: medium.

## 4. Other data files in the install

| File | Status |
|---|---|
| `DetectionEx.ini` | Plain UTF-16, described above. |
| `Database/DriverCleanIgnore.dat`, `Update/ExpSlist.dat` | `.dbd` containers holding `DriverCleanIgnore.ini` (1,848 B) and `ExpSlist.txt` (369 B). Directory readable without key; password **not found** among ~450k strings (digits or `_`, 4–24 chars) from all 113 carved binaries (owners: `DriverClean.dll`, which has no carved copy, and `SgbUpdater.exe`). Purpose from the owners' notes: driver-cleaner ignore list; updater signature-check exceptions. |
| `Database/SpnnDef.sgdt` (310 KB), `SgamDef.sgdt` (7 KB), `orgbase.sgdt` (104 B) | Not decoded. Network accelerator defaults: SpnnDef = server list cache (`GBseverList.sgdt`), SgamDef = game list (`GBgameList.sgdt`), from SgbMain 0x6bb262 / 0x6be192. Entropy 8.0 (orgbase 6.4), lengths not block multiples; not `.dbd` containers. Encryption sits in the accelerator code, not followed. Not scanner data. |
| `Database/Ngulocal.zip` | Encrypted zip of `Ngulocal.json` (already noted). |
| `Database/InBoxDriverFeature/*.ini` | Plain INI per Windows version: `[md5]` → `INFFileName`, `INFFileVersion`, `INFData`, `InfProvider` of in-box drivers (driver updater uses it to skip Windows' own drivers). `vista64.ini` is empty. |
| `LocalData/InstConf.ini` | UTF-16 INI: `[ImportantDevice]`, `[SpecialIgnore]`, `[PopUp]`, `[GC_SpecialIgnore]`, lists of PCI class codes (driver updater filters). |
| `LocalData/HotGame.ini` | Hot-games list (`id=name`). |
| `winid.dat` | UTF-16 INI: language → product-locale map and per-language lists of numeric IDs. |
| `lang.dat` | UTF-16 `[CusLan] Name=` (empty; custom language choice). |
| `GameSource/<md5>.zip` | Plain zips of game artwork (`gm_background.png`, `gm_logo.png`, …, dated 2026-04). |
| `smgb.spritefont` | DirectXTK `DXTKfont` bitmap font for the in-game overlay. |
| `Update/Update.ini` | UTF-16 updater config (`[MainControl]`, `[smgb_config]`); see `SgbUpdater.exe/notes.md`. |
| `Alarm.wav`, `Language/*.lng`, `HUD64/*.txt` | Sound, UI strings, HUD lists (described in their components' notes). |
