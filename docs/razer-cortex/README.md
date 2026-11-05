# Razer Cortex: what we could check so far

Razer Cortex (razer.com/cortex) is Razer's free PC "game booster" and launcher. This page covers what
reading the official download (static reverse engineering, **nothing was run**) shows. Technical notes:
[../../decomp/razer-cortex/README.md](../../decomp/razer-cortex/README.md).

## Short version

- The download from razer.com (`RazerCortexInstaller.exe`, Razer Installer 2.6.0.890, fetched 2026-11-05)
  is only a **web installer**. It contains no Cortex code; it asks Razer's servers for the current Cortex
  package and installs that. Asking those servers was outside the rules of this research, so **Cortex
  itself (Game Booster, System Booster, overlay, service) has not been analysed yet**. Nothing below says
  what Cortex's boost does.
- The installer is properly code-signed by Razer and downloads over HTTPS with a checksum check, but it
  does **not** check the signature of what it downloads before running it as administrator.
- It sends install telemetry **before you accept the licence**, with **no opt-out**, including a device
  fingerprint made of your network card's MAC address, your system drive's serial number and Windows'
  machine ID (sent readable, not hashed), plus your Razer ID if any Razer software was logged in before.
- It shows server-chosen adverts for other Razer apps during install and reports every view and click.
- After installing, it takes over the app's Windows uninstall entry, so uninstalling goes through the Razer
  installer, which asks for a reason and sends it to Razer.
- It sends your PC's manufacturer, model and SKU to Razer to pick the product list.

## Every part checked, in one table

Verdicts as in [the SGB overview](../sgb/README.md): **useful**, **placebo**, **harmful**, **risky** (can
break things, cost privacy or security, or trip anti-cheat), **bloat**.

| Part | What it does | Verdict |
|---|---|---|
| Web installer | Unpacks itself to `C:\Windows\Installer\Razer\Installer2\App`, asks for admin, downloads and runs the Cortex package chosen by Razer's server. | Necessary to get Cortex; **risky** in that the server fully decides what runs as admin (checksum only, no signature check). |
| Install telemetry (AWS Kinesis) | Install funnel events with IP, MAC + disk serial + machine ID fingerprint, Razer ID; starts before the licence screen; no opt-out. | **risky** (privacy) |
| Razer Analytics channel | Launch / keep-alive / diagnostics, keyed by a hash of the MAC address. | **risky** (privacy), minor |
| Cross-promotion popup and cards | Server-chosen ads for other Razer apps during install; impressions and clicks reported. | **bloat** / mild dark pattern |
| Progress bar | Timer-driven while the package installer runs, not real progress. | cosmetic (placebo) |
| Uninstall takeover | Points Windows' uninstall entry at the Razer installer, which asks for a reason and reports it. | mild dark pattern |
| Razer device detection, Bluetooth helpers | Find plugged-in Razer mice/keyboards/dongles to offer their software. Self-updating from Razer's CDN. | **bloat** for a Cortex-only install; harmless |
| Services, drivers, scheduled tasks, Run keys | **None from the installer itself.** Whatever Cortex adds comes from its package (not yet analysed). | not verified |
| Game Booster, System Booster, FPS overlay, auto-boost, Razer Silver rewards, updater | In the Cortex package. | **not verified** |

## For a 2019 MacBook Pro under Boot Camp playing BF6

Nothing verified yet about the boost itself. The installer alone does not touch games, so it is no
anti-cheat risk for BF6 (EA Javelin); an overlay or injection in Cortex would be, and needs the package to
judge. Until then, nothing here is worth installing for that machine.

## Compared with SGB, PGWARE and gamebooster-rs

- **Installer hygiene:** better than PGWARE (signed, HTTPS, checksums vs. unsigned plain-HTTP updater)
  and SGB; worse than gamebooster-rs, which has no installer, no network and no telemetry.
- **Privacy:** the most aggressive so far at install time (raw hardware identifiers, account linking,
  pre-licence telemetry, no opt-out).
- **Worth adopting into gamebooster-rs:** nothing yet. The installer has no tweaks; the boost logic is in
  the package not yet analysed.

## Open question

To finish this research, one of: allow two anonymous requests to Razer's product-list servers (the same
requests the installer makes) to find the Cortex package on Razer's CDN, or take the downloaded package
from a PC that already installed Cortex.
