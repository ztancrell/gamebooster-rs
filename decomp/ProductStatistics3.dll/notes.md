# ProductStatistics3.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0809b3ee_1150008.bin`. iTop "Product Statistics".

In-process twin of `ProductStat3.exe` (see `../ProductStat3.exe/notes.md`): exports `SetStatParam3`,
`DoStatistic3`, `DoStatCusEx3`, `DoStatCommon`, `PostLocalCache3`, `ClearCacheByApp3`,
`HttpPostContent`. Default endpoint `https://stats.itopvpn.com/iusage_v2.php`; country lookup
`ip-api.com`/`ipinfo.io`; AntiBan lists. SgbTray loads it by name (`SetStatParam3`, `DoStatistic3`,
code near `0x525acb`).
