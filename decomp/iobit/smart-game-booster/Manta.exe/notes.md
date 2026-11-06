# Manta.exe

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_07a647b6_2076672.bin`. Delphi.

"Can You Run It" matcher: `/MacthCYRI`, `/normal`, `DataBase\CYRI.db`, `DataBase\CYRIUpdate.db`,
`CYRI.ini`, `CYRIupdate.ini`, `DataBase\ScoreCFG.ini`, `HotGame.ini`, `GameBooster.ini`. Compares the
PC against game requirements out of process. SgbMain launches `Manta.exe`. See
`../SgbMain.exe/features.md` (CYRI).
