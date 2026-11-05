# ProductStat3.exe notes (usage-statistics sender)

Static only, nothing executed. Analysed copy: `sgb-extract/carved/pe_081b4026_1235512.bin`
(1,235,512 bytes, same size as the named `-db` copy; sha256 prefix `e0a0685d8f267046`). Method:
`strings` (ASCII + UTF-16) on the carved copy. No Ghidra pass; findings are string-level, medium confidence.

## What it is
iTop's shared stats daemon (Delphi; strings name `iTop\StatsData3`, `StatCache3.db`, `[ProductStat]`).
Other SGB programs launch it with a command line built by `uSMGBProductStat3` (in SgbMain and SgbInit):
`/statcom /stflag <dyonce|dyeach|dyweek> /appid "…" /pd "…" /url "…" /user <type> /insur "…" /type …
/pr "…" /ver "…" /ExtParam "…" [/postnow] /cachepath "…"` and `/postcache /days … /cachepath "…"`.

## What it sends and where
- Default endpoint `https://stats.itopvpn.com/iusage_v2.php` (SgbMain passes
  `https://stats.gameboosta.com/gamebooster/iusage_v2.php` via `/url`). The stats host can be
  overridden from `Update\Update.ini` (`statIP`, `Server`).
- Payload: JSON-ish fields `pd`, `app`, `post_id`, `content`, `extparam`; counters `TotalStat`,
  `WekStat`, `Maj_%s_Stat`, `Min_%s_Stat`, `OtherStat` (feature-usage counts).
- Frequency flags: once per day (`dyonce`), every event (`dyeach`), weekly (`dyweek`), or immediately
  (`postnow`). Unsent posts are cached in `%ProgramData%\iTop\StatsData3\StatCache3.db` and retried
  (`PostLocalCacheEx`, `/postspcache`).
- Country lookup: `http://ip-api.com/json/` then `https://ipinfo.io/` (plain HTTP first); result
  cached in `HKLM\SOFTWARE\iTop` `LocalCC`/`CCdtid`.
- "AntiBan" fallback: if the stats host is unreachable, it downloads a backup host list from
  `https://s3.amazonaws.com/myfilelist/list[-smgb|-itop|-dpm|-pdf|-ied|-ivc]` or the same paths on
  `storage.googleapis.com`, stored in `%ProgramData%\iTop\BkListData\`. This is a block-evasion
  mechanism: blocking the main host does not stop the stats.
- Uses the user's proxy settings from `SmartGameBooster.ini` (`FEnableProxy`, `FProxyServer`, ...).

## Opt-out
No opt-out string or setting was found in this binary. Whether SgbMain skips launching it for some
setting was not traced.
