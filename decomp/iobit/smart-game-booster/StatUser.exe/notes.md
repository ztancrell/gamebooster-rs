# StatUser.exe (not shipped)

Static only. **No `StatUser.exe` is in the 6.0.1 install**: absent from `sgb-extract-db/data/` (all
`{app}`, `{tmp}`, per-OS dirs) and from the installer script `embedded/script.ps`; no carved PE carries
the name. It exists only as a string in SgbMain (carved `pe_00444dac_9008696.bin`).

## The SgbMain code that would run it

One function in SgbMain (`TFormCrack` area, starts `0x963cf0`; no direct `call` to it was found, so
it is reached through a method pointer):

1. Opens `<app>\Update\Update.ini`. If `[smgb_config] STAT_DayActive` (default 1) = 1, builds a stats
   object (`0x95f43c`, product `smgb`) and, when its check `0x9631d4` passes, sends the day-active
   ping (`0x9634f8`), i.e. the in-process path behind AUpdate's `active_v4.php` fields.
2. If `[smgb_config] STAT_MonActive` (default 1) = 1, posts message `0x3506` to a window
   (`0x963dc4`/`0x40ba18`), which triggers the monthly ping.
3. Then `jmp 0x963ed9` (function exit, `0x963dd7`). The code after it (`0x963ddc`..`0x963ed1`) has no
   branch into it: it reads `[smgb4_Control] STAT_ActUser` and `gap_day` and would `ShellExecute(open)`
   `StatUser.exe /u http://ascstats.pcgameboost.com/active.php /a smgb4 /p smgb /v <ver> /t 1 /d <gap_day>`
   (strings `0x96404c`, `0x964074`, call `0x963eb7`).

**Conclusion** (medium-high confidence): the StatUser path is dead legacy code from SGB 4 (`smgb4`);
6.0 never sends `ascstats.pcgameboost.com/active.php`. Daily/monthly active pings come from the
in-process stats object (steps 1-2) and from `AUpdate.exe` (`active_v4.php`, `active_month.php`).
