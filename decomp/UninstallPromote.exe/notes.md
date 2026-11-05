# UninstallPromote.exe notes (install/uninstall beacon)

Static only. Analysed copy: `sgb-extract/carved/pe_020d7fa4_1917496.bin` (1,917,496 bytes; sha256
prefix `7a878fe00a89d38e`). Method: `strings`.

Despite the name, in SGB 6 it shows no promotion: its only form `TfrmUnistallPromote` has no
controls (caption left over from "Advanced SystemCare Uninstall"). The installer runs it as
`UninstallPromote.exe /install smgb6` and `/uninstall smgb6` (also `/Upgrade=`), elevated
(manifest `requireAdministrator`). It reports the event:
`operate=%s&user=%s&app=%s&ver=%s&pr=%s&system=%s&type=%s&lang=%s&geo=%s&insur=%s[&ref=]` to
`https://stats.gameboosta.com/gamebooster/install.php?` or `.../aff/install.php?` (affiliate
installs), with iTop/IObit siblings going to `stats.reportcpanel.com` / `stats.itopupdate.com`.
It reads affiliate IDs (`aff_param`, `a_aid`, `data1`) and first-install dates of other installed
iTop products (Screen Recorder, Data Recovery, Voicy, PDF, Easy Desktop, DualSafe, Top Data
Protector) to attribute the install. Uses the same S3/GCS "AntiBan" host fallback.
