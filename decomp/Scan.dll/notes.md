# Scan.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0034a3ac_1026560.bin`. Delphi, "Malware Scan Library for Smart Game Booster".

Factory exports `CreateScanObj`, `CreateScanThreadObj`, `CreateScanConfigObj`, `CreateCacheObj`,
`CreateDbObj`, `CreateEnumObj`, `CreateRepairObj`. Signature/rule databases `BHO.db`, `Cookie.db`,
`Coreext.db`, `File.db`, `FileInfo.db`, `Folder.db`, `GUID.db`, `GuidKey.db`, `GuidVal.db`, `info.db`.
The malware/add-on scanner used by SgbMain and AutoGuard. Internals not studied.
