# ScanGameInfo.exe

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0c352c8f_5278408.bin`. .NET 8 NativeAOT, x64 (iTop). PDB
`E:\Code\ISR5\Source\SteamAppInfo-master\SteamAppInfoParser\...\ScanGameInfo.pdb`.

Built from the open-source SteamAppInfo parser: reads Steam's `appinfo.vdf` to list installed Steam
games (names, install dirs, executables). SgbMain runs it and waits (`RunAndWaitScanGameInfo …`) during
game detection. No network.
