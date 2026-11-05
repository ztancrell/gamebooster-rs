# Smart Game Booster 6: telemetry, licensing, updates, promotion

What SGB 6.0 sends home, how it licenses and updates itself, and what the installer does. Found by
reading the shipped files (strings, some decompiled code); **nothing was run**, so how often
things fire and server-controlled switches are partly inferred. Details and evidence per program
are in `decomp/<program>/notes.md` (ProductStat3.exe, Register.dll, SgbUpdater.exe, AUpdate.exe,
DDownloader.exe, FeedBack.exe, BugReport.exe, UninstallPromote.exe, SgbInit.exe, Setup).

Who makes it: the code signers and build paths point at IObit / iTop (Chengdu-based companies:
"Chengdu Zhagu", "Chengdu ShanHe", "ORANGE VIEW LIMITED"). SGB shares libraries, stats servers
and update code with Advanced SystemCare, Driver Booster and iTop VPN. Many URLs inside the files
(iobit.com, Driver Booster pages) are unused leftovers of those shared libraries; the table lists
only endpoints that belong to SGB or to code SGB really uses.

## In short

- **Telemetry is on and has no off switch** that we could find. Daily and monthly "active" pings,
  feature-usage counters, install/uninstall events, game searches, hardware specs (CPU/GPU/RAM/OS),
  network-acceleration stats, and your country (looked up from your IP over plain HTTP).
- **It routes around blocking.** If a stats or licence server is unreachable, it downloads backup
  server lists from Amazon S3 / Google Cloud Storage ("AntiBan" in the code).
- **Crash reports can be uploaded silently**, including a list of all running processes, logs and
  dumps. Whether silent mode is on is controlled by a server-updated config file.
- **Licensing is online**, tied to your CPU and system-disk serial number. To defeat people who
  block the licence server, it **edits your hosts file** and flushes DNS without asking. Trial
  counters are hidden in `desktop.ini` files so a reinstall does not reset them.
- **Updates are reasonably safe**: manifest over HTTPS, each file checked by MD5 and by a valid
  code signature from an allow-listed company.
- **Installer**: no bundled offers, no browser changes. It adds scheduled tasks (one to start
  elevated without a UAC prompt, one to start with Windows), installs a packet driver (WinDivert),
  and DirectX DLLs. Refuses to install in China.
- **Upselling** is heavy: "Limited Time 85% OFF" pop-ups, SVIP expiry nags, timed free
  network-acceleration that disconnects your game when the balance runs out.

## Every network endpoint found

