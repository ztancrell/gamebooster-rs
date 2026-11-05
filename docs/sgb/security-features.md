# Smart Game Booster: security features

What SGB 6.0's malware scanner, security check and antivirus detection actually do. Found by
reading the program and its data files; nothing was run. Technical details and addresses:
[decomp/SgbData/scanner/README.md](../../decomp/SgbData/scanner/README.md).

## Malware scanner ("Account Guard", "Auto Scan Spyware")

- **Engine.** IObit's own small scanner, carried over from Advanced SystemCare (around 2010). No
  licensed antivirus engine is included.
- **Signatures.** One encrypted file, `Database\Def.dbd`, holding about 609,000 simple rules:
  550,000 file fingerprints (MD5 of the whole file), 22,000 file paths, 6,700 folder paths,
  17,000 registry keys, 11,000 COM IDs, a few hundred browser add-on names, and 7,245 tracking-cookie
  domains. No patterns, no behaviour checks, no real-time protection.
- **Age.** The database says `LastUpdate=2021-03-15`. No way to download newer signatures was found;
  only a full program update would replace it.
- **What it scans.** About 90 fixed folders, mostly only their top level (Windows, System32,
  Program Files, Temp, Desktop, Startup, Recycle Bin, several Windows XP-era paths), and only
  `.exe .txt .ocx .sys .cpl .cmd .ser .bin .com .bat .vxd` files (not `.dll`). Also registry keys,
  running programs, shortcuts and browser cookies (Firefox, Chrome, Edge).
- **On detection.** You choose Repair, No action or Ignore. Repair moves files into a
  `QuarantineZone` folder and deletes registry entries and cookies; Ignore remembers the item in
  `MalwareIgnore.ini`.

**Verdict.** Close to no protection. Old, rarely updated fingerprints that miss any changed file, in a
few folders, with no real-time scanning. Windows Defender, which is already on your PC, does far more.
Tracking cookies are counted as "Spyware threats", so on almost any PC that has used a web browser it
reports "N threats found" and urges you to "remove now to ensure the security of your game account".
That number is mostly cookies.

## Security check ("Security Guard")

Reads the antivirus and firewall status that Windows reports. Its "fix" turns Windows Defender
real-time protection **on** (`Set-MpPreference -DisableRealtimeMonitoring 0`) and switches the Windows
firewall on. It never turns protection off. Harmless, and it duplicates the Windows Security app.

## Antivirus "uninstaller"

In this version it is a **detector**, not an uninstall tool. A plain-text list, `DetectionEx.ini`,
names about 100 antivirus products (Avast, AVG, Avira, Bitdefender, Kaspersky, Norton, McAfee, ESET,
Trend Micro, Sophos, 360 and many smaller ones; no IObit product). SGB matches it against your
installed programs. The network accelerator uses it to refuse to start while Kaspersky is installed
("Please uninstall Kaspersky first"). SGB contains generic code that can run a program's uninstaller,
but no menu or text in 6.0 offers to uninstall antivirus software.

## Other data files

Every data file in the install is now described in the technical note above. Three remain encrypted
and undecoded: the accelerator's server/game lists (`SpnnDef.sgdt`, `SgamDef.sgdt`, `orgbase.sgdt`) and
two small lists (`DriverCleanIgnore.dat`, `ExpSlist.dat`) whose password is not in any shipped program
we could read. None of them is part of the malware scanner.
