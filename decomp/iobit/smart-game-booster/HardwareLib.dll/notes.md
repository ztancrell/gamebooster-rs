# HardwareLib.dll (+ Temperature.dll)

Static strings, imports and exports only (objdump/strings); nothing executed. Carved: HardwareLib `pe_07c8d3ee_189952.bin` (MSVC, internal name `Hardware.dll`); Temperature
`pe_07c5f7b6_187448.bin` (MSVC, "Temperature Module").

- **HardwareLib.dll**: `HTLIB_CreateInstance`, `HTLIB_Get{CPU,GPU,MBD,FAN}Temperature`,
  `HTLIB_UninstallServices`. Installs a kernel driver for sensor access
  (`drivers\Monitor_win10_x64.sys`, `Monitor_win7_x64.sys`, `Monitor_x86.sys`, `GetWinRingDriverName`:
  a WinRing0-style port/MSR driver), mutex `Global\{C8A27592-6739-4722-B53D-28204CF79C50}`, log
  `hardinfo.log`. Also reads display-driver registry values (`UserModeDriverName`, Vulkan/OpenCL vendor
  keys). PDB `...\TemperatureMonitor\HardwareLib\...`.
- **Temperature.dll**: `CPUIDInterface_*` (init, sensor values, dump): a wrapper over `cpuidsdk.dll`
  (CPUID SDK, third-party). PDB `CPUIDInterface_ohm_TEST`.

Used for the tray's temperature warnings and the in-app hardware monitor.