| Host / URL | Used by | What goes there | When |
|---|---|---|---|
| `https://stats.gameboosta.com/gamebooster/active_v4.php` | AUpdate | install ID, app, version, language, Windows version, licence tier, install source, likely hardware hash (`hx`) | daily |
| `https://stats.gameboosta.com/gamebooster/active_month.php` | AUpdate | same fields | monthly |
| `https://stats.gameboosta.com/gamebooster/moreuse.php` | AUpdate | months of continued use, first-use date, **count and date of promotion clicks** | periodic |
| `https://stats.gameboosta.com/gamebooster/register.php` | AUpdate | first registration: code type, install GUID, hardware hash, language | once |
| `https://stats.gameboosta.com/gamebooster/iusage_v2.php` | SgbMain via ProductStat3 | feature-usage counters (daily/weekly/immediate), cached and retried | daily / per event |
| `https://stats.itopvpn.com/iusage_v2.php` | ProductStat3 (default) | same, when no URL is passed | as above |
| `https://stats.gameboosta.com/gamebooster/smgb_hw.php` | SgbMain (game-requirements "Can You Run It") | CPU, GPU(s), RAM, OS name/bitness, scores | when the feature is used |
| `https://stats.gameboosta.com/gamebooster/app_stats.php?a=smgb_gamesearch` | SgbMain | game ID searched | on search |
| `https://stats.gameboosta.com/gamebooster/gethot.php` | SgbMain | download of "hot games" list | periodic |
| `https://stats.gameboosta.com/app_stats.php?a=ftmusage` / `portusage` / `tsxzusage` / `cnt` / `cntt` | SgbMain (accelerator) | connection stats: your IP, country, city, server, user ID, errors, latency/loss | per accelerator session |
| `https://stats.gameboosta.com/multi_app_lang.php`, `multi_app_new.php` | SgbMain, BugReport | language / crash event counters | periodic / on crash |
| `https://stats.gameboosta.com/gamebooster/smgb_bugreport_v2.php` | BugReport | zip of logs, crash dumps, settings, **process list**, modules, volumes, hardware, screenshot (optional), e-mail | on crash (may be silent) |
| `https://stats.gameboosta.com/smgbfeedback/feedback.php` | FeedBack | your message, e-mail, category, licence tier | when you send feedback |
| `https://stats.gameboosta.com/gamebooster/install.php`, `/aff/install.php` | UninstallPromote | install/uninstall/upgrade event, OS, language, country, affiliate ID, other iTop installs | install, uninstall, upgrade |
| `http://ascstats.pcgameboost.com/kwstats.php` (HTTP) | SgbInit | `k,s,c,p` keyword/source attribution, `type=newinstall` | first run |
| `http://ascstats.pcgameboost.com/active.php` (HTTP) | SgbMain via `StatUser.exe` | legacy active ping (`/a smgb4`) | periodic (legacy) |
| `http://stats.reportcpanel.com/gamebooster/proportion.php` (HTTP) | SgbMain startup manager | startup-item disable rates (request) | periodic |
| `https://ascstats.iobit.com/base-info.php` | SgbMain, Register | country lookup | as needed |
| `http://ip-api.com/json/` (HTTP), `https://ipinfo.io/` | installer, nearly every program | your public IP → country code | install and often |
| `https://1.1.1.1/`, `https://8.8.8.8:443`, `https://www.google.com` | SgbMain | connectivity / IPv6 tests | accelerator |
| `https://s3.amazonaws.com/myfilelist/list-*`, `https://storage.googleapis.com/myfilelist/list-*` | all helpers | download of backup server lists ("AntiBan") | when main host fails |
| `https://api.gameboosta.com/` (also hard-coded IPs `3.229.56.169`, `54.165.29.74`, `13.248.190.80`, `76.223.44.67`) | SgbMain, BugReport | account login, device ID/name, accelerator server lists, signed API calls | login, accelerator |
| `https://smgblicense.itopupdate.com/check.php` | Register.dll / SgbMain | activation code, CPU ID, disk serial | activation and periodic re-check |
| `https://update.gameboosta.com/infofiles/smartgb6/update[_vip|_svip].upt` | SgbUpdater, SgbInit | update manifest download | `UpdateCheckTime` schedule |
| `http://update.pcgameboost.com/infofiles/smartgb/pub/sgb.upt`, `http://update.superbgameboost.com/…`, `http://www.pcgameboost.com/infofiles/up/param_info.php` (HTTP) | SgbMain | legacy game-database / parameter updates | not traced |
| `https://s.pcgameboostdrivers.com/scan.php` (fallback `https://s1|s2.driverboosterscan.com/worker.php`), `http://f.pcgameboostdrivers.com/drivers/`, `http://www.cd4o.com/drivers/wlst/v.json` | AutoDriver (Scanner.dll), SgbMain / DDownloader (driver updater) | hardware IDs for driver matching, driver downloads | when driver update runs |
| `https://goto.gameboosta.com/appgoto?name=smgb&ver=&lan=&type=` (+ `&insday=&insur=&user=&idata=`) | SgbMain | links to buy/renew pages; `idata` = **which other IObit/iTop products you have installed** | when you click a buy/promo link |
| `https://goto.gameboosta.com/online` | SgbMain | promo/news check | periodic |
| `https://gamebooster.itopvpn.com/index.php?d=`, Facebook / X share dialogs | SgbMain | "share my PC score" | user action |
| `https://graph.microsoft.com/v1.0/me/photos/…`, Google auth link | SgbMain | profile picture after Microsoft/Google sign-in | user login |
| `https://download.gameboosta.com/dl/smart_game_booster_setup.exe` | SgbMain, SgbInit | link shown when a crack is detected | anti-crack |

