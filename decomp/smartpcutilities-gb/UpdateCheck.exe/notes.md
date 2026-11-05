# UpdateCheck.exe and UpdateInstaller.exe

Both ConfuserEx-protected; names only. `UpdateCheck.exe` asks the vendor for update info
(`AppUpdateInfo`: version, download link, MD5/SHA-256, channel stable/beta), shows `FrmUpdateNotify`,
and downloads to an updater cache (`DirUpdaterCache`, `DownloadUpdate`, `backgrounddownload`).
`UpdateInstaller.exe` runs the downloaded installer (`BWUpdateInstaller`).

Integrity: the shared library verifies files with `WinVerifyTrustEx` (`VerifyFileSignature`,
`FileNotSigned`, `InvalidSignature`, `SignatureOrFileCorrupt`) and has XML-signature verification
with embedded RSA keys, so updates are most likely checked by Authenticode and a signed manifest.
The update URL is encrypted and was not recovered. Caveat: the signing certificate expired on
2026-04-25; how the check treats that is not visible.
