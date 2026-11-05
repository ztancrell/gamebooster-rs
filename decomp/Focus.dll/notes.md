# Focus.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_03172595_555008.bin`. Delphi (runtime packages rtl120/vcl120).

Exports `PopConditionMet`, `PopConditionMetEx`: decides whether a promotion/notification popup may
show now (conditions: running full-screen apps, installed products, Steam app uninstall keys, CPU
info). Carries the IObit `appgoto.php?to=…` link table (buy, expired, compare, …) and Driver Booster
stat URLs as library leftovers. Callers not traced (not referenced by name in SgbMain).
