# decomp/pgware-gameboost/

Static analysis of **PGWARE GameBoost** (pgware.com/products/gameboost), done for comparison with
Smart Game Booster. Nothing was executed (no Wine). Plain-language overview with verdicts:
[docs/pgware/README.md](../../docs/pgware/README.md).

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05 from `https://www.pgware.com/downloads/gameboost.exe` (link on the official product page) |
| Installer | `gameboost.exe`, 5,997,123 bytes, sha256 `78af5430b7d58575803bf0b845f8cd98b112a584f898166e708059c5bc0ec8b7` |
| Installer type | Inno Setup 6.3.0 (`Inno Setup Setup Data (6.3.0)`), LZMA1, `PrivilegesRequired=Admin`, 32-bit |
| Version | Installer `AppVersion` **4.6.8.2026**; the programs themselves report **3.10.21.2024** (update/register URLs, `TGBMainForm`), About box "GameBoost 3", "2001-2021"; the product page still says 3.10.21.2024 |
| Signatures | **None.** The installer and all three programs have an empty Authenticode (Security) directory. |
| Tech stack | Delphi (VCL, "Recompiled with Delphi 2023" per the product page), Indy `TIdHTTP` for HTTP, no .NET, no drivers, no DLLs |
| Local copies | `/home/zpt/re/pgware-gameboost/dl/` (installer), `x/` (binary-refinery `xtinno`: `data/{app}/…`, `meta/setup.json`, `embedded/script.ps`), `carved/` (`inno_carve.py`). The three carved PEs are byte-identical to the `xtinno` ones, so the Inno call filter is not a concern here. |

## Installer (`meta/setup.json`, `embedded/script.ps`)

- Installs to `C:\Program Files (x86)\PGWARE\GameBoost`: `GameBoost.exe`, `GameBoostRegister.exe`,
  `GameBoostUpdate.exe`, `GameBoost.chm`, `ReadMe.rtf`, `License.rtf`, `History.rtf`, `uninsimg.dat`
  (the Inno uninstaller data, via `CustomDestName`).
- Registry: only `HKLM\Software\PGWARE\GameBoost` (`InstalledDate`, `Name`, `Serial`, `ProcessorType`,
  `InternetType`, `SpeedSetting`, `Restore`=0, `Updated`=1, `UpdatedDate`, `Language`).
- No services, no scheduled tasks, no Run key, no drivers, no bundled third-party offers, no
  `UninstallRun`. One optional `[Run]` entry ("Launch GameBoost", post-install checkbox).
- Start-menu icons: app, help, "GameBoost Support" (web link), "Update GameBoost"; optional desktop icon.
- Pascal script (`script.ps`): only `ShowIcon` (Windows-version check), `GetLanguage` (always
  "english") and a progress-bar repaint. Nothing hidden.
- **Uninstall does not undo the tweaks.** No `UninstallRun` calls Restore; the uninstaller deletes the
  `Restore` flag and settings values but leaves the changed Windows values (and the backup copies in
  `HKLM\Software\PGWARE\GameBoost`) behind.

## Components

| Binary | What | Status |
|---|---|---|
| [`GameBoost.exe`](GameBoost.exe/notes.md) (4.1 MB) | The whole product: one window with OS / CPU / internet-type / speed pickers, **Optimize now** and **Restore**. Registry-only tweaks, then asks to reboot. Also starts the licence and update checks. | **Complete** for Optimize/Restore (every registry write listed by `tweaks.py`); startup, licence and update-check paths targeted. |
| [`GameBoostUpdate.exe`](GameBoostUpdate.exe/notes.md) (3.9 MB) | Downloads the full installer over plain HTTP and runs it. No signature or hash check. | **Targeted** (strings, imports, flow). |
| [`GameBoostRegister.exe`](GameBoostRegister.exe/notes.md) (3.7 MB) | Trial / buy / activate screens. | **Strings** + imports (licensing at a high level only, by design). |

## Helper

| Script | What |
|---|---|
| [`tweaks.py`](tweaks.py) | `python -I tweaks.py GameBoost.exe`: disassembles the Optimize and Restore handlers with `objdump` and prints every registry get/set/delete (hive, key, value name, data). Has an `assert` self-check against the hand-read values. Specific to the sha256 above (hard-coded VAs). |
