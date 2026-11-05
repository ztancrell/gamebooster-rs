# LocalLang.exe

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0d32483a_240696.bin`. "Check Language".

Reads `GetSystemDefaultLangID` and writes the result (`LangID`, `_Lang`) to `Lang.dat`; also touches
`pr.dat`, `winid.dat`. Used at install/first run to pick the UI language. No network.
