# UWPLink.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_07de5bee_991232.bin`. Delphi.

Export `GetUWPLinkInfo`: resolves Microsoft Store (UWP) app shortcuts to package/app IDs so Store
games can be added and launched. SgbMain references `UWPLinkDataDefine`. No network.
