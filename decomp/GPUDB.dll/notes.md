# GPUDB.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_03237a65_183808.bin`. Delphi, "Overclock for Smart Game Booster".

Export `GetGpuInfo`. Imports `sqlite3.dll`; opens `GPUInfo.db` and runs
`Select * from GPU_NVIDIA where N_GPUName = '%s1' and N_GPUChip = '%s2' and replace(N_GPUMemInfo,' ','') = "%s3"`.
Looks up per-GPU limits for the overclock feature (see `../OverDrive.dll/notes.md`). Local only.
