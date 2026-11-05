# sdcore.dll (+ dataexchange.dll, sdassist.dll, sdproxy.exe): Smart Defrag engine

Static strings, imports and exports only (objdump/strings); nothing executed. Carved: sdcore `pe_08691e5e_219136.bin`, dataexchange `pe_0864c65e_78336.bin`, sdassist
`pe_0865f85e_206336.bin`, sdproxy `pe_086c765e_96768.bin`.

IObit **Smart Defrag 5** engine, rebranded (PDB paths `E:\SVN\SmartDefrag\SD5 Engine\sdcore\...`,
`E:\SVN\SmartDefrag\SmartDefrag V5\sdassist5.5\...`). Backs SGB's disk-defrag feature.

| File | Role |
|---|---|
| `sdcore.dll` | Engine API `SDAPI_*` (`SDAPI_Defrag`, `SDAPI_GetFragmentInfo`, `SDAPI_GetFreqFile*`, result lists); reads `Config.ini`, `sdcore.dat`, `Update\Update.ini`. |
| `dataexchange.dll` | Shared progress state `DCAPI_*` (current path/position, fragment info, map cells, notify window/message); imports only kernel32. SgbMain imports `DCAPI_Initialize`, `DCAPI_GetCurrentPos`, `DCAPI_GetFragmentInfo`. |
| `sdassist.dll` | Report/UI helper `sdassist_*` (add file/HDD/junk-file items, `sdassist_GetAllUsbVolume`, language). SgbMain imports `sdassist_GetAllUsbVolume`. |
| `sdproxy.exe` | Out-of-process worker (`gsdresult.db` results, `Update\Update.ini`). |

No network URLs in any of the four.
