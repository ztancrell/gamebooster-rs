# decomp/smart-pc-utilities/game-booster/

Static analysis of **PC Game Booster** by Smart PC Utilities (smartpcutilities.com/game-booster.html),
done for comparison with Smart Game Booster and PGWARE GameBoost. Nothing was executed (no Wine).
Plain-language overview with verdicts: [docs/smartpcutilities-gb/README.md](../../../docs/smartpcutilities-gb/README.md).

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05 from `https://www.smartpcutilities.com/files/gamebooster.exe` (the "Download Now" link on the official download page), which redirects to `www.smartdownloadcloud.com/downloads/download.html?id=7921` |
| Installer | `gamebooster.exe` (internal name `gamebooster-8.0.5002-setup.exe`), 26,719,552 bytes, sha256 `87640009bae425af657336efebf20470d6e80e7842e4adc6b2b5f66116fc8117` |
| Installer type | **Advanced Installer** EXE bootstrapper (`ExternalUi.pdb`, `ADVINSTSFX` footer) carrying `FILES.7z` (program files) and a 7z with two MSIs (x86 and x64). The 7z blobs have their first 0x200 bytes XORed with 0xFF; after undoing that, plain `7z x` works. |
| Version | **8.0.5002** (product and all vendor binaries `8.0.5002.1`); the product page calls it "PC Game Booster 8". Earlier name: "Game Fire" (the MSI migrates `Game Fire` settings). |
| Signatures | Installer and every vendor binary are Authenticode-signed by **"Ahmed Fathi", Minya, EG** (an individual, not the company), GoGetSSL code-signing CA, certificate valid 2025-04-26 to **2026-04-25** (expired at download time; whether a timestamp keeps the signatures valid was not checked). Bundled `Contig.exe` is Microsoft-signed (Sysinternals). |
| Tech stack | VB.NET on **.NET Framework 4.8** (WinForms, Krypton/FlatUI/ObjectListView controls), x64 and x86 builds. Third-party libraries: LibreHardwareMonitorLib 0.9.4 (embeds the **WinRing0** kernel driver), HidSharp, Microsoft.Win32.TaskScheduler, System.Management.Automation (PowerShell hosting), SQLCipher/SQLite, Newtonsoft.Json, protobuf-net, YamlDotNet, VDFParser. |
| Protection | All eight vendor assemblies are protected with **ConfuserEx**: renamed private members, flattened control flow, encrypted strings/constants, and **anti-tamper** (every method body is encrypted in an extra first PE section and decrypted at module load). Type names, public member names and a few literals survive. A static undo of the anti-tamper was attempted and failed (this build's runtime differs from stock ConfuserEx in a detail not found within budget), so **no method body was read**. Findings below come from metadata names, surviving strings, the installer database, shipped data files and the power-plan file. |
| Local copies | `/home/zpt/re/smartpcutilities-gb/dl/` (installer), `x/` (`FILES/`, MSIs, `setup.ini`), `msi/x64/` (MSI streams). Decompiled output was deleted after the research, as the user asked. |

## Hashes of the vendor binaries (x64 build)

| File | Size | sha256 |
|---|---|---|
| `GameBooster.exe` | 2,033,880 | `41f5a537f30b038c0498cfc33c127aaec3201465682ff370bf7457ca1ade8e77` |
| `GameBoosterTray.exe` | 824,536 | `d35f89eea389596ccc6595f871239303ad4cd76796f801dc476871e9752135a6` |
| `GameBoosterCore.dll` | 632,024 | `e6aff9277f257c9e77f0d4b2a4f9047fb9a8d4b44ec27b51c0707e8169e6b7c2` |
| `GameBoosterUI.dll` | 769,752 | `ac4c2fdfe93d6ffe7599275eaf6d3b93062c45ec81b608fd8f72395d78283926` |
| `SmartPCUtilitiesCore.dll` | 237,272 | `38f02db6b0e643ea109e9ec0bff0110add3f1b6f2ba2f197b8449df9d7b61de0` |
| `UpdateCheck.exe` | 469,720 | `410a0c830dbb4981ff1d53229af6a94e42784768681eac1d1095f6bf54162f34` |
| `UpdateInstaller.exe` | 301,784 | `fefdbf66f1fb9bca4952aa64669349d5fa2438eddf72c71351467e7f62f3658e` |
| `Feedback.exe` | 359,128 | `0cba037c39c4c0f3e3fe70d3eeaca008b778c587e6cdf5bab4b81beec63127ae` |
| `GameBooster-PowerPlan.pow` | | `cbb586ac10f618f3ff8343544e0d49fe01145415294945354ed8dfb7ed50c40f` |
| `Games.db` (SQLCipher, encrypted) | 2,859,008 | `044d3887e326b1af19130015fb51303f7324b151221df08609567276a137a962` |
| `LibreHardwareMonitorLib.dll` 0.9.4 | | `c01da3efb7f5c4533412b05359434e42e3daeef522b226b5dc2134dd0659796b` |
| `gamebooster-8.0.5002-setup.x64.msi` | | `347fdad838711c55f04eccc182c80d3631b19da2fa355380b4638a6993aad46e` |

## Components

Status words as in [../../iobit/smart-game-booster/README.md](../../iobit/smart-game-booster/README.md#components): **Targeted** (the parts that matter read), **Strings**
(names, strings, resources, no code). Here "Names" means metadata names only, because every method
body is encrypted.

| Part | What | Status |
|---|---|---|
| [Setup](Setup/notes.md) | Advanced Installer MSI: files, registry, scheduled tasks, power-plan import, install/uninstall tracking, uninstall cleanup. | **Complete** (all tables read with `msitables.py`). |
| [Data](Data/notes.md) | Power plan `.pow` (decoded), exception lists, vendor list, `Games.db`, `Contig.exe`. | **Decoded** (except `Games.db`, encrypted). |
| [`GameBoosterCore.dll`](GameBoosterCore.dll/notes.md) | Game Mode manager, game library scanners, licensing, updates, preferences. | **Names**. |
| [`SmartPCUtilitiesCore.dll`](SmartPCUtilitiesCore.dll/notes.md) | Shared helpers: power plans, services, processes, registry, Defender, PowerShell, signatures, promotions. | **Names**. |
| [`GameBooster.exe`](GameBooster.exe/notes.md) | Main window: My Games, Game Mode, Live Optimization, System Optimizer tweak cards, defrag, hardware monitor. | **Names**. |
| [`GameBoosterUI.dll`](GameBoosterUI.dll/notes.md) | Forms: Game Mode editor/progress/report, licence and promotion dialogs. | **Names**. |
| [`GameBoosterTray.exe`](GameBoosterTray.exe/notes.md) | Tray resident: auto Game Mode, Live Optimization, memory trimming, hotkeys. | **Names**. |
| [`UpdateCheck.exe`, `UpdateInstaller.exe`](UpdateCheck.exe/notes.md) | Update check, download, signature check, install. | **Names**. |
| [`Feedback.exe`](Feedback.exe/notes.md) | Feedback / error report form. | **Names**. |

## Helper

| Script | What |
|---|---|
| [`msitables.py`](msitables.py) | `python3 -I msitables.py STREAMDIR [Table…]` dumps MSI tables from the streams `7z x setup.msi` writes (string pool, `_Columns`, column-major rows). Generic; contains no vendor code; `assert` self-check of the column-width rule. |

The power plan was decoded with `hivexregedit --export GameBooster-PowerPlan.pow '\'` (a `.pow` file
is a registry hive). The ConfuserEx anti-tamper attempt is not kept, since it did not work.