## Can it be turned off?

- No setting, command-line flag or registry value that disables statistics was found. The
  settings file has `Autoupdate`/`update_method` (updates) and `FRunWithWindows` (tray at startup),
  but nothing for stats.
- Blocking the hosts helps only partly: the programs fall back to S3/GCS lists of other hosts.
- Licence hosts must not be blocked in the hosts file: Register.dll deletes such lines.
- For our project (gamebooster-rs): none of this is needed for Boost. Boost is fully local.

## Licensing (high level only)

- Tiers: **Free**, **VIP** (older lifetime-style codes, "Expiration date: Never (V6)") and **SVIP**
  (time-limited subscription). Codes now migrate into an online account.
- Check: online POST of code + CPU ID + system-disk serial; the server answers valid, invalid,
  expired, or over the seat limit. Result cached locally with last-check and server dates.
- Paid unlocks: mainly the **network accelerator** (VPN-style tunnel, free users get limited time),
  "Max Mode" GPU overclock, and a separate update channel. The local Boost (services, processes,
  RAM, power plan) is not tied to the licence in these strings.
- Anti-piracy: hosts-file cleanup, hidden trial markers in `desktop.ini`, proxy-DLL crack detection
  with an "unofficial version" warning.

## Updates

Manifest over HTTPS from `update.gameboosta.com`; each file is checked by MD5 **and** Windows
Authenticode with a signer allow-list. Two weak spots: an encrypted exception list
(`ExpSlist.dat`) and per-OS keys can skip the signature check, and some legacy game-database and
driver downloads use plain HTTP (protected only by MD5).

## Installer and first run

No third-party offers, no browser or search changes, no services of its own, no Run key. It adds
the scheduled tasks `SmartGameBooster SkipUAC` and `SmartGameBooster Startup`, a WinDivert packet
driver (`itvwd32/64.sys`, used by the accelerator), DirectX DLLs in System32, pins itself to the
taskbar (if ticked), silently removes older Game Booster versions, and refuses to install for IPs
in China. First run (SgbInit) also reads Explorer's UserAssist (Windows' record of programs you
ran); purpose unclear.

`UninstallPromote.exe` shows nothing in SGB 6: it is an install/uninstall/upgrade beacon that
reports affiliate IDs and other installed iTop products.

## Dark patterns

- Fake urgency: "Limited Time 85% OFF", "Save 85% today", repeated "Restore SVIP today" pop-ups
  (`Need Show SVIP salePop`), news/recommendation icons enabled by default.
- Timer pressure: "Your game will disconnect when time runs out. Upgrade to SVIP now".
- Tracking of promotion clicks (`Use_PromClickCount`) and of other installed products sent with
  purchase links.
- Silent system changes for licence enforcement (hosts file, hidden trial files).
- Silent crash uploads that include the process list.
- Not found: fake scan results or fake problem counts in the Boost/licence code we read (the
  junk-file/registry scanner was not analysed), and no "auto-renew" consent text in the client
  (renewal happens on the web store, not visible here).

## Verdict

Not malware, and the local Boost does what it says. But it is a commercial upsell funnel built on
top of a small free feature: heavy and non-optional telemetry (with blocking evasion), hardware
fingerprinting, silent edits to system files for licence enforcement, and aggressive discount
nags pushing a subscription whose main paid feature is a VPN-style network accelerator. "Money-grab"
is fair for the paid tier; the free Boost itself is a handful of local tweaks that gamebooster-rs
already reproduces without any of the above.
