# Smart Game Booster 6.0: the cleaners and the tweak scanner

Plain-language guide to SGB's "System Clean" (junk files), "Privacy Sweep" and "System Tweak" pages.
Found by reading `SgbMain.exe` and its data files; nothing was run. Addresses and evidence:
`decomp/iobit/smart-game-booster/SgbMain.exe/features.md` (section "Cleaners, System Tweak scanner…").

## Junk files and privacy

**Where the list comes from.** An encrypted file, `Database\PriTemp.dbd`, holds two lists: 164 junk
categories and about 365 privacy items (browsers, apps). Each entry names folders and file masks to
delete, registry keys or values to delete, a "is this program installed?" check, and a
"Recommended" flag. Some entries use built-in routines instead (Recycle Bin, Windows temp, DNS cache,
clipboard, browser databases).

**Scan.** SGB really looks: it walks the listed folders, adds up file sizes, and checks which
registry entries exist. Browser history and cookies are removed from the browsers' own databases with
SQL. No random numbers or made-up results were found in the cleaner code.

**Clean.**
- Files in use are not deleted now: Windows is asked to delete them at the next restart, and the list
  shows them as "(Delete on Restart)".
- Registry entries can be saved as `.reg` text before deletion (when a backup is open).
- It also loads **other Windows users' registry files** and cleans their entries.
- Optional "Secure Deletion" overwrites files before deleting them.

**How honest are the numbers?** Partly. After cleaning, "N items fixed" and the size are simply what you
had ticked, worked out from the scan. They are not reduced when a file couldn't be deleted, and files
waiting for a restart count as cleaned. Registry traces (recent-file lists, "UserAssist" history,
tray and menu caches) count as items just like files. Nothing is invented, but the result overstates
what actually went.

**Defaults to watch.** Marked "Recommended": the **Downloads folder** and "Windows Installer unused
files" (both can remove things you need). Not recommended: saved passwords, cookies and form data in
every browser (good).

## Registry cleaner

There isn't one. SGB 6.0 has no "invalid registry entries" scanner; the only registry deletions are the
cleaner items above.

## System Tweak

51 + 48 registry tweaks (`SysSpeed.dbd`, see `decomp/iobit/smart-game-booster/SgbData/dbd/README.md`). An item counts as a
problem whenever the current value differs from SGB's preferred value, **including when the value isn't
set at all**, which is Windows' normal state. So a fresh Windows install shows most items as problems.
The scan also creates any missing registry key it checks (empty keys, harmless but a write during a
"scan").

**Undo exists.** Before changing a value SGB saves the old one (or "not set") in `Tweaks.ini`, and
Restore puts it back or deletes the value. Two GB3 items have misspelled names and write junk values
(see the dbd README).

## For gamebooster-rs

None of this helps games run faster. Do not copy it.
