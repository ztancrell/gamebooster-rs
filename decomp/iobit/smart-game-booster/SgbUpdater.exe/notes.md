# SgbUpdater.exe notes (program updater)

Static only. Analysed copy: `sgb-extract/carved/pe_0123748c_3749432.bin` (3,749,432 bytes; sha256
prefix `8c3eccb069adf958`). Method: `strings`. Most URLs in the file (iobit.com, Driver Booster,
cd4o.com) belong to a shared IObit library linked into every helper and are not used by SGB.

## Update source
- Manifest (HTTPS): `https://update.gameboosta.com/infofiles/smartgb6/update.upt`, with
  `update_vip.upt` / `update_svip.upt` for paid tiers. Overridable by `UpdateUrl` in
  `TestUpdate.ini` (a test hook read from the program dir / `C:\Testupdate.ini`).
- Settings in `Update\update.ini`: `MainVer`, `BigUpgrade`, `ProServer`, `FileServer`,
  `UpdateCheckTime`, `ShowType`, `AutoPmt`, `IngoreVer`.

## Integrity checks (good)
Each downloaded file is checked by MD5 (`download failed: md5 error`), PE header sanity
(`CheckValidPE`), Authenticode via `wintrust.dll` WinVerifyTrust (`MsFileTrust`), and signer name
against an allow-list (`SignInfoIsIObitFile`): IObit Information Technology / IObit CO., LTD /
Chengdu Zhagu / Chengdu (Xiao) ShanHu / Chengdu ShanHe / ORANGE VIEW LIMITED. Failure:
`download failed: Sign error`. Exceptions: files listed in the encrypted `Update\ExpSlist.dat`
skip the signature check, and `32blackos`/`64blackos` keys in `update.ini` (OS versions, most
likely ones without SHA-2 support) relax it. Not traced which OSes.

## Crash reporting
Has the shared crash hook that runs `Bugreport.exe /bugreport /smgb … /silent` (see BugReport.exe).
