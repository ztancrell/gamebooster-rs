# ICONPIN32.exe (+ ICONPIN32.dll, ICONPIN64.exe)

Static strings, imports and exports only (objdump/strings); nothing executed. ICONPIN32.exe carved `pe_0bc38738_1048264.bin`; ICONPIN32.dll (1,052,360 B) has no carved copy
of that size (`-db` strings only); ICONPIN64.exe in the per-OS dirs. iTop "Common Component".

Pins the program's icon to the taskbar/Start (`ICONPIN.exe Done`, `Commu.ini`). Installer helper. No
network.
