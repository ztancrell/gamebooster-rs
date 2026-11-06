# Diagnosis.exe (+ dxhelper.dll)

Static strings, imports and exports only (objdump/strings); nothing executed. Carved: Diagnosis `pe_01a78134_2845240.bin` ("PC Diagnosis for Smart Game Booster", Delphi);
dxhelper `pe_07dce9ee_94720.bin` (MSVC).

PC diagnosis report window. Imports `dxhelper.dll` (23 exports `DXHelperGetDisplay*`: display/adapter
info and DXVA deinterlace caps; PDB `\trunk\plugins\dxhelper\...`). Carries the Driver Booster
library leftovers seen in AutoDriver (`idb.iobit.com/check.php`, `s1/s2.driverboosterscan.com/worker.php`,
`ascstats.iobit.com/...`, "2 drivers need to be updated", `bDriverBeta`) and posts usage to
`https://stats.gameboosta.com/gamebooster/iusage_v2.php`. Logs `DiagnosisAppRun.log` (gathered by
BugReport). SgbMain launches `Diagnosis.exe`.
