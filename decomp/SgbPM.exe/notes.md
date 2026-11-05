# SgbPM.exe (+ HUD64/SgbPM64.exe)

Static strings, imports and exports only (objdump/strings); nothing executed. Carved: `pe_0c04820f_2218040.bin` (32-bit), `pe_03da9e3d_3555896.bin` (64-bit). Delphi (newer
RTL), "Process Info for Smart Game Booster".

Small helper that reports process information (path/command line, `/Path="`) back to SgbMain; the
64-bit copy exists so 64-bit game processes can be queried. Strings are almost all VCL; no network.
Exact protocol not traced.
