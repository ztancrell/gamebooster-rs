# GPUlib.dll notes (GPU usage monitor, read-only)

Carved copy: `carved/pe_0c2bc647_130048.bin` (130,048 bytes, i386, PDB `SMGB_gongjie\GPUlib`).
Exports `EtGpuMonitorInitialization`, `EtGpuProcessesUpdatedCallback`, `EtGetGpuNodeName`,
`_SetCallBackFun@4`: the "Et" names and code shape are Process Hacker's ExtendedTools GPU monitor.
Uses `D3DKMTOpenAdapterFromDeviceName` / `D3DKMTQueryStatistics` (gdi32) and CfgMgr32 device
enumeration to read GPU engine utilisation per process. Read-only; changes nothing. Feeds the
HUD/monitor GPU-usage figure.
