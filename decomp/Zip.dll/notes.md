# Zip.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_07ef33ee_594432.bin`. Delphi, "Zip Library".

Exports `ExtractZipFile`, `IsValid`, `IsZipValid`, `ZipSingleFile`, `ZipDirectoryContents`,
`ZipBackupDriverFiles`. Used by DbzInst (driver packages) and BugReport. `ZipBackupDriverFiles` is a
driver-backup helper; no caller was found in AutoDriver/Scanner/DbzInst strings, so the "no driver
backup" finding in `../AutoDriver.exe/notes.md` stands (not traced in code).
