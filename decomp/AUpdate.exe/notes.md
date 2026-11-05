# AUpdate.exe notes (daily activity pings, cross-product detection)

Static only. Analysed copy: `sgb-extract/carved/pe_018c12fc_1797688.bin` (1,797,688 bytes; sha256
prefix `9e4424efcec0b6a5`). Method: `strings`. Despite the name it is IObit's generic "AUpdate"
(mutex `Global\IObit_AUpdate`, config `PCGameBoost\Smart Game Booster\AUpdate.ini`); the many
`iobit.com`/Driver Booster URLs are library leftovers.

## SGB-specific network calls (HTTPS, GET query strings)
- `stats.gameboosta.com/gamebooster/active_v4.php?uid=&app=&pr=&ver=&ln=&os=&type=&actv=&user=&insur=`
  (daily active, `dayactive`/`dayuser`; a second form adds `hx=` and `dd=`).
- `.../active_month.php?uid=&app=&pr=&ver=&ln=&os=&hx=&pd=&user=&type=&insur=` (monthly active).
- `.../moreuse.php?app=&ver=&user=&Use_ContinMonths=&Use_ContinFirstDate=&type=[&Use_PromClickCount=&Use_PromClickDate=]`
  (retention, and how often the user clicked promotions).
- `.../register.php?app=&pr=&ver=&user=&cidate=&cd=&hx=&lang=&guid=&type=&insur=&nrt=` (first
  registration; `cd` = code type, `guid` = install ID).
Field meaning (inferred): `uid` install/user ID, `os` Windows version, `ln`/`lang` language, `user`
licence tier, `insur` install source/affiliate, `hx` most likely a hardware hash.

## Cross-product detection
Checks the Uninstall keys / folders of Advanced SystemCare, Driver Booster, IObit Malware Fighter,
IObit Software Updater, IOTransfer, iTop VPN, iTop Screen Recorder, iTop Screenshot, iTop Data
Recovery, IObit SysInfo and SGB itself (each with a `*Rtt.ept` marker file). Used to coordinate the
family's updaters and stats.
