# Installer (Inno Setup, SetupExitLag-5.24.0-x64.exe)

Read from the installer's setup data and its compiled Pascal script, with binary-refinery `xtinno`.
Nothing was executed.

## Files

- `{app}` = `C:\Program Files\ExitLag`: the Qt client `ExitLag.exe`, `libexitlag.dll`, Qt 6
  plugins and QML, OpenSSL 3, Boost 1.87, `ndisapi.dll`, `nfapi.dll`, `PresentMonAPI2.dll`,
  `WebView2Loader.dll`, `handler.exe`, `exitlag_autostart.exe`, `exitlag_reboot_required.exe`,
  `ExitLagPmService.exe`, the VC++ runtime (x64 or ARM64 copy), `snetcfg.exe` and `drivers\`
  (`ndextlag.sys/.cat/_lwf.inf`, per architecture).
- `{tmp}` (used during setup only): the WinpkFilter driver set for Windows 7/8 (i386, amd64) with a
  `DriverSigning.exe` tool, the NetFilter SDK set (`nfextlag.sys/.inf/.cat` for x86, x64, ARM64,
  `nfregdrv.exe`, `nfapi.dll`, a version stamp `nfsdk_ext_version.iss`) and `DriverCacheCleaner.exe`.
- Shortcuts: Start menu, optional public desktop icon. Post-install "Launch ExitLag" runs as the
  original (non-elevated) user.

## What the script does

- Uninstalls older or beta ExitLag builds first (three known uninstall keys), with a prompt.
- **Refuses to continue while "TibiaTunnel" is installed** and offers to uninstall it (a tunnelling
  tool for the game Tibia). No other competitor is checked.
- **Two kernel drivers are installed at setup time**, not on first use:
  1. NetFilter SDK WFP driver `nfextlag`: copied to `System32\drivers` and registered with
     `nfregdrv.exe`; the script compares the installed driver version with the shipped one and
     replaces it when needed (`sc stop nfextlag` first). Failure only warns that "NFSDK may not be
     available as a redirection method".
  2. WinpkFilter NDIS lightweight filter `ndextlag`: installed into every adapter's binding with
     `snetcfg.exe -l ndextlag_lwf.inf -c s -i nt_ndextlag`; the script then checks the device
     `\\.\NDEXTLAG`. Failure warns that "NDIS may not be available as a redirection method".
- **Service**: `sc create ExitLagPmService start= auto binPath=...` then `sc start`. It is a copy of
  Intel's PresentMon service (frame-time data for the overlay), so it runs as LocalSystem and starts
  with Windows whether or not ExitLag is open.
- Removes the drivers from the Windows driver store cache with `DriverCacheCleaner.exe`.
- Kills `exitlag_reboot_required.exe` before replacing files.
- No registry Run key and no scheduled task is created by the installer itself. Autostart is
  created by the app on first run (see `../app/notes.md`).

## Uninstall

`snetcfg.exe -v -u nt_ndextlag` (NDIS filter), `nfregdrv.exe -u nfextlag` (WFP driver),
`sc stop/delete ExitLagPmService`, driver cache clean-up. So the uninstaller does remove both
drivers and the service. Settings the app changed while running (power plan, services, DNS, NTFS
last-access, performance counters, GPU clocks) are not touched by the uninstaller; they are only
undone by switching them off inside the app before uninstalling.
