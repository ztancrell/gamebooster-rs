# Razer Installer (LWI2) 2.6.0.890: notes

What the Cortex download (`RazerCortexInstaller.exe`) does, from the decompiled Razer assemblies
(`RazerInstaller.exe`, `Razer.Installer.Core/Hub/UI/CompatibleV1.dll`, `Razer.Analytics.dll`). Written in
our own words; no vendor code is reproduced here.

## Launch chain

1. The outer signed native exe (internally "Razer.Installer.Extractor", runs as invoker) carries one
   resource: a .NET DotNetZip self-extractor. It unpacks 81 files to
   `%windir%\Installer\Razer\Installer2\App\` and starts `RazerInstaller.exe` there, which requires
   administrator rights (UAC prompt).
2. The app to install comes from the `--app` argument or, failing that, from the launcher's file name:
   a lookup table maps a name containing `cortex` to the internal product name **`gamebooster2`**
   (display name "Razer Cortex"). The same bootstrapper installs Synapse, Chroma, Axon, etc. under other
   file names. A file name containing `staging` switches to Razer's staging servers.
3. Command-line options: `--app`, `--name`, `--silent`, `--install`, `--uninstall`, `--culture`,
   `--staging`, `--devicestaging`, `--beta`, `--betatag`, `--ep`, `--hash`, `--host`, `--showdevice`,
   `--dmqa`, `--dmstaging`, `--ad` (marks the run as coming from an advert, for telemetry).

## Product manifest and download

- Step 1: ask `discovery3.razerapi.com/api/v1/endpoints…` for the hash of the release channel ("prod",
  or a beta tag).
- Step 2: GET `manifest3.razerapi.com/api/v1/releases/{hash}/tags/{channel}/products?…` with the OS name and
  version, CPU architecture, **PC manufacturer, model and SKU** (read from the BIOS registry key) and
  locale. The answer lists every product with its modules: download URL, file size, MD5 and/or SHA-256,
  silent-install arguments, whether admin or reboot is needed, whether the module is **optional or hidden**
  in the UI, uninstall command, and promotion material.
- If the server is unreachable it falls back to a cached manifest in `…\Installer2\ManifesetCache`
  (sic).
- Each module installer is downloaded to `%windir%\Installer\Razer\Installer2\DownloadCache\` (admin-only
  folder), its **MD5** is checked against the manifest (SHA-256 only when the manifest gives no MD5), and
  then it is run with the manifest's arguments and waited for.
- **No code-signature check** of downloaded module installers. Integrity rests on HTTPS plus the checksum
  in the same HTTPS answer, so whoever controls the manifest controls what runs as admin. MD5 is weak but,
  with the hash delivered over HTTPS from the same server, the practical risk is the server, not
  collisions. Better than PGWARE's plain-HTTP, unchecked updater; weaker than signing.
- The install progress bar is driven by a timer ("progress simulator") while the module installer runs,
  not by real progress. Cosmetic.
- Installed apps are tracked in an `installed.json` file under the installer's data folder. After an app
  installs, the installer **rewrites that app's Windows "Uninstall" and "Modify" entries** to point at
  `RazerInstaller.exe /uninstall true`, so removing Cortex from Windows Settings goes through the Razer
  installer (which asks for an uninstall reason and reports it, see Telemetry).
- Downloaded module installers stay in `DownloadCache` (the "space cleaner" routine is empty); they are
  deleted only when their checksum fails.
- The installer itself creates **no services, scheduled tasks, Run keys or drivers**. What Cortex installs
  (Razer Central service, any drivers, autostart) is decided by the module packages listed in the manifest;
  for Cortex see [../cortex/notes.md](../cortex/notes.md). A helper class for creating Windows services exists in the legacy
  `CompatibleV1` assembly (Razer Central / Synapse 3 era) but is not on the Cortex path we could trace.

## Cross-promotion

- Before installing, the UI may show a **cross-promotion popup** and promo cards for other Razer apps,
  chosen by the server (`promote_apps`, `promotional_materials`). Impressions and clicks are reported.
- When the product is Synapse 4, Chroma is added **by default** (pre-ticked). Nothing similar is hard-coded
  for Cortex; additional Cortex modules can only come from the manifest.
- App ratings and user counts shown in the UI come from `bespoke-analytics.razerapi.com`.
- The finish screen links Razer's social media, Discord and Insider forum.

## Telemetry (no opt-out found, starts before the licence is accepted)

Two channels:

1. **AWS Kinesis "big data"**: the installer fetches short-lived AWS credentials (and its own public IP)
   from an Amazon API Gateway endpoint (`u05srooyhc.execute-api.us-east-1.amazonaws.com/sts`) and writes
   events to the Kinesis stream `lwi_unauth_prod`. Each event carries an event id, session id, a user id,
   IP, app version, a **device fingerprint**, timestamp, category/action/label and extra fields.
   - The device fingerprint is the default network adapter's **MAC address, the system drive's volume
     serial number and the Windows MachineGuid**, joined and base64-encoded. Base64 is not hashing: Razer
     receives the raw hardware identifiers.
   - The user id is the most recently used **Razer ID** found as a folder name under
     `C:\ProgramData\Razer\Razer Central\Accounts`, if Razer software was used before, so installs are
     linked to an account even without logging in.
   - Events: installer launched (sent while the splash screen loads, **before** the licence screen),
     licence accepted, install clicked/started/finished/cancelled, promo popup and card impressions and
     clicks, social-link clicks, launch after install, and uninstall with the **reason and free-text
     feedback** the user typed.
2. **Razer Analytics** (`maxineapi.razersynapse.com/v1`, app id `lwi2`): launch, keep-alive and
   diagnostics reports; when no Razer ID is known it uses an id derived from a SHA-1 of the MAC address.

No setting, flag or prompt to disable either channel was found in the installer.

## Other components bundled with the installer

- Razer **device detection** (native `rzS3detmgr.dll` plus wrapper, PID lists in small JSON files): detects
  plugged-in Razer USB/dongle/dock devices so the installer can offer the matching software (`--showdevice`).
  It can update itself from `manifest-assets.razersynapse.com/lwi/` (a zip; checksum handling not
  traced).
- **BLE** pairing helpers for Razer Bluetooth peripherals.
- Neither is a game-boosting component; neither installs a driver from this package.

## Anti-cheat relevance

The installer touches no game process. Nothing here matters for EA Javelin / BF6. For the Cortex app see
[../cortex/notes.md](../cortex/notes.md#anti-cheat-bf6--ea-javelin).
