# DriverClean.dll

Static strings only; no carved copy with the same size (821,760 bytes) was found, so read from the
`-db` extraction. Nothing executed.

Removes old driver packages from the Windows DriverStore ("Slim Folder size", `CleanDriver`,
`clean success`). Evidence: `DriverStoreDeleteDriverPackageW`, `DriverStoreInfList`,
`SetupUninstallOEMInf`-family SetupAPI imports, per-device reads of `Provider`, `DriverVersion`,
`DriverDateData`, `ClassGUID`, `Rollback`, and a list of named INFs (`amdkmafd.INF`, `AtihdWT6.INF`,
`nv_ref_pubwu.inf`, `netwsw04.inf`, ...) that is probably a keep-list.

Consequence: after a clean, Device Manager's "Roll Back Driver" may have nothing to roll back to,
since that feature uses the previous package in the DriverStore. Combined with the updater's lack of
its own backup (`../AutoDriver.exe/notes.md`), an update plus a clean can be one-way.
