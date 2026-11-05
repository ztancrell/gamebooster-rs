# Hone helper programs: notes

Our own words. The .NET helpers (HoneSvc, hone-bios) were decompiled locally and the output deleted.
The native ones were read as strings and imports.

## HoneSvc.exe 1.0.1 (Windows service)

- A .NET service named `HoneSvc` ("Hone Service"), start type **Automatic**. HoneUtility creates
  it with `CreateService` when "Optimize device affinity" or "Optimize Message Signaled Interrupts" is
  applied. The account is set there, inside encrypted code. It is most likely LocalSystem, since the
  service starts an elevated HoneUtility.
- It watches for device arrival and removal, default-audio-device changes, power events and session
  changes. After each event (with a start-up delay so it doesn't run twice), it reads the
  `HKLM\SOFTWARE\Hone` value `TweaksSvc` and re-runs HoneUtility with **AES-encrypted** arguments for
  the affinity and/or MSI tweak. In effect it re-applies those device settings whenever Windows or a
  driver resets them. Logs go to `hone_service.log`.
- It is permanent: it stays installed, running at every boot as SYSTEM, until the tweak is reverted
  or Hone is removed.

## hone-elevation-helper.exe 1.1.3 (native C++)

- "Secure privilege elevation service for Hone". It is started once with UAC and then serves the named
  pipe `\\.\pipe\HoneElevator` (with an SDDL security descriptor). It checks the client process,
  runs only executables on its allow-list whose Authenticode signature matches a **pinned signing
  certificate** (WinVerifyTrust plus a pin), caps file sizes, and refuses registry paths not on its
  allow-list.
- This design is better than Razer Cortex's SYSTEM service that starts anything. It is still a
  standing admin broker while Hone runs.

## hone-bios.exe (.NET) with AMI SCEWIN

- A wrapper around **AMI SCEWIN** (`SCEWIN\SCEWIN_64.exe`, not shipped in the package; the app fetches
  it as a "hone resource"). It exports the BIOS setup variables to `nvram.txt`, lists the questions and
  options as JSON, and writes changed values back from `new-nvram.txt` (import, silent).
- SCEWIN loads AMI's flash driver (`amifldrv64.sys`) to reach NVRAM, so this is a **kernel driver load**
  and a **firmware setting write**. It works only on AMI Aptio boards. Revert depends on Hone keeping
  the exported file. BIOS defaults are available from setup.
- What it changes is the renderer's BIOS table (see [../app/notes.md](../app/notes.md)): C-states and
  C1E off, SpeedStep/Speed Shift off, spread spectrum off, Turbo on, Resizable BAR on, vendor
  auto-installers off, and more.

## SysInfo.exe 2.0.5 (native) with HWiNFO64.dll 8.55

- Hardware inventory: CPU, GPU (CUDA probe), RAM, disks, display resolution per process, Secure Boot,
  VBS and HVCI state, and the vulnerable-driver blocklist state.
- Live sensors through the **HWiNFO SDK** (`HWiNFO64.dll`, signed by REALiX). The HWiNFO SDK loads
  HWiNFO's signed **kernel driver** to read sensors. SysInfo has a "persistentDriver" option. It also
  uses PresentMon-style frame data for benchmarks.
- Writes `hone_sensors_report.txt`. Its output feeds the hardware fingerprint and the uploaded device
  info.

## Other helpers

- **GameManager.exe 1.3.9** (native; version info suggests .NET Native AOT): detects installed games across launchers (Steam,
  Epic, Riot, Battle.net, EA and others) and launches them. Controlled with JSON over stdin.
- **hone-system-validator.exe** (native AOT): validates the system before and after benchmark runs.
- **HoneRedirect.exe**: an Edge-redirect helper. It rewrites `microsoft-edge:` links to the default
  browser (reads the https `UserChoice`). It backs a "stop Edge hijacking links" type item.
- **SetTimerResolution.exe**, **SetDpi.exe**: small tools used by the timer and mouse/display items.
- **reg.exe** (AuraSide's own, not Windows'), **TotalReg.exe** (third-party registry editor), 7-Zip,
  **nvidiaProfileInspector.zip** with `HoneProfile.nip` / `Base_Profile.nip`.
- **hone-notifier.exe**: Windows toast notifications.
- **openmouse-host** (AGPL Node helper with `node-hid` and `koffi`): configures supported gaming mice
  over HID (device settings; not studied further).
- **No Hone kernel driver** ships in the package. The kernel code Hone can cause to load is
  third-party: HWiNFO's sensor driver (SysInfo) and AMI's flash driver (SCEWIN for BIOS edits).
